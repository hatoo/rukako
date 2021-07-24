use spirv_std::glam::{vec3, Vec3};
use spirv_std::num_traits::Float;

use crate::{
    hittable::HitRecord,
    math::{random_in_unit_sphere, IsNearZero},
    rand::DefaultRng,
    ray::Ray,
};

#[derive(Clone, Default)]
pub struct Scatter {
    pub color: Vec3,
    pub ray: Ray,
}

pub trait Material {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut DefaultRng,
        sucatter: &mut Scatter,
    ) -> u32;
}

pub struct Lambertian {
    pub albedo: Vec3,
}

pub struct Metal {
    pub albedo: Vec3,
    pub fuzz: f32,
}

impl Material for Lambertian {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut DefaultRng,
        scatter: &mut Scatter,
    ) -> u32 {
        let scatter_direction = hit_record.normal + random_in_unit_sphere(rng).normalize();

        let scatter_direction = if scatter_direction.is_near_zero() != 0 {
            hit_record.normal
        } else {
            scatter_direction
        };

        let scatterd = Ray {
            origin: hit_record.position,
            direction: scatter_direction,
            time: ray.time,
        };

        *scatter = Scatter {
            color: self.albedo,
            ray: scatterd,
        };
        1
    }
}

/*
fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord, rng: &mut DefaultRng) -> Option<Scatter> {
        let reflected = reflect(ray.direction, hit_record.normal);
        let scatterd = reflected + self.fuzz * random_in_unit_sphere(rng);
        if scatterd.dot(hit_record.normal) > 0.0 {
            Some(Scatter {
                color: self.albedo,
                ray: Ray {
                    origin: hit_record.position,
                    direction: scatterd,
                    time: ray.time,
                },
            })
        } else {
            None
        }
    }
}

fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

pub struct Dielectric {
    pub ir: f32,
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord, rng: &mut DefaultRng) -> Option<Scatter> {
        let refraction_ratio = if hit_record.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = ray.direction.normalize();
        let cos_theta = (-unit_direction).dot(hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction =
            if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng.next_f32() {
                reflect(unit_direction, hit_record.normal)
            } else {
                refract(unit_direction, hit_record.normal, refraction_ratio)
            };

        Some(Scatter {
            color: vec3(1.0, 1.0, 1.0),
            ray: Ray {
                origin: hit_record.position,
                direction: direction,
                time: ray.time,
            },
        })
    }
}
*/
