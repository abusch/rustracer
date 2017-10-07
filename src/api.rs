use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use na::{Matrix4, Similarity3};

use {Point3f, Transform, Vector3f};
use bvh::BVH;
use camera::{Camera, PerspectiveCamera};
use display::NoopDisplayUpdater;
use errors::*;
use filter::Filter;
use filter::boxfilter::BoxFilter;
use filter::gaussian::GaussianFilter;
use film::Film;
use light::{AreaLight, DiffuseAreaLight, DistantLight, InfiniteAreaLight, Light, PointLight};
use integrator::{DirectLightingIntegrator, Normal, PathIntegrator, SamplerIntegrator, Whitted};
use material::Material;
use material::matte::MatteMaterial;
use material::plastic::Plastic;
use paramset::{ParamSet, TextureParams};
use primitive::{GeometricPrimitive, Primitive};
use renderer;
use sampler::Sampler;
use sampler::zerotwosequence::ZeroTwoSequence;
use scene::Scene;
use shapes::Shape;
use shapes::sphere::Sphere;
use shapes::disk::Disk;
use shapes::mesh::TriangleMesh;
use spectrum::Spectrum;
use texture::Texture;

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

impl Default for ApiState {
    fn default() -> Self {
        ApiState::Uninitialized
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

pub struct RenderOptions {
    transform_start_time: f32,
    transform_end_time: f32,
    film_name: String,
    film_params: ParamSet,
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
    camera_to_world: Transform,
    lights: Vec<Arc<Light + Send + Sync>>,
    primitives: Vec<Box<Primitive + Send + Sync>>,
}

impl RenderOptions {
    pub fn make_filter(&mut self) -> Result<Box<Filter + Send + Sync>> {
        info!("Making filter");
        let filter = match self.filter_name.as_ref() {
            "box" => BoxFilter::create(&mut self.filter_params),
            "gaussian" => GaussianFilter::create(&mut self.film_params),
            _ => bail!(format!("Filter \"{}\" unknown.", self.filter_name)),
        };

        Ok(filter)
    }

    pub fn make_film(&mut self, filter: Box<Filter + Send + Sync>) -> Result<Box<Film>> {
        info!("Making film");
        let film = if self.film_name == "image" {
            Film::create(&mut self.film_params, filter)
        } else {
            bail!(format!("Film \"{}\" unknown.", self.film_name));
        };

        Ok(film)
    }

    pub fn make_sampler(&mut self) -> Result<Box<Sampler + Send + Sync>> {
        let sampler = if self.sampler_name == "lowdiscrepancy" || self.sampler_name == "02sequence"
        {
            ZeroTwoSequence::create(&mut self.sampler_params)
        } else {
            bail!(format!("Sampler \"{}\" unknown.", self.sampler_name));
        };

        Ok(sampler)
    }

    pub fn make_camera(&mut self) -> Result<Box<Camera + Send + Sync>> {
        info!("Making camera");
        let filter = self.make_filter()?;
        let film = self.make_film(filter)?;

        let camera = if self.camera_name == "perspective" {
            PerspectiveCamera::create(&mut self.camera_params, &self.camera_to_world, film)
        } else {
            bail!("Camera \"{}\" unknown.", self.camera_name);
        };

        Ok(camera)
    }

    pub fn make_integrator(&mut self) -> Result<Box<SamplerIntegrator + Send + Sync>> {
        info!("Making integrator");
        let integrator = if self.integrator_name == "whitted" {
            Whitted::create(&mut self.integrator_params)
        // Box::new(Normal {})
        } else if self.integrator_name == "directlighting" {
            DirectLightingIntegrator::create(&mut self.integrator_params)
        } else if self.integrator_name == "path" {
            unimplemented!();
        } else if self.integrator_name == "normal" {
            Box::new(Normal {})
        } else {
            bail!(format!("Integrator \"{}\" unknown.", self.integrator_name));
        };

        Ok(integrator)
    }

    pub fn make_scene(&mut self) -> Result<Box<Scene>> {
        info!("Making scene");
        let accelerator = Arc::new(BVH::new(1, &mut self.primitives));
        Ok(Box::new(Scene::new(accelerator, self.lights.clone())))
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            transform_start_time: 0.0,
            transform_end_time: 1.0,
            film_name: "image".to_owned(),
            film_params: ParamSet::default(),
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
            camera_to_world: Transform::default(),
            lights: Vec::new(),
            primitives: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct GraphicsState {
    float_textures: HashMap<String, Arc<Texture<f32> + Send + Sync>>,
    spectrum_textures: HashMap<String, Arc<Texture<Spectrum> + Send + Sync>>,
    material_param: ParamSet,
    material: String,
    named_material: HashMap<String, Arc<Material + Send + Sync>>,
    current_named_material: String,
    area_light_params: ParamSet,
    area_light: String,
    reverse_orientation: bool,
}

impl GraphicsState {
    pub fn create_material(&mut self, params: &mut ParamSet) -> Arc<Material + Send + Sync> {
        let mut mp = TextureParams::new(
            params,
            &mut self.material_param,
            &self.float_textures,
            &self.spectrum_textures,
        );
        if !self.current_named_material.is_empty() {
            self.named_material
                .get(&self.current_named_material)
                .map(|v| v.clone())
                .unwrap_or_else(|| make_material("matte", &mut mp))
        } else {
            make_material(&self.material, &mut mp)
        }
    }
}

impl Default for GraphicsState {
    fn default() -> Self {
        GraphicsState {
            float_textures: HashMap::new(),
            spectrum_textures: HashMap::new(),
            material_param: ParamSet::default(),
            material: "matte".to_owned(),
            named_material: HashMap::new(),
            current_named_material: String::new(),
            area_light_params: ParamSet::default(),
            area_light: String::new(),
            reverse_orientation: false,
        }
    }
}

#[derive(Default)]
pub struct State {
    api_state: ApiState,
    render_options: RenderOptions,
    cur_transform: Transform,
    pushed_transforms: Vec<Transform>,
    graphics_state: GraphicsState,
    pushed_graphics_states: Vec<GraphicsState>,
}

impl State {
    pub fn save_graphics_state(&mut self) {
        let gs = self.graphics_state.clone();
        self.pushed_graphics_states.push(gs);
    }

    pub fn save_transform(&mut self) {
        let t = self.cur_transform.clone();
        self.pushed_transforms.push(t);
    }

    pub fn restore_graphics_state(&mut self) {
        self.graphics_state = self.pushed_graphics_states.pop().unwrap();
    }

    pub fn restore_transform(&mut self) {
        self.cur_transform = self.pushed_transforms.pop().unwrap();
    }
}

pub trait Api {
    fn init(&self) -> Result<()>;
    // TODO cleanup
    fn identity(&self) -> Result<()>;
    fn translate(&self, dx: f32, dy: f32, dz: f32) -> Result<()>;
    fn rotate(&self, angle: f32, dx: f32, dy: f32, dz: f32) -> Result<()>;
    fn scale(&self, sx: f32, sy: f32, sz: f32) -> Result<()>;
    fn look_at(
        &self,
        ex: f32,
        ey: f32,
        ez: f32,
        lx: f32,
        ly: f32,
        lz: f32,
        ux: f32,
        uy: f32,
        uz: f32,
    ) -> Result<()>;
    // TODO concat_Transform
    fn transform(
        &self,
        tr00: f32,
        tr01: f32,
        tr02: f32,
        tr03: f32,
        tr04: f32,
        tr05: f32,
        tr06: f32,
        tr07: f32,
        tr08: f32,
        tr09: f32,
        tr10: f32,
        tr11: f32,
        tr12: f32,
        tr13: f32,
        tr14: f32,
        tr15: f32,
    ) -> Result<()>;
    // TODO coordinate_system
    // TODO coordinate_sys_transform
    // TODO active_transform_all
    // TODO active_transform_end_time
    // TODO active_transform_start_time
    // TODO transform_times
    fn pixel_filter(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn film(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn sampler(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn accelerator(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn integrator(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn camera(&self, name: String, params: &mut ParamSet) -> Result<()>;
    // TODO make_named_medium
    // TODO medium_interface
    fn world_begin(&self) -> Result<()>;
    fn attribute_begin(&self) -> Result<()>;
    fn attribute_end(&self) -> Result<()>;
    fn transform_begin(&self) -> Result<()>;
    fn transform_end(&self) -> Result<()>;
    // TODO texture
    fn material(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn make_named_material(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn named_material(&self, name: String) -> Result<()>;
    fn lightsource(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn arealightsource(&self, name: String, params: &mut ParamSet) -> Result<()>;
    fn shape(&self, name: String, params: &mut ParamSet) -> Result<()>;
    // TODO reverse_orientation
    // TODO object_begin
    // TODO object_end
    // TODO object_instance
    fn world_end(&self) -> Result<()>;
}

#[derive(Default)]
pub struct RealApi {
    state: RefCell<State>,
}

impl RealApi {
    fn make_light(
        &self,
        name: &str,
        param_set: &mut ParamSet,
        light_2_world: &Transform,
    ) -> Result<Arc<Light + Send + Sync>> {
        if name == "point" {
            let light = PointLight::create(light_2_world, param_set);
            Ok(light)
        } else if name == "distant" {
            let light = DistantLight::create(light_2_world, param_set);
            Ok(light)
        } else if name == "infinite" {
            let light = InfiniteAreaLight::create(light_2_world, param_set);
            Ok(light)
        } else {
            warn!("Light {} unknown", name);
            bail!("Unsupported light type");
        }
    }
}

impl Api for RealApi {
    fn init(&self) -> Result<()> {
        info!("API initialized!");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_uninitialized()?;

        state.api_state = ApiState::OptionsBlock;
        Ok(())
    }

    fn identity(&self) -> Result<()> {
        info!("Identity called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_initialized()?;
        state.cur_transform = Transform::default();
        Ok(())
    }

    fn translate(&self, dx: f32, dy: f32, dz: f32) -> Result<()> {
        info!("Translate called with {} {} {}", dx, dy, dz);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_initialized()?;
        let t = Transform::translate(Vector3f::new(dx, dy, dz));
        state.cur_transform = &state.cur_transform * &t;
        Ok(())
    }

    fn rotate(&self, angle: f32, dx: f32, dy: f32, dz: f32) -> Result<()> {
        info!("Rotate called with {} {} {} {}", angle, dx, dy, dz);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_initialized()?;
        let t = Transform::rotate(angle, Vector3f::new(dx, dy, dz));
        state.cur_transform = &state.cur_transform * &t;
        Ok(())
    }

    fn scale(&self, sx: f32, sy: f32, sz: f32) -> Result<()> {
        info!("Scale called with {} {} {}", sx, sy, sz);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_initialized()?;
        let t = Transform::scale(sx, sy, sz);
        state.cur_transform = &state.cur_transform * &t;
        Ok(())
    }

    fn transform(
        &self,
        tr00: f32,
        tr01: f32,
        tr02: f32,
        tr03: f32,
        tr04: f32,
        tr05: f32,
        tr06: f32,
        tr07: f32,
        tr08: f32,
        tr09: f32,
        tr10: f32,
        tr11: f32,
        tr12: f32,
        tr13: f32,
        tr14: f32,
        tr15: f32,
    ) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.api_state.verify_initialized()?;
        let mat = Matrix4::new(
            tr00,
            tr04,
            tr08,
            tr12,
            tr01,
            tr05,
            tr09,
            tr13,
            tr02,
            tr06,
            tr10,
            tr14,
            tr03,
            tr07,
            tr11,
            tr15,
        );
        state.cur_transform = Transform {
            m: mat,
            m_inv: mat.try_inverse().expect("Non invertible matrix"),
        };
        Ok(())
    }

    fn look_at(
        &self,
        ex: f32,
        ey: f32,
        ez: f32,
        lx: f32,
        ly: f32,
        lz: f32,
        ux: f32,
        uy: f32,
        uz: f32,
    ) -> Result<()> {
        info!("look_at called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_initialized()?;
        let look_at = Transform::from_similarity(&Similarity3::look_at_lh(
            &Point3f::new(ex, ey, ez),
            &Point3f::new(lx, ly, lz),
            &Vector3f::new(ux, uy, uz),
            1.0,
        ));
        state.cur_transform = &state.cur_transform * &look_at;
        Ok(())
    }

    fn pixel_filter(&self, name: String, params: &mut ParamSet) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        info!("pixel_filter called");
        state.render_options.filter_name = name;
        state.render_options.filter_params = params.clone();
        Ok(())
    }

    fn film(&self, name: String, params: &mut ParamSet) -> Result<()> {
        info!("Film called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        state.render_options.film_name = name;
        state.render_options.film_params = params.clone();
        Ok(())
    }

    fn sampler(&self, name: String, params: &mut ParamSet) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        info!("sampler called");
        state.render_options.sampler_name = name;
        state.render_options.sampler_params = params.clone();
        Ok(())
    }

    fn accelerator(&self, name: String, params: &mut ParamSet) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        info!("accelerator called");
        state.render_options.accelerator_name = name;
        state.render_options.accelerator_params = params.clone();
        Ok(())
    }

    fn integrator(&self, name: String, params: &mut ParamSet) -> Result<()> {
        info!("Integrator called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        state.render_options.integrator_name = name;
        state.render_options.integrator_params = params.clone();
        Ok(())
    }

    fn camera(&self, name: String, params: &mut ParamSet) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        info!("Camera called with {} and {:?}", name, params);
        state.render_options.camera_name = name;
        state.render_options.camera_params = params.clone();
        state.render_options.camera_to_world = state.cur_transform.inverse();
        // TODO named coordinate system
        Ok(())
    }

    fn world_begin(&self) -> Result<()> {
        info!("world_begin called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_options()?;
        state.api_state = ApiState::WorldBlock;
        state.cur_transform = Transform::default();
        Ok(())
    }

    fn attribute_begin(&self) -> Result<()> {
        info!("attribute_begin called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_world()?;
        state.save_graphics_state();
        state.save_transform();

        Ok(())
    }

    fn attribute_end(&self) -> Result<()> {
        info!("attribute_end called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_world()?;
        if state.pushed_graphics_states.is_empty() {
            error!("Unmatched AttributeEnd encountered. Ignoring it.");
            return Ok(());
        }
        state.restore_graphics_state();
        state.restore_transform();

        Ok(())
    }

    fn transform_begin(&self) -> Result<()> {
        info!("transform_begin called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_world()?;
        state.save_transform();

        Ok(())
    }

    fn transform_end(&self) -> Result<()> {
        info!("transform_end called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_world()?;
        if state.pushed_transforms.is_empty() {
            error!("Unmatched TransformEnd encountered. Ignoring it.");
            return Ok(());
        }
        state.restore_transform();

        Ok(())
    }

    fn make_named_material(&self, name: String, params: &mut ParamSet) -> Result<()> {
        info!("MakeNamedMaterial called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();

        let mtl = {
            let mut empty_params = ParamSet::default();
            let mut mp = TextureParams::new(
                params,
                &mut empty_params,
                &state.graphics_state.float_textures,
                &state.graphics_state.spectrum_textures,
            );

            let mat_name = mp.find_string("type");
            if mat_name == "" {
                bail!("No parameter string \"type\" found in named_material");
            }
            make_material(&mat_name, &mut mp)
        };
        if state
            .graphics_state
            .named_material
            .insert(name.clone(), mtl)
            .is_some()
        {
            warn!("Named material {} redefined", name);
        }
        Ok(())
    }

    fn material(&self, name: String, params: &mut ParamSet) -> Result<()> {
        info!("Material called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.graphics_state.material = name;
        state.graphics_state.material_param = params.clone();
        state.graphics_state.current_named_material = String::new();
        Ok(())
    }

    fn named_material(&self, name: String) -> Result<()> {
        info!("NamedMaterial called with {}", name);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_world()?;
        state.graphics_state.current_named_material = name;
        Ok(())
    }

    fn lightsource(&self, name: String, params: &mut ParamSet) -> Result<()> {
        info!("Lightsource called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_world()?;
        let lt = self.make_light(&name, params, &state.cur_transform)?;
        state.render_options.lights.push(lt);
        Ok(())
    }

    fn arealightsource(&self, name: String, params: &mut ParamSet) -> Result<()> {
        info!("Arealightsource called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.graphics_state.area_light = name;
        state.graphics_state.area_light_params = params.clone();
        Ok(())
    }

    fn shape(&self, name: String, params: &mut ParamSet) -> Result<()> {
        info!("Shape called with {} and {:?}", name, params);
        let mut state = self.state.borrow_mut();
        state.api_state.verify_world()?;

        let mut prims: Vec<Box<Primitive + Send + Sync>> = Vec::new();
        let mut area_lights: Vec<Arc<Light + Send + Sync>> = Vec::new();
        let shapes = make_shapes(
            &name,
            &state.cur_transform,
            &state.cur_transform.inverse(),
            state.graphics_state.reverse_orientation,
            params,
            &state.graphics_state,
        );
        let mat = if !shapes.is_empty() {
            Some(state.graphics_state.create_material(params))
        } else {
            None
        };
        for s in shapes {
            let area = if state.graphics_state.area_light != "" {
                let mut ps = state.graphics_state.area_light_params.clone();
                let (area_light, light) = make_area_light(
                    &state.graphics_state.area_light,
                    &state.cur_transform,
                    &mut ps,
                    s.clone(),
                )?;
                area_lights.push(light);
                Some(area_light)
            } else {
                None
            };
            let prim: Box<Primitive + Send + Sync> = Box::new(GeometricPrimitive {
                shape: s,
                area_light: area,
                material: mat.clone(),
            });
            prims.push(prim);
        }
        state.render_options.primitives.append(&mut prims);
        state.render_options.lights.append(&mut area_lights);
        Ok(())
    }

    fn world_end(&self) -> Result<()> {
        info!("world_end called");
        let mut state = self.state.borrow_mut();
        state.api_state.verify_world()?;

        while !state.pushed_graphics_states.is_empty() {
            warn!("Missing AttributeEnd");
            let _ = state.pushed_graphics_states.pop();
            let _ = state.pushed_transforms.pop();
        }
        while !state.pushed_transforms.is_empty() {
            warn!("Missing TransformEnd!");
            let _ = state.pushed_transforms.pop();
        }

        let mut integrator = state.render_options.make_integrator()?;
        let mut sampler = state.render_options.make_sampler()?;
        let scene = state.render_options.make_scene()?;
        let camera = state.render_options.make_camera()?;

        // TODO finish
        let stats = renderer::render(
            scene,
            &mut integrator,
            camera,
            7,
            &mut sampler,
            16,
            Box::new(NoopDisplayUpdater {}),
        )?;
        println!("{:?}", stats);

        Ok(())
    }
}

fn make_shapes(
    name: &str,
    object2world: &Transform,
    world2object: &Transform,
    reverse_orientation: bool,
    ps: &mut ParamSet,
    graphics_state: &GraphicsState,
) -> Vec<Arc<Shape + Send + Sync>> {
    let mut shapes: Vec<Arc<Shape + Send + Sync>> = Vec::new();
    if name == "sphere" {
        shapes.push(Sphere::create(object2world, reverse_orientation, ps));
    } else if name == "cylinder" {
        unimplemented!();
    } else if name == "disk" {
        shapes.push(Disk::create(object2world, reverse_orientation, ps));
    } else if name == "cone" {
        unimplemented!();
    } else if name == "paraboloid" {
        unimplemented!();
    } else if name == "hyperboloid" {
        unimplemented!();
    } else if name == "curve" {
        unimplemented!();
    } else if name == "trianglemesh" {
        let mut tris = TriangleMesh::create(
            object2world,
            world2object,
            reverse_orientation,
            ps,
            &graphics_state.float_textures,
        );
        shapes.append(&mut tris);
    } else {
        warn!("Unknown shape {}", name);
    }

    shapes
}

fn make_material(name: &str, mp: &mut TextureParams) -> Arc<Material + Send + Sync> {
    if name == "matte" {
        MatteMaterial::create(mp)
    } else if name == "plastic" {
        Plastic::create(mp)
    } else {
        warn!("Unknown material {}. Using matte.", name);
        MatteMaterial::create(mp)
    }
}

fn make_area_light(
    name: &str,
    light2world: &Transform,
    params: &mut ParamSet,
    shape: Arc<Shape + Send + Sync>,
) -> Result<(Arc<AreaLight + Send + Sync>, Arc<Light + Send + Sync>)> {
    if name == "area" || name == "diffuse" {
        let l = DiffuseAreaLight::create(light2world, params, shape);
        let light: Arc<Light + Send + Sync> = l.clone();
        let area_light: Arc<AreaLight + Send + Sync> = l.clone();
        Ok((area_light, light))
    } else {
        bail!("Area light {} unknown", name);
    }
}
