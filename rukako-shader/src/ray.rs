use spirv_std::glam::Vec3;

#[derive(Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub time: f32,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}
