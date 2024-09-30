mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod light;
mod material;
mod cube; 

use minifb::{ Window, WindowOptions, Key };
use nalgebra_glm::{Vec3, normalize};
use std::time::Duration;
use std::f32::consts::PI;

use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;
use crate::cube::Cube;

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

pub fn cast_ray<T: RayIntersect>(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    object: &T,  
    light: &Light,
    depth: u32,
    skybox: &Skybox,
) -> Color {
    let mut intersect = object.ray_intersect(ray_origin, ray_direction);
    if !intersect.is_intersecting {
        return skybox.sample(*ray_direction);
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity;

    let ambient = intersect.material.diffuse * 0.2; 

    diffuse + specular + ambient
}


pub fn render(
    framebuffer: &mut Framebuffer,
    plane: &Plane,
    cubes: &[Cube],  
    camera: &Camera,
    light: &Light,
    skybox: &Skybox,
) {
    let aspect_ratio = framebuffer.width as f32 / framebuffer.height as f32;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / framebuffer.width as f32 - 1.0;
            let screen_y = -(2.0 * y as f32) / framebuffer.height as f32 + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.base_change(&ray_direction);

            
            let mut pixel_color = if plane.ray_intersect(&camera.eye, &rotated_direction).is_intersecting {
                cast_ray(&camera.eye, &rotated_direction, plane, light, 0, skybox)
            } else {
                skybox.sample(rotated_direction)  
            };

            
            let mut nearest_intersection = f32::INFINITY;
            for cube in cubes {
                let intersect = cube.ray_intersect(&camera.eye, &rotated_direction);
                if intersect.is_intersecting && intersect.distance < nearest_intersection {
                    nearest_intersection = intersect.distance;
                    pixel_color = cast_ray(&camera.eye, &rotated_direction, cube, light, 0, skybox);
                }
            }

            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}



pub struct Plane {
    pub point: Vec3,  
    pub normal: Vec3, 
    pub material: Material,
}

impl RayIntersect for Plane {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let denom = self.normal.dot(ray_direction);
        
        
        if denom.abs() > 1e-6 {
            let p0l0 = self.point - ray_origin;
            let t = p0l0.dot(&self.normal) / denom;
            if t >= 0.0 {
                let point = ray_origin + ray_direction * t;

                
                if point.x.abs() <= 1.0 && point.z.abs() <= 1.0 {
                    
                    let normal = if denom < 0.0 { self.normal } else { -self.normal };
                    
                    
                    return Intersect::new(point, normal, t, self.material);
                }
            }
        }
        Intersect::empty()
    }
}




pub struct Skybox {
    pub day_material: Material,    
    pub night_material: Material,  
    pub current_material: Material, 
}

impl Skybox {
    pub fn new(day_material: Material, night_material: Material) -> Self {
        Skybox { 
            day_material,
            night_material,
            current_material: day_material, 
        }
    }

    pub fn sample(&self, _direction: Vec3) -> Color {
        
        self.current_material.diffuse
    }

    pub fn set_day(&mut self) {
        self.current_material = self.day_material.clone();
    }

    pub fn set_night(&mut self) {
        self.current_material = self.night_material.clone();
    }
}


fn load_skybox() -> Skybox {
    let day_material = Material::new(
        Color::new(135, 206, 235),  
        50.0,
        [1.0, 0.0, 0.0, 0.0],       
        1.0,
    );

    let night_material = Material::new(
        Color::new(10, 10, 30),  
        50.0,
        [1.0, 0.0, 0.0, 0.0],    
        1.0,
    );
    

    Skybox::new(day_material, night_material)
}



fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 400;
    let framebuffer_height = 300;
    let frame_delay = Duration::from_millis(16);
    let mut is_day = true; 


    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Refractor",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    let mut skybox = load_skybox();

    let plane_material = Material::new(
        Color::new(34, 139, 34),  
        50.0,
        [1.0, 0.0, 0.0, 0.0],     
        1.0,
    );    

    let plane = Plane {
        point: Vec3::new(0.0, 0.0, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
        material: plane_material,
    };

    let tronco = Material::new(
        Color::new(139, 69, 19),  
        50.0,
        [0.8, 0.2, 0.0, 0.0],     
        1.0,
    );    

    let hojas = Material::new(
        Color::new(0, 255, 0),  
        50.0,
        [0.8, 0.2, 0.0, 0.0],
        1.0,
    );
    let agua = Material::new(
        Color::new(0, 0, 255),  
        50.0,
        [0.5, 0.5, 0.0, 0.0],  
        1.0,
    );
    let mut tiempo = 0.0;

    
    let mut cubos_agua = vec![
        Cube::new(Vec3::new(0.0, 0.0, 0.0), 0.10, agua.clone()),
        Cube::new(Vec3::new(-0.1, 0.0, 0.0), 0.10, agua.clone()),
        Cube::new(Vec3::new(-0.1, 0.0, 0.1), 0.10, agua.clone()),
        Cube::new(Vec3::new(0.0, 0.0, 0.1), 0.10, agua.clone()),
    ];

    

    let cubes = vec![
        
        Cube::new(Vec3::new(-0.8, 0.10, -0.8), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.8, 0.20, -0.8), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.8, 0.30, -0.8), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.8, 0.40, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.9, 0.40, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.7, 0.40, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.8, 0.50, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.8, 0.40, -0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.8, 0.40, -0.7), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(-0.5, 0.10, -0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.5, 0.20, -0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.5, 0.30, -0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.5, 0.40, -0.5), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.5, 0.50, -0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.5, 0.60, -0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.6, 0.50, -0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.4, 0.50, -0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.5, 0.50, -0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.5, 0.50, -0.4), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(-0.1, 0.10, -0.8), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.1, 0.20, -0.8), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.1, 0.30, -0.8), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.1, 0.40, -0.8), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.1, 0.50, -0.8), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.1, 0.60, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.1, 0.70, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.2, 0.60, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.0, 0.60, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.1, 0.60, -0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.1, 0.60, -0.7), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.6, 0.10, -0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.6, 0.20, -0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.6, 0.30, -0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.6, 0.40, -0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.6, 0.50, -0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.6, 0.60, -0.6), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.6, 0.70, -0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.6, 0.80, -0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.5, 0.70, -0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.7, 0.70, -0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.6, 0.70, -0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.6, 0.70, -0.5), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(-0.9, 0.10, 0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.9, 0.20, 0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.9, 0.30, 0.5), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.9, 0.40, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.9, 0.50, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-1.0, 0.40, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.8, 0.40, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.9, 0.50, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.9, 0.40, 0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.9, 0.40, 0.4), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.3, 0.10, 0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.3, 0.20, 0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.3, 0.30, 0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.3, 0.40, 0.9), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.3, 0.50, 0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.3, 0.60, 0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.2, 0.50, 0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.4, 0.50, 0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.3, 0.50, 1.0), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.3, 0.50, 0.8), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.8, 0.10, 0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.8, 0.20, 0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.8, 0.30, 0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.8, 0.40, 0.6), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.8, 0.50, 0.6), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.8, 0.60, 0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.8, 0.70, 0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.7, 0.60, 0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.9, 0.60, 0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.8, 0.60, 0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.8, 0.60, 0.5), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.4, 0.10, -0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.4, 0.20, -0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.4, 0.30, -0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.4, 0.40, -0.9), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.4, 0.50, -0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.3, 0.50, -0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.5, 0.50, -0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.4, 0.60, -0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.4, 0.50, -1.0), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.4, 0.50, -0.8), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.9, 0.10, 0.4), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.9, 0.20, 0.4), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.9, 0.30, 0.4), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.9, 0.40, 0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(1.0, 0.40, 0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.8, 0.40, 0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.9, 0.50, 0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.9, 0.40, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.9, 0.40, 0.3), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(-0.4, 0.10, 0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.4, 0.20, 0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.4, 0.30, 0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.4, 0.40, 0.9), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.4, 0.50, 0.9), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.4, 0.60, 0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.3, 0.60, 0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.5, 0.60, 0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.4, 0.70, 0.9), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.4, 0.60, 1.0), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.4, 0.60, 0.8), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.7, 0.10, 0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.7, 0.20, 0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.7, 0.30, 0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.7, 0.40, 0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.7, 0.50, 0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.7, 0.60, 0.7), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.7, 0.70, 0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.6, 0.70, 0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.8, 0.70, 0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.7, 0.80, 0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.7, 0.70, 0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.7, 0.70, 0.6), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(-0.6, 0.10, -0.4), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.6, 0.20, -0.4), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.6, 0.30, -0.4), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.6, 0.40, -0.4), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.6, 0.50, -0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.7, 0.50, -0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.5, 0.50, -0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.6, 0.60, -0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.6, 0.50, -0.3), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.6, 0.50, -0.5), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.3, 0.10, 0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.3, 0.20, 0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.3, 0.30, 0.5), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.3, 0.40, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.2, 0.40, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.4, 0.40, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.3, 0.50, 0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.3, 0.40, 0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.3, 0.40, 0.4), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(-0.2, 0.10, -0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.2, 0.20, -0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.2, 0.30, -0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.2, 0.40, -0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.2, 0.50, -0.2), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.2, 0.60, -0.2), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.3, 0.60, -0.2), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.1, 0.60, -0.2), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.2, 0.70, -0.2), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.2, 0.60, -0.3), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.2, 0.60, -0.1), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.8, 0.10, -0.3), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.8, 0.20, -0.3), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.8, 0.30, -0.3), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.8, 0.40, -0.3), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.7, 0.40, -0.3), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.9, 0.40, -0.3), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.8, 0.50, -0.3), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.8, 0.40, -0.4), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.8, 0.40, -0.2), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(-0.7, 0.10, 0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.7, 0.20, 0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.7, 0.30, 0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.7, 0.40, 0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.7, 0.50, 0.2), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.7, 0.60, 0.2), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.7, 0.70, 0.2), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.8, 0.70, 0.2), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.6, 0.70, 0.2), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.7, 0.80, 0.2), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.7, 0.70, 0.3), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.7, 0.70, 0.1), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(0.1, 0.10, -0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.1, 0.20, -0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.1, 0.30, -0.5), 0.10, tronco.clone()),
        Cube::new(Vec3::new(0.1, 0.40, -0.5), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(0.1, 0.50, -0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.0, 0.50, -0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.2, 0.50, -0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.1, 0.60, -0.5), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.1, 0.50, -0.6), 0.10, hojas.clone()),
        Cube::new(Vec3::new(0.1, 0.50, -0.4), 0.10, hojas.clone()),

        
        Cube::new(Vec3::new(-0.6, 0.10, -0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.6, 0.20, -0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.6, 0.30, -0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.6, 0.40, -0.7), 0.10, tronco.clone()),
        Cube::new(Vec3::new(-0.6, 0.50, -0.7), 0.10, tronco.clone()),
        
        Cube::new(Vec3::new(-0.6, 0.60, -0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.7, 0.60, -0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.5, 0.60, -0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.6, 0.70, -0.7), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.6, 0.60, -0.8), 0.10, hojas.clone()),
        Cube::new(Vec3::new(-0.6, 0.60, -0.6), 0.10, hojas.clone()),


    ];

    

    let mut camera = Camera::new(
        Vec3::new(0.0, 3.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut light = Light::new(
        Vec3::new(5.0, 5.0, 5.0),  
        Color::new(255, 255, 255),  
        1.0,                        
    );

    
    

    let rotation_speed = PI / 10.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        
        tiempo += 0.5;  
        for (i, cubo) in cubos_agua.iter_mut().enumerate() {
            let desplazamiento = (tiempo + i as f32).sin() * 0.05;  
            cubo.center.y = 0.0 + desplazamiento;  
        }
    
        
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0); 
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }
        if window.is_key_down(Key::W) {
            camera.zoom(0.1);
        }
        if window.is_key_down(Key::S) {
            camera.zoom(-0.1);
        }
        if window.is_key_down(Key::D) {
            is_day = true;
            skybox.set_day();
            light.position = Vec3::new(5.0, 5.0, 5.0);
            light.color = Color::new(255, 255, 255);
            light.intensity = 1.0;
        }
        if window.is_key_down(Key::N) {
            is_day = false;
            skybox.set_night();
            light.position = Vec3::new(1.0, 1.0, 1.0);
            light.color = Color::new(20, 20, 50);
            light.intensity = 0.05;
        }
    
        
        let mut todos_los_cubos = cubes.clone();  
        todos_los_cubos.extend_from_slice(&cubos_agua);  
    
        render(&mut framebuffer, &plane, &todos_los_cubos, &camera, &light, &skybox);
    
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();
    
        std::thread::sleep(frame_delay);
    }    
}

