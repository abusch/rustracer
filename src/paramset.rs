#![macro_use]

use std::fmt::Debug;

use Point3f;
use api::{ParamListEntry, ParamType};
use spectrum::Spectrum;

macro_rules! find_one(
    ($x:ident, $y:ident, $t:ty) => (
        pub fn $x(&mut self, name: &str, d: $t) -> $t {
            let mut res = self.$y.iter_mut().find(|ref mut e| e.name == name);

            if let Some(e) = res.as_mut() {
                e.looked_up = true;
            }

            res.map(|e| e.values[0].clone()).unwrap_or(d)
        }
    );
);

macro_rules! find(
    ($x:ident, $y:ident, $t:ty) => (
        pub fn $x(&mut self, name: &str) -> Option<Vec<$t>> {
            let mut res = self.$y.iter_mut().find(|ref mut e| e.name == name);

            if let Some(e) = res.as_mut() {
                e.looked_up = true;
            }

            res.map(|e| e.values.clone())
        }
    );
);


#[derive(Default, Debug, Clone)]
pub struct ParamSet {
    ints: Vec<ParamSetItem<i32>>,
    floats: Vec<ParamSetItem<f32>>,
    strings: Vec<ParamSetItem<String>>,
    spectra: Vec<ParamSetItem<Spectrum>>,
    point3fs: Vec<ParamSetItem<Point3f>>,
}

impl ParamSet {
    pub fn init(&mut self, entries: Vec<ParamListEntry>) {
        for entry in entries {
            match entry.param_type {
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
                _ => {
                    error!(format!("Parameter type {:?} is not implemented yet!",
                                   entry.param_type))
                }
            }
        }
    }

    fn add_int(&mut self, name: String, values: Vec<i32>) {
        self.ints
            .push(ParamSetItem {
                      name: name,
                      values: values,
                      looked_up: false,
                  });
    }

    fn add_float(&mut self, name: String, values: Vec<f32>) {
        self.floats
            .push(ParamSetItem {
                      name: name,
                      values: values,
                      looked_up: false,
                  });
    }

    fn add_string(&mut self, name: String, values: Vec<String>) {
        self.strings
            .push(ParamSetItem {
                      name: name,
                      values: values,
                      looked_up: false,
                  });
    }

    fn add_rgb_spectrum(&mut self, name: String, values: Vec<Spectrum>) {
        self.spectra
            .push(ParamSetItem {
                      name: name,
                      values: values,
                      looked_up: false,
                  });
    }

    fn add_point3f(&mut self, name: String, values: Vec<Point3f>) {
        self.point3fs
            .push(ParamSetItem {
                      name: name,
                      values: values,
                      looked_up: false,
                  });
    }

    find!(find_int, ints, i32);
    find!(find_float, floats, f32);
    find!(find_string, strings, String);
    find!(find_spectrum, spectra, Spectrum);
    find!(find_point3fs, point3fs, Point3f);
    find_one!(find_one_int, ints, i32);
    find_one!(find_one_float, floats, f32);
    find_one!(find_one_string, strings, String);
    find_one!(find_one_spectrum, spectra, Spectrum);
    find_one!(find_one_point3f, point3fs, Point3f);
}

#[derive(Debug, Clone)]
struct ParamSetItem<T: Debug> {
    name: String,
    values: Vec<T>,
    looked_up: bool,
}

impl<T: Debug> Default for ParamSetItem<T> {
    fn default() -> Self {
        ParamSetItem {
            name: String::new(),
            values: Vec::new(),
            looked_up: false,
        }
    }
}

pub struct TextureParams<'a> {
    geom_params: &'a mut ParamSet,
    material_params: &'a mut ParamSet,
}

impl<'a> TextureParams<'a> {
    pub fn new(gp: &'a mut ParamSet, mp: &'a mut ParamSet) -> TextureParams<'a> {
        TextureParams {
            geom_params: gp,
            material_params: mp,
        }
    }

    pub fn find_string(&mut self, n: &str) -> String {
        let mat_string = self.material_params.find_one_string(n, "".to_owned());
        self.geom_params.find_one_string(n, mat_string)
    }
}
