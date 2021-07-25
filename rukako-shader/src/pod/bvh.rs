use crate::aabb::{surrounding_box, AABB};
use bytemuck::{Pod, Zeroable};
use float_ord::FloatOrd;
use rand::prelude::*;
use spirv_std::glam::Vec3;

use super::SpherePod;

#[derive(Clone, Copy, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct BVHNodePod {
    minimum: [f32; 4],
    maximum: [f32; 4],
    child: [u32; 4],
}

enum BVHChildInner {
    One(usize),
    Two(usize, usize),
    World(usize),
}

struct BVHNodeInner {
    aabb: AABB,
    child: BVHChildInner,
}

fn create_bvh_inner(
    world: &mut [SpherePod],
    time0: f32,
    time1: f32,
    l: usize,
    r: usize,
    out: &mut Vec<BVHNodeInner>,
    rng: &mut impl Rng,
) -> Option<usize> {
    if !(l < r) {
        return None;
    }

    if r - l == 1 {
        let i = out.len();
        out.push(BVHNodeInner {
            aabb: world[l].bounding_box(time0, time1),
            child: BVHChildInner::World(l),
        });
        return Some(i);
    }

    let axis = rng.gen_range(0..=2);

    world[l..r].sort_by_key(|w| FloatOrd(w.bounding_box(time0, time1).minimum[axis]));

    let dummy_child = BVHNodeInner {
        aabb: AABB {
            minimum: Vec3::ZERO,
            maximum: Vec3::ZERO,
        },
        child: BVHChildInner::One(0),
    };

    let i = out.len();
    out.push(dummy_child);

    let mid = (l + r) / 2;

    let left = create_bvh_inner(world, time0, time1, l, mid, out, rng);
    let right = create_bvh_inner(world, time0, time1, mid, r, out, rng);

    match (left, right) {
        (Some(left), Some(right)) => {
            out[i] = BVHNodeInner {
                aabb: surrounding_box(out[left].aabb, out[right].aabb),
                child: BVHChildInner::Two(left, right),
            };
        }
        (Some(left), None) => {
            out[i] = BVHNodeInner {
                aabb: out[left].aabb,
                child: BVHChildInner::One(left),
            };
        }
        _ => unreachable!(),
    }

    Some(i)
}

impl Into<BVHNodePod> for BVHNodeInner {
    fn into(self) -> BVHNodePod {
        let child = match self.child {
            BVHChildInner::One(l) => [0, l as u32, 0, 0],
            BVHChildInner::Two(l, r) => [1, l as u32, r as u32, 0],
            BVHChildInner::World(w) => [2, 0, 0, w as u32],
        };
        BVHNodePod {
            minimum: [
                self.aabb.minimum.x,
                self.aabb.minimum.y,
                self.aabb.minimum.z,
                0.0,
            ],
            maximum: [
                self.aabb.maximum.x,
                self.aabb.maximum.y,
                self.aabb.maximum.z,
                0.0,
            ],
            child,
        }
    }
}

pub fn create_bvh(
    world: &mut [SpherePod],
    time0: f32,
    time1: f32,
    rng: &mut impl Rng,
) -> Vec<BVHNodePod> {
    let mut ret = Vec::new();

    create_bvh_inner(world, time0, time1, 0, world.len(), &mut ret, rng);

    ret.into_iter().map(Into::into).collect()
}
