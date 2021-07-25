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
}

#[repr(C)]
pub struct BVH<'a> {
    pub nodes: &'a [BVHNode],
    pub len: u32,
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
        world_len: u32,
    ) -> Bool32 {
        let mut stack = Stack::default();
        let mut hit = Bool32::FALSE;
        stack.push(0);

        while (!stack.is_empty()).into() {
            let i = stack.pop();

            if (!self.nodes[i as usize].aabb.hit(ray, t_min, t_max)).into() {
                continue;
            }

            if i * 2 + 1 >= self.len {
                let index = world_len - (self.len - i);
                if world[index as usize]
                    .hit(ray, t_min, t_max, hit_record)
                    .into()
                {
                    t_max = hit_record.t;
                    hit = Bool32::TRUE;
                }
            } else {
                if i * 2 + 1 < self.len {
                    stack.push(i * 2 + 1);
                }
                if i * 2 + 2 < self.len {
                    stack.push(i * 2 + 2);
                }
            }
        }

        hit
    }
}
