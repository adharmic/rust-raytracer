use std::{
    fs::{self, File},
    ops::Deref,
    path::PathBuf,
};

use clap::Parser;
use glam::{Vec3, Vec4};
use image::{DynamicImage, ImageReader};
use serde_json::Value;
use wavefront_obj::obj::{self, ObjSet, Primitive};

use crate::structures::{AABB, Color, Light, Material, Scene, Sphere, Traceable, Triangle};

#[derive(Parser)]
struct Arguments {
    #[arg(short, long)]
    scene: Option<PathBuf>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    background: Option<String>,
}

pub struct ExecutionContext {
    pub scene: Scene,
    pub output: String,
}

pub fn initialize() -> ExecutionContext {
    let arguments = Arguments::parse();

    let mut scene: Scene;
    match arguments.scene.as_deref() {
        Some(scene_file_path) => scene = parse_scene_json(scene_file_path),
        None => scene = default_scene(),
    }

    let background: DynamicImage;
    match arguments.background.as_deref() {
        Some(background_file_path) => background = load_background(background_file_path),
        None => background = load_background("envmap.jpg"),
    }
    scene.background = Some(background);

    let output: String;
    match arguments.output.as_deref() {
        Some(output_file_path) => output = String::from(output_file_path),
        None => output = String::from("out.png"),
    }

    ExecutionContext { scene, output }
}

fn parse_scene_json(scene_file_path: &std::path::Path) -> Scene {
    // TODO: If any errors while trying to read the scene JSON, return the default scene.
    // replace expects with unwrap_or and handle more gracefully
    let scene_json = File::open(scene_file_path).expect("Scene file not found!");
    let raw_data: Value =
        serde_json::from_reader(scene_json).expect("Scene file is not valid JSON.");
    let lights: Vec<Light> = serde_json::from_value(raw_data["lights"].clone())
        .expect("Scene file does not contain any lights.");
    let objects_json: Vec<Value> = serde_json::from_value(raw_data["objects"].clone())
        .expect("Scene file does not contain any objects.");

    let mut objects: Vec<Box<dyn Traceable>> = Vec::new();
    for object in objects_json {
        let object_name: String = serde_json::from_value(object["name"].clone()).unwrap();
        match object_name.deref() {
            "sphere" => {
                let radius: f32 = serde_json::from_value(object["radius"].clone()).unwrap();
                let center: Vec3 = serde_json::from_value(object["center"].clone()).unwrap();
                let material: Material =
                    serde_json::from_value(object["material"].clone()).unwrap();
                objects.push(Box::new(Sphere {
                    center,
                    radius,
                    material,
                }));
            }
            "box" => {
                let min: Vec3 = serde_json::from_value(object["min"].clone()).unwrap();
                let max: Vec3 = serde_json::from_value(object["max"].clone()).unwrap();
                let material: Material =
                    serde_json::from_value(object["material"].clone()).unwrap();
                objects.push(Box::new(AABB { min, max, material }));
            }
            _ => {
                println!("Unknown object: {}", object_name)
            }
        }
    }

    return Scene {
        lights,
        objects,
        background: None,
    };
}

fn load_background(background: &str) -> DynamicImage {
    // TODO: Fallback to using background color and provide a warning instead of panicking if image is not found.
    // Should be simple. Just handle errors and return None.
    return ImageReader::open(background)
        .expect("Environment map not found!")
        .decode()
        .expect("Unable to decode environment map. Is the file intact?");
}

fn default_scene() -> Scene {
    let mut objects: Vec<Box<dyn Traceable>> = Vec::new();
    let mut lights = Vec::new();
    let _ivory = Material {
        diffuse_color: Color {
            r: 102,
            g: 102,
            b: 76,
        },
        albedo: Vec4::new(0.6, 0.3, 0.05, 0.0),
        specular_exponent: 50f32,
        refractive_index: 1f32,
    };
    let red = Material {
        diffuse_color: Color {
            r: 76,
            g: 25,
            b: 25,
        },
        albedo: Vec4::new(0.9, 0.1, 0.0, 0.0),
        specular_exponent: 10f32,
        refractive_index: 1f32,
    };
    let _mirror = Material {
        diffuse_color: Color {
            r: 255,
            g: 255,
            b: 255,
        },
        albedo: Vec4::new(0.0, 10.0, 0.8, 0.0),
        specular_exponent: 1425f32,
        refractive_index: 1f32,
    };
    let _glass = Material {
        diffuse_color: Color {
            r: 150,
            g: 175,
            b: 200,
        },
        albedo: Vec4::new(0.0, 0.5, 0.1, 0.8),
        specular_exponent: 125f32,
        refractive_index: 1.5f32,
    };
    // objects.push(Box::new(Sphere {
    //     center: Vec3::new(-3f32, 0f32, -8f32),
    //     radius: 2f32,
    //     material: ivory,
    // }));
    // objects.push(Box::new(Sphere {
    //     center: Vec3::new(-1f32, -1.5f32, -6f32),
    //     radius: 2f32,
    //     material: ivory,
    // }));
    // objects.push(Box::new(Sphere {
    //     center: Vec3::new(1.5f32, -0.5f32, -9f32),
    //     radius: 2f32,
    //     material: red,
    // }));
    // objects.push(Box::new(Sphere {
    //     center: Vec3::new(-1f32, 3.5f32, -7f32),
    //     radius: 2f32,
    //     material: red,
    // }));
    // objects.push(Box::new(Sphere {
    //     center: Vec3::new(3f32, -3f32, -6f32),
    //     radius: 2f32,
    //     material: mirror,
    // }));
    // objects.push(Box::new(Sphere {
    //     center: Vec3::new(-3f32, 3f32, -5f32),
    //     radius: 2f32,
    //     material: glass,
    // }));

    // objects.push(Box::new(Triangle {
    //     v0: Vec3::new(0f32, 3f32, -8f32),
    //     v1: Vec3::new(-3f32, 0f32, -8f32),
    //     v2: Vec3::new(3f32, 0f32, -8f32),
    //     material: mirror,
    // }));

    let model_read = fs::read_to_string("model.obj").unwrap();
    let model = obj::parse(model_read).unwrap();
    let dragon_read = fs::read_to_string("dragon.obj").unwrap();
    let dragon_model = obj::parse(dragon_read).unwrap();

    add_triangulated_mesh(model, &mut objects, red, Vec3::new(2f32, 0f32, 2f32));
    add_triangulated_mesh(
        dragon_model,
        &mut objects,
        red,
        Vec3::new(-2f32, 0f32, 2f32),
    );

    lights.push(Light {
        position: Vec3::new(5f32, 5f32, -2f32),
        intensity: 1.5,
    });

    return Scene {
        lights,
        objects,
        background: None,
    };
}

pub fn add_triangulated_mesh(
    model: ObjSet,
    objects: &mut Vec<Box<dyn Traceable>>,
    default_mat: Material,
    offset: Vec3,
) {
    model.objects.iter().for_each(|object| {
        object.geometry.iter().for_each(|geo| {
            geo.shapes.iter().for_each(|shape| {
                if let Primitive::Triangle(x, y, z) = shape.primitive {
                    objects.push(Box::new(Triangle {
                        v0: Vec3::new(
                            object.vertices[x.0].x as f32 - offset.x,
                            object.vertices[x.0].y as f32 - offset.y,
                            object.vertices[x.0].z as f32 - offset.z,
                        ),
                        v1: Vec3::new(
                            object.vertices[y.0].x as f32 - offset.x,
                            object.vertices[y.0].y as f32 - offset.y,
                            object.vertices[y.0].z as f32 - offset.z,
                        ),
                        v2: Vec3::new(
                            object.vertices[z.0].x as f32 - offset.x,
                            object.vertices[z.0].y as f32 - offset.y,
                            object.vertices[z.0].z as f32 - offset.z,
                        ),
                        material: default_mat,
                    }));
                }
            });
        });
    });
}
