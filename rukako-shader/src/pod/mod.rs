use bytemuck::{Pod, Zeroable};
use spirv_std::glam::{vec3, Vec3};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

use crate::aabb::AABB;

#[cfg(not(target_arch = "spirv"))]
pub mod bvh;

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SpherePod {
    center: [f32; 3],
    _pad0: f32,
    radius: f32,
    _pad1: [f32; 3],
    material: EnumMaterialPod,
}
#[derive(Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct EnumMaterialPod {
    data: [f32; 4],
    t: u32,
    _pad: [f32; 3],
}

impl SpherePod {
    pub fn new(center: Vec3, radius: f32, material: EnumMaterialPod) -> Self {
        Self {
            center: [center.x, center.y, center.z],
            _pad0: 0.0,
            radius,
            _pad1: [0.0, 0.0, 0.0],
            material,
        }
    }

    pub fn center(&self) -> Vec3 {
        vec3(self.center[0], self.center[1], self.center[2])
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn bounding_box(&self, _time0: f32, _time1: f32) -> AABB {
        AABB {
            minimum: self.center() - vec3(self.radius, self.radius, self.radius),
            maximum: self.center() + vec3(self.radius, self.radius, self.radius),
        }
    }
}

impl EnumMaterialPod {
    pub fn new_lambertian(albedo: Vec3) -> Self {
        Self {
            data: [albedo.x, albedo.y, albedo.z, 0.0],
            t: 0,
            _pad: [0.0, 0.0, 0.0],
        }
    }

    pub fn new_metal(albedo: Vec3, fuzz: f32) -> Self {
        Self {
            data: [albedo.x, albedo.y, albedo.z, fuzz],
            t: 1,
            _pad: [0.0, 0.0, 0.0],
        }
    }

    pub fn new_dielectric(ir: f32) -> Self {
        Self {
            data: [ir, 0.0, 0.0, 0.0],
            t: 2,
            _pad: [0.0, 0.0, 0.0],
        }
    }
}
