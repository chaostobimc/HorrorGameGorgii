//! Hand written geometry helpers.
//!
//! The whole hotel is built from a handful of primitives that are batched into
//! one mesh per material. UVs are projected in world space so that a tiling
//! texture keeps the same physical scale on every surface - the single most
//! important trick for making procedural architecture not look procedural.

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

#[derive(Default, Clone)]
pub struct MeshBuilder {
    pub pos: Vec<[f32; 3]>,
    pub nrm: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
    pub idx: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.idx.is_empty()
    }

    pub fn vertex_count(&self) -> usize {
        self.pos.len()
    }

    pub fn append(&mut self, other: &MeshBuilder) {
        let base = self.pos.len() as u32;
        self.pos.extend(other.pos.iter().copied());
        self.nrm.extend(other.nrm.iter().copied());
        self.uv.extend(other.uv.iter().copied());
        self.idx.extend(other.idx.iter().map(|i| i + base));
    }

    fn push_vertex(&mut self, p: Vec3, n: Vec3, tile: f32) -> u32 {
        let i = self.pos.len() as u32;
        self.pos.push([p.x, p.y, p.z]);
        self.nrm.push([n.x, n.y, n.z]);
        let (u, v) = planar_uv(p, n, tile);
        self.uv.push([u, v]);
        i
    }

    /// Generic quad: a -> b -> c -> d counter clockwise when seen from the
    /// side the normal points to.
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, tile: f32) {
        let n = (b - a).cross(c - a).normalize_or_zero();
        if n.length_squared() < 0.5 {
            return;
        }
        let i0 = self.push_vertex(a, n, tile);
        let i1 = self.push_vertex(b, n, tile);
        let i2 = self.push_vertex(c, n, tile);
        let i3 = self.push_vertex(d, n, tile);
        self.idx.extend([i0, i1, i2, i0, i2, i3]);
    }

    /// Quad with explicit UVs (0..1) - used for props that need a specific map
    /// such as the clock face or the piano keys.
    pub fn quad_uv(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, uvs: [Vec2; 4]) {
        let n = (b - a).cross(c - a).normalize_or_zero();
        let i0 = self.pos.len() as u32;
        for (p, uv) in [(a, uvs[0]), (b, uvs[1]), (c, uvs[2]), (d, uvs[3])] {
            self.pos.push([p.x, p.y, p.z]);
            self.nrm.push([n.x, n.y, n.z]);
            self.uv.push([uv.x, uv.y]);
        }
        self.idx.extend([i0, i0 + 1, i0 + 2, i0, i0 + 2, i0 + 3]);
    }

    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, tile: f32) {
        let n = (b - a).cross(c - a).normalize_or_zero();
        if n.length_squared() < 0.5 {
            return;
        }
        let i0 = self.push_vertex(a, n, tile);
        let i1 = self.push_vertex(b, n, tile);
        let i2 = self.push_vertex(c, n, tile);
        self.idx.extend([i0, i1, i2]);
    }

    /// Horizontal face with the normal pointing up or down.
    pub fn slab(&mut self, x0: f32, z0: f32, x1: f32, z1: f32, y: f32, tile: f32, up: bool) {
        let a = Vec3::new(x0, y, z0);
        let b = Vec3::new(x0, y, z1);
        let c = Vec3::new(x1, y, z1);
        let d = Vec3::new(x1, y, z0);
        if up {
            self.quad(a, b, c, d, tile);
        } else {
            self.quad(d, c, b, a, tile);
        }
    }

    pub fn floor(&mut self, x0: f32, z0: f32, x1: f32, z1: f32, y: f32, tile: f32) {
        self.slab(x0, z0, x1, z1, y, tile, true);
    }

    pub fn ceiling(&mut self, x0: f32, z0: f32, x1: f32, z1: f32, y: f32, tile: f32) {
        self.slab(x0, z0, x1, z1, y, tile, false);
    }

    /// Wall lying in a plane of constant z. `n_sign` is +1 when the visible
    /// face points towards +z.
    pub fn wall_z(
        &mut self,
        x0: f32,
        x1: f32,
        y0: f32,
        y1: f32,
        z: f32,
        tile: f32,
        n_sign: f32,
    ) {
        let a = Vec3::new(x0, y0, z);
        let b = Vec3::new(x1, y0, z);
        let c = Vec3::new(x1, y1, z);
        let d = Vec3::new(x0, y1, z);
        if n_sign >= 0.0 {
            self.quad(a, b, c, d, tile);
        } else {
            self.quad(b, a, d, c, tile);
        }
    }

    /// Wall lying in a plane of constant x. `n_sign` is +1 when the visible
    /// face points towards +x.
    pub fn wall_x(
        &mut self,
        z0: f32,
        z1: f32,
        y0: f32,
        y1: f32,
        x: f32,
        tile: f32,
        n_sign: f32,
    ) {
        let a = Vec3::new(x, y0, z0);
        let b = Vec3::new(x, y0, z1);
        let c = Vec3::new(x, y1, z1);
        let d = Vec3::new(x, y1, z0);
        if n_sign >= 0.0 {
            self.quad(b, a, d, c, tile);
        } else {
            self.quad(a, b, c, d, tile);
        }
    }

    /// Full closed box (all six faces).
    pub fn box_all(&mut self, center: Vec3, size: Vec3, tile: f32) {
        let h = size * 0.5;
        let (x0, x1) = (center.x - h.x, center.x + h.x);
        let (y0, y1) = (center.y - h.y, center.y + h.y);
        let (z0, z1) = (center.z - h.z, center.z + h.z);
        // +Z
        self.quad(
            Vec3::new(x0, y0, z1),
            Vec3::new(x1, y0, z1),
            Vec3::new(x1, y1, z1),
            Vec3::new(x0, y1, z1),
            tile,
        );
        // -Z
        self.quad(
            Vec3::new(x1, y0, z0),
            Vec3::new(x0, y0, z0),
            Vec3::new(x0, y1, z0),
            Vec3::new(x1, y1, z0),
            tile,
        );
        // +X
        self.quad(
            Vec3::new(x1, y0, z1),
            Vec3::new(x1, y0, z0),
            Vec3::new(x1, y1, z0),
            Vec3::new(x1, y1, z1),
            tile,
        );
        // -X
        self.quad(
            Vec3::new(x0, y0, z0),
            Vec3::new(x0, y0, z1),
            Vec3::new(x0, y1, z1),
            Vec3::new(x0, y1, z0),
            tile,
        );
        // +Y
        self.quad(
            Vec3::new(x0, y1, z1),
            Vec3::new(x1, y1, z1),
            Vec3::new(x1, y1, z0),
            Vec3::new(x0, y1, z0),
            tile,
        );
        // -Y
        self.quad(
            Vec3::new(x0, y0, z0),
            Vec3::new(x1, y0, z0),
            Vec3::new(x1, y0, z1),
            Vec3::new(x0, y0, z1),
            tile,
        );
    }

    /// Box without its bottom face (cheaper for things sitting on the floor).
    pub fn box_no_bottom(&mut self, center: Vec3, size: Vec3, tile: f32) {
        let h = size * 0.5;
        let (x0, x1) = (center.x - h.x, center.x + h.x);
        let (y0, y1) = (center.y - h.y, center.y + h.y);
        let (z0, z1) = (center.z - h.z, center.z + h.z);
        self.quad(
            Vec3::new(x0, y0, z1),
            Vec3::new(x1, y0, z1),
            Vec3::new(x1, y1, z1),
            Vec3::new(x0, y1, z1),
            tile,
        );
        self.quad(
            Vec3::new(x1, y0, z0),
            Vec3::new(x0, y0, z0),
            Vec3::new(x0, y1, z0),
            Vec3::new(x1, y1, z0),
            tile,
        );
        self.quad(
            Vec3::new(x1, y0, z1),
            Vec3::new(x1, y0, z0),
            Vec3::new(x1, y1, z0),
            Vec3::new(x1, y1, z1),
            tile,
        );
        self.quad(
            Vec3::new(x0, y0, z0),
            Vec3::new(x0, y0, z1),
            Vec3::new(x0, y1, z1),
            Vec3::new(x0, y1, z0),
            tile,
        );
        self.quad(
            Vec3::new(x0, y1, z1),
            Vec3::new(x1, y1, z1),
            Vec3::new(x1, y1, z0),
            Vec3::new(x0, y1, z0),
            tile,
        );
    }

    /// Vertical cylinder. `segments` of 12-24 is plenty for furniture legs.
    pub fn cylinder(
        &mut self,
        center: Vec3,
        radius: f32,
        height: f32,
        segments: u32,
        tile: f32,
        caps: bool,
    ) {
        let segs = segments.max(3);
        let y0 = center.y - height * 0.5;
        let y1 = center.y + height * 0.5;
        // side
        let mut prev = 0.0f32;
        for i in 1..=segs {
            let ang = i as f32 / segs as f32 * std::f32::consts::TAU;
            let a0 = Vec3::new(
                center.x + prev.cos() * radius,
                y0,
                center.z + prev.sin() * radius,
            );
            let b0 = Vec3::new(
                center.x + ang.cos() * radius,
                y0,
                center.z + ang.sin() * radius,
            );
            let c1 = Vec3::new(
                center.x + ang.cos() * radius,
                y1,
                center.z + ang.sin() * radius,
            );
            let d1 = Vec3::new(
                center.x + prev.cos() * radius,
                y1,
                center.z + prev.sin() * radius,
            );
            self.quad(a0, b0, c1, d1, tile);
            prev = ang;
        }
        if caps {
            self.disc(Vec3::new(center.x, y1, center.z), radius, segs, true, tile);
            self.disc(Vec3::new(center.x, y0, center.z), radius, segs, false, tile);
        }
    }

    pub fn disc(&mut self, center: Vec3, radius: f32, segments: u32, up: bool, tile: f32) {
        let segs = segments.max(3);
        let n = if up { Vec3::Y } else { Vec3::NEG_Y };
        let origin = self.pos.len() as u32;
        self.pos.push([center.x, center.y, center.z]);
        self.nrm.push([n.x, n.y, n.z]);
        let (u, v) = planar_uv(center, n, tile);
        self.uv.push([u, v]);
        for i in 0..=segs {
            let ang = i as f32 / segs as f32 * std::f32::consts::TAU;
            let p = Vec3::new(
                center.x + ang.cos() * radius,
                center.y,
                center.z + ang.sin() * radius,
            );
            self.pos.push([p.x, p.y, p.z]);
            self.nrm.push([n.x, n.y, n.z]);
            let (u, v) = planar_uv(p, n, tile);
            self.uv.push([u, v]);
        }
        for i in 0..segs {
            if up {
                self.idx
                    .extend([origin, origin + 1 + i, origin + 2 + i]);
            } else {
                self.idx
                    .extend([origin, origin + 2 + i, origin + 1 + i]);
            }
        }
    }

    pub fn ring(
        &mut self,
        center: Vec3,
        inner: f32,
        outer: f32,
        segments: u32,
        up: bool,
        tile: f32,
    ) {
        let segs = segments.max(3);
        let n = if up { Vec3::Y } else { Vec3::NEG_Y };
        for i in 0..segs {
            let a0 = i as f32 / segs as f32 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / segs as f32 * std::f32::consts::TAU;
            let p0 = Vec3::new(
                center.x + a0.cos() * inner,
                center.y,
                center.z + a0.sin() * inner,
            );
            let p1 = Vec3::new(
                center.x + a1.cos() * inner,
                center.y,
                center.z + a1.sin() * inner,
            );
            let p2 = Vec3::new(
                center.x + a1.cos() * outer,
                center.y,
                center.z + a1.sin() * outer,
            );
            let p3 = Vec3::new(
                center.x + a0.cos() * outer,
                center.y,
                center.z + a0.sin() * outer,
            );
            let base = self.pos.len() as u32;
            for p in [p0, p1, p2, p3] {
                self.pos.push([p.x, p.y, p.z]);
                self.nrm.push([n.x, n.y, n.z]);
                let (u, v) = planar_uv(p, n, tile);
                self.uv.push([u, v]);
            }
            if up {
                self.idx
                    .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
            } else {
                self.idx
                    .extend([base, base + 2, base + 1, base, base + 3, base + 2]);
            }
        }
    }

    /// A tube along Y between two heights (pipes, columns, curtain rods).
    pub fn pipe(&mut self, x: f32, z: f32, y0: f32, y1: f32, radius: f32, segs: u32, tile: f32) {
        self.cylinder(
            Vec3::new(x, (y0 + y1) * 0.5, z),
            radius,
            y1 - y0,
            segs,
            tile,
            false,
        );
    }

    /// Horizontal bar from a to b with a square cross section (beams, rails).
    pub fn bar(&mut self, a: Vec3, b: Vec3, thickness: f32, tile: f32) {
        let dir = b - a;
        let len = dir.length();
        if len < 1e-4 {
            return;
        }
        let dir = dir / len;
        let mid = (a + b) * 0.5;
        let up = if dir.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y };
        let side = dir.cross(up).normalize_or_zero() * (thickness * 0.5);
        let upv = side.cross(dir).normalize_or_zero() * (thickness * 0.5);
        let p0 = mid - dir * (len * 0.5);
        let p1 = mid + dir * (len * 0.5);
        let corners = |p: Vec3| {
            [
                p - side - upv,
                p + side - upv,
                p + side + upv,
                p - side + upv,
            ]
        };
        let c0 = corners(p0);
        let c1 = corners(p1);
        for i in 0..4 {
            let j = (i + 1) % 4;
            self.quad(c0[i], c0[j], c1[j], c1[i], tile);
        }
    }

    pub fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.pos);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.nrm);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uv);
        mesh.insert_indices(Indices::U32(self.idx));
        mesh
    }
}

/// Projects a world position onto the two axes that best match the surface
/// normal, giving seamless, constant-scale tiling.
fn planar_uv(p: Vec3, n: Vec3, tile: f32) -> (f32, f32) {
    let (ax, ay, az) = (n.x.abs(), n.y.abs(), n.z.abs());
    let tile = if tile <= 0.0001 { 1.0 } else { tile };
    let (u, v) = if ay >= ax && ay >= az {
        (p.x, p.z)
    } else if ax >= az {
        (p.z, p.y)
    } else {
        (p.x, p.y)
    };
    (u / tile, v / tile)
}

/// Small helper to build a standalone mesh from a closure.
pub fn build_mesh<F: FnOnce(&mut MeshBuilder)>(f: F) -> Mesh {
    let mut mb = MeshBuilder::new();
    f(&mut mb);
    mb.into_mesh()
}
