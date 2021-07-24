use bytemuck::{Pod, Zeroable};
use spirv_std::glam::{vec3, Vec3};
use spirv_std::num_traits::Float;

use crate::hittable::{HitRecord, Hittable};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct Sphere {
    pub data: [f32; 4],
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self {
            data: [center.x, center.y, center.z, radius],
        }
    }

    pub fn center(&self) -> Vec3 {
        vec3(self.data[0], self.data[1], self.data[2])
    }

    pub fn radius(&self) -> f32 {
        self.data[3]
    }
}

impl Hittable for Sphere {
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f32,
        t_max: f32,
        hit_record: &mut HitRecord,
    ) -> u32 {
        let oc = ray.origin - self.center();
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius() * self.radius();

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return 0;
        }
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;

        if root < t_min {
            root = (-half_b + sqrtd) / a;
            if root < t_min {
                return 0;
            } else if t_max < root {
                return 0;
            }
        } else if t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min {
                return 0;
            } else if t_max < root {
                return 0;
            }
        }

        let position = ray.at(root);

        *hit_record = HitRecord::new(
            position,
            (position - self.center()) / self.radius(),
            root,
            ray,
            // self.material.clone(),
        );
        1
    }
}
