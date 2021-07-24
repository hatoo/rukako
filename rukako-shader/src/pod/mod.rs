use bytemuck::{Pod, Zeroable};
use spirv_std::glam::{vec3, Vec3};
use spirv_std::num_traits::Float;

use crate::hittable::{HitRecord, Hittable};
use crate::material::{Material, Scatter};
use crate::math::{random_in_unit_sphere, IsNearZero};
use crate::rand::DefaultRng;
use crate::ray::Ray;

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub material: EnumMaterial,
}

#[derive(Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct Lambertian {
    pub albedo: [f32; 3],
}

#[derive(Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct EnumMaterial {
    pub data: [f32; 4],
    pub t: u32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: EnumMaterial) -> Self {
        Self {
            center: [center.x, center.y, center.z],
            radius,
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

impl Hittable for Sphere {
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f32,
        t_max: f32,
        hit_record: &mut HitRecord,
    ) -> u32 {
        let oc = ray.origin - self.center();
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius() * self.radius();

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return 0;
        }
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;

        if root < t_min {
            root = (-half_b + sqrtd) / a;
            if root < t_min {
                return 0;
            } else if t_max < root {
                return 0;
            }
        } else if t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min {
                return 0;
            } else if t_max < root {
                return 0;
            }
        }

        let position = ray.at(root);

        *hit_record = HitRecord::new(
            position,
            (position - self.center()) / self.radius(),
            root,
            ray,
            self.material,
        );
        1
    }
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Lambertian {
        Self {
            albedo: [albedo.x, albedo.y, albedo.z],
        }
    }

    pub fn albedo(&self) -> Vec3 {
        vec3(self.albedo[0], self.albedo[1], self.albedo[2])
    }
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
            color: self.albedo(),
            ray: scatterd,
        };
        1
    }
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
    pub fn new_lambertian(albedo: Vec3) -> Self {
        Self {
            data: [albedo.x, albedo.y, albedo.z, 0.0],
            t: 0,
        }
    }

    pub fn new_metal(albedo: Vec3, fuzz: f32) -> Self {
        Self {
            data: [albedo.x, albedo.y, albedo.z, fuzz],
            t: 1,
        }
    }

    pub fn new_dielectric(ir: f32) -> Self {
        Self {
            data: [ir, 0.0, 0.0, 0.0],
            t: 2,
        }
    }

    pub fn albedo(&self) -> Vec3 {
        vec3(self.data[0], self.data[1], self.data[2])
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
        let fuzz = self.data[3];
        let reflected = reflect(ray.direction, hit_record.normal);
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
        let ir = self.data[0];
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
