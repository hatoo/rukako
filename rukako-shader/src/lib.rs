#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr, lang_items),
    register_attr(spirv)
)]

use crate::rand::DefaultRng;
use camera::Camera;
use hittable::HitRecord;
use material::{Material, Scatter};
use ray::Ray;
use spirv_std::glam::{vec2, vec3, UVec3, Vec2, Vec3, Vec4};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::num_traits::FloatConst;

use bytemuck::{Pod, Zeroable};

pub mod aabb;
pub mod bool;
pub mod bvh;
pub mod camera;
pub mod hittable;
pub mod material;
pub mod math;
pub mod pod;
pub mod rand;
pub mod ray;
pub mod sphere;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub seed: u32,
}

/*
fn hit(
    ray: &Ray,
    world: &[sphere::Sphere],
    len: usize,
    t_min: f32,
    t_max: f32,
    hit_record: &mut HitRecord,
) -> Bool32 {
    let mut closest_so_far = t_max;
    let mut hit = Bool32::FALSE;

    for i in 0..len {
        if world[i].hit(ray, t_min, closest_so_far, hit_record).into() {
            closest_so_far = hit_record.t;
            hit = Bool32::TRUE;
        }
    }

    hit
}
*/

fn ray_color(
    mut ray: Ray,
    world: &[sphere::Sphere],
    bvh: &[bvh::BVHNode],
    rng: &mut DefaultRng,
) -> Vec3 {
    let mut color = vec3(1.0, 1.0, 1.0);
    let mut hit_record = HitRecord::default();
    let mut scatter = Scatter::default();

    for _ in 0..50 {
        if (bvh::BVH { nodes: bvh })
            .hit(&ray, 0.001, f32::INFINITY, &mut hit_record, world)
            .into()
        {
            let material = hit_record.material;

            if material
                .scatter(&ray, &hit_record, rng, &mut scatter)
                .into()
            {
                color *= scatter.color;
                ray = scatter.ray;
            } else {
                break;
            }
        } else {
            let unit_direction = ray.direction.normalize();
            let t = 0.5 * (unit_direction.y + 1.0);
            color *= vec3(1.0, 1.0, 1.0).lerp(vec3(0.5, 0.7, 1.0), t);
            break;
        };
    }

    color
}

pub const NUM_THREADS_X: u32 = 8;
pub const NUM_THREADS_Y: u32 = 8;

#[spirv(compute(threads(/* NUM_THREADS_X */ 8, /* NUM_THREADS_Y */ 8, 1)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] world: &[sphere::Sphere],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] bvh: &[bvh::BVHNode],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] out: &mut [Vec4],
) {
    let x = id.x;
    let y = id.y;

    if x >= constants.width {
        return;
    }

    if y >= constants.height {
        return;
    }

    let seed = constants.seed ^ (constants.width * y + x);
    let mut rng = DefaultRng::new(seed);

    let camera = Camera::new(
        vec3(13.0, 2.0, 3.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        20.0 / 180.0 * f32::PI(),
        constants.width as f32 / constants.height as f32,
        0.1,
        10.0,
        0.0,
        1.0,
    );

    let u = (x as f32 + rng.next_f32()) / (constants.width - 1) as f32;
    let v = (y as f32 + rng.next_f32()) / (constants.height - 1) as f32;

    let ray = camera.get_ray(u, v, &mut rng);
    let color = ray_color(ray, world, bvh, &mut rng);

    out[((constants.height - y - 1) * constants.width + x) as usize] += color.extend(1.0);
}

#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] world: &[sphere::Sphere],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] bvh: &[bvh::BVHNode],
    instance_index: f32,
    output: &mut Vec4,
) {
    let x = in_frag_coord.x;
    let y = constants.height as f32 - in_frag_coord.y;

    /*
    if x >= constants.width {
        return;
    }

    if y >= constants.height {
        return;
    }
    */

    let seed = constants.seed
        ^ ((1024.0 * instance_index) as u32)
        ^ (constants.width * y as u32 + x as u32);
    let mut rng = DefaultRng::new(seed);

    let camera = Camera::new(
        vec3(13.0, 2.0, 3.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        20.0 / 180.0 * f32::PI(),
        constants.width as f32 / constants.height as f32,
        0.1,
        10.0,
        0.0,
        1.0,
    );

    let u = (x as f32 + rng.next_f32()) / (constants.width - 1) as f32;
    let v = (y as f32 + rng.next_f32()) / (constants.height - 1) as f32;

    let ray = camera.get_ray(u, v, &mut rng);
    let color = ray_color(ray, world, bvh, &mut rng);

    *output = color.extend(1.0);
    //out[((constants.height - y - 1) * constants.width + x) as usize] += color.extend(1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_idx: i32,
    #[spirv(position)] builtin_pos: &mut Vec4,
    #[spirv(instance_index)] instance_index: u32,
    index: &mut f32,
) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
    *index = instance_index as f32;
}
