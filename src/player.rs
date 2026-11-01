//! First person body, camera, flashlight and footsteps.

use std::f32::consts::FRAC_PI_2;

use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{DistanceFog, FogFalloff, VolumetricFog, VolumetricLight};
use bevy::prelude::*;
use bevy::render::camera::Exposure;
use bevy::window::CursorGrabMode;

use crate::audio::{Listener, PlaySfx};
use crate::collision::{
    ceiling_height, collect, is_free, resolve_axis, support_height, SolidBox,
};
use crate::core::{AppState, GameAssets, Rng, Settings, UiState};

pub const EYE_HEIGHT: f32 = 1.66;
pub const RADIUS: f32 = 0.33;
pub const HEIGHT: f32 = 1.76;
pub const GRAVITY: f32 = 21.0;
pub const SPAWN: Vec3 = Vec3::new(0.0, 0.0, 15.0);

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
    pub vel_y: f32,
    pub vel_xz: Vec2,
    pub on_ground: bool,
    pub bob: f32,
    pub travel: f32,
    pub step_accum: f32,
    pub last_safe: Vec3,
    pub flashlight: bool,
    pub flicker: f32,
    pub sway: Vec2,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            vel_y: 0.0,
            vel_xz: Vec2::ZERO,
            on_ground: true,
            bob: 0.0,
            travel: 0.0,
            step_accum: 0.0,
            last_safe: SPAWN,
            flashlight: true,
            flicker: 1.0,
            sway: Vec2::ZERO,
        }
    }
}

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct Flashlight {
    pub base_intensity: f32,
    pub flicker_timer: f32,
}

/// Marker for the "eye" transform used by interaction and spooky-event systems.
#[derive(Component)]
pub struct Eye;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(OnExit(AppState::Playing), despawn_player)
            .add_systems(
                Update,
                (
                    mouse_look,
                    movement,
                    head_bob,
                    cursor_grab,
                    flashlight_toggle,
                    flashlight_sway,
                    update_listener,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands, settings: Res<Settings>) {
    let fov = settings.fov.to_radians();
    commands
        .spawn((
            Player {
                yaw: 0.0,
                pitch: -0.03,
                ..default()
            },
            Transform::from_xyz(SPAWN.x, SPAWN.y, SPAWN.z),
            Visibility::default(),
            Name::new("Gorgii"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Camera3d::default(),
                    Camera {
                        hdr: true,
                        clear_color: ClearColorConfig::Custom(Color::srgb(0.012, 0.016, 0.02)),
                        ..default()
                    },
                    Projection::Perspective(PerspectiveProjection {
                        fov,
                        near: 0.04,
                        far: 240.0,
                        ..default()
                    }),
                    Exposure { ev100: 6.2 },
                    Tonemapping::TonyMcMapface,
                    Bloom {
                        intensity: 0.10,
                        low_frequency_boost: 0.4,
                        ..Bloom::NATURAL
                    },
                    DistanceFog {
                        color: Color::srgba(0.045, 0.058, 0.072, 1.0),
                        falloff: FogFalloff::Exponential { density: 0.021 },
                        ..default()
                    },
                    VolumetricFog {
                        ambient_intensity: 0.06,
                        jitter: 0.5,
                        step_count: 32,
                        ..default()
                    },
                    Msaa::Sample4,
                    Transform::from_xyz(0.0, EYE_HEIGHT, 0.0),
                    PlayerCamera,
                    Eye,
                    Name::new("PlayerCamera"),
                ))
                .with_children(|cam| {
                    cam.spawn((
                        SpotLight {
                            color: Color::srgb(1.0, 0.92, 0.78),
                            intensity: 26_000.0,
                            range: 26.0,
                            radius: 0.02,
                            inner_angle: 0.10,
                            outer_angle: 0.44,
                            shadows_enabled: true,
                            shadow_depth_bias: 0.03,
                            ..default()
                        },
                        VolumetricLight,
                        Transform::from_xyz(0.14, -0.10, 0.0),
                        Flashlight {
                            base_intensity: 26_000.0,
                            flicker_timer: 4.0,
                        },
                        Name::new("Flashlight"),
                    ));
                });
        });
}

fn despawn_player(mut commands: Commands, q: Query<Entity, With<Player>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Locks the mouse while we are actually in control.
fn cursor_grab(ui: Res<UiState>, state: Res<State<AppState>>, mut windows: Query<&mut Window>) {
    let playing = *state.get() == AppState::Playing;
    let want = playing && ui.can_play();
    if let Ok(mut window) = windows.get_single_mut() {
        let (grab, visible) = if want {
            (CursorGrabMode::Locked, false)
        } else {
            (CursorGrabMode::None, true)
        };
        if window.cursor_options.grab_mode != grab {
            window.cursor_options.grab_mode = grab;
        }
        window.cursor_options.visible = visible;
    }
}

fn mouse_look(
    mut mouse: EventReader<MouseMotion>,
    settings: Res<Settings>,
    ui: Res<UiState>,
    mut q: Query<&mut Player>,
) {
    let mut delta = Vec2::ZERO;
    for ev in mouse.read() {
        delta += ev.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }
    if !ui.can_play() {
        return;
    }
    if let Ok(mut player) = q.get_single_mut() {
        player.yaw -= delta.x * settings.sensitivity;
        let dy = if settings.invert_y { -delta.y } else { delta.y };
        player.pitch -= dy * settings.sensitivity;
        player.pitch = player.pitch.clamp(-FRAC_PI_2 + 0.05, FRAC_PI_2 - 0.05);
        player.sway += delta * 0.0006;
        player.sway = player.sway.clamp(Vec2::splat(-0.09), Vec2::splat(0.09));
        // yaw wrap keeps the number space small over a 45 minute session
        if player.yaw.abs() > std::f32::consts::TAU * 4.0 {
            player.yaw = player.yaw.rem_euclid(std::f32::consts::TAU);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    ui: Res<UiState>,
    assets: Res<GameAssets>,
    mut rng: ResMut<Rng>,
    mut sfx: EventWriter<PlaySfx>,
    mut q: Query<(&mut Player, &mut Transform)>,
    solids: Query<(&GlobalTransform, &SolidBox)>,
) {
    let Ok((mut player, mut transform)) = q.get_single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    let solid_list = collect(&solids);

    let mut input = Vec2::ZERO;
    if ui.can_play() {
        if keys.pressed(KeyCode::KeyW) {
            input.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            input.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            input.x += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            input.x -= 1.0;
        }
    }
    let walking = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if input.length_squared() > 0.0 {
        input = input.normalize();
    }

    let forward = Vec3::new(-player.yaw.sin(), 0.0, -player.yaw.cos());
    let right = Vec3::new(player.yaw.cos(), 0.0, -player.yaw.sin());
    let wish = forward * input.y + right * input.x;
    let target_speed = if walking { 1.15 } else { 2.35 };
    let target = Vec2::new(wish.x, wish.z) * target_speed;
    let accel = if player.on_ground { 14.0 } else { 4.0 };
    let vx = player.vel_xz;
    player.vel_xz = vx + (target - vx) * (accel * dt).min(1.0);

    // Horizontal movement, resolved one axis at a time.
    let dx = player.vel_xz.x * dt;
    let dz = player.vel_xz.y * dt;
    let mut p = transform.translation;
    p.x += dx;
    resolve_axis(&mut p, dx, 0, RADIUS, HEIGHT, &solid_list);
    p.z += dz;
    resolve_axis(&mut p, dz, 2, RADIUS, HEIGHT, &solid_list);

    // Vertical: gravity, floor and ceiling.
    player.vel_y -= GRAVITY * dt;
    p.y += player.vel_y * dt;

    let support = support_height(p, RADIUS, &solid_list);
    if p.y <= support {
        p.y = support;
        player.vel_y = 0.0;
        player.on_ground = true;
    } else if p.y > support + 0.06 {
        player.on_ground = false;
    }

    let ceil = ceiling_height(p, RADIUS, &solid_list);
    if p.y + HEIGHT > ceil {
        p.y = ceil - HEIGHT;
        if player.vel_y > 0.0 {
            player.vel_y = 0.0;
        }
    }

    // Failsafes: never leave the player stuck in a wall or falling forever.
    if !is_free(p, RADIUS * 0.6, HEIGHT, &solid_list) {
        p = player.last_safe;
        player.vel_y = 0.0;
    }
    if p.y < -30.0 {
        p = SPAWN;
    }

    transform.translation = p;
    if player.on_ground {
        player.last_safe = p;
    }

    // Head bob & footsteps.
    let speed = player.vel_xz.length();
    if player.on_ground && speed > 0.25 {
        player.bob += dt * speed * 3.3;
        player.step_accum += speed * dt;
        let stride = if walking { 1.45 } else { 1.85 };
        if player.step_accum > stride {
            player.step_accum = 0.0;
            let i = rng.index(assets.snd.step_wood.len());
            let s = match surface_at(p) {
                Surface::Wood => assets.snd.step_wood[i % assets.snd.step_wood.len()].clone(),
                Surface::Stone => assets.snd.step_stone[i % assets.snd.step_stone.len()].clone(),
            };
            sfx.send(PlaySfx::flat(s, 0.32 + rng.f32() * 0.08));
        }
    } else {
        player.step_accum = player.step_accum.max(0.55);
    }
    player.travel += speed * dt;
}

fn surface_at(pos: Vec3) -> Surface {
    if pos.y < 0.7 {
        Surface::Stone
    } else if pos.y > 8.4 {
        if pos.x > 6.5 {
            Surface::Stone
        } else {
            Surface::Wood
        }
    } else if pos.x > 15.8 && pos.y > 4.0 {
        Surface::Stone
    } else {
        Surface::Wood
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Surface {
    Wood,
    Stone,
}

fn head_bob(
    settings: Res<Settings>,
    time: Res<Time>,
    q_player: Query<&Player>,
    mut q_cam: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Ok(player) = q_player.get_single() else {
        return;
    };
    let Ok(mut cam) = q_cam.get_single_mut() else {
        return;
    };
    let speed = player.vel_xz.length().min(3.0);
    let amount = 0.028 * settings.head_bob * (speed / 2.35).min(1.5);
    let bob_y = (player.bob * 2.0).sin() * amount;
    let bob_x = (player.bob).cos() * amount * 0.6;
    cam.translation = Vec3::new(bob_x * 0.5, EYE_HEIGHT + bob_y, 0.0);
    let t = time.elapsed_secs();
    let breath = (t * 1.1).sin() * 0.004 * settings.head_bob;
    cam.translation.y += breath;
    let roll = -player.vel_xz.dot(Vec2::new(
        player.yaw.cos(),
        -player.yaw.sin(),
    )) * 0.006;
    cam.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, roll);
}

fn flashlight_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    ui: Res<UiState>,
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
    mut q_player: Query<&mut Player>,
    mut q_light: Query<(&mut Visibility, &mut SpotLight), With<Flashlight>>,
) {
    if !keys.just_pressed(KeyCode::KeyF) || !ui.can_play() {
        return;
    }
    let Ok(mut player) = q_player.get_single_mut() else {
        return;
    };
    player.flashlight = !player.flashlight;
    if let Ok((mut vis, mut light)) = q_light.get_single_mut() {
        *vis = if player.flashlight {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        light.intensity = if player.flashlight { 26_000.0 } else { 0.0 };
    }
    sfx.send(PlaySfx::flat(assets.snd.switch_click.clone(), 0.35));
}

fn flashlight_sway(
    time: Res<Time>,
    mut q_player: Query<&mut Player>,
    mut q_light: Query<(&mut Transform, &mut SpotLight, &mut Flashlight), Without<PlayerCamera>>,
) {
    let Ok(mut player) = q_player.get_single_mut() else {
        return;
    };
    let Ok((mut t, mut light, mut fl)) = q_light.get_single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    player.sway *= (1.0 - 6.0 * dt).max(0.0);
    t.rotation = Quat::from_euler(EulerRot::YXZ, player.sway.x * 2.2, -player.sway.y * 2.2, 0.0);

    // Occasional dying-bulb flicker makes the beam feel like real hardware.
    fl.flicker_timer -= dt;
    if fl.flicker_timer <= 0.0 {
        fl.flicker_timer = 3.0 + (player.travel * 0.37).sin().abs() * 9.0;
        player.flicker = 1.0;
    }
    if !player.flashlight {
        light.intensity = 0.0;
        return;
    }
    let wobble = 1.0 + (time.elapsed_secs() * 37.0).sin() * 0.03;
    light.intensity = fl.base_intensity * wobble * player.flicker;
}

fn update_listener(
    mut listener: ResMut<Listener>,
    q: Query<&GlobalTransform, With<PlayerCamera>>,
) {
    if let Ok(t) = q.get_single() {
        listener.0 = t.translation();
    }
}

/// Helper for other modules: gives the world position and forward direction of
/// the player's eye.
pub fn eye_transform(q: &Query<&GlobalTransform, With<PlayerCamera>>) -> Option<(Vec3, Vec3)> {
    q.get_single().ok().map(|t| {
        let f = t.rotation() * Vec3::NEG_Z;
        (t.translation(), f)
    })
}
