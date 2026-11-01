//! Night light: a cold moon outside, and lightning that finds its way in
//! through every window at once.

use bevy::prelude::*;

use crate::audio::LightningStrike;
use crate::core::{AppState, Rng};
use crate::ui::screen_fx::ScreenGrade;

#[derive(Component)]
pub struct MoonLight;

#[derive(Component)]
pub struct FlashLight;

#[derive(Resource, Default)]
pub struct Lightning {
    pub t: f32,
    pub strength: f32,
}

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Lightning>()
            .add_systems(OnEnter(AppState::Title), spawn_lights)
            .add_systems(
                Update,
                (strike, flash_curve).run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_lights(mut commands: Commands, existing: Query<&MoonLight>) {
    if existing.iter().next().is_some() {
        return;
    }
    // moonlight, raking through the south windows
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.52, 0.62, 0.86),
            illuminance: 260.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.028,
            shadow_normal_bias: 1.4,
            ..default()
        },
        Transform::from_xyz(30.0, 60.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        MoonLight,
        Name::new("moon"),
    ));
    // flash source, dark until the sky opens
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.85, 0.90, 1.0),
            illuminance: 0.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-40.0, 70.0, -55.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlashLight,
        Name::new("lightning"),
    ));
}

fn strike(
    mut strikes: EventReader<LightningStrike>,
    mut lightning: ResMut<Lightning>,
    mut rng: ResMut<Rng>,
) {
    for s in strikes.read() {
        if lightning.t > 0.0 {
            continue;
        }
        lightning.t = if s.near { 0.9 } else { 0.55 };
        lightning.strength = if s.near { 1.0 } else { 0.55 } * (0.75 + rng.f32() * 0.45);
    }
}

fn flash_curve(
    time: Res<Time>,
    mut lightning: ResMut<Lightning>,
    mut q: Query<&mut DirectionalLight, With<FlashLight>>,
    mut grade: ResMut<ScreenGrade>,
) {
    if lightning.t <= 0.0 {
        if grade.flash != 0.0 {
            grade.flash = 0.0;
        }
        return;
    }
    let dt = time.delta_secs();
    lightning.t -= dt;
    let elapsed = 0.9 - lightning.t;
    // three uneven pulses, like a real bolt
    let env = if elapsed < 0.05 {
        elapsed / 0.05
    } else if elapsed < 0.11 {
        1.0
    } else if elapsed < 0.16 {
        0.25
    } else if elapsed < 0.24 {
        0.9
    } else {
        (1.0 - (elapsed - 0.24) / 0.6).max(0.0)
    };
    let lux = 6_500.0 * lightning.strength * env;
    grade.flash = (env * lightning.strength * 0.8).clamp(0.0, 1.0);
    for mut light in q.iter_mut() {
        light.illuminance = lux;
    }
    if lightning.t <= 0.0 {
        for mut light in q.iter_mut() {
            light.illuminance = 0.0;
        }
        grade.flash = 0.0;
    }
}
