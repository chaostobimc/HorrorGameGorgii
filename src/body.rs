//! The player's own body, seen from the first person camera.
//!
//! Everything here is parented to the camera, so the body follows the view.
//! The animation is procedural: a walk cycle, idle breathing and one
//! interaction pose that fires whenever something in the world is used
//! (a door, a pickup, a lever ...). No animation assets needed.

use bevy::prelude::*;

use crate::core::AppState;
use crate::interact::Interacted;
use crate::palette::Palette;
use crate::player::Player;

const HIP_Y: f32 = -0.72;
const SHOULDER_X: f32 = 0.215;
const SHOULDER_Y: f32 = 0.46;
const ARM_DOWN: f32 = -1.30;
const ARM_IN: f32 = 0.18;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FpKind {
    Root,
    Hips,
    Torso,
    Thigh,
    Calf,
    Boot,
    UpperArm,
    Forearm,
    Hand,
}

#[derive(Component)]
pub struct FpLimb {
    pub kind: FpKind,
    /// -1.0 for the left side of the body, +1.0 for the right, 0.0 in between.
    pub side: f32,
    pub rest: Quat,
}

/// Builds the visible body as children of the player camera.
pub fn spawn_fp_body(
    parent: &mut ChildBuilder,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
) {
    let arms = |side: f32| {
        (
            Quat::from_rotation_z(-side * ARM_IN) * Quat::from_rotation_x(ARM_DOWN),
            side * SHOULDER_X,
        )
    };

    parent
        .spawn((
            Transform::IDENTITY,
            Visibility::default(),
            FpLimb {
                kind: FpKind::Root,
                side: 0.0,
                rest: Quat::IDENTITY,
            },
            Name::new("FpBodyRoot"),
        ))
        .with_children(|body| {
            body.spawn((
                Transform::from_translation(Vec3::new(0.0, HIP_Y, 0.01)),
                Visibility::default(),
                FpLimb {
                    kind: FpKind::Hips,
                    side: 0.0,
                    rest: Quat::IDENTITY,
                },
                Name::new("FpBodyHips"),
            ))
            .with_children(|hips| {
                hips.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.30, 0.20, 0.22))),
                    MeshMaterial3d(pal.trouser.clone()),
                    Transform::from_xyz(0.0, 0.02, 0.0),
                ));

                // ---- legs -------------------------------------------------
                for side in [-1.0f32, 1.0] {
                    hips.spawn((
                        Transform::from_translation(Vec3::new(side * 0.105, 0.0, 0.0)),
                        Visibility::default(),
                        FpLimb {
                            kind: FpKind::Thigh,
                            side,
                            rest: Quat::IDENTITY,
                        },
                        Name::new("FpThigh"),
                    ))
                    .with_children(|thigh| {
                        thigh.spawn((
                            Mesh3d(meshes.add(Cuboid::new(0.165, 0.44, 0.175))),
                            MeshMaterial3d(pal.trouser.clone()),
                            Transform::from_xyz(0.0, -0.22, 0.0),
                        ));
                        thigh
                            .spawn((
                                Transform::from_translation(Vec3::new(0.0, -0.44, 0.0)),
                                Visibility::default(),
                                FpLimb {
                                    kind: FpKind::Calf,
                                    side,
                                    rest: Quat::IDENTITY,
                                },
                                Name::new("FpCalf"),
                            ))
                            .with_children(|calf| {
                                calf.spawn((
                                    Mesh3d(meshes.add(Cuboid::new(0.135, 0.42, 0.15))),
                                    MeshMaterial3d(pal.trouser.clone()),
                                    Transform::from_xyz(0.0, -0.21, 0.0),
                                ));
                                calf
                                    .spawn((
                                        Transform::from_translation(Vec3::new(
                                            0.0, -0.42, 0.0,
                                        )),
                                        Visibility::default(),
                                        FpLimb {
                                            kind: FpKind::Boot,
                                            side,
                                            rest: Quat::IDENTITY,
                                        },
                                        Name::new("FpBoot"),
                                    ))
                                    .with_children(|boot| {
                                        boot.spawn((
                                            Mesh3d(meshes.add(Cuboid::new(0.15, 0.12, 0.27))),
                                            MeshMaterial3d(pal.leather.clone()),
                                            Transform::from_xyz(0.0, -0.05, -0.055),
                                        ));
                                    });
                            });
                    });
                }

                // ---- torso ------------------------------------------------
                let torso_rest = Quat::from_rotation_x(-0.06);
                hips.spawn((
                    Transform::from_translation(Vec3::ZERO).with_rotation(torso_rest),
                    Visibility::default(),
                    FpLimb {
                        kind: FpKind::Torso,
                        side: 0.0,
                        rest: torso_rest,
                    },
                    Name::new("FpTorso"),
                ))
                .with_children(|torso| {
                    torso.spawn((
                        Mesh3d(meshes.add(Cuboid::new(0.38, 0.52, 0.225))),
                        MeshMaterial3d(pal.shirt.clone()),
                        Transform::from_xyz(0.0, 0.26, 0.0),
                    ));
                    torso.spawn((
                        Mesh3d(meshes.add(Cuboid::new(0.34, 0.14, 0.20))),
                        MeshMaterial3d(pal.skin_dark.clone()),
                        Transform::from_xyz(0.0, 0.53, 0.0),
                    ));

                    // ---- arms ---------------------------------------------
                    for side in [-1.0f32, 1.0] {
                        let (rest, x) = arms(side);
                        torso.spawn((
                            Transform::from_translation(Vec3::new(x, SHOULDER_Y, 0.0))
                                .with_rotation(rest),
                            Visibility::default(),
                            FpLimb {
                                kind: FpKind::UpperArm,
                                side,
                                rest,
                            },
                            Name::new("FpUpperArm"),
                        ))
                        .with_children(|upper| {
                            upper.spawn((
                                Mesh3d(meshes.add(Cuboid::new(0.115, 0.30, 0.115))),
                                MeshMaterial3d(pal.sleeve.clone()),
                                Transform::from_xyz(0.0, -0.15, 0.0),
                            ));
                            let fore_rest = Quat::from_rotation_x(-0.42);
                            upper
                                .spawn((
                                    Transform::from_translation(Vec3::new(0.0, -0.30, 0.0))
                                        .with_rotation(fore_rest),
                                    Visibility::default(),
                                    FpLimb {
                                        kind: FpKind::Forearm,
                                        side,
                                        rest: fore_rest,
                                    },
                                    Name::new("FpForearm"),
                                ))
                                .with_children(|fore| {
                                    fore.spawn((
                                        Mesh3d(meshes.add(Cuboid::new(0.10, 0.28, 0.10))),
                                        MeshMaterial3d(pal.sleeve.clone()),
                                        Transform::from_xyz(0.0, -0.14, 0.0),
                                    ));
                                    let hand_rest = Quat::from_rotation_x(0.18);
                                    fore.spawn((
                                        Transform::from_translation(Vec3::new(0.0, -0.28, 0.0))
                                            .with_rotation(hand_rest),
                                        Visibility::default(),
                                        FpLimb {
                                            kind: FpKind::Hand,
                                            side,
                                            rest: hand_rest,
                                        },
                                        Name::new("FpHand"),
                                    ))
                                    .with_children(|hand| {
                                        hand.spawn((
                                            Mesh3d(meshes.add(Cuboid::new(0.09, 0.13, 0.105))),
                                            MeshMaterial3d(pal.skin.clone()),
                                            Transform::from_xyz(0.0, -0.055, 0.0),
                                        ));
                                        hand.spawn((
                                            Mesh3d(meshes.add(Cuboid::new(0.085, 0.075, 0.095))),
                                            MeshMaterial3d(pal.skin.clone()),
                                            Transform::from_xyz(0.0, -0.145, -0.01),
                                        ));

                                        // the torch itself sits in the right hand
                                        if side > 0.0 {
                                            hand.spawn((
                                                Mesh3d(meshes.add(Cylinder::new(0.033, 0.17))),
                                                MeshMaterial3d(pal.black_metal.clone()),
                                                Transform::from_xyz(0.0, -0.075, -0.10)
                                                    .with_rotation(Quat::from_rotation_x(
                                                        -1.35,
                                                    )),
                                            ));
                                            hand.spawn((
                                                Mesh3d(meshes.add(Cylinder::new(0.040, 0.03))),
                                                MeshMaterial3d(pal.bulb.clone()),
                                                Transform::from_xyz(0.0, -0.075, -0.185)
                                                    .with_rotation(Quat::from_rotation_x(
                                                        -1.35,
                                                    )),
                                            ));
                                        }
                                    });
                                });
                        });
                    }
                });
            });
        });
}

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_fp_body.run_if(in_state(AppState::Playing)),
        );
    }
}

fn animate_fp_body(
    time: Res<Time>,
    mut events: EventReader<Interacted>,
    player: Query<&Player>,
    mut reach: Local<f32>,
    mut q: Query<(&FpLimb, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    if events.read().count() > 0 {
        *reach = 1.0;
    }

    // reach_k: 0 while idle, 1 while the hand is out at the door handle
    let mut reach_k = 0.0f32;
    if *reach > 0.0 {
        *reach = (*reach - dt / 0.78).max(0.0);
        let t = 1.0 - *reach;
        reach_k = if t < 0.32 {
            (t / 0.32).powf(0.55)
        } else {
            ((1.0 - t) / 0.68).powf(1.5)
        };
    }

    let Ok(player) = player.get_single() else {
        return;
    };
    let speed = player.vel_xz.length();
    let walk = (speed / 2.15).min(1.0);
    let idle = 1.0 - walk;
    let p = player.bob;
    let swing = p.sin() * 0.52 * walk;
    let breath = (time.elapsed_secs() * 1.35).sin();

    for (limb, mut t) in q.iter_mut() {
        let s = limb.side;
        let hand = if s > 0.0 { reach_k } else { 0.0 };
        let extra = match limb.kind {
            FpKind::Root => {
                t.translation = Vec3::new(
                    (p * 0.5).sin() * 0.008 * walk,
                    (p * 2.0).sin() * 0.011 * walk,
                    0.0,
                );
                Quat::IDENTITY
            }
            FpKind::Hips => Quat::from_rotation_y((p * 0.5).sin() * 0.05 * walk),
            FpKind::Torso => {
                Quat::from_rotation_z((p).cos() * 0.022 * walk + breath * 0.004 * idle)
                    * Quat::from_rotation_x(
                        -0.06 + reach_k * 0.07 + breath * 0.006 * idle,
                    )
            }
            FpKind::Thigh => Quat::from_rotation_x(swing * s),
            FpKind::Calf => {
                // knees never bend the wrong way: always a negative X offset
                let bend = 0.07 + (0.10 + (p - s * 1.45).cos().max(0.0) * 0.55) * walk;
                Quat::from_rotation_x(-bend)
            }
            FpKind::Boot => Quat::from_rotation_x(0.06 * walk * s),
            FpKind::UpperArm => Quat::from_rotation_x(
                -swing * s * 0.34 - hand * 0.18 + breath * 0.018 * idle,
            ),
            FpKind::Forearm => Quat::from_rotation_x(
                swing * s * 0.13 + hand * 0.44 - breath * 0.016 * idle,
            ),
            FpKind::Hand => Quat::from_rotation_x(hand * 0.10),
        };
        t.rotation = limb.rest * extra;
    }
}
