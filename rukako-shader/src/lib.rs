#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr, lang_items),
    register_attr(spirv)
)]

use camera::Camera;
use hittable::{HitRecord, Hittable};
use material::{Lambertian, Material, Scatter};
use rand::DefaultRng;
use ray::Ray;
use sphere::Sphere;
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
pub mod hittable;
pub mod material;
pub mod math;
pub mod rand;
pub mod ray;
pub mod sphere;

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
}

fn ray_color(ray: &Ray, world: &Sphere, material: &Lambertian, rng: &mut DefaultRng) -> Vec3 {
    let mut color = vec3(1.0, 1.0, 1.0);
    let mut ray = *ray;

    for _ in 0..50 {
        let mut hit_record = HitRecord::default();
        if world.hit(&ray, 0.001, f32::INFINITY, &mut hit_record) != 0 {
            let mut scatter = Scatter::default();

            if material.scatter(&ray, &hit_record, rng, &mut scatter) != 0 {
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

    /*
    let mut hit_record = HitRecord::default();
    if world.hit(ray, 0.001, f32::INFINITY, &mut hit_record) != 0 {
        let mut scatter = Scatter::default();

        return if material.scatter(ray, &hit_record, rng, &mut scatter) != 0 {
            // scatter.color * ray_color(&scatter.ray, world, material, depth - 1, rng)
            ray_color(&scatter.ray, world, material, rng)
        } else {
            vec3(0.0, 0.0, 0.0)
        };
    }
    */
    /*
    match world.hit(ray, 0.001, f32::INFINITY).as_ref() {
        Some(hit_record) => {}
        None => {} /*
                   return if let Some(Scatter {
                       color,
                       ray: scatterd,
                   }) = material.scatter(ray, &hit_record, rng)
                   {
                       color * ray_color(&scatterd, world, material, depth - 1, rng)
                   } else {
                       vec3(0.0, 0.0, 0.0)
                   };
                   */
    }
    */
    /*
    let unit_direction = ray.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    vec3(1.0, 1.0, 1.0).lerp(vec3(0.5, 0.7, 1.0), t)
    */
}

fn ray_color_test(center: Vec3, radius: f32, ray: &Ray) -> Vec3 {
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

    let world = Sphere {
        center: vec3(0.0, 1.0, 0.0),
        radius: 1.0,
    };

    let material = Lambertian {
        albedo: vec3(0.4, 0.2, 0.1),
    };

    let seed = id.x * (constants.width * constants.height) + constants.width * y + x;
    let mut rng = DefaultRng::new(seed);

    let camera = Camera::new(
        vec3(13.0, 2.0, 3.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        20.0 / 180.0 * f32::PI(),
        constants.width as f32 / constants.height as f32,
        0.0,
        10.0,
        0.0,
        1.0,
    );

    let u = (x as f32 + rng.next_f32()) / (constants.width - 1) as f32;
    let v = (y as f32 + rng.next_f32()) / (constants.height - 1) as f32;

    let ray = camera.get_ray(u, v, &mut rng);
    let color = ray_color(&ray, &world, &material, &mut rng); // ray_color_test(vec3(0.0, 0.0, 1.0), 0.5, &ray);

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
