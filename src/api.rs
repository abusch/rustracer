use std::cell::RefCell;

use na::Similarity3;

use {Transform, Vector3f, Point3f};
use errors::*;
use paramset::ParamSet;

#[derive(Debug, Copy, Clone)]
pub enum ApiState {
    Uninitialized,
    OptionsBlock,
    WorldBlock,
}

impl ApiState {
    pub fn verify_uninitialized(&self) -> Result<()> {
        match *self {
            ApiState::Uninitialized => Ok(()),
            _ => bail!("Api::init() has already been called!"),
        }
    }

    pub fn verify_initialized(&self) -> Result<()> {
        match *self {
            ApiState::Uninitialized => bail!("Api::init() has not been called!"),
            _ => Ok(()),
        }
    }

    pub fn verify_options(&self) -> Result<()> {
        self.verify_initialized()?;
        match *self {
            ApiState::WorldBlock => bail!("Options cannot be set inside world block."),
            _ => Ok(()),
        }
    }

    pub fn verify_world(&self) -> Result<()> {
        self.verify_initialized()?;
        match *self {
            ApiState::OptionsBlock => bail!("Scene description must be inside world block."),
            _ => Ok(()),
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
#[derive(Debug, PartialEq)]
pub enum Array {
    NumArray(Vec<f32>),
    StrArray(Vec<String>),
}

impl Array {
    pub fn as_num_array(&self) -> Vec<f32> {
        // TODO proper error handling
        return match *self {
                   Array::NumArray(ref a) => a.clone(),
                   _ => panic!("Attempted to cast a num array to a String array"),
               };
    }

    // TODO proper error handling
    pub fn as_str_array(&self) -> Vec<String> {
        return match *self {
                   Array::StrArray(ref a) => a.clone(),
                   _ => panic!("Attempted to cast a string array to a num array"),
               };
    }
}

#[derive(Debug)]
pub struct ParamListEntry {
    pub param_type: ParamType,
    pub param_name: String,
    pub values: Array,
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
    cur_transform: Transform,
}

impl Default for State {
    fn default() -> Self {
        State {
            api_state: ApiState::Uninitialized,
            render_options: RenderOptions::default(),
            cur_transform: Transform::default(),
        }
    }
}

pub trait Api {
    fn init(&self) -> Result<()>;

    fn attribute_begin(&self) -> Result<()>;

    fn attribute_end(&self) -> Result<()>;

    fn transform_begin(&self) -> Result<()>;

    fn transform_end(&self) -> Result<()>;

    fn world_begin(&self) -> Result<()>;

    fn world_end(&self) -> Result<()>;

    fn look_at(&self,
               ex: f32,
               ey: f32,
               ez: f32,
               lx: f32,
               ly: f32,
               lz: f32,
               ux: f32,
               uy: f32,
               uz: f32)
               -> Result<()>;

    fn camera(&self, name: String, params: &ParamSet) -> Result<()>;
    fn film(&self, name: String, params: &ParamSet) -> Result<()>;
    fn integrator(&self, name: String, params: &ParamSet) -> Result<()>;
    fn arealightsource(&self, name: String, params: &ParamSet) -> Result<()>;
    fn lightsource(&self, name: String, params: &ParamSet) -> Result<()>;
    fn material(&self, name: String, params: &ParamSet) -> Result<()>;
    fn shape(&self, name: String, params: &ParamSet) -> Result<()>;
    fn rotate(&self, angle: f32, dx: f32, dy: f32, dz: f32) -> Result<()>;
}

#[derive(Default)]
pub struct RealApi {
    state: RefCell<State>,
}

impl Api for RealApi {
    fn init(&self) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.api_state.verify_uninitialized()?;

        state.api_state = ApiState::OptionsBlock;
        Ok(())
    }

    fn attribute_begin(&self) -> Result<()> {
        self.state.borrow().api_state.verify_world()?;

        println!("attribute_begin called");
        Ok(())
    }

    fn attribute_end(&self) -> Result<()> {
        self.state.borrow().api_state.verify_world()?;
        
        println!("attribute_end called");
        Ok(())
    }

    fn transform_begin(&self) -> Result<()> {
        self.state.borrow().api_state.verify_world()?;

        println!("transform_begin called");
        Ok(())
    }

    fn transform_end(&self) -> Result<()> {
        self.state.borrow().api_state.verify_world()?;

        println!("transform_end called");
        Ok(())
    }

    fn world_begin(&self) -> Result<()> {
        self.state.borrow().api_state.verify_options()?;
        println!("world_begin called");
        Ok(())
    }

    fn world_end(&self) -> Result<()> {
        self.state.borrow().api_state.verify_world()?;

        println!("world_end called");
        Ok(())
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
               uz: f32)
               -> Result<()> {
        println!("look_at called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_initialized()?;
        let look_at =
            Transform::from_similarity(&Similarity3::look_at_lh(&Point3f::new(ex, ey, ez),
                                                                &Point3f::new(lx, ly, lz),
                                                                &Vector3f::new(ux, uy, uz),
                                                                1.0));
        state.cur_transform = &state.cur_transform * &look_at;
        Ok(())
    }

    fn camera(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Camera called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        state.render_options.camera_name = name;
        state.render_options.camera_params = params.clone();
        Ok(())
    }

    fn film(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Film called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        state.render_options.filter_name = name;
        state.render_options.filter_params = params.clone();
        Ok(())
    }

    fn integrator(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Integrator called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        state.render_options.integrator_name = name;
        state.render_options.integrator_params = params.clone();
        Ok(())
    }

    fn arealightsource(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Arealightsource called with {} and {:?}", name, params);
        Ok(())
    }

    fn lightsource(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Lightsource called with {} and {:?}", name, params);
        Ok(())
    }

    fn material(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Material called with {} and {:?}", name, params);
        Ok(())
    }

    fn shape(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Shape called with {} and {:?}", name, params);
        Ok(())
    }

    fn rotate(&self, angle: f32, dx: f32, dy: f32, dz: f32) -> Result<()> {
        println!("Rotate called with {} {} {} {}", angle, dx, dy, dz);
        Ok(())
    }
}

/*
#[derive(Default)]
pub struct DummyApi {
    state: RefCell<State>,
}

impl Api for DummyApi {
    fn init(&self) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.api_state = ApiState::OptionsBlock;
        Ok(())
    }

    fn attribute_begin(&self) -> Result<()> {
        self.state.borrow().api_state.verify_options()?;

        println!("attribute_begin called");
        Ok(())
    }

    fn attribute_end(&self) -> Result<()> {
        println!("attribute_end called");
        Ok(())
    }

    fn transform_begin(&self) -> Result<()> {
        self.state.borrow().api_state.verify_options()?;

        println!("transform_begin called");
        Ok(())
    }

    fn transform_end(&self) -> Result<()> {
        println!("transform_end called");
        Ok(())
    }

    fn world_begin(&self) -> Result<()> {
        println!("world_begin called");
        Ok(())
    }

    fn world_end(&self) -> Result<()> {
        println!("world_end called");
        Ok(())
    }

    fn look_at(&self,
               _ex: f32,
               _ey: f32,
               _ez: f32,
               _lx: f32,
               _ly: f32,
               _lz: f32,
               _ux: f32,
               _uy: f32,
               _uz: f32)
               -> Result<()> {
        println!("look_at called");
        Ok(())
    }

    fn camera(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Camera called with {} and {:?}", name, params);
        Ok(())
    }

    fn film(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Film called with {} and {:?}", name, params);
        Ok(())
    }

    fn integrator(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Integrator called with {} and {:?}", name, params);
        Ok(())
    }

    fn arealightsource(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Arealightsource called with {} and {:?}", name, params);
        Ok(())
    }

    fn lightsource(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Lightsource called with {} and {:?}", name, params);
        Ok(())
    }

    fn material(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Material called with {} and {:?}", name, params);
        Ok(())
    }

    fn shape(&self, name: String, params: &ParamSet) -> Result<()> {
        println!("Shape called with {} and {:?}", name, params);
        Ok(())
    }

    fn rotate(&self, angle: f32, dx: f32, dy: f32, dz: f32) -> Result<()> {
        println!("Rotate called with {} {} {} {}", angle, dx, dy, dz);

        Ok(())
    }
}
*/