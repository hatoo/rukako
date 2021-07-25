use spirv_std::glam::UVec4;

use crate::{
    aabb::AABB,
    bool::Bool32,
    hittable::{HitRecord, Hittable},
    ray::Ray,
    sphere::Sphere,
};

#[repr(C)]
pub struct BVHNode {
    aabb: AABB,
    child: UVec4,
}

#[repr(C)]
pub struct BVH<'a> {
    pub nodes: &'a [BVHNode],
}

#[derive(Default)]
struct Stack {
    len: u32,
    data: [u32; 32],
}

impl Stack {
    fn push(&mut self, element: u32) {
        self.data[self.len as usize] = element;
        self.len += 1;
    }

    fn pop(&mut self) -> u32 {
        self.len -= 1;
        self.data[self.len as usize]
    }

    fn is_empty(&self) -> Bool32 {
        (self.len == 0).into()
    }
}

impl<'a> BVH<'a> {
    pub fn hit(
        &self,
        ray: &Ray,
        t_min: f32,
        mut t_max: f32,
        hit_record: &mut HitRecord,
        world: &[Sphere],
    ) -> Bool32 {
        let mut stack = Stack::default();
        let mut hit = Bool32::FALSE;
        stack.push(0);

        while (!stack.is_empty()).into() {
            let i = stack.pop();

            if (!self.nodes[i as usize].aabb.hit(ray, t_min, t_max)).into() {
                continue;
            }

            match self.nodes[i as usize].child.x {
                0 => {
                    stack.push(self.nodes[i as usize].child.y);
                }
                1 => {
                    stack.push(self.nodes[i as usize].child.y);
                    stack.push(self.nodes[i as usize].child.z);
                }
                _ => {
                    if world[self.nodes[i as usize].child.w as usize]
                        .hit(ray, t_min, t_max, hit_record)
                        .into()
                    {
                        t_max = hit_record.t;
                        hit = Bool32::TRUE;
                    }
                }
            }
        }

        hit
    }
}
