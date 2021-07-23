#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr, lang_items),
    register_attr(spirv)
)]

use camera::Camera;
use rand::DefaultRng;
use ray::Ray;
use spirv_std::glam::{vec3, vec4, UVec3, Vec3, Vec4};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
use spirv_std::num_traits::FloatConst;

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
    /*
    pub camera_origin: [f32; 3],
    pub camera_lower_left_corner: [f32; 3],
    pub camera_horizontal: [f32; 3],
    pub camera_vertical: [f32; 3],
    pub camera_u: [f32; 3],
    pub camera_v: [f32; 3],
    pub camera_w: [f32; 3],
    pub camera_lens_radius: f32,
    pub time0: f32,
    pub time1: f32,
    */
}

/*
#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] slice: &[f32],
    output: &mut Vec4,
) {
    let r = in_frag_coord.x as f32 / (constants.width - 1) as f32;
    let g = in_frag_coord.y as f32 / (constants.height - 1) as f32;
    *output = in_frag_coord;
    output.x = r;
    output.y = slice[0];
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
}
*/

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

// LocalSize/numthreads of (x = 64, y = 1, z = 1)
#[spirv(compute(threads(8, 8, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] out: &mut [Vec4],
) {
    let x = id.x;
    let y = id.y;

    if x >= constants.width {
        return;
    }

    if y >= constants.height {
        return;
    }

    let mut rng = DefaultRng::new(y * constants.width + x);
    let scale = rng.next_f32();

    out[(y * constants.width + x) as usize] = (Vec3::ONE * scale).extend(1.0);
    /*
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

    let u = x as f32 / (constants.width - 1) as f32;
    let v = y as f32 / (constants.height - 1) as f32;

    let ray = camera.get_ray(u, v);
    let color = ray_color(vec3(0.0, 0.0, 1.0), 0.5, &ray);

    // let mut rng = Pcg32::

    // let r = x as f32 / (constants.width - 1) as f32;
    out[(y * constants.width + x) as usize] = color.extend(1.0);
    */
}
