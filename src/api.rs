use std::cell::RefCell;

#[derive(Debug, Copy, Clone)]
pub enum ApiState {
    Uninitialized,
    OptionsBlock,
    WorldBlock,
}

impl ApiState {
    pub fn verify_initialized(&self) {
        match *self {
            ApiState::Uninitialized => panic!("Api::init() has not been called!"),
            _ => (),
        }
    }

    pub fn verify_options(&self) {
        self.verify_initialized();
        match *self {
            ApiState::WorldBlock => panic!("Options are not allowed in a World block"),
            _ => (),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParamType {
    Int,
    Bool,
    Float,
    Point2,
    Vector2,
    Point3,
    Vector3,
    Normal,
    Rgb,
    Xyz,
    Blackbody,
    Spectrum,
    String,
    Texture,
}

#[derive(Debug)]
pub struct ParamSet {
    pub params: Vec<ParamListEntry>,
}

impl Default for ParamSet {
    fn default() -> Self {
        ParamSet { params: Vec::new() }
    }
}

#[derive(Debug, PartialEq)]
pub enum Array {
    NumArray(Vec<f32>),
    StrArray(Vec<String>),
}

#[derive(Debug)]
pub struct ParamListEntry {
    param_type: ParamType,
    param_name: String,
    values: Array,
}

impl ParamListEntry {
    pub fn new(t: ParamType, name: String, values: Array) -> ParamListEntry {
        ParamListEntry {
            param_type: t,
            param_name: name,
            values: values,
        }
    }
}

#[derive(Debug)]
pub struct TransformSet {}
impl Default for TransformSet {
    fn default() -> Self {
        TransformSet {}
    }
}

#[derive(Debug)]
pub struct RenderOptions {
    transform_start_time: f32,
    transform_end_time: f32,
    filter_name: String,
    filter_params: ParamSet,
    sampler_name: String,
    sampler_params: ParamSet,
    accelerator_name: String,
    accelerator_params: ParamSet,
    integrator_name: String,
    integrator_params: ParamSet,
    camera_name: String,
    camera_params: ParamSet,
    camera_to_world: TransformSet,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            transform_start_time: 0.0,
            transform_end_time: 1.0,
            filter_name: "box".to_owned(),
            filter_params: ParamSet::default(),
            sampler_name: "halton".to_owned(),
            sampler_params: ParamSet::default(),
            accelerator_name: "bvh".to_owned(),
            accelerator_params: ParamSet::default(),
            integrator_name: "path".to_owned(),
            integrator_params: ParamSet::default(),
            camera_name: "perspective".to_owned(),
            camera_params: ParamSet::default(),
            camera_to_world: TransformSet::default(),
        }
    }
}

#[derive(Debug)]
pub struct State {
    api_state: ApiState,
    render_options: RenderOptions,
}

impl Default for State {
    fn default() -> Self {
        State {
            api_state: ApiState::Uninitialized,
            render_options: RenderOptions::default(),
        }
    }
}

pub trait Api {
    fn init(&self);

    fn attribute_begin(&self);

    fn attribute_end(&self);

    fn world_begin(&self);

    fn world_end(&self);

    fn look_at(&self,
               ex: f32,
               ey: f32,
               ez: f32,
               lx: f32,
               ly: f32,
               lz: f32,
               ux: f32,
               uy: f32,
               uz: f32);

    fn camera(&self, name: String, params: &ParamSet);
    fn film(&self, name: String, params: &ParamSet);
    fn integrator(&self, name: String, params: &ParamSet);
    fn arealightsource(&self, name: String, params: &ParamSet);
    fn lightsource(&self, name: String, params: &ParamSet);
    fn material(&self, name: String, params: &ParamSet);
    fn shape(&self, name: String, params: &ParamSet);
    fn rotate(&self, angle: f32, dx: f32, dy: f32, dz: f32);
}

#[derive(Default)]
pub struct DummyApi {
    state: RefCell<State>,
}

impl Api for DummyApi {
    fn init(&self) {
        let mut state = self.state.borrow_mut();
        state.api_state = ApiState::OptionsBlock;
    }

    fn attribute_begin(&self) {
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options();

        println!("attribute_begin called");
    }

    fn attribute_end(&self) {
        println!("attribute_end called");
    }

    fn world_begin(&self) {
        println!("world_begin called");
    }

    fn world_end(&self) {
        println!("world_end called");
    }

    fn look_at(&self,
               ex: f32,
               ey: f32,
               ez: f32,
               lx: f32,
               ly: f32,
               lz: f32,
               ux: f32,
               uy: f32,
               uz: f32) {
        println!("look_at called");
    }

    fn camera(&self, name: String, params: &ParamSet) {
        println!("Camera called with {} and {:?}", name, params);
    }

    fn film(&self, name: String, params: &ParamSet) {
        println!("Film called with {} and {:?}", name, params);
    }

    fn integrator(&self, name: String, params: &ParamSet) {
        println!("Integrator called with {} and {:?}", name, params);
    }

    fn arealightsource(&self, name: String, params: &ParamSet) {
        println!("Arealightsource called with {} and {:?}", name, params);
    }

    fn lightsource(&self, name: String, params: &ParamSet) {
        println!("Lightsource called with {} and {:?}", name, params);
    }

    fn material(&self, name: String, params: &ParamSet) {
        println!("Material called with {} and {:?}", name, params);
    }

    fn shape(&self, name: String, params: &ParamSet) {
        println!("Shape called with {} and {:?}", name, params);
    }

    fn rotate(&self, angle: f32, dx: f32, dy: f32, dz: f32) {
        println!("Rotate called with {} {} {} {}", angle, dx, dy, dz);
    }
}
