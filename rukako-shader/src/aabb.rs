use core::mem::swap;
use spirv_std::glam::{vec3, Vec3};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::{bool::Bool32, ray::Ray};

#[derive(Clone, Copy)]
pub struct AABB {
    pub minimum: Vec3,
    pub maximum: Vec3,
}

impl AABB {
    pub fn hit(&self, ray: &Ray, mut t_min: f32, mut t_max: f32) -> Bool32 {
        {
            let inv_d = 1.0 / ray.direction.x;
            let mut t0 = (self.minimum.x - ray.origin.x) * inv_d;
            let mut t1 = (self.maximum.x - ray.origin.x) * inv_d;

            if inv_d < 0.0 {
                swap(&mut t0, &mut t1);
            }

            t_min = t0.max(t_min);
            t_max = t1.min(t_max);

            if t_max <= t_min {
                return Bool32::FALSE;
            }
        }
        {
            let inv_d = 1.0 / ray.direction.y;
            let mut t0 = (self.minimum.y - ray.origin.y) * inv_d;
            let mut t1 = (self.maximum.y - ray.origin.y) * inv_d;

            if inv_d < 0.0 {
                swap(&mut t0, &mut t1);
            }

            t_min = t0.max(t_min);
            t_max = t1.min(t_max);

            if t_max <= t_min {
                return Bool32::FALSE;
            }
        }
        {
            let inv_d = 1.0 / ray.direction.z;
            let mut t0 = (self.minimum.z - ray.origin.z) * inv_d;
            let mut t1 = (self.maximum.z - ray.origin.z) * inv_d;

            if inv_d < 0.0 {
                swap(&mut t0, &mut t1);
            }

            t_min = t0.max(t_min);
            t_max = t1.min(t_max);

            if t_max <= t_min {
                return Bool32::FALSE;
            }
        }

        Bool32::TRUE
    }
}

pub fn surrounding_box(box0: AABB, box1: AABB) -> AABB {
    let small = vec3(
        box0.minimum.x.min(box1.minimum.x),
        box0.minimum.y.min(box1.minimum.y),
        box0.minimum.z.min(box1.minimum.z),
    );

    let big = vec3(
        box0.maximum.x.max(box1.maximum.x),
        box0.maximum.y.max(box1.maximum.y),
        box0.maximum.z.max(box1.maximum.z),
    );

    AABB {
        minimum: small,
        maximum: big,
    }
}
