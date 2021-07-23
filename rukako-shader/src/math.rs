/*
use rand::prelude::*;

use spirv_std::glam::{vec3, Vec3};
use spirv_std::num_traits::FloatConst;

pub fn random_in_unit_sphere(rng: &mut impl Rng) -> Vec3 {
    loop {
        let v = vec3(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );

        if v.length_squared() < 1.0 {
            break v;
        }
    }
}

#[allow(dead_code)]
pub fn random_in_hemisphere(normal: Vec3, rng: &mut impl Rng) -> Vec3 {
    let v = random_in_unit_sphere(rng).normalize();
    if normal.dot(v) > 0.0 {
        v
    } else {
        -v
    }
}

pub fn random_in_unit_disk(rng: &mut impl Rng) -> Vec3 {
    loop {
        let p = vec3(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
        if p.length_squared() < 1.0 {
            break p;
        }
    }
}

pub fn random_cosine_direction(rng: &mut impl Rng) -> Vec3 {
    let r1: f32 = rng.gen();
    let r2: f32 = rng.gen();
    let z = (1.0 - r2).sqrt();

    let phi = 2.0 * f32::PI() * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();

    vec3(x, y, z)
}

pub fn random_to_sphere(radius: f32, distance_squared: f32, rng: &mut impl Rng) -> Vec3 {
    let r1 = rng.gen::<f32>();
    let r2 = rng.gen::<f32>();
    let z = 1.0 + r2 * ((1.0 - radius * radius / distance_squared).sqrt() - 1.0);

    let phi = 2.0 * f32::PI() * r1;
    let x = phi.cos() * (1.0 - z * z).sqrt();
    let y = phi.sin() * (1.0 - z * z).sqrt();

    vec3(x, y, z)
}

pub fn sphere_uv(point: Vec3) -> (f32, f32) {
    let theta = (-point.y).acos();
    let phi = (-point.z).atan2(point.x) + f32::PI();
    (phi / (2.0 * f32::PI()), theta / f32::PI())
}

pub trait IsNearZero {
    fn is_near_zero(&self) -> bool;
}

impl IsNearZero for Vec3 {
    fn is_near_zero(&self) -> bool {
        const S: f32 = 1e-8;
        self.x.abs() < S && self.y.abs() < S && self.z.abs() < S
    }
}
*/
