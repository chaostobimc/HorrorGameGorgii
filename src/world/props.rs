//! Furniture, fixtures and set dressing.
//!
//! Each prop is a small composite of primitives with its own merged mesh, so
//! the hotel still draws in a few dozen calls while looking hand built.

use std::cell::Cell;

use bevy::prelude::*;

use crate::geom::MeshBuilder;
use crate::palette::Palette;

use super::{Flicker, Sway};

thread_local! {
    /// While the level editor or the level file spawns a prop, every entity it
    /// creates is parented to this root, so the prop can be moved, rotated and
    /// deleted as one object. `None` while the hand built hotel is generated.
    static SPAWN_PARENT: Cell<Option<Entity>> = Cell::new(None);
    /// The prop currently being built, so every piece can be tagged.
    static CURRENT_GROUP: Cell<Option<u32>> = Cell::new(None);
    static PART_COUNTER: Cell<u32> = Cell::new(0);
    static GROUP_COUNTER: Cell<u32> = Cell::new(0);
}

/// One piece of a hand built prop (a door leaf, a chair leg, a lamp glass).
///
/// The numbers are handed out in build order and the hotel is built the same
/// way in the editor and in the game, so `HotelPart(12)` means the same piece
/// in both programs. That is what makes "replace this door with my model"
/// survive a save and a restart.
#[derive(Component, Clone, Copy, Debug)]
pub struct HotelPart {
    pub id: u32,
    pub group: u32,
}

/// A whole hand built prop: the editor moves and deletes these as one object.
#[derive(Component, Clone, Copy, Debug)]
pub struct HotelGroup(pub u32);

/// Starts a new prop group around `f`.
///
/// Does nothing while the level editor spawns a prop, because the editor
/// already packs everything under one root.
pub fn with_group<R>(commands: &mut Commands, f: impl FnOnce(&mut Commands) -> R) -> R {
    if SPAWN_PARENT.with(|slot| slot.get()).is_some() {
        return f(commands);
    }
    let group = GROUP_COUNTER.with(|slot| {
        let value = slot.get();
        slot.set(value + 1);
        value
    });
    let root = commands
        .spawn((
            Transform::IDENTITY,
            Visibility::default(),
            HotelGroup(group),
        ))
        .id();
    let previous = CURRENT_GROUP.with(|slot| slot.replace(Some(group)));
    let out = with_parent(Some(root), || f(commands));
    CURRENT_GROUP.with(|slot| slot.set(previous));
    out
}

/// Hands out the numbers again: called at the start of every level build.
pub fn reset_parts() {
    PART_COUNTER.with(|slot| slot.set(0));
    GROUP_COUNTER.with(|slot| slot.set(0));
    CURRENT_GROUP.with(|slot| slot.set(None));
}

/// Runs `f` with all props spawned inside it attached to `parent`.
pub fn with_parent<R>(parent: Option<Entity>, f: impl FnOnce() -> R) -> R {
    let previous = SPAWN_PARENT.with(|slot| slot.replace(parent));
    let out = f();
    SPAWN_PARENT.with(|slot| slot.set(previous));
    out
}

pub(crate) fn mesh_of<F: FnOnce(&mut MeshBuilder)>(meshes: &mut Assets<Mesh>, f: F) -> Handle<Mesh> {
    let mut mb = MeshBuilder::new();
    f(&mut mb);
    meshes.add(mb.into_mesh())
}

/// Spawns anything inside the current prop group, so the editor can pick it up.
pub(crate) fn spawn_any<B: Bundle>(commands: &mut Commands, bundle: B) -> Entity {
    let group = CURRENT_GROUP.with(|slot| slot.get());
    let entity = match SPAWN_PARENT.with(|slot| slot.get()) {
        Some(parent) => {
            let mut spawned: Option<Entity> = None;
            commands.entity(parent).with_children(|child| {
                spawned = Some(child.spawn(bundle).id());
            });
            spawned.unwrap()
        }
        None => commands.spawn(bundle).id(),
    };
    if let Some(group) = group {
        let id = PART_COUNTER.with(|slot| {
            let value = slot.get();
            slot.set(value + 1);
            value
        });
        commands.entity(entity).insert(HotelPart { id, group });
    }
    entity
}

pub(crate) fn spawn_prop(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    mat: Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    spawn_any(
        commands,
        (
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            transform,
            Visibility::default(),
        ),
    )
}

/// Text that lives in the world (door numbers, signs, the clock readout).
pub fn text3d(
    commands: &mut Commands,
    font: Handle<Font>,
    pos: Vec3,
    yaw: f32,
    cap_height: f32,
    text: &str,
    color: Color,
) -> Entity {
    // Text2d is sized in world units; render at a fixed atlas size and scale.
    let scale = (cap_height / 48.0).max(0.0005);
    commands
        .spawn((
            Text2d::new(text.to_string()),
            TextFont {
                font,
                font_size: 48.0,
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(pos.x, pos.y, pos.z)
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(Vec3::splat(scale)),
        ))
        .id()
}

// ---------------------------------------------------------------------------
// lighting fixtures
// ---------------------------------------------------------------------------

pub fn chandelier(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    radius: f32,
    lit: bool,
) {
    with_group(commands, |commands| {
        // chain + stem
        let chain = mesh_of(meshes, |mb| {
            mb.cylinder(Vec3::new(0.0, 0.55, 0.0), 0.025, 1.1, 8, 1.0, false);
            mb.cylinder(Vec3::new(0.0, 0.0, 0.0), 0.09, 0.22, 12, 1.0, true);
            mb.ring(Vec3::new(0.0, -0.10, 0.0), radius * 0.2, radius, 32, true, 1.0);
            mb.ring(Vec3::new(0.0, -0.10, 0.0), radius * 0.2, radius, 32, false, 1.0);
            mb.ring(Vec3::new(0.0, -0.42, 0.0), radius * 0.12, radius * 0.62, 24, true, 1.0);
            // arms
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                let p = Vec3::new(a.cos() * radius * 0.86, -0.10, a.sin() * radius * 0.86);
                mb.bar(Vec3::ZERO, p, 0.035, 1.0);
                mb.cylinder(p + Vec3::Y * 0.09, 0.03, 0.18, 8, 1.0, true);
            }
        });
        let e = spawn_prop(
            commands,
            chain,
            pal.brass_dark.clone(),
            Transform::from_translation(pos + Vec3::Y * 1.0),
        );
        commands.entity(e).insert(Sway {
            amount: 0.012,
            speed: 0.35,
        });

        // candle bulbs
        let bulbs = mesh_of(meshes, |mb| {
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                let p = Vec3::new(a.cos() * radius * 0.86, 0.24, a.sin() * radius * 0.86);
                mb.cylinder(p, 0.045, 0.2, 8, 1.0, true);
            }
        });
        spawn_prop(
            commands,
            bulbs,
            pal.bulb.clone(),
            Transform::from_translation(pos + Vec3::Y * 1.0),
        );

        // crystals
        let crystals = mesh_of(meshes, |mb| {
            let mut seed = 0.0f32;
            for i in 0..14 {
                let a = i as f32 / 14.0 * std::f32::consts::TAU;
                seed += 1.7;
                let p = Vec3::new(
                    a.cos() * radius * 0.75,
                    -0.55 - (seed.sin().abs() * 0.35),
                    a.sin() * radius * 0.75,
                );
                mb.box_all(p, Vec3::splat(0.05), 1.0);
            }
        });
        spawn_prop(
            commands,
            crystals,
            pal.ivory.clone(),
            Transform::from_translation(pos + Vec3::Y * 1.0),
        );

        if lit {
            spawn_any(
                commands,
                (
                PointLight {
                    color: Color::srgb(1.0, 0.80, 0.55),
                    intensity: 5_500.0,
                    range: 16.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_translation(pos + Vec3::Y * 0.9),
                Flicker {
                    base: 5_500.0,
                    amp: 0.25,
                    speed: 2.1,
                    phase: 0.4,
                    dying: true,
                },
            ));
        }

    });
}

pub fn sconce(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    yaw: f32,
    lit: bool,
    dying: bool,
) {
    with_group(commands, |commands| {
        let holder = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(0.0, 0.0, 0.06), Vec3::new(0.10, 0.26, 0.03), 1.0);
            mb.cylinder(Vec3::new(0.0, 0.02, 0.16), 0.035, 0.16, 8, 1.0, true);
            mb.bar(
                Vec3::new(0.0, -0.10, 0.02),
                Vec3::new(0.0, 0.02, 0.16),
                0.025,
                1.0,
            );
        });
        let rot = Quat::from_rotation_y(yaw);
        spawn_prop(
            commands,
            holder,
            pal.brass_dark.clone(),
            Transform::from_translation(pos).with_rotation(rot),
        );
        let shade = mesh_of(meshes, |mb| {
            mb.cylinder(Vec3::new(0.0, 0.16, 0.16), 0.11, 0.16, 12, 1.0, false);
        });
        spawn_prop(
            commands,
            shade,
            if lit { pal.bulb.clone() } else { pal.bulb_dim.clone() },
            Transform::from_translation(pos).with_rotation(rot),
        );
        if lit {
            spawn_any(
                commands,
                (
                PointLight {
                    color: Color::srgb(1.0, 0.78, 0.52),
                    intensity: 2_600.0,
                    range: 9.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_translation(pos + rot * Vec3::new(0.0, 0.16, 0.16)),
                Flicker {
                    base: 2_600.0,
                    amp: 0.18,
                    speed: 1.4,
                    phase: pos.x * 0.7 + pos.z * 1.3,
                    dying,
                },
            ));
        }

    });
}

/// Ugly fluorescent tube, used in the machine room.
pub fn fluor_tube(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    lit: bool,
) {
    with_group(commands, |commands| {
        let fixture = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::ZERO, Vec3::new(1.5, 0.10, 0.22), 1.0);
        });
        spawn_prop(
            commands,
            fixture,
            pal.metal.clone(),
            Transform::from_translation(pos),
        );
        let tube = mesh_of(meshes, |mb| {
            mb.cylinder(Vec3::new(0.0, -0.08, 0.0), 0.045, 1.4, 10, 1.0, false);
        });
        let e = spawn_prop(
            commands,
            tube,
            if lit { pal.bulb.clone() } else { pal.bulb_dim.clone() },
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(
                std::f32::consts::FRAC_PI_2,
            )),
        );
        let _ = e;
        if lit {
            spawn_any(
                commands,
                (
                PointLight {
                    color: Color::srgb(0.82, 0.92, 0.88),
                    intensity: 4_200.0,
                    range: 11.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_translation(pos + Vec3::NEG_Y * 0.1),
                Flicker {
                    base: 4_200.0,
                    amp: 0.45,
                    speed: 6.3,
                    phase: 1.7,
                    dying: true,
                },
            ));
        }

    });
}

// ---------------------------------------------------------------------------
// plants
// ---------------------------------------------------------------------------

pub(crate) fn leaf_card_mesh(meshes: &mut Assets<Mesh>, size: f32, cards: usize) -> Handle<Mesh> {
    mesh_of(meshes, |mb| {
        for i in 0..cards {
            let a = i as f32 / cards as f32 * std::f32::consts::TAU;
            let dir = Vec3::new(a.cos(), 0.0, a.sin());
            let side = Vec3::new(-a.sin(), 0.0, a.cos());
            let w = size * 0.5;
            let h = size;
            let base = Vec3::new(0.0, -h * 0.5, 0.0);
            let c0 = base - side * w;
            let c1 = base + side * w;
            let c2 = base + side * w * 0.25 + Vec3::Y * h;
            let c3 = base - side * w * 0.25 + Vec3::Y * h;
            let _ = dir;
            mb.quad_uv(c0, c1, c2, c3, [Vec2::new(0.0, 1.0), Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.0)]);
        }
    })
}

pub fn potted_plant(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    scale: f32,
    variant: usize,
) {
    with_group(commands, |commands| {
        let pot = mesh_of(meshes, |mb| {
            mb.cylinder(Vec3::new(0.0, 0.26, 0.0), 0.30, 0.52, 16, 1.0, true);
            mb.ring(Vec3::new(0.0, 0.52, 0.0), 0.22, 0.32, 16, true, 1.0);
        });
        spawn_prop(
            commands,
            pot,
            pal.terracotta.clone(),
            Transform::from_translation(pos).with_scale(Vec3::splat(scale)),
        );
        let card = leaf_card_mesh(meshes, 1.9, 3);
        let mat = if variant % 2 == 0 {
            pal.leaf.clone()
        } else {
            pal.leaf2.clone()
        };
        spawn_prop(
            commands,
            card,
            mat.clone(),
            Transform::from_translation(pos + Vec3::Y * 1.35 * scale)
                .with_scale(Vec3::splat(scale * 1.15))
                .with_rotation(Quat::from_rotation_y(variant as f32 * 1.1)),
        );
        // a second, offset card gives real volume
        spawn_prop(
            commands,
            leaf_card_mesh(meshes, 1.5, 2),
            mat,
            Transform::from_translation(pos + Vec3::new(0.1, 1.05, -0.1) * scale)
                .with_scale(Vec3::splat(scale))
                .with_rotation(Quat::from_rotation_y(variant as f32 * 2.3 + 0.7)),
        );

    });
}

pub fn hanging_vines(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    tex: Handle<Image>,
    pos: Vec3,
    count: usize,
) {
    with_group(commands, |commands| {
        let _ = tex;
        let mesh = mesh_of(meshes, |mb| {
            for i in 0..count {
                let a = i as f32 * 1.7;
                let off = Vec3::new(a.cos() * 0.5, 0.0, a.sin() * 0.5);
                let h = 1.2 + (i as f32 * 0.37).sin().abs() * 1.4;
                let w = 0.22;
                let c0 = pos + off + Vec3::new(-w, 0.0, 0.0);
                let c1 = pos + off + Vec3::new(w, 0.0, 0.0);
                let c2 = pos + off + Vec3::new(w, -h, 0.0);
                let c3 = pos + off + Vec3::new(-w, -h, 0.0);
                mb.quad_uv(
                    c0,
                    c1,
                    c2,
                    c3,
                    [
                        Vec2::new(0.0, 0.0),
                        Vec2::new(1.0, 0.0),
                        Vec2::new(1.0, 1.0),
                        Vec2::new(0.0, 1.0),
                    ],
                );
                let c0 = pos + off + Vec3::new(0.0, 0.0, -w);
                let c1 = pos + off + Vec3::new(0.0, 0.0, w);
                let c2 = pos + off + Vec3::new(0.0, -h, w);
                let c3 = pos + off + Vec3::new(0.0, -h, -w);
                mb.quad_uv(
                    c0,
                    c1,
                    c2,
                    c3,
                    [
                        Vec2::new(0.0, 0.0),
                        Vec2::new(1.0, 0.0),
                        Vec2::new(1.0, 1.0),
                        Vec2::new(0.0, 1.0),
                    ],
                );
            }
        });
        spawn_prop(
            commands,
            mesh,
            pal.vine.clone(),
            Transform::IDENTITY,
        );

    });
}

// ---------------------------------------------------------------------------
// seating and tables
// ---------------------------------------------------------------------------

pub fn sofa(commands: &mut Commands, meshes: &mut Assets<Mesh>, pal: &Palette, pos: Vec3, yaw: f32) {
    with_group(commands, |commands| {
        let body = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(0.0, 0.22, 0.0), Vec3::new(2.1, 0.44, 0.9), 1.0);
            mb.box_all(Vec3::new(0.0, 0.62, -0.34), Vec3::new(2.1, 0.55, 0.22), 1.0);
            mb.box_all(Vec3::new(-0.98, 0.50, 0.0), Vec3::new(0.2, 0.5, 0.9), 1.0);
            mb.box_all(Vec3::new(0.98, 0.50, 0.0), Vec3::new(0.2, 0.5, 0.9), 1.0);
            for x in [-0.9, 0.9] {
                for z in [-0.32, 0.32] {
                    mb.cylinder(Vec3::new(x, 0.06, z), 0.05, 0.12, 8, 1.0, false);
                }
            }
        });
        spawn_prop(
            commands,
            body,
            pal.velvet_green.clone(),
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
        );

    });
}

pub fn armchair(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    yaw: f32,
) {
    with_group(commands, |commands| {
        let body = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(0.0, 0.24, 0.0), Vec3::new(0.9, 0.44, 0.85), 1.0);
            mb.box_all(Vec3::new(0.0, 0.62, -0.34), Vec3::new(0.9, 0.5, 0.18), 1.0);
            mb.box_all(Vec3::new(-0.42, 0.48, 0.0), Vec3::new(0.12, 0.5, 0.85), 1.0);
            mb.box_all(Vec3::new(0.42, 0.48, 0.0), Vec3::new(0.12, 0.5, 0.85), 1.0);
            for x in [-0.36, 0.36] {
                for z in [-0.3, 0.3] {
                    mb.cylinder(Vec3::new(x, 0.08, z), 0.04, 0.16, 8, 1.0, false);
                }
            }
        });
        spawn_prop(
            commands,
            body,
            pal.velvet.clone(),
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
        );

    });
}

pub fn coffee_table(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    yaw: f32,
) {
    with_group(commands, |commands| {
        let body = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(0.0, 0.42, 0.0), Vec3::new(1.4, 0.06, 0.75), 1.0);
            for x in [-0.62, 0.62] {
                for z in [-0.3, 0.3] {
                    mb.cylinder(Vec3::new(x, 0.21, z), 0.035, 0.42, 8, 1.0, false);
                }
            }
        });
        spawn_prop(
            commands,
            body,
            pal.wood.clone(),
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
        );

    });
}

pub fn round_table(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    cloth: bool,
) {
    with_group(commands, |commands| {
        let body = mesh_of(meshes, |mb| {
            mb.cylinder(Vec3::new(0.0, 0.74, 0.0), 0.62, 0.06, 20, 1.0, true);
            mb.cylinder(Vec3::new(0.0, 0.37, 0.0), 0.08, 0.74, 10, 1.0, false);
            mb.cylinder(Vec3::new(0.0, 0.03, 0.0), 0.34, 0.06, 14, 1.0, true);
        });
        spawn_prop(
            commands,
            body,
            pal.wood.clone(),
            Transform::from_translation(pos),
        );
        if cloth {
            let cloth_mesh = mesh_of(meshes, |mb| {
                mb.cylinder(Vec3::new(0.0, 0.62, 0.0), 0.66, 0.32, 20, 1.0, false);
                mb.disc(Vec3::new(0.0, 0.78, 0.0), 0.66, 20, true, 1.0);
            });
            spawn_prop(
                commands,
                cloth_mesh,
                pal.cloth.clone(),
                Transform::from_translation(pos),
            );
        }

    });
}

pub fn chair(commands: &mut Commands, meshes: &mut Assets<Mesh>, pal: &Palette, pos: Vec3, yaw: f32) {
    with_group(commands, |commands| {
        let body = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(0.0, 0.45, 0.0), Vec3::new(0.44, 0.05, 0.44), 1.0);
            mb.box_all(Vec3::new(0.0, 0.72, -0.20), Vec3::new(0.44, 0.5, 0.05), 1.0);
            for x in [-0.19, 0.19] {
                for z in [-0.19, 0.19] {
                    mb.cylinder(Vec3::new(x, 0.22, z), 0.022, 0.44, 6, 1.0, false);
                }
            }
        });
        spawn_prop(
            commands,
            body,
            pal.wood_h.clone(),
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
        );

    });
}

/// Chair pushed over / knocked down - used to make the ballroom feel wrong.
pub fn fallen_chair(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    yaw: f32,
) {
    with_group(commands, |commands| {
        let body = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(0.0, 0.05, 0.0), Vec3::new(0.44, 0.05, 0.44), 1.0);
            mb.box_all(Vec3::new(0.0, 0.05, 0.24), Vec3::new(0.44, 0.5, 0.05), 1.0);
        });
        spawn_prop(
            commands,
            body,
            pal.wood_h.clone(),
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
        );

    });
}

// ---------------------------------------------------------------------------
// architectural fittings
// ---------------------------------------------------------------------------

pub fn rug(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    center: Vec3,
    size: Vec2,
    yaw: f32,
) {
    with_group(commands, |commands| {
        let mesh = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::ZERO, Vec3::new(size.x, 0.012, size.y), 1.0);
        });
        spawn_prop(
            commands,
            mesh,
            pal.carpet.clone(),
            Transform::from_translation(center + Vec3::Y * 0.012)
                .with_rotation(Quat::from_rotation_y(yaw)),
        );

    });
}

#[allow(clippy::too_many_arguments)]
pub fn window(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    center: Vec3,
    width: f32,
    height: f32,
    yaw: f32,
) {
    with_group(commands, |commands| {
        let rot = Quat::from_rotation_y(yaw);
        let frame = mesh_of(meshes, |mb| {
            let t = 0.09;
            mb.box_all(Vec3::new(0.0, height * 0.5, 0.0), Vec3::new(width + t * 2.0, t, 0.16), 1.0);
            mb.box_all(Vec3::new(0.0, -height * 0.5, 0.0), Vec3::new(width + t * 2.0, t, 0.16), 1.0);
            mb.box_all(Vec3::new(-width * 0.5, 0.0, 0.0), Vec3::new(t, height, 0.16), 1.0);
            mb.box_all(Vec3::new(width * 0.5, 0.0, 0.0), Vec3::new(t, height, 0.16), 1.0);
            mb.box_all(Vec3::ZERO, Vec3::new(0.06, height, 0.10), 1.0);
            mb.box_all(Vec3::new(0.0, height * 0.22, 0.0), Vec3::new(width, 0.05, 0.08), 1.0);
        });
        spawn_prop(
            commands,
            frame,
            pal.wood_h.clone(),
            Transform::from_translation(center).with_rotation(rot),
        );
        let glass = mesh_of(meshes, |mb| {
            mb.quad_uv(
                Vec3::new(-width * 0.5, -height * 0.5, 0.0),
                Vec3::new(width * 0.5, -height * 0.5, 0.0),
                Vec3::new(width * 0.5, height * 0.5, 0.0),
                Vec3::new(-width * 0.5, height * 0.5, 0.0),
                [
                    Vec2::new(0.0, 1.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(0.0, 0.0),
                ],
            );
        });
        spawn_prop(
            commands,
            glass.clone(),
            pal.glass.clone(),
            Transform::from_translation(center).with_rotation(rot),
        );
        // rain running down the outside of the pane
        spawn_prop(
            commands,
            glass,
            pal.rain_glass.clone(),
            Transform::from_translation(center + rot * Vec3::new(0.0, 0.0, 0.06)).with_rotation(rot),
        );

    });
}

#[allow(clippy::too_many_arguments)]
pub fn curtain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    center: Vec3,
    width: f32,
    height: f32,
    yaw: f32,
) {
    with_group(commands, |commands| {
        let rot = Quat::from_rotation_y(yaw);
        let rod = mesh_of(meshes, |mb| {
            mb.cylinder(Vec3::new(0.0, height * 0.5 + 0.12, 0.0), 0.035, width + 0.3, 8, 1.0, false);
        });
        spawn_prop(
            commands,
            rod,
            pal.brass_dark.clone(),
            Transform::from_translation(center).with_rotation(rot * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        );
        // folds
        let folds = mesh_of(meshes, |mb| {
            let n = 7;
            for i in 0..n {
                let t = i as f32 / (n - 1) as f32;
                let x = -width * 0.5 + t * width;
                let w = width / n as f32 * 0.9;
                let bulge = 0.06 + (i as f32 * 1.7).sin().abs() * 0.05;
                mb.box_all(
                    Vec3::new(x, 0.0, bulge),
                    Vec3::new(w, height, 0.04 + bulge),
                    1.0,
                );
            }
        });
        spawn_prop(
            commands,
            folds,
            pal.velvet.clone(),
            Transform::from_translation(center).with_rotation(rot),
        );

    });
}

pub fn railing(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    from: Vec3,
    to: Vec3,
    height: f32,
) {
    with_group(commands, |commands| {
        let mesh = mesh_of(meshes, |mb| {
            let len = (to - from).length();
            let dir = (to - from).normalize_or_zero();
            let n = (len / 1.1).max(1.0) as usize;
            mb.bar(from + Vec3::Y * height, to + Vec3::Y * height, 0.06, 1.0);
            mb.bar(from + Vec3::Y * (height * 0.5), to + Vec3::Y * (height * 0.5), 0.03, 1.0);
            for i in 0..=n {
                let p = from + dir * (len * i as f32 / n as f32);
                mb.cylinder(p + Vec3::Y * (height * 0.5), 0.022, height, 6, 1.0, false);
            }
        });
        spawn_prop(commands, mesh, pal.black_metal.clone(), Transform::IDENTITY);

    });
}

/// A door leaf plus its frame. Returns the leaf entity so it can swing.
pub fn door(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    hinge: Vec3,
    width: f32,
    height: f32,
    yaw: f32,
    number: Option<&str>,
    font: Option<Handle<Font>>,
) -> Entity {
    with_group(commands, |commands| {
        let frame = mesh_of(meshes, |mb| {
            let t = 0.10;
            let rot = Quat::from_rotation_y(yaw);
            // `hinge` is the left jamb: the frame sits from there to the right, so
            // it lines up with the leaf instead of hanging half a door beside it.
            let off = |v: Vec3| hinge + rot * v;
            mb.box_all(off(Vec3::new(-t * 0.5, height * 0.5, 0.0)), Vec3::new(t, height + t, 0.30), 1.0);
            mb.box_all(off(Vec3::new(width + t * 0.5, height * 0.5, 0.0)), Vec3::new(t, height + t, 0.30), 1.0);
            mb.box_all(off(Vec3::new(width * 0.5, height + t * 0.5, 0.0)), Vec3::new(width + t * 2.0, t, 0.30), 1.0);
        });
        spawn_prop(
            commands,
            frame,
            pal.trim.clone(),
            Transform::IDENTITY,
        );

        let leaf = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(width * 0.5, height * 0.5, 0.0), Vec3::new(width, height, 0.06), 1.0);
            // panels
            for py in [height * 0.30, height * 0.72] {
                mb.box_all(
                    Vec3::new(width * 0.5, py, 0.045),
                    Vec3::new(width * 0.72, height * 0.30, 0.012),
                    1.0,
                );
            }
            // handle
            mb.cylinder(Vec3::new(width * 0.88, height * 0.48, 0.06), 0.028, 0.07, 10, 1.0, true);
        });
        let e = spawn_prop(
            commands,
            leaf,
            pal.wood_h.clone(),
            Transform::from_translation(hinge).with_rotation(Quat::from_rotation_y(yaw)),
        );
        if let (Some(n), Some(font)) = (number, font) {
            let rot = Quat::from_rotation_y(yaw);
            let pos = hinge + rot * Vec3::new(width * 0.5, height * 0.80, 0.05);
            text3d(commands, font, pos, yaw, 0.13, n, Color::srgba(0.82, 0.78, 0.68, 0.95));
        }
        e

    })
}

pub fn grand_piano(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    pos: Vec3,
    yaw: f32,
) {
    with_group(commands, |commands| {
        let rot = Quat::from_rotation_y(yaw);
        let body = mesh_of(meshes, |mb| {
            // body: three overlapping boxes approximate the curved case
            mb.box_all(Vec3::new(0.0, 0.95, 0.0), Vec3::new(1.5, 0.32, 2.4), 1.0);
            mb.box_all(Vec3::new(0.55, 0.95, -0.4), Vec3::new(1.1, 0.32, 1.5), 1.0);
            mb.box_all(Vec3::new(-0.55, 0.95, 0.2), Vec3::new(0.9, 0.32, 1.9), 1.0);
            // lid
            mb.box_all(Vec3::new(0.0, 1.14, 0.0), Vec3::new(1.55, 0.04, 2.45), 1.0);
            // keyboard shelf
            mb.box_all(Vec3::new(0.0, 0.86, 1.32), Vec3::new(1.5, 0.12, 0.34), 1.0);
            // legs
            for (x, z) in [(-0.6, 1.0), (0.6, 1.0), (-0.5, -1.0)] {
                mb.cylinder(Vec3::new(x, 0.42, z), 0.07, 0.84, 10, 1.0, false);
            }
            // pedal lyre
            mb.box_all(Vec3::new(0.0, 0.30, 0.75), Vec3::new(0.18, 0.6, 0.10), 1.0);
        });
        spawn_prop(
            commands,
            body,
            pal.black_metal.clone(),
            Transform::from_translation(pos).with_rotation(rot),
        );
        let keys = mesh_of(meshes, |mb| {
            // wound so the normal faces up (the key material is single sided)
            mb.quad_uv(
                Vec3::new(-0.68, 0.94, 1.62),
                Vec3::new(0.68, 0.94, 1.62),
                Vec3::new(0.68, 0.94, 1.36),
                Vec3::new(-0.68, 0.94, 1.36),
                [
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                ],
            );
        });
        spawn_prop(
            commands,
            keys,
            pal.piano_keys.clone(),
            Transform::from_translation(pos).with_rotation(rot),
        );
        // bench
        let bench = mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(0.0, 0.5, 2.15), Vec3::new(0.9, 0.08, 0.36), 1.0);
            for x in [-0.38, 0.38] {
                mb.cylinder(Vec3::new(x, 0.25, 2.15), 0.03, 0.5, 8, 1.0, false);
            }
        });
        spawn_prop(
            commands,
            bench,
            pal.wood.clone(),
            Transform::from_translation(pos).with_rotation(rot),
        );

    });
}
