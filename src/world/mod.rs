//! The Diplomat Peak Resort.
//!
//! One continuous building, built once when the menu appears:
//!
//! ```text
//!   y = 9.2   PENTHOUSE  ballroom (x -16..6, z -20..4)
//!                        terrace (x 6..22, z -14..4) -> gate corridor
//!   y = 4.6   F1         corridor (x -16..16, z -22..-18), machine room (x 16..23)
//!   y = 0     GROUND     porch (z 10..20), lobby (x -12..12, z -10..10)
//!                        stair hall (x 4..11, z -18..-10)
//! ```
//!
//! Everything is axis aligned so the collision solver stays exact.

pub mod corridor;
pub mod foyer;
pub mod penthouse;
pub mod props;

use bevy::prelude::*;

use crate::core::{AppState, Rng};
use crate::geom::MeshBuilder;
use crate::palette::Palette;

pub const GROUND: f32 = 0.0;
pub const F1: f32 = 4.6;
pub const F2: f32 = 9.2;

#[derive(Resource, Default)]
pub struct LevelBuilt(pub bool);

/// Accumulates geometry per material and the collision boxes of the level.
pub struct BuildCtx {
    pub batches: Vec<(Handle<StandardMaterial>, MeshBuilder)>,
    pub solids: Vec<(Vec3, Vec3)>,
    cur: Option<usize>,
}

impl BuildCtx {
    pub fn new() -> Self {
        Self {
            batches: Vec::new(),
            solids: Vec::new(),
            cur: None,
        }
    }

    pub fn mat(&mut self, m: &Handle<StandardMaterial>) {
        if let Some(i) = self.batches.iter().position(|(h, _)| h == m) {
            self.cur = Some(i);
        } else {
            self.batches.push((m.clone(), MeshBuilder::new()));
            self.cur = Some(self.batches.len() - 1);
        }
    }

    pub fn mb(&mut self) -> &mut MeshBuilder {
        let i = self.cur.expect("BuildCtx::mat must be called first");
        &mut self.batches[i].1
    }

    pub fn solid(&mut self, center: Vec3, size: Vec3) {
        self.solids.push((center, size * 0.5));
    }

    // ---- surfaces ---------------------------------------------------------
    pub fn floor(&mut self, x0: f32, z0: f32, x1: f32, z1: f32, y: f32, tile: f32) {
        self.mb().floor(x0, z0, x1, z1, y, tile);
    }

    pub fn ceiling(&mut self, x0: f32, z0: f32, x1: f32, z1: f32, y: f32, tile: f32) {
        self.mb().ceiling(x0, z0, x1, z1, y, tile);
    }

    pub fn slab_solid(&mut self, x0: f32, z0: f32, x1: f32, z1: f32, y: f32, thick: f32, tile: f32) {
        let c = Vec3::new((x0 + x1) * 0.5, y - thick * 0.5, (z0 + z1) * 0.5);
        let size = Vec3::new((x1 - x0).abs(), thick, (z1 - z0).abs());
        self.mb().box_all(c, size, tile);
        self.solid(c, size);
    }

    pub fn ceiling_solid(&mut self, x0: f32, z0: f32, x1: f32, z1: f32, y: f32, thick: f32, tile: f32) {
        let c = Vec3::new((x0 + x1) * 0.5, y + thick * 0.5, (z0 + z1) * 0.5);
        let size = Vec3::new((x1 - x0).abs(), thick, (z1 - z0).abs());
        self.mb().box_all(c, size, tile);
        self.solid(c, size);
    }

    /// Wall along Z (i.e. facing +-Z) with a rectangular opening.
    #[allow(clippy::too_many_arguments)]
    pub fn wall_z(
        &mut self,
        x0: f32,
        x1: f32,
        y0: f32,
        y1: f32,
        z: f32,
        th: f32,
        tile: f32,
        opening: Option<(f32, f32, f32)>,
    ) {
        let c = Vec3::new((x0 + x1) * 0.5, (y0 + y1) * 0.5, z);
        let size = Vec3::new((x1 - x0).abs(), (y1 - y0).abs(), th);
        if let Some((ox0, ox1, oy1)) = opening {
            let left = (ox0 - x0).abs();
            let right = (x1 - ox1).abs();
            if left > 0.01 {
                let sc = Vec3::new(x0 + (ox0 - x0) * 0.5, c.y, z);
                let ss = Vec3::new(left, size.y, th);
                self.mb().box_all(sc, ss, tile);
                self.solid(sc, ss);
            }
            if right > 0.01 {
                let sc = Vec3::new(ox1 + (x1 - ox1) * 0.5, c.y, z);
                let ss = Vec3::new(right, size.y, th);
                self.mb().box_all(sc, ss, tile);
                self.solid(sc, ss);
            }
            if (y1 - oy1).abs() > 0.01 {
                let sc = Vec3::new((ox0 + ox1) * 0.5, (oy1 + y1) * 0.5, z);
                let ss = Vec3::new((ox1 - ox0).abs(), (y1 - oy1).abs(), th);
                self.mb().box_all(sc, ss, tile);
                self.solid(sc, ss);
            }
        } else {
            self.mb().box_all(c, size, tile);
            self.solid(c, size);
        }
    }

    /// Wall along X (i.e. facing +-X) with a rectangular opening.
    #[allow(clippy::too_many_arguments)]
    pub fn wall_x(
        &mut self,
        z0: f32,
        z1: f32,
        y0: f32,
        y1: f32,
        x: f32,
        th: f32,
        tile: f32,
        opening: Option<(f32, f32, f32)>,
    ) {
        let c = Vec3::new(x, (y0 + y1) * 0.5, (z0 + z1) * 0.5);
        let size = Vec3::new(th, (y1 - y0).abs(), (z1 - z0).abs());
        if let Some((oz0, oz1, oy1)) = opening {
            let before = (oz0 - z0).abs();
            let after = (z1 - oz1).abs();
            if before > 0.01 {
                let sc = Vec3::new(x, c.y, z0 + (oz0 - z0) * 0.5);
                let ss = Vec3::new(th, size.y, before);
                self.mb().box_all(sc, ss, tile);
                self.solid(sc, ss);
            }
            if after > 0.01 {
                let sc = Vec3::new(x, c.y, oz1 + (z1 - oz1) * 0.5);
                let ss = Vec3::new(th, size.y, after);
                self.mb().box_all(sc, ss, tile);
                self.solid(sc, ss);
            }
            if (y1 - oy1).abs() > 0.01 {
                let sc = Vec3::new(x, (oy1 + y1) * 0.5, (oz0 + oz1) * 0.5);
                let ss = Vec3::new(th, (y1 - oy1).abs(), (oz1 - oz0).abs());
                self.mb().box_all(sc, ss, tile);
                self.solid(sc, ss);
            }
        } else {
            self.mb().box_all(c, size, tile);
            self.solid(c, size);
        }
    }

    pub fn box_at(&mut self, center: Vec3, size: Vec3, tile: f32) {
        self.mb().box_all(center, size, tile);
    }

    pub fn box_collide(&mut self, center: Vec3, size: Vec3, tile: f32) {
        self.mb().box_all(center, size, tile);
        self.solid(center, size);
    }

    /// Simple staircase going towards -Z between two heights.
    pub fn stairs(&mut self, x0: f32, x1: f32, z_start: f32, z_end: f32, y0: f32, y1: f32, steps: u32, tile: f32) {
        let steps = steps.max(1);
        let dz = (z_end - z_start) / steps as f32;
        let dy = (y1 - y0) / steps as f32;
        for i in 0..steps {
            let z0 = z_start + dz * i as f32;
            let z1 = z0 + dz;
            let top = y0 + dy * (i + 1) as f32;
            let c = Vec3::new((x0 + x1) * 0.5, top - 0.14, (z0 + z1) * 0.5);
            let size = Vec3::new((x1 - x0).abs(), 0.28, dz.abs());
            self.mb().box_all(c, size, tile);
            self.solid(c, size);
            // riser face
            let rc = Vec3::new((x0 + x1) * 0.5, top - 0.14, z0);
            let rs = Vec3::new((x1 - x0).abs(), 0.28, 0.04);
            let _ = (rc, rs);
        }
    }

    /// Sends every batch to the world as one mesh per material.
    pub fn finish(self, commands: &mut Commands, meshes: &mut Assets<Mesh>, name: &str) {
        for (mat, mb) in self.batches.into_iter() {
            if mb.is_empty() {
                continue;
            }
            let handle = meshes.add(mb.into_mesh());
            commands.spawn((
                Mesh3d(handle),
                MeshMaterial3d(mat),
                Transform::IDENTITY,
                Name::new(name.to_string()),
            ));
        }
        for (center, half) in self.solids.into_iter() {
            commands.spawn((
                Transform::from_translation(center),
                crate::collision::SolidBox { half },
                Name::new("collision"),
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// animated props
// ---------------------------------------------------------------------------

/// Slides an entity between two local translations (doors, gates).
#[derive(Component)]
pub struct Slide {
    pub from: Vec3,
    pub to: Vec3,
    pub t: f32,
    pub duration: f32,
    pub active: bool,
}

impl Slide {
    pub fn new(from: Vec3, to: Vec3, duration: f32) -> Self {
        Self {
            from,
            to,
            t: 0.0,
            duration,
            active: false,
        }
    }
}

/// Rotates an entity from one quaternion to another (swinging doors, clock hands).
#[derive(Component)]
pub struct Swing {
    pub from: Quat,
    pub to: Quat,
    pub t: f32,
    pub duration: f32,
    pub active: bool,
}

impl Swing {
    pub fn new(from: Quat, to: Quat, duration: f32) -> Self {
        Self {
            from,
            to,
            t: 0.0,
            duration,
            active: false,
        }
    }
}

/// A light that breathes or flickers, with a per-entity phase so a room full
/// of bulbs never pulses in sync.
#[derive(Component)]
pub struct Flicker {
    pub base: f32,
    pub amp: f32,
    pub speed: f32,
    pub phase: f32,
    pub dying: bool,
}

// ---------------------------------------------------------------------------
// markers: how the story code finds the moving parts
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct SafeDoor;
#[derive(Component)]
pub struct SafeKey;
#[derive(Component)]
pub struct CorridorDoor;
#[derive(Component)]
pub struct StairDoor;
#[derive(Component)]
pub struct LeverHandle(pub usize);
#[derive(Component)]
pub struct ClockHand {
    pub hour: bool,
    /// Orientation of the clock face itself; the hands spin in front of it.
    pub base: Quat,
}
#[derive(Component)]
pub struct SecretDoor;
#[derive(Component)]
pub struct ExitGate;
#[derive(Component)]
pub struct GateLed;
#[derive(Component)]
pub struct EmergencySwitch;
#[derive(Component)]
pub struct JumpscareTrigger;
#[derive(Component)]
pub struct ElevatorLight;
#[derive(Component)]
pub struct DoorBlocker;

#[derive(Component)]
pub struct GateBlocker;

#[derive(Component)]
pub struct SecretBlocker;

#[derive(Component)]
pub struct EntryDoor {
    pub left: bool,
}

/// A wall along Z with any number of rectangular holes `(x0, x1, y0, y1)`.
#[allow(clippy::too_many_arguments)]
pub fn wall_z_win(
    b: &mut BuildCtx,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    z: f32,
    th: f32,
    tile: f32,
    holes: &[(f32, f32, f32, f32)],
) {
    let mut holes = holes.to_vec();
    holes.sort_by(|a, c| a.0.partial_cmp(&c.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut cursor = x0.min(x1);
    let end = x0.max(x1);
    for (hx0, hx1, hy0, hy1) in holes {
        if hx0 > cursor + 0.01 {
            b.wall_z(cursor, hx0, y0, y1, z, th, tile, None);
        }
        if hy0 > y0 + 0.01 {
            b.wall_z(hx0, hx1, y0, hy0, z, th, tile, None);
        }
        if hy1 < y1 - 0.01 {
            b.wall_z(hx0, hx1, hy1, y1, z, th, tile, None);
        }
        cursor = cursor.max(hx1);
    }
    if end > cursor + 0.01 {
        b.wall_z(cursor, end, y0, y1, z, th, tile, None);
    }
}

/// A wall along X with any number of rectangular holes `(z0, z1, y0, y1)`.
#[allow(clippy::too_many_arguments)]
pub fn wall_x_win(
    b: &mut BuildCtx,
    z0: f32,
    z1: f32,
    y0: f32,
    y1: f32,
    x: f32,
    th: f32,
    tile: f32,
    holes: &[(f32, f32, f32, f32)],
) {
    let mut holes = holes.to_vec();
    holes.sort_by(|a, c| a.0.partial_cmp(&c.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut cursor = z0.min(z1);
    let end = z0.max(z1);
    for (hz0, hz1, hy0, hy1) in holes {
        if hz0 > cursor + 0.01 {
            b.wall_x(cursor, hz0, y0, y1, x, th, tile, None);
        }
        if hy0 > y0 + 0.01 {
            b.wall_x(hz0, hz1, y0, hy0, x, th, tile, None);
        }
        if hy1 < y1 - 0.01 {
            b.wall_x(hz0, hz1, hy1, y1, x, th, tile, None);
        }
        cursor = cursor.max(hz1);
    }
    if end > cursor + 0.01 {
        b.wall_x(cursor, end, y0, y1, x, th, tile, None);
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelBuilt>().add_systems(
            OnEnter(AppState::Title),
            build_level.run_if(not_built),
        )
        .add_systems(Update, (animate_slides, animate_swings, flicker_lights, sway_props));
    }
}

fn not_built(built: Res<LevelBuilt>) -> bool {
    !built.0
}

fn build_level(
    mut commands: Commands,
    assets: Res<crate::core::GameAssets>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: ResMut<LevelBuilt>,
    mut rng: ResMut<Rng>,
    mut ambient: ResMut<AmbientLight>,
) {
    if built.0 {
        return;
    }
    ambient.color = Color::srgb(0.42, 0.50, 0.62);
    ambient.brightness = 26.0;

    let mut b = BuildCtx::new();
    foyer::build(&mut commands, &mut b, &palette, &mut meshes, &assets, &mut rng);
    corridor::build(&mut commands, &mut b, &palette, &mut meshes, &assets, &mut rng);
    penthouse::build(&mut commands, &mut b, &palette, &mut meshes, &assets, &mut rng);
    b.finish(&mut commands, &mut meshes, "architecture");

    built.0 = true;
}

fn animate_slides(time: Res<Time>, mut q: Query<(&mut Transform, &mut Slide)>) {
    let dt = time.delta_secs();
    for (mut tf, mut slide) in q.iter_mut() {
        if !slide.active {
            continue;
        }
        if slide.t < 1.0 {
            slide.t = (slide.t + dt / slide.duration.max(0.01)).min(1.0);
            let e = ease(slide.t);
            tf.translation = slide.from.lerp(slide.to, e);
        }
    }
}

fn animate_swings(time: Res<Time>, mut q: Query<(&mut Transform, &mut Swing)>) {
    let dt = time.delta_secs();
    for (mut tf, mut swing) in q.iter_mut() {
        if !swing.active {
            continue;
        }
        if swing.t < 1.0 {
            swing.t = (swing.t + dt / swing.duration.max(0.01)).min(1.0);
            let e = ease(swing.t);
            tf.rotation = swing.from.slerp(swing.to, e);
        }
    }
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn flicker_lights(
    time: Res<Time>,
    mut q: Query<(&mut PointLight, &Flicker)>,
    mut spot: Query<(&mut SpotLight, &Flicker), Without<PointLight>>,
) {
    let t = time.elapsed_secs();
    for (mut light, fl) in q.iter_mut() {
        let n = (t * fl.speed + fl.phase).sin() * 0.5 + 0.5;
        let mut k = 1.0 + (n - 0.5) * fl.amp;
        if fl.dying {
            let stutter = ((t * 13.0 + fl.phase).sin() * (t * 7.3 + fl.phase).cos()).abs();
            k *= 0.45 + 0.55 * stutter;
        }
        light.intensity = fl.base * k.max(0.0);
    }
    for (mut light, fl) in spot.iter_mut() {
        let n = (t * fl.speed + fl.phase).sin() * 0.5 + 0.5;
        light.intensity = fl.base * (1.0 + (n - 0.5) * fl.amp);
    }
}

/// Very slow sway for chandeliers and hanging things.
#[derive(Component)]
pub struct Sway {
    pub amount: f32,
    pub speed: f32,
}

fn sway_props(time: Res<Time>, mut q: Query<(&mut Transform, &Sway)>) {
    let t = time.elapsed_secs();
    for (mut tf, sway) in q.iter_mut() {
        let a = (t * sway.speed).sin() * sway.amount;
        let b = (t * sway.speed * 0.77 + 1.3).cos() * sway.amount * 0.6;
        tf.rotation = Quat::from_euler(EulerRot::XYZ, a, 0.0, b);
    }
}
