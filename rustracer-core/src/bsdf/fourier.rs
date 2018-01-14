use std::path::Path;
use std::f32::consts::PI;

use num::zero;

use {clamp, Point2f, Vector3f};
use bsdf::{BxDF, BxDFType};
use geometry::{cos_theta, sin2_theta};
use interpolation::{catmull_rom_weights, fourier, sample_fourier, sample_catmull_rom_2d};
use material::TransportMode;
use spectrum::Spectrum;

#[derive(Debug, Clone)]
pub struct FourierBSDF {
    bsdf_table: Box<FourierBSDFTable>,
    mode: TransportMode,
}

impl FourierBSDF {}

impl BxDF for FourierBSDF {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        // Find the zenith angle cosines and azimuth difference angle
        let muI = cos_theta(&(-wi));
        let muO = cos_theta(wo);
        let cos_phi = cos_d_phi(&(-wi), wo);

        // Compute Fourier coefficients $a_k$ for $(\mui, \muo)$

        // Determine offsets and weights for $\mui$ and $\muo$
        let mut weightsI = [0.0; 4];
        let mut weightsO = [0.0; 4];
        let maybe_offsetI = self.bsdf_table.get_weights_and_offset(muI, &mut weightsI);
        let maybe_offsetO = self.bsdf_table.get_weights_and_offset(muO, &mut weightsO);
        if maybe_offsetI.is_none() || maybe_offsetO.is_none() {
            return Spectrum::black();
        }
        let offsetI = maybe_offsetI.unwrap_or(0);
        let offsetO = maybe_offsetO.unwrap_or(0);

        // Allocate storage to accumulate _ak_ coefficients
        // TODO use ALLOCA when available
        let mut ak = vec![0.0; (self.bsdf_table.m_max * self.bsdf_table.n_channels) as usize];

        // Accumulate weighted sums of nearby $a_k$ coefficients
        let mut m_max = 0;
        for b in 0..4 {
            for a in 0..4 {
                // Add contribution of _(a, b)_ to $a_k$ values
                let weight = weightsI[a] * weightsO[b];
                if weight != 0.0 {
                    let (ap, m) = self.bsdf_table.get_ak(offsetI + a, offsetO + b);
                    m_max = u32::max(m_max, m);
                    for c in 0..self.bsdf_table.n_channels {
                        for k in 0..m {
                            ak[(c * self.bsdf_table.m_max + k) as usize] +=
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
                1.0 / self.bsdf_table.eta
            } else {
                self.bsdf_table.eta
            };
            scale *= eta * eta;
        }
        if self.bsdf_table.n_channels == 1 {
            Spectrum::from(Y * scale)
        } else {
            // Compute and return RGB colors for tabulated BSDF
            let R = fourier(&ak[1 * self.bsdf_table.m_max as usize..], m_max, cos_phi);
            let B = fourier(&ak[2 * self.bsdf_table.m_max as usize..], m_max, cos_phi);
            let G = 1.39829 * Y - 0.100913 * B - 0.297375 * R;
            Spectrum::rgb(R * scale, G * scale, B * scale).clamp()
        }
    }

    fn sample_f(&self, wo: &Vector3f, u: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        // Sample zenith angle component for _FourierBSDF_
        let muO = cos_theta(wo);
        let (muI, _, pdfMu) = sample_catmull_rom_2d(
            &self.bsdf_table.mu,
            &self.bsdf_table.mu,
            &self.bsdf_table.a0,
            &self.bsdf_table.cdf,
            muO,
            u[1],
        );

        // Compute Fourier coefficients $a_k$ for $(\mui, \muo)$

        // Determine offsets and weights for $\mui$ and $\muo$
        let mut weightsI = [0.0; 4];
        let mut weightsO = [0.0; 4];
        let maybe_offsetI = self.bsdf_table.get_weights_and_offset(muI, &mut weightsI);
        let maybe_offsetO = self.bsdf_table.get_weights_and_offset(muO, &mut weightsO);
        if maybe_offsetI.is_none() || maybe_offsetO.is_none() {
            return (Spectrum::black(), zero(), 0.0, self.get_type());
        }
        let offsetI = maybe_offsetI.unwrap_or(0);
        let offsetO = maybe_offsetO.unwrap_or(0);

        // Allocate storage to accumulate _ak_ coefficients
        // TODO use ALLOCA when available
        let mut ak = vec![0.0; (self.bsdf_table.m_max * self.bsdf_table.n_channels) as usize];

        // Accumulate weighted sums of nearby $a_k$ coefficients
        let mut m_max = 0;
        for b in 0..4 {
            for a in 0..4 {
                // Add contribution of _(a, b)_ to $a_k$ values
                let weight = weightsI[a] * weightsO[b];
                if weight != 0.0 {
                    let (ap, m) = self.bsdf_table.get_ak(offsetI + a, offsetO + b);
                    m_max = u32::max(m_max, m);
                    for c in 0..self.bsdf_table.n_channels {
                        for k in 0..m {
                            ak[(c * self.bsdf_table.m_max + k) as usize] +=
                                weight * ap[(c * m + k) as usize];
                        }
                    }
                }
            }
        }

        // Importance sample the luminance Fourier expansion
        let (Y, pdfPhi, phi) = sample_fourier(&ak, &self.bsdf_table.recip, m_max, u[0]);
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
                1.0 / self.bsdf_table.eta
            } else {
                self.bsdf_table.eta
            };
            scale *= eta * eta;
        }

        if self.bsdf_table.n_channels == 1 {
            return (Spectrum::from(Y * scale), wi, pdf, self.get_type());
        }
        let R = fourier(&ak[(1 * self.bsdf_table.m_max) as usize..], m_max, cosPhi);
        let B = fourier(&ak[(2 * self.bsdf_table.m_max) as usize..], m_max, cosPhi);
        let G = 1.39829 * Y - 0.100913 * B - 0.297375 * R;
        return (
            Spectrum::rgb(R * scale, G * scale, B * scale).clamp(),
            wi,
            pdf,
            self.get_type(),
        );
    }

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> f32 {
        // Find the zenith angle cosines and azimuth difference angle
        let muI = cos_theta(&(-wi));
        let muO = cos_theta(wo);
        let cosPhi = cos_d_phi(&(-wi), wo);

        // Compute luminance Fourier coefficients $a_k$ for $(\mui, \muo)$
        let mut weightsI = [0.0; 4];
        let mut weightsO = [0.0; 4];
        let maybe_offsetI = self.bsdf_table.get_weights_and_offset(muI, &mut weightsI);
        let maybe_offsetO = self.bsdf_table.get_weights_and_offset(muO, &mut weightsO);
        if maybe_offsetI.is_none() || maybe_offsetO.is_none() {
            return 0.0;
        }
        let offsetI = maybe_offsetI.unwrap_or(0);
        let offsetO = maybe_offsetO.unwrap_or(0);

        // Allocate storage to accumulate _ak_ coefficients
        // TODO use ALLOCA when available
        let mut ak = vec![0.0; (self.bsdf_table.m_max * self.bsdf_table.n_channels) as usize];

        let mut mMax = 0;
        for o in 0..4 {
            for i in 0..4 {
                let weight = weightsI[i] * weightsO[o];
                if weight == 0.0 {
                    continue;
                }

                let (coeffs, order) = self.bsdf_table.get_ak(offsetI + i, offsetO + o);
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
                * self.bsdf_table.cdf[(offsetO + o) * self.bsdf_table.n_mu as usize
                                          + self.bsdf_table.n_mu as usize
                                          - 1] * (2.0 * PI);
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
struct FourierBSDFTable {
    eta: f32,
    m_max: u32,
    n_channels: u32,
    n_mu: u32,
    mu: Box<[f32]>,
    m: Box<[u32]>,
    a_offset: Box<[usize]>,
    a: Box<[f32]>,
    a0: Box<[f32]>,
    cdf: Box<[f32]>,
    recip: Box<[f32]>,
}

impl FourierBSDFTable {
    pub fn read<P: AsRef<Path>>(filename: P) -> FourierBSDFTable {
        unimplemented!();
    }

    pub fn get_ak(&self, offset_i: usize, offset_o: usize) -> (&[f32], u32) {
        (
            &self.a[self.a_offset[offset_o * self.mu.len() + offset_i]..],
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
