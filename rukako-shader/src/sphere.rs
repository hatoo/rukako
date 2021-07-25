use spirv_std::glam::{vec3, Vec3};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::{
    aabb::AABB,
    bool::Bool32,
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
    ) -> Bool32 {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return Bool32::FALSE;
        }
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;

        if Bool32::new(root < t_min)
            .or(Bool32::new(t_max < root))
            .into()
        {
            root = (-half_b + sqrtd) / a;
            if Bool32::new(root < t_min)
                .or(Bool32::new(t_max < root))
                .into()
            {
                return Bool32::FALSE;
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
        Bool32::TRUE
    }

    fn bounding_box(&self, _time0: f32, _time1: f32) -> AABB {
        AABB {
            minimum: self.center - vec3(self.radius, self.radius, self.radius),
            maximum: self.center + vec3(self.radius, self.radius, self.radius),
        }
    }
}
