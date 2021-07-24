#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr, lang_items),
    register_attr(spirv)
)]

use camera::Camera;
use rand::DefaultRng;
use ray::Ray;
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
use spirv_std::num_traits::FloatConst;
use spirv_std::{
    arch::control_barrier,
    glam::{vec3, UVec3, Vec3, Vec4},
    memory::Semantics,
};

use bytemuck::{Pod, Zeroable};

pub mod camera;
pub mod math;
pub mod rand;
pub mod ray;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
}

fn ray_color(center: Vec3, radius: f32, ray: &Ray) -> Vec3 {
    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant > 0.0 {
        vec3(1.0, 0.0, 0.0)
    } else {
        vec3(0.5, 0.5, 0.5)
    }
}

#[spirv(compute(threads(1024, 1, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(local_invocation_id)] local_id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] out: &mut [Vec4],
) {
    let x = id.y;
    let y = id.z;

    if x >= constants.width {
        return;
    }

    if y >= constants.height {
        return;
    }

    let seed = id.x * (constants.width * constants.height) + constants.width * y + x;
    let mut rng = DefaultRng::new(seed);

    let camera = Camera::new(
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 1.0, 0.0),
        90.0 / 180.0 * f32::PI(),
        constants.width as f32 / constants.height as f32,
        0.0,
        10.0,
        0.0,
        1.0,
    );

    let u = (x as f32 + rng.next_f32()) / (constants.width - 1) as f32;
    let v = (y as f32 + rng.next_f32()) / (constants.height - 1) as f32;

    let ray = camera.get_ray(u, v, &mut rng);
    let color = ray_color(vec3(0.0, 0.0, 1.0), 0.5, &ray);

    let scale = 1.0 / 1024.0;

    unsafe {
        control_barrier::<0, 0, { Semantics::NONE.bits() }>();
        for i in 0..1024 {
            if i == local_id.x {
                out[(y * constants.width + x) as usize] += color.extend(1.0) * scale;
            }
            control_barrier::<0, 0, { Semantics::NONE.bits() }>();
        }
    }
}
