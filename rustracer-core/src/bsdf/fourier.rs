use std::path::Path;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufReader, Read};

use byteorder::{NativeEndian, ReadBytesExt};
use failure::Error;
use num::zero;

use {clamp, Point2f, Vector3f};
use bsdf::{BxDF, BxDFType};
use geometry::{cos_theta, sin2_theta};
use interpolation::{catmull_rom_weights, fourier, sample_fourier, sample_catmull_rom_2d};
use material::TransportMode;
use spectrum::Spectrum;

// This is ugly, but FourierBSDF has to be Copy, because it implements BxDF and needs to be
// allocated from the arena, but I can't make it work with FourierBSDFTable... There must be a way
// to do it with proper lifetime annotations but I couldn't figure it out, so I resorted to using
// a raw pointer. It *should* be safe, since FourierBSDFTable lives for the duration of the
// material (i.e the whole render), while FourierBSDF lives for a single ray trace. But TODO
// revisit this at some point.

#[derive(Debug, Copy, Clone)]
pub struct FourierBSDF {
    bsdf_table: *const FourierBSDFTable,
    mode: TransportMode,
}

unsafe impl Send for FourierBSDF {}
unsafe impl Sync for FourierBSDF {}

impl FourierBSDF {
    pub fn new(bsdf_table: &FourierBSDFTable, mode: TransportMode) -> FourierBSDF {
        FourierBSDF {
            bsdf_table: bsdf_table as *const FourierBSDFTable,
            mode,
        }
    }
}

impl BxDF for FourierBSDF {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let bsdf_table = unsafe { &*self.bsdf_table };
        // Find the zenith angle cosines and azimuth difference angle
        let muI = cos_theta(&(-wi));
        let muO = cos_theta(wo);
        let cos_phi = cos_d_phi(&(-wi), wo);

        // Compute Fourier coefficients $a_k$ for $(\mui, \muo)$

        // Determine offsets and weights for $\mui$ and $\muo$
        let mut weightsI = [0.0; 4];
        let mut weightsO = [0.0; 4];
        let maybe_offsetI = bsdf_table.get_weights_and_offset(muI, &mut weightsI);
        let maybe_offsetO = bsdf_table.get_weights_and_offset(muO, &mut weightsO);
        if maybe_offsetI.is_none() || maybe_offsetO.is_none() {
            return Spectrum::black();
        }
        let offsetI = maybe_offsetI.unwrap_or(0);
        let offsetO = maybe_offsetO.unwrap_or(0);

        // Allocate storage to accumulate _ak_ coefficients
        // TODO use ALLOCA when available
        let mut ak = vec![0.0; (bsdf_table.m_max * bsdf_table.n_channels) as usize];

        // Accumulate weighted sums of nearby $a_k$ coefficients
        let mut m_max = 0;
        for b in 0..4 {
            for a in 0..4 {
                // Add contribution of _(a, b)_ to $a_k$ values
                let weight = weightsI[a] * weightsO[b];
                if weight != 0.0 {
                    let (ap, m) = bsdf_table.get_ak(offsetI + a, offsetO + b);
                    m_max = u32::max(m_max, m);
                    for c in 0..bsdf_table.n_channels {
                        for k in 0..m {
                            ak[(c * bsdf_table.m_max + k) as usize] +=
                                weight * ap[(c * m + k) as usize];
                        }
                    }
                }
            }
        }

        // Evaluate Fourier expansion for angle $\phi$
        let Y = f32::max(0.0, fourier(&ak, m_max, cos_phi));
        let mut scale = if muI != 0.0 {
            (1.0 / f32::abs(muI))
        } else {
            0.0
        };

        // Update _scale_ to account for adjoint light transport
        if self.mode == TransportMode::RADIANCE && muI * muO > 0.0 {
            let eta = if muI > 0.0 {
                1.0 / bsdf_table.eta
            } else {
                bsdf_table.eta
            };
            scale *= eta * eta;
        }
        if bsdf_table.n_channels == 1 {
            Spectrum::from(Y * scale)
        } else {
            // Compute and return RGB colors for tabulated BSDF
            let R = fourier(&ak[1 * bsdf_table.m_max as usize..], m_max, cos_phi);
            let B = fourier(&ak[2 * bsdf_table.m_max as usize..], m_max, cos_phi);
            let G = 1.39829 * Y - 0.100913 * B - 0.297375 * R;
            Spectrum::rgb(R * scale, G * scale, B * scale).clamp()
        }
    }

    fn sample_f(&self, wo: &Vector3f, u: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        let bsdf_table = unsafe { &*self.bsdf_table };
        // Sample zenith angle component for _FourierBSDF_
        let muO = cos_theta(wo);
        let (muI, _, pdfMu) = sample_catmull_rom_2d(
            &bsdf_table.mu,
            &bsdf_table.mu,
            &bsdf_table.a0,
            &bsdf_table.cdf,
            muO,
            u[1],
        );

        // Compute Fourier coefficients $a_k$ for $(\mui, \muo)$

        // Determine offsets and weights for $\mui$ and $\muo$
        let mut weightsI = [0.0; 4];
        let mut weightsO = [0.0; 4];
        let maybe_offsetI = bsdf_table.get_weights_and_offset(muI, &mut weightsI);
        let maybe_offsetO = bsdf_table.get_weights_and_offset(muO, &mut weightsO);
        if maybe_offsetI.is_none() || maybe_offsetO.is_none() {
            return (Spectrum::black(), zero(), 0.0, self.get_type());
        }
        let offsetI = maybe_offsetI.unwrap_or(0);
        let offsetO = maybe_offsetO.unwrap_or(0);

        // Allocate storage to accumulate _ak_ coefficients
        // TODO use ALLOCA when available
        let mut ak = vec![0.0; (bsdf_table.m_max * bsdf_table.n_channels) as usize];

        // Accumulate weighted sums of nearby $a_k$ coefficients
        let mut m_max = 0;
        for b in 0..4 {
            for a in 0..4 {
                // Add contribution of _(a, b)_ to $a_k$ values
                let weight = weightsI[a] * weightsO[b];
                if weight != 0.0 {
                    let (ap, m) = bsdf_table.get_ak(offsetI + a, offsetO + b);
                    m_max = u32::max(m_max, m);
                    for c in 0..bsdf_table.n_channels {
                        for k in 0..m {
                            ak[(c * bsdf_table.m_max + k) as usize] +=
                                weight * ap[(c * m + k) as usize];
                        }
                    }
                }
            }
        }

        // Importance sample the luminance Fourier expansion
        let (Y, pdfPhi, phi) = sample_fourier(&ak, &bsdf_table.recip, m_max, u[0]);
        let pdf = f32::max(0.0, pdfPhi * pdfMu);

        // Compute the scattered direction for _FourierBSDF_
        let sin2ThetaI = f32::max(0.0, 1.0 - muI * muI);
        let mut norm = f32::sqrt(sin2ThetaI / sin2_theta(wo));
        if f32::is_infinite(norm) {
            norm = 0.0;
        }
        let sinPhi = f32::sin(phi);
        let cosPhi = f32::cos(phi);

        let wi = -Vector3f::new(
            norm * (cosPhi * wo.x - sinPhi * wo.y),
            norm * (sinPhi * wo.x + cosPhi * wo.y),
            muI,
        );

        // Mathematically, wi will be normalized (if wo was). However, in
        // practice, floating-point rounding error can cause some error to
        // accumulate in the computed value of wi here. This can be
        // catastrophic: if the ray intersects an object with the FourierBSDF
        // again and the wo (based on such a wi) is nearly perpendicular to the
        // surface, then the wi computed at the next intersection can end up
        // being substantially (like 4x) longer than normalized, which leads to
        // all sorts of errors, including negative spectral values. Therefore,
        // we normalize again here.
        let wi = wi.normalize();

        // Evaluate remaining Fourier expansions for angle $\phi$
        let mut scale = if muI != 0.0 {
            (1.0 / f32::abs(muI))
        } else {
            0.0
        };
        if self.mode == TransportMode::RADIANCE && muI * muO > 0.0 {
            let eta = if muI > 0.0 {
                1.0 / bsdf_table.eta
            } else {
                bsdf_table.eta
            };
            scale *= eta * eta;
        }

        if bsdf_table.n_channels == 1 {
            return (Spectrum::from(Y * scale), wi, pdf, self.get_type());
        }
        let R = fourier(&ak[(1 * bsdf_table.m_max) as usize..], m_max, cosPhi);
        let B = fourier(&ak[(2 * bsdf_table.m_max) as usize..], m_max, cosPhi);
        let G = 1.39829 * Y - 0.100913 * B - 0.297375 * R;
        return (
            Spectrum::rgb(R * scale, G * scale, B * scale).clamp(),
            wi,
            pdf,
            self.get_type(),
        );
    }

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> f32 {
        let bsdf_table = unsafe { &*self.bsdf_table };
        // Find the zenith angle cosines and azimuth difference angle
        let muI = cos_theta(&(-wi));
        let muO = cos_theta(wo);
        let cosPhi = cos_d_phi(&(-wi), wo);

        // Compute luminance Fourier coefficients $a_k$ for $(\mui, \muo)$
        let mut weightsI = [0.0; 4];
        let mut weightsO = [0.0; 4];
        let maybe_offsetI = bsdf_table.get_weights_and_offset(muI, &mut weightsI);
        let maybe_offsetO = bsdf_table.get_weights_and_offset(muO, &mut weightsO);
        if maybe_offsetI.is_none() || maybe_offsetO.is_none() {
            return 0.0;
        }
        let offsetI = maybe_offsetI.unwrap_or(0);
        let offsetO = maybe_offsetO.unwrap_or(0);

        // Allocate storage to accumulate _ak_ coefficients
        // TODO use ALLOCA when available
        let mut ak = vec![0.0; (bsdf_table.m_max * bsdf_table.n_channels) as usize];

        let mut mMax = 0;
        for o in 0..4 {
            for i in 0..4 {
                let weight = weightsI[i] * weightsO[o];
                if weight == 0.0 {
                    continue;
                }

                let (coeffs, order) = bsdf_table.get_ak(offsetI + i, offsetO + o);
                mMax = u32::max(mMax, order);

                for k in 0..order {
                    ak[k as usize] += coeffs[k as usize] * weight;
                }
            }
        }

        // Evaluate probability of sampling _wi_
        let mut rho = 0.0;
        for o in 0..4 {
            if weightsO[o] == 0.0 {
                continue;
            }
            rho += weightsO[o]
                * bsdf_table.cdf
                    [(offsetO + o) * bsdf_table.n_mu as usize + bsdf_table.n_mu as usize - 1]
                * (2.0 * PI);
        }
        let Y = fourier(&ak, mMax, cosPhi);
        if rho > 0.0 && Y > 0.0 {
            Y / rho
        } else {
            0.0
        }
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_GLOSSY
    }
}

#[derive(Debug, Clone)]
pub struct FourierBSDFTable {
    eta: f32,
    m_max: u32,
    n_channels: u32,
    n_mu: u32,
    mu: Box<[f32]>,
    m: Box<[u32]>,
    a_offset: Box<[u32]>,
    a: Box<[f32]>,
    a0: Box<[f32]>,
    cdf: Box<[f32]>,
    recip: Box<[f32]>,
}

impl FourierBSDFTable {
    //  File format description:
    //
    //  This is the file format generated by the material designer of the paper
    //
    //  'A Comprehensive Framework for Rendering Layered Materials' by
    //  Wenzel Jakob, Eugene D'Eon, Otto Jakob and Steve Marschner
    //  Transactions on Graphics (Proceedings of SIGGRAPH 2014)
    //
    //  A standalone Python plugin for generating such data files is available
    //  on GitHub: https://github.com/wjakob/layerlab
    //
    //  This format specifies an isotropic BSDF expressed in a Spline x Fourier
    //  directional basis. It begins with a header of the following type:
    //
    // struct Header {
    //     uint8_t identifier[7];     // Set to 'SCATFUN'
    //     uint8_t version;           // Currently version is 1
    //     uint32_t flags;            // 0x01: file contains a BSDF, 0x02: uses
    // harmonic extrapolation
    //     int nMu;                   // Number of samples in the elevational
    // discretization
    //
    //     int nCoeffs;               // Total number of Fourier series coefficients
    // stored in the file
    //     int mMax;                  // Coeff. count for the longest series occurring
    // in the file
    //     int nChannels;             // Number of color channels (usually 1 or 3)
    //     int nBases;                // Number of BSDF basis functions (relevant for
    // texturing)
    //
    //     int nMetadataBytes;        // Size of descriptive metadata that follows the
    // BSDF data
    //     int nParameters;           // Number of textured material parameters
    //     int nParameterValues;      // Total number of BSDF samples for all textured
    // parameters
    //     float eta;                 // Relative IOR through the material
    // (eta(bottom) / eta(top))
    //
    //     float alpha[2];            // Beckmann-equiv. roughness on the top (0) and
    // bottom (1) side
    //     float unused[2];           // Unused fields to pad the header to 64 bytes
    // };
    //
    //  Due to space constraints, two features are not currently implemented in PBRT,
    //  namely texturing and harmonic extrapolation (though it would be
    // straightforward
    //  to port them from Mitsuba.)
    pub fn read<P: AsRef<Path>>(filename: P) -> Result<FourierBSDFTable, Error> {
        let filename = filename.as_ref();
        let file = File::open(filename)?;
        let mut f = BufReader::new(file);

        info!("Loading BSDF file \"{}\"", filename.display());

        const HEADER_EXP: [u8; 8] = [
            'S' as u8,
            'C' as u8,
            'A' as u8,
            'T' as u8,
            'F' as u8,
            'U' as u8,
            'N' as u8,
            '\x01' as u8,
        ];
        let mut header = [0; 8];
        f.read_exact(&mut header)?;
        if header != HEADER_EXP {
            bail!(
                "BSDF file \"{}\" has an invalid header!",
                filename.display()
            );
        }
        let flags = f.read_u32::<NativeEndian>()?;
        let n_mu = f.read_u32::<NativeEndian>()?;
        let n_coeffs = f.read_u32::<NativeEndian>()?;
        let m_max = f.read_u32::<NativeEndian>()?;
        let n_channels = f.read_u32::<NativeEndian>()?;
        let n_bases = f.read_u32::<NativeEndian>()?;
        let _n_metadata_bytes = f.read_u32::<NativeEndian>()?;
        let _n_parameters = f.read_u32::<NativeEndian>()?;
        let _n_parameter_values = f.read_u32::<NativeEndian>()?;
        let eta = f.read_f32::<NativeEndian>()?;
        let _alpha_0 = f.read_f32::<NativeEndian>()?;
        let _alpha_1 = f.read_f32::<NativeEndian>()?;
        let _unused = f.read_f32::<NativeEndian>()?;
        let _unused = f.read_f32::<NativeEndian>()?;

        if flags != 1 || (n_channels != 1 && n_channels != 3) || n_bases != 1 {
            bail!("Unsupported BSDF file \"{}\"", filename.display());
        }

        let mut mu = vec![0.0; n_mu as usize];
        let mut cdf = vec![0.0; (n_mu * n_mu) as usize];
        let mut a0 = vec![0.0; (n_mu * n_mu) as usize];
        let mut offset_and_length = vec![0; (n_mu * n_mu * 2) as usize];
        let mut a_offset = vec![0; (n_mu * n_mu) as usize];
        let mut m = vec![0; (n_mu * n_mu) as usize];
        let mut a = vec![0.0; n_coeffs as usize];

        f.read_f32_into::<NativeEndian>(&mut mu)?;
        f.read_f32_into::<NativeEndian>(&mut cdf)?;
        f.read_u32_into::<NativeEndian>(&mut offset_and_length)?;
        f.read_f32_into::<NativeEndian>(&mut a)?;

        for i in 0..(n_mu * n_mu) {
            let offset = offset_and_length[2 * i as usize];
            let length = offset_and_length[2 * i as usize + 1];
            a_offset[i as usize] = offset;
            m[i as usize] = length;
            a0[i as usize] = if length > 0 { a[offset as usize] } else { 0.0 };
        }

        let mut recip = vec![0.0; m_max as usize];
        for i in 0..m_max {
            recip[i as usize] = 1.0 / i as f32;
        }

        Ok(FourierBSDFTable {
            n_mu,
            m_max,
            n_channels,
            eta,
            mu: mu.into_boxed_slice(),
            cdf: cdf.into_boxed_slice(),
            a0: a0.into_boxed_slice(),
            a_offset: a_offset.into_boxed_slice(),
            m: m.into_boxed_slice(),
            a: a.into_boxed_slice(),
            recip: recip.into_boxed_slice(),
        })
    }

    pub fn get_ak(&self, offset_i: usize, offset_o: usize) -> (&[f32], u32) {
        (
            &self.a[self.a_offset[offset_o * self.mu.len() + offset_i] as usize..],
            self.m[offset_o * self.mu.len() + offset_i],
        )
    }

    pub fn get_weights_and_offset(&self, cos_theta: f32, weights: &mut [f32; 4]) -> Option<usize> {
        catmull_rom_weights(&self.mu, cos_theta, weights)
    }
}

#[inline]
fn cos_d_phi(wa: &Vector3f, wb: &Vector3f) -> f32 {
    clamp(
        (wa.x * wb.x + wa.y * wb.y)
            / f32::sqrt((wa.x * wa.x + wa.y * wa.y) * (wb.x * wb.x + wb.y * wb.y)),
        -1.0,
        1.0,
    )
}

#[ignore]
#[test]
fn test_fourier_bsdf_file() {
    let filename = "/home/abusch/code/pbrt-v3-scenes/dragon/bsdfs/ceramic.bsdf";
    let res = FourierBSDFTable::read(filename);
    assert!(res.is_ok());
    let _ = res.unwrap();
}
