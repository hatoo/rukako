#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr, lang_items),
    register_attr(spirv)
)]

use camera::Camera;
use spirv_std::glam::{vec2, Vec2, Vec3, Vec4};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

use bytemuck::{Pod, Zeroable};

pub mod camera;
pub mod math;
pub mod ray;

fn f() -> Option<f32> {
    Some(0.2)
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    // pub camera: Camera,
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let r = in_frag_coord.x as f32 / (constants.width - 1) as f32;
    let g = in_frag_coord.y as f32 / (constants.height - 1) as f32;
    *output = in_frag_coord;
    output.x = r;
    if let Some(a) = f() {
        output.y = a;
    }
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
}
