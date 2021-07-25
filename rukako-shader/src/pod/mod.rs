use bytemuck::{Pod, Zeroable};
use spirv_std::glam::{vec3, Vec3, Vec4, Vec4Swizzles};
use spirv_std::num_traits::Float;

use crate::hittable::{HitRecord, Hittable};
use crate::material::{Material, Scatter};
use crate::math::{random_in_unit_sphere, IsNearZero};
use crate::rand::DefaultRng;
use crate::ray::Ray;

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SpherePod {
    pub center: [f32; 3],
    pub _pad0: f32,
    pub radius: f32,
    pub _pad1: [f32; 3],
    pub material: EnumMaterialPod,
}
#[derive(Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct EnumMaterialPod {
    pub data: [f32; 4],
    pub t: u32,
    pub _pad: [f32; 3],
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
