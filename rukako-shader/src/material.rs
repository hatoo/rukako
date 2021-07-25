use spirv_std::glam::{vec3, Vec3, Vec4, Vec4Swizzles};
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

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct EnumMaterial {
    pub data: Vec4,
    pub t: u32,
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
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

impl EnumMaterial {
    #[inline(always)]
    pub fn albedo(&self) -> Vec3 {
        self.data.xyz()
    }

    fn lambertian_scatter(
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
            color: self.albedo(),
            ray: scatterd,
        };
        1
    }

    fn metal_scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut DefaultRng,
        scatter: &mut Scatter,
    ) -> u32 {
        let fuzz = self.data.w;
        let reflected = reflect(ray.direction.normalize(), hit_record.normal);
        let scatterd = reflected + fuzz * random_in_unit_sphere(rng);
        if scatterd.dot(hit_record.normal) > 0.0 {
            *scatter = Scatter {
                color: self.albedo(),
                ray: Ray {
                    origin: hit_record.position,
                    direction: scatterd,
                    time: ray.time,
                },
            };
            1
        } else {
            0
        }
    }

    fn dielectric_scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut DefaultRng,
        scatter: &mut Scatter,
    ) -> u32 {
        let ir = self.data.x;
        let refraction_ratio = if hit_record.front_face != 0 {
            1.0 / ir
        } else {
            ir
        };

        let unit_direction = ray.direction.normalize();
        let cos_theta = (-unit_direction).dot(hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction = if cannot_refract {
            reflect(unit_direction, hit_record.normal)
        } else if reflectance(cos_theta, refraction_ratio) > rng.next_f32() {
            reflect(unit_direction, hit_record.normal)
        } else {
            refract(unit_direction, hit_record.normal, refraction_ratio)
        };

        *scatter = Scatter {
            color: vec3(1.0, 1.0, 1.0),
            ray: Ray {
                origin: hit_record.position,
                direction,
                time: ray.time,
            },
        };
        1
    }
}

impl Material for EnumMaterial {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut DefaultRng,
        scatter: &mut Scatter,
    ) -> u32 {
        match self.t {
            0 => self.lambertian_scatter(ray, hit_record, rng, scatter),
            1 => self.metal_scatter(ray, hit_record, rng, scatter),
            _ => self.dielectric_scatter(ray, hit_record, rng, scatter),
        }
    }
}
