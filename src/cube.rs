use nalgebra_glm::Vec3;
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};

#[derive(Clone, Debug)]
pub struct Cube {
    pub center: Vec3, 
    pub size: f32,     
    pub material: Material,
}


impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        Cube {
            center,
            size,
            material,
        }
    }
}


impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let min = self.center - Vec3::new(self.size / 2.0, self.size / 2.0, self.size / 2.0);
        let max = self.center + Vec3::new(self.size / 2.0, self.size / 2.0, self.size / 2.0);

        
        let inv_dir = Vec3::new(1.0, 1.0, 1.0).component_div(ray_direction);

        
        let t_min = (min - ray_origin).component_mul(&inv_dir);
        let t_max = (max - ray_origin).component_mul(&inv_dir);

        
        let t1 = t_min.zip_map(&t_max, |a, b| a.min(b));
        let t2 = t_min.zip_map(&t_max, |a, b| a.max(b));

        let t_near = t1.max();  
        let t_far = t2.min();   

        if t_near > t_far || t_far < 0.0 {
            return Intersect::empty();  
        }

        
        let point = ray_origin + ray_direction * t_near;

        
        let normal = self.compute_normal(point);

        Intersect::new(point, normal, t_near, self.material)
    }
}

impl Cube {
    fn compute_normal(&self, point: Vec3) -> Vec3 {
        let local_point = point - self.center;
        let bias = 0.001;  

        
        if (local_point.x - self.size / 2.0).abs() < bias {
            Vec3::new(1.0, 0.0, 0.0)  
        } else if (local_point.x + self.size / 2.0).abs() < bias {
            Vec3::new(-1.0, 0.0, 0.0)  
        } else if (local_point.y - self.size / 2.0).abs() < bias {
            Vec3::new(0.0, 1.0, 0.0)  
        } else if (local_point.y + self.size / 2.0).abs() < bias {
            Vec3::new(0.0, -1.0, 0.0)  
        } else if (local_point.z - self.size / 2.0).abs() < bias {
            Vec3::new(0.0, 0.0, 1.0)  
        } else {
            Vec3::new(0.0, 0.0, -1.0)  
        }
    }
}

