#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr, lang_items),
    register_attr(spirv)
)]

use camera::Camera;
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
use spirv_std::{
    arch::control_barrier,
    glam::{vec2, vec4, UVec3, Vec2, Vec3, Vec4},
    memory::Semantics,
};

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

// LocalSize/numthreads of (x = 64, y = 1, z = 1)
#[spirv(compute(threads(8, 8, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] out: &mut [Vec4],
) {
    // let index = id.x as usize;
    // prime_indices[index] = index as u32;

    let x = id.x;
    let y = id.y;

    if x >= constants.width {
        return;
    }

    if y >= constants.height {
        return;
    }

    let id = y * constants.width + x;
    let r = x as f32 / (constants.width - 1) as f32;
    unsafe {
        control_barrier::<0, 0, { Semantics::NONE.bits() }>();
        for i in 0..constants.width * constants.height {
            if i == id {
                out[0].x += 1.0;
            }
            control_barrier::<1, 0, { Semantics::NONE.bits() }>();
        }
        // out[(y * constants.width + x) as usize] = vec4(r, 0.0, 1.0, 1.0);
    }
}
