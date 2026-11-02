//! Imported 3D models (Blender → glTF) and the collision built from them.
//!
//! The editor and the game share this module, so a model that is placed in the
//! editor behaves in the game exactly as it did there.
//!
//! Collision is generated from the actual triangles: the surface is sampled
//! into a voxel grid and every hit cell becomes a box collider. That means a
//! model with a hollow inside (a room, a corridor, a broken wall) is walkable
//! through and only the geometry itself is solid — no invisible block cube
//! around it.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};

/// Where Blender exports go.
pub const MODEL_DIR: &str = "assets/models";

/// Extensions Bevy can read without extra plugins.
pub const MODEL_EXTENSIONS: [&str; 2] = ["glb", "gltf"];

/// A loaded model: the scene to draw and the meshes used for collision.
#[derive(Clone)]
pub struct LoadedModel {
    pub scene: Handle<Scene>,
    pub meshes: Vec<Handle<Mesh>>,
}

thread_local! {
    /// `models/chair.glb` → what came out of it.
    static LOADED: RefCell<HashMap<String, LoadedModel>> = RefCell::new(HashMap::new());
    /// Cached voxel hulls, keyed by file and voxel size.
    static HULLS: RefCell<HashMap<String, Vec<(Vec3, Vec3)>>> = RefCell::new(HashMap::new());
}

/// Files in `assets/models` that can be placed, as asset paths.
pub fn scan_dir() -> Vec<String> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(MODEL_DIR) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        let extension = extension.to_ascii_lowercase();
        if !MODEL_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        found.push(format!("models/{}.{}", stem, extension));
    }
    found.sort();
    found
}

/// Remembers a loaded model. Called once per file by the game and the editor.
pub fn register(file: &str, scene: Handle<Scene>, meshes: Vec<Handle<Mesh>>) {
    LOADED.with(|slot| {
        slot.borrow_mut()
            .insert(file.to_string(), LoadedModel { scene, meshes });
    });
}

pub fn is_loaded(file: &str) -> bool {
    LOADED.with(|slot| slot.borrow().contains_key(file))
}

pub fn scene(file: &str) -> Option<Handle<Scene>> {
    LOADED.with(|slot| slot.borrow().get(file).map(|model| model.scene.clone()))
}

pub fn mesh_count(file: &str) -> usize {
    LOADED.with(|slot| {
        slot.borrow()
            .get(file)
            .map(|model| model.meshes.len())
            .unwrap_or(0)
    })
}

/// Voxel hull of a model, in model space: `(center, half)` pairs.
///
/// `voxel` is the edge length of the sampling grid: 0.2 is fine for furniture,
/// 0.35 for a whole room. Runs of filled cells are merged along X, so a wall
/// costs a handful of boxes instead of hundreds.
pub fn hull(file: &str, meshes: &Assets<Mesh>, voxel: f32) -> Vec<(Vec3, Vec3)> {
    let voxel = voxel.clamp(0.05, 2.0);
    let key = format!("{}@{:.3}", file, voxel);
    if let Some(cached) = HULLS.with(|slot| slot.borrow().get(&key).cloned()) {
        return cached;
    }

    let mut cells: HashSet<(i32, i32, i32)> = HashSet::new();
    let step = (voxel * 0.5).max(0.02);

    if let Some(model) = LOADED.with(|slot| slot.borrow().get(file).cloned()) {
        for handle in model.meshes.iter() {
            let Some(mesh) = meshes.get(handle) else {
                continue;
            };
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                continue;
            };
            let indices: Vec<u32> = match mesh.indices() {
                Some(Indices::U32(data)) => data.to_vec(),
                Some(Indices::U16(data)) => data.iter().map(|value| *value as u32).collect(),
                None => (0..positions.len() as u32).collect(),
            };
            for triangle in indices.chunks(3) {
                if triangle.len() < 3 {
                    continue;
                }
                let a = Vec3::from(positions[triangle[0] as usize]);
                let b = Vec3::from(positions[triangle[1] as usize]);
                let c = Vec3::from(positions[triangle[2] as usize]);
                let longest = (b - a).length().max((c - a).length()).max((c - b).length());
                let steps = ((longest / step).ceil() as i32).clamp(1, 40);
                for i in 0..=steps {
                    for j in 0..=(steps - i) {
                        let u = i as f32 / steps as f32;
                        let v = j as f32 / steps as f32;
                        let point = a + (b - a) * u + (c - a) * v;
                        cells.insert((
                            (point.x / voxel).floor() as i32,
                            (point.y / voxel).floor() as i32,
                            (point.z / voxel).floor() as i32,
                        ));
                    }
                }
            }
        }
    }

    let boxes = merge(&cells, voxel);
    HULLS.with(|slot| {
        slot.borrow_mut().insert(key, boxes.clone());
    });
    boxes
}

/// Merges filled cells into runs along X, one run per (y, z) row.
fn merge(cells: &HashSet<(i32, i32, i32)>, voxel: f32) -> Vec<(Vec3, Vec3)> {
    let mut rows: HashMap<(i32, i32), Vec<i32>> = HashMap::new();
    for &(x, y, z) in cells.iter() {
        rows.entry((y, z)).or_default().push(x);
    }

    let mut out = Vec::new();
    for ((y, z), mut xs) in rows {
        xs.sort_unstable();
        let mut start = xs[0];
        let mut previous = xs[0];
        for &x in xs.iter().skip(1) {
            if x == previous + 1 {
                previous = x;
                continue;
            }
            push_run(&mut out, start, previous, y, z, voxel);
            start = x;
            previous = x;
        }
        push_run(&mut out, start, previous, y, z, voxel);
    }

    // deterministic order, so the same model always collides the same way
    out.sort_by(|left, right| {
        left.0
            .x
            .partial_cmp(&right.0.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                left.0
                    .y
                    .partial_cmp(&right.0.y)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then(
                left.0
                    .z
                    .partial_cmp(&right.0.z)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    out
}

fn push_run(out: &mut Vec<(Vec3, Vec3)>, x0: i32, x1: i32, y: i32, z: i32, voxel: f32) {
    let min = Vec3::new(
        x0 as f32 * voxel,
        y as f32 * voxel,
        z as f32 * voxel,
    );
    let max = Vec3::new(
        (x1 + 1) as f32 * voxel,
        (y + 1) as f32 * voxel,
        (z + 1) as f32 * voxel,
    );
    out.push(((min + max) * 0.5, (max - min) * 0.5));
}

/// Bounding box of a mesh, used for picking in the editor.
pub fn mesh_bounds(mesh: &Mesh) -> Option<(Vec3, Vec3)> {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return None;
    };
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for position in positions.iter() {
        let point = Vec3::from(*position);
        min = min.min(point);
        max = max.max(point);
    }
    if min.x > max.x {
        None
    } else {
        Some((min, max))
    }
}
