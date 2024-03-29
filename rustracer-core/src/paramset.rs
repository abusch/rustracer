#![macro_use]

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use log::{error, warn};

use crate::api::{ParamListEntry, ParamType};
use crate::cie::CIE_LAMBDA;
use crate::fileutil::resolve_filename;
use crate::floatfile::read_float_file;
use crate::spectrum::{blackbody_normalized, Spectrum};
use crate::texture::ConstantTexture;
use crate::texture::Texture;
use crate::{Normal3f, Point2f, Point3f, Vector3f};

macro_rules! find_one(
    ($x:ident, $y:ident, $t:ty) => (
        pub fn $x(&self, name: &str, d: $t) -> $t {
            let res = self.$y.iter().find(|ref e| e.name == name);

            if let Some(e) = res.as_ref() {
                e.looked_up.set(true);
            }

            res.map(|e| e.values[0].clone()).unwrap_or(d)
        }
    );
);

macro_rules! find(
    ($x:ident, $y:ident, $t:ty) => (
        pub fn $x(&self, name: &str) -> Option<Vec<$t>> {
            let res = self.$y.iter().find(|ref e| e.name == name);

            if let Some(e) = res.as_ref() {
                e.looked_up.set(true);
            }

            res.map(|e| e.values.clone())
        }
    );
);

#[derive(Default, Debug, Clone)]
pub struct ParamSet {
    bools: Vec<ParamSetItem<bool>>,
    ints: Vec<ParamSetItem<i32>>,
    floats: Vec<ParamSetItem<f32>>,
    strings: Vec<ParamSetItem<String>>,
    spectra: Vec<ParamSetItem<Spectrum>>,
    point2fs: Vec<ParamSetItem<Point2f>>,
    point3fs: Vec<ParamSetItem<Point3f>>,
    vector3fs: Vec<ParamSetItem<Vector3f>>,
    normal3fs: Vec<ParamSetItem<Normal3f>>,
    textures: Vec<ParamSetItem<String>>,
}

impl ParamSet {
    pub fn init(&mut self, entries: Vec<ParamListEntry>) {
        for entry in entries {
            match entry.param_type {
                ParamType::Bool => {
                    let bools = entry
                        .values
                        .as_str_array()
                        .iter()
                        .map(|x| x == "true")
                        .collect();
                    self.add_bool(entry.param_name.clone(), bools);
                }
                ParamType::Int => {
                    let ints = entry
                        .values
                        .as_num_array()
                        .iter()
                        .map(|x| *x as i32)
                        .collect::<Vec<_>>();
                    self.add_int(entry.param_name.clone(), ints);
                }
                ParamType::Float => {
                    self.add_float(entry.param_name.clone(), entry.values.as_num_array())
                }
                ParamType::String => {
                    self.add_string(entry.param_name.clone(), entry.values.as_str_array())
                }
                ParamType::Rgb => {
                    let spectra = entry
                        .values
                        .as_num_array()
                        .chunks(3)
                        .filter(|s| s.len() == 3)
                        .map(|s| Spectrum::rgb(s[0], s[1], s[2]))
                        .collect();
                    self.add_rgb_spectrum(entry.param_name.clone(), spectra);
                }
                ParamType::Point2 => {
                    let points = entry
                        .values
                        .as_num_array()
                        .chunks(2)
                        .filter(|s| s.len() == 2)
                        .map(|s| Point2f::new(s[0], s[1]))
                        .collect();
                    self.add_point2f(entry.param_name.clone(), points);
                }
                ParamType::Point3 => {
                    let points = entry
                        .values
                        .as_num_array()
                        .chunks(3)
                        .filter(|s| s.len() == 3)
                        .map(|s| Point3f::new(s[0], s[1], s[2]))
                        .collect();
                    self.add_point3f(entry.param_name.clone(), points);
                }
                ParamType::Vector3 => {
                    let vectors = entry
                        .values
                        .as_num_array()
                        .chunks(3)
                        .filter(|s| s.len() == 3)
                        .map(|s| Vector3f::new(s[0], s[1], s[2]))
                        .collect();
                    self.add_vector3f(entry.param_name.clone(), vectors);
                }
                ParamType::Normal => {
                    let vectors = entry
                        .values
                        .as_num_array()
                        .chunks(3)
                        .filter(|s| s.len() == 3)
                        .map(|s| Normal3f::new(s[0], s[1], s[2]))
                        .collect();
                    self.add_normal3f(entry.param_name.clone(), vectors);
                }
                ParamType::Texture => {
                    self.add_texture(entry.param_name.clone(), entry.values.as_str_array())
                }
                ParamType::Spectrum => {
                    let filenames = entry.values.as_str_array();
                    self.add_sampled_spectrum_files(entry.param_name.clone(), filenames);
                    // TODO handle case where floats are specified inline
                }
                ParamType::Blackbody => {
                    self.add_blackbody_spectrum(
                        entry.param_name.clone(),
                        &entry.values.as_num_array(),
                    );
                }
                ParamType::Vector2 => error!(
                    "Parameter type {:?} is not implemented yet!",
                    entry.param_type
                ),
                ParamType::Xyz => error!(
                    "Parameter type {:?} is not implemented yet!",
                    entry.param_type
                ),
            }
        }
    }

    pub fn find_one_filename(&self, name: &str, d: String) -> String {
        let filename = self.find_one_string(name, "".to_owned());
        if filename.is_empty() {
            d
        } else {
            resolve_filename(&filename)
        }
    }

    fn add_bool(&mut self, name: String, values: Vec<bool>) {
        self.bools.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_int(&mut self, name: String, values: Vec<i32>) {
        self.ints.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_float(&mut self, name: String, values: Vec<f32>) {
        self.floats.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_string(&mut self, name: String, values: Vec<String>) {
        self.strings.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_rgb_spectrum(&mut self, name: String, values: Vec<Spectrum>) {
        self.spectra.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_point2f(&mut self, name: String, values: Vec<Point2f>) {
        self.point2fs.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_point3f(&mut self, name: String, values: Vec<Point3f>) {
        self.point3fs.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_vector3f(&mut self, name: String, values: Vec<Vector3f>) {
        self.vector3fs.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_normal3f(&mut self, name: String, values: Vec<Normal3f>) {
        self.normal3fs.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_texture(&mut self, name: String, values: Vec<String>) {
        self.textures.push(ParamSetItem {
            name,
            values,
            looked_up: Cell::new(false),
        });
    }

    fn add_sampled_spectrum_files(&mut self, name: String, values: Vec<String>) {
        let mut s = Vec::with_capacity(values.len());
        for filename in values {
            let filename = resolve_filename(&filename);
            match read_float_file(&filename) {
                Err(_) => {
                    warn!(
                        "Unable to read SPD file \"{}\". Using black distribution.",
                        filename
                    );
                    s.push(Spectrum::black());
                }
                Ok(floats) => {
                    let mut wls = Vec::new();
                    let mut v = Vec::new();
                    floats.chunks(2).for_each(|chunk| {
                        if chunk.len() == 2 {
                            wls.push(chunk[0]);
                            v.push(chunk[1]);
                        } else {
                            warn!(
                                "Extra value found in spectrum file \"{}\". Ignoring it.",
                                filename
                            );
                        }
                    });
                    s.push(Spectrum::from_sampled(&wls, &v, wls.len()));
                }
            }
        }
        self.spectra.push(ParamSetItem {
            name,
            values: s,
            looked_up: Cell::new(false),
        });
    }

    fn add_blackbody_spectrum(&mut self, name: String, values: &[f32]) {
        let spectra = values
            .chunks(2)
            .filter(|s| s.len() == 2)
            .map(|v| {
                let temp = v[0];
                let scale = v[1];
                let Le = blackbody_normalized(&CIE_LAMBDA, temp);
                scale * Spectrum::from_sampled(&CIE_LAMBDA, &Le, CIE_LAMBDA.len())
            })
            .collect();

        self.spectra.push(ParamSetItem {
            name,
            values: spectra,
            looked_up: Cell::new(false),
        });
    }

    find!(find_bool, bools, bool);
    find!(find_int, ints, i32);
    find!(find_float, floats, f32);
    find!(find_string, strings, String);
    find!(find_spectrum, spectra, Spectrum);
    find!(find_point2f, point2fs, Point2f);
    find!(find_point3f, point3fs, Point3f);
    find!(find_vector3f, vector3fs, Vector3f);
    find!(find_normal3f, normal3fs, Normal3f);
    find_one!(find_one_bool, bools, bool);
    find_one!(find_one_int, ints, i32);
    find_one!(find_one_float, floats, f32);
    find_one!(find_one_string, strings, String);
    find_one!(find_one_spectrum, spectra, Spectrum);
    find_one!(find_one_point2f, point2fs, Point2f);
    find_one!(find_one_point3f, point3fs, Point3f);
    find_one!(find_one_vector3f, vector3fs, Vector3f);
    find_one!(find_one_normal3f, normal3fs, Normal3f);

    find_one!(find_texture, textures, String);
}

#[derive(Debug, Clone)]
struct ParamSetItem<T: Debug> {
    name: String,
    values: Vec<T>,
    looked_up: Cell<bool>,
}

impl<T: Debug> Default for ParamSetItem<T> {
    fn default() -> Self {
        ParamSetItem {
            name: String::new(),
            values: Vec::new(),
            looked_up: Cell::new(false),
        }
    }
}

pub struct TextureParams<'a> {
    geom_params: &'a ParamSet,
    material_params: &'a ParamSet,
    float_textures: &'a HashMap<String, Arc<dyn Texture<f32>>>,
    spectrum_textures: &'a HashMap<String, Arc<dyn Texture<Spectrum>>>,
}

impl<'a> TextureParams<'a> {
    pub fn new(
        gp: &'a ParamSet,
        mp: &'a ParamSet,
        ft: &'a HashMap<String, Arc<dyn Texture<f32>>>,
        st: &'a HashMap<String, Arc<dyn Texture<Spectrum>>>,
    ) -> TextureParams<'a> {
        TextureParams {
            geom_params: gp,
            material_params: mp,
            float_textures: ft,
            spectrum_textures: st,
        }
    }

    pub fn find_int(&self, n: &str, d: i32) -> i32 {
        let d = self.material_params.find_one_int(n, d);
        self.geom_params.find_one_int(n, d)
    }

    pub fn find_string(&self, n: &str, d: &str) -> String {
        let mat_string = self.material_params.find_one_string(n, d.to_owned());
        self.geom_params.find_one_string(n, mat_string)
    }

    pub fn find_filename(&self, n: &str, d: &str) -> String {
        let mat_string = self.material_params.find_one_filename(n, d.to_owned());
        self.geom_params.find_one_filename(n, mat_string)
    }

    pub fn find_bool(&self, n: &str, d: bool) -> bool {
        let d = self.material_params.find_one_bool(n, d);
        self.geom_params.find_one_bool(n, d)
    }

    pub fn find_float(&self, n: &str, d: f32) -> f32 {
        let d = self.material_params.find_one_float(n, d);
        self.geom_params.find_one_float(n, d)
    }

    pub fn find_vector3f(&self, n: &str, d: Vector3f) -> Vector3f {
        let d = self.material_params.find_one_vector3f(n, d);
        self.geom_params.find_one_vector3f(n, d)
    }

    pub fn find_spectrum(&self, n: &str, d: Spectrum) -> Spectrum {
        let d = self.material_params.find_one_spectrum(n, d);
        self.geom_params.find_one_spectrum(n, d)
    }

    pub fn get_spectrum_texture(&self, n: &str, default: &Spectrum) -> Arc<dyn Texture<Spectrum>> {
        let mut name = self.geom_params.find_texture(n, "".to_owned());
        if name.is_empty() {
            name = self.material_params.find_texture(n, "".to_owned());
        }
        if !name.is_empty() {
            if let Some(tex) = self.spectrum_textures.get(&name) {
                return Arc::clone(tex);
            } else {
                error!(
                    "Couldn't find spectrum texture {} for parameter {}",
                    name, n
                );
            }
        }
        // If texture wasn't found
        let val = self.material_params.find_one_spectrum(n, *default);
        let val = self.geom_params.find_one_spectrum(n, val);
        Arc::new(ConstantTexture::new(val))
    }

    pub fn get_float_texture(&self, n: &str, default: f32) -> Arc<dyn Texture<f32>> {
        let mut name = self.geom_params.find_texture(n, "".to_owned());
        if name.is_empty() {
            name = self.material_params.find_texture(n, "".to_owned());
        }
        if !name.is_empty() {
            if let Some(tex) = self.float_textures.get(&name) {
                return Arc::clone(tex);
            } else {
                error!("Couldn't find float texture {} for parameter {}", name, n);
            }
        }
        // If texture wasn't found
        let val = self.material_params.find_one_float(n, default);
        let val = self.geom_params.find_one_float(n, val);
        Arc::new(ConstantTexture::new(val))
    }

    pub fn get_float_texture_or_none(&self, n: &str) -> Option<Arc<dyn Texture<f32>>> {
        let mut name = self.geom_params.find_texture(n, "".to_owned());
        if name.is_empty() {
            name = self.material_params.find_texture(n, "".to_owned());
        }
        if !name.is_empty() {
            if let Some(tex) = self.float_textures.get(&name) {
                return Some(Arc::clone(tex));
            } else {
                error!("Couldn't find float texture {} for parameter {}", name, n);
                return None;
            }
        }
        // If texture wasn't found
        self.geom_params
            .find_float(n)
            .or_else(|| self.material_params.find_float(n))
            .map(|val| {
                let tex: Arc<dyn Texture<f32>> = Arc::new(ConstantTexture::new(val[0]));
                tex
            })
    }
}
