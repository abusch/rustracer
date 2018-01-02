use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

use ply_rs::ply;
use ply_rs::parser;

use {Normal3f, Point2f, Point3f};
use paramset::ParamSet;
use shapes::Shape;
use shapes::mesh::create_triangle_mesh;
use texture::Texture;
use transform::Transform;

pub fn create(o2w: &Transform,
              _w2o: &Transform,
              reverse_orientation: bool,
              params: &mut ParamSet,
              _float_textures: &HashMap<String, Arc<Texture<f32>>>)
              -> Vec<Arc<Shape>> {
    let filename = params.find_one_filename("filename", "".into());
    let f = File::open(&filename).unwrap();
    let mut f = BufReader::new(f);

    // create a parser
    let vertex_parser = parser::Parser::<Vertex>::new();
    let face_parser = parser::Parser::<Face>::new();

    // use the parser: read the entire file
    let header = vertex_parser.read_header(&mut f).unwrap();
    let mut vertex_count = 0;
    let mut face_count = 0;
    let mut has_normals = false;
    let mut has_texture = false;
    for (key, elem) in &header.elements {
        if key == "vertex" {
            vertex_count = elem.count;
            if !elem.properties.contains_key("x") || !elem.properties.contains_key("y") ||
               !elem.properties.contains_key("z") {
                error!("PLY file \"{}\": Vertex coordinate property not found",
                       filename);
                return Vec::new();
            }
            if elem.properties.contains_key("nx") && elem.properties.contains_key("ny") &&
               elem.properties.contains_key("nz") {
                has_normals = true;
            }
            if (elem.properties.contains_key("u") && elem.properties.contains_key("v")) ||
               (elem.properties.contains_key("s") && elem.properties.contains_key("t")) ||
               (elem.properties.contains_key("texture_u") &&
                elem.properties.contains_key("texture_v")) ||
               (elem.properties.contains_key("texture_s") &&
                elem.properties.contains_key("texture_t")) {
                has_texture = true;
            }
        } else if key == "face" {
            face_count = elem.count;
        }
    }

    if vertex_count == 0 || face_count == 0 {
        error!("PLY file \"{}\" is invalid! No face/vertex elements found!",
               filename);
        return Vec::new();
    } else {
        info!("Loading PLY file with {} vertices and {} faces",
              vertex_count,
              face_count);
    }

    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for (_key, elem) in &header.elements {
        match elem.name.as_ref() {
            "vertex" => {
                vertices = vertex_parser
                    .read_payload_for_element(&mut f, elem, &header)
                    .unwrap();
                // TODO normals + texture
            }
            "face" => {
                faces = face_parser
                    .read_payload_for_element(&mut f, elem, &header)
                    .unwrap();
            }
            _ => panic!("Unexpected element \"{}\"", elem.name),
        }
    }

    let vertex_indices: Vec<usize> = faces.into_iter().flat_map(|f| {
        let length = f.vertex_indices.len();
        if length != 3 && length != 4 {
            warn!("plymesh: Ignoring face with {} vertices (only triangles and quads are supported!", f.vertex_indices.len());
            Vec::new()
        } else {
            let mut vec = Vec::new();
            vec.push(f.vertex_indices[0] as usize);
            vec.push(f.vertex_indices[1] as usize);
            vec.push(f.vertex_indices[2] as usize);
            if length == 4 {
                // If it's a quad, split it into 2 triangles
                vec.push(f.vertex_indices[3] as usize);
                vec.push(f.vertex_indices[0] as usize);
                vec.push(f.vertex_indices[2] as usize);
            }

            vec
        }
    }).collect();

    let mut p = Vec::with_capacity(vertex_count);
    let mut n = Vec::with_capacity(vertex_count);
    let mut uv = Vec::with_capacity(vertex_count);

    for v in vertices {
        p.push(v.p);
        if has_normals {
            n.push(v.n);
        }
        if has_texture {
            uv.push(v.uv);
        }
    }

    create_triangle_mesh(o2w,
                         reverse_orientation,
                         &vertex_indices,
                         &p,
                         None,
                         if has_normals { Some(&n) } else { None },
                         if has_texture { Some(&uv) } else { None })
}

struct Vertex {
    p: Point3f,
    n: Normal3f,
    uv: Point2f,
}

impl ply::PropertyAccess for Vertex {
    fn new() -> Vertex {
        Vertex {
            p: Point3f::default(),
            n: Normal3f::default(),
            uv: Point2f::default(),
        }
    }

    fn set_property(&mut self, key: String, prop: ply::Property) {
        match (key.as_ref(), prop) {
            // point
            ("x", ply::Property::Float(v)) => self.p.x = v,
            ("y", ply::Property::Float(v)) => self.p.y = v,
            ("z", ply::Property::Float(v)) => self.p.z = v,
            // normal
            ("nx", ply::Property::Float(v)) => self.n.x = v,
            ("ny", ply::Property::Float(v)) => self.n.y = v,
            ("nz", ply::Property::Float(v)) => self.n.z = v,
            // texture coordinates
            ("u", ply::Property::Float(v)) |
            ("texture_u", ply::Property::Float(v)) |
            ("s", ply::Property::Float(v)) |
            ("texture_s", ply::Property::Float(v)) => self.uv.x = v,
            ("v", ply::Property::Float(v)) |
            ("t", ply::Property::Float(v)) |
            ("texture_v", ply::Property::Float(v)) |
            ("texture_t", ply::Property::Float(v)) => self.uv.y = v,
            _ => debug!("Unknown property \"{}\" found for vertex element", key),
        }
    }
}

struct Face {
    vertex_indices: Vec<i32>,
}

impl ply::PropertyAccess for Face {
    fn new() -> Face {
        Face { vertex_indices: Vec::new() }
    }

    fn set_property(&mut self, key: String, prop: ply::Property) {
        match (key.as_ref(), prop) {
            ("vertex_indices", ply::Property::ListInt(v)) => self.vertex_indices = v,
            (_k, p) => debug!("Face: Invalid combination key/value for key {} / prop {:?}", key, p),
        }
    }
}
