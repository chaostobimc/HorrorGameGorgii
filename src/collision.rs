//! Axis aligned collision volumes.
//!
//! The hotel is entirely made of boxes, so a simple swept-AABB style solver is
//! both exact and cheap. Every wall, step, counter and doorframe registers a
//! `SolidBox`; doors simply drop theirs when they open.

use bevy::prelude::*;

/// Half extents of a solid, centred on the entity translation.
#[derive(Component, Clone, Copy, Debug)]
pub struct SolidBox {
    pub half: Vec3,
}

impl SolidBox {
    pub fn new(half: Vec3) -> Self {
        Self { half }
    }

    pub fn from_size(size: Vec3) -> Self {
        Self { half: size * 0.5 }
    }
}

/// How high a ledge the player can walk over without jumping.
pub const STEP_UP: f32 = 0.44;

pub type SolidList = Vec<(Vec3, Vec3)>; // (centre, half)

pub fn collect(solids: &Query<(&GlobalTransform, &SolidBox)>) -> SolidList {
    solids
        .iter()
        .map(|(t, s)| (t.translation(), s.half))
        .collect()
}

fn xz_overlap(p: Vec3, radius: f32, c: Vec3, half: Vec3) -> bool {
    let min = c - half;
    let max = c + half;
    p.x + radius > min.x && p.x - radius < max.x && p.z + radius > min.z && p.z - radius < max.z
}

/// Push the player out of any box they are intersecting, one axis at a time.
/// Low ledges (below `STEP_UP`) are ignored so stairs can be climbed.
pub fn resolve_axis(pos: &mut Vec3, delta: f32, axis: usize, radius: f32, height: f32, solids: &SolidList) {
    if delta.abs() < 1e-6 {
        return;
    }
    let feet = pos.y;
    let head = pos.y + height;
    for (c, half) in solids.iter() {
        let s_min = c - half;
        let s_max = c + half;
        // ignore ledges we can step onto, and boxes above our head
        if s_max.y <= feet + STEP_UP {
            continue;
        }
        if s_min.y >= head - 0.02 {
            continue;
        }
        if !xz_overlap(*pos, radius, *c, *half) {
            continue;
        }
        if axis == 0 {
            if delta > 0.0 {
                pos.x = s_min.x - radius - 0.001;
            } else {
                pos.x = s_max.x + radius + 0.001;
            }
        } else if delta > 0.0 {
            pos.z = s_min.z - radius - 0.001;
        } else {
            pos.z = s_max.z + radius + 0.001;
        }
    }
}

/// Highest floor under the player that they can stand on.
pub fn support_height(pos: Vec3, radius: f32, solids: &SolidList) -> f32 {
    let mut best = f32::NEG_INFINITY;
    for (c, half) in solids.iter() {
        let top = c.y + half.y;
        if top > pos.y + STEP_UP {
            continue;
        }
        if !xz_overlap(pos, radius * 0.85, *c, *half) {
            continue;
        }
        if top > best {
            best = top;
        }
    }
    best
}

/// Lowest ceiling above the player.
pub fn ceiling_height(pos: Vec3, radius: f32, solids: &SolidList) -> f32 {
    let mut best = f32::INFINITY;
    for (c, half) in solids.iter() {
        let bottom = c.y - half.y;
        if bottom <= pos.y + 0.25 {
            continue;
        }
        if !xz_overlap(pos, radius * 0.8, *c, *half) {
            continue;
        }
        if bottom < best {
            best = bottom;
        }
    }
    best
}

/// True if the player can stand at this spot (used for the "stuck" failsafe).
pub fn is_free(pos: Vec3, radius: f32, height: f32, solids: &SolidList) -> bool {
    for (c, half) in solids.iter() {
        let s_min = c - half;
        let s_max = c + half;
        if s_max.y <= pos.y + STEP_UP || s_min.y >= pos.y + height - 0.02 {
            continue;
        }
        if xz_overlap(pos, radius, *c, *half) {
            return false;
        }
    }
    true
}

/// Approximate distance from a ray to a sphere, used by the interaction
/// system. Returns the distance along the ray if it hits.
pub fn ray_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let b = oc.dot(dir);
    let c = oc.length_squared() - radius * radius;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let sqrt_d = disc.sqrt();
    let t0 = -b - sqrt_d;
    let t1 = -b + sqrt_d;
    if t0 > 0.0 {
        Some(t0)
    } else if t1 > 0.0 {
        Some(t1)
    } else {
        None
    }
}
