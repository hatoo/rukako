use spirv_std::glam::Vec3;

use crate::{aabb::AABB, bool::Bool32, material::EnumMaterial, ray::Ray};

#[derive(Clone, Default)]
pub struct HitRecord {
    pub position: Vec3,
    pub normal: Vec3,
    pub material: EnumMaterial,
    pub t: f32,
    pub front_face: Bool32,
}

impl HitRecord {
    pub fn new(
        position: Vec3,
        outward_normal: Vec3,
        t: f32,
        ray: &Ray,
        material: EnumMaterial,
    ) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Self {
            position,
            normal,
            t,
            front_face: front_face.into(),
            material,
        }
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32, hit_record: &mut HitRecord) -> Bool32;
    fn bounding_box(&self, time0: f32, time1: f32) -> AABB;
}

/*
impl<T: Hittable> Hittable for [T] {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32, hit_record: &mut HitRecord) -> u32 {
        let mut closest_so_far = t_max;
        let mut hit = 0;

        for hittable in self {
            if hittable.hit(ray, t_min, closest_so_far, hit_record) != 0 {
                closest_so_far = hit_record.t;
                hit = 1;
            }
        }

        hit
    }
}
*/
