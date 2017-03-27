#![macro_use]

use std::fmt::Debug;

use api::{ParamListEntry, ParamType};

macro_rules! find_one(
    ($x:ident, $y:ident, $t:ty) => (
        pub fn $x(&mut self, name: &str, d: $t) -> $t {
            let mut res = self.$y.iter_mut().find(|ref mut e| e.name == name);

            if let Some(e) = res.as_mut() {
                e.looked_up = true;
            }

            res.map(|e| e.values[0]).unwrap_or(d)
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


#[derive(Debug, Clone)]
pub struct ParamSet {
    ints: Vec<ParamSetItem<i32>>,
    floats: Vec<ParamSetItem<f32>>,
}

impl ParamSet {
    pub fn init(&mut self, entries: Vec<ParamListEntry>) {
        for entry in entries {
            match entry.param_type {
                ParamType::Int => {
                    let ints = entry.values
                        .as_num_array()
                        .iter()
                        .map(|x| *x as i32)
                        .collect::<Vec<_>>();
                    self.add_int(entry.param_name.clone(), ints);
                }
                ParamType::Float => {
                    self.add_float(entry.param_name.clone(), entry.values.as_num_array())
                }
                _ => {
                    error!(format!("Parameter type {:?} is not implemented yet!",
                                   entry.param_type))
                }
            }
        }
    }

    fn add_int(&mut self, name: String, values: Vec<i32>) {
        self.ints.push(ParamSetItem {
                           name: name,
                           values: values,
                           looked_up: false,
                       });
    }

    fn add_float(&mut self, name: String, values: Vec<f32>) {
        self.floats.push(ParamSetItem {
                             name: name,
                             values: values,
                             looked_up: false,
                         });
    }

    find!(find_int, ints, i32);
    find!(find_float, floats, f32);
    find_one!(find_one_int, ints, i32);
    find_one!(find_one_float, floats, f32);
}

impl Default for ParamSet {
    fn default() -> Self {
        ParamSet {
            ints: Vec::new(),
            floats: Vec::new(),
        }
    }
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
