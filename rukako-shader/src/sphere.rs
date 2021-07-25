use spirv_std::glam::Vec3;
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::{
    hittable::{HitRecord, Hittable},
    material::EnumMaterial,
};

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub matelial: EnumMaterial,
}

impl Hittable for Sphere {
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f32,
        t_max: f32,
        hit_record: &mut HitRecord,
    ) -> u32 {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;

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
            (position - self.center) / self.radius,
            root,
            ray,
            self.matelial,
        );
        1
    }
}
