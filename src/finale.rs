//! The last five seconds of the night: the switch, the silence, the gate, and
//! what waits at the end of the service corridor.

use bevy::prelude::*;

use crate::audio::{PlaySfx, Silence};
use crate::core::{AppState, GameAssets, UiState};
use crate::interact::{Action, Interacted};
use crate::palette::Palette;
use crate::player::{Eye, Player};
use crate::story::{Progress, Subtitle};
use crate::ui::Fade;
use crate::world::{ExitGate, GateBlocker, GateLed, JumpscareTrigger, Slide};

#[derive(Resource, Default)]
pub struct Finale {
    pub stage: u8,
    pub t: f32,
}

pub struct FinalePlugin;

impl Plugin for FinalePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Finale>().add_systems(
            Update,
            (switch_pressed, finale_tick, scare_trigger)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn switch_pressed(
    mut events: EventReader<Interacted>,
    progress: Res<Progress>,
    mut finale: ResMut<Finale>,
    mut sub: ResMut<Subtitle>,
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
) {
    for ev in events.read() {
        if ev.action == Action::Gate {
            if progress.gate_open {
                sub.say("Das Gitter steht offen. Dahinter: nur Regen und Ziegel.", 4.0);
            } else {
                sfx.send(PlaySfx::flat(assets.snd.metal_ring.clone(), 0.4));
                sub.say(
                    "Das Gitter ist verriegelt. Der Notfallschalter hängt neben der Tür.",
                    5.0,
                );
            }
            continue;
        }
        if ev.action != Action::EmergencySwitch || finale.stage != 0 {
            continue;
        }
        if progress.gate_open {
            sub.say("Das Tor steht bereits offen.", 3.0);
            continue;
        }
        sfx.send(PlaySfx::flat(assets.snd.switch_click.clone(), 0.9));
        // the click still rings out, then everything dies at once
        finale.stage = 1;
        finale.t = 0.0;
        sub.say("", 0.1);
    }
}

#[allow(clippy::too_many_arguments)]
fn finale_tick(
    time: Res<Time>,
    mut finale: ResMut<Finale>,
    mut progress: ResMut<Progress>,
    mut silence: ResMut<Silence>,
    mut sub: ResMut<Subtitle>,
    assets: Res<GameAssets>,
    palette: Res<Palette>,
    mut sfx: EventWriter<PlaySfx>,
    mut gate: Query<&mut Slide, With<ExitGate>>,
    mut led: Query<&mut MeshMaterial3d<StandardMaterial>, With<GateLed>>,
    mut fade: ResMut<Fade>,
    mut ui: ResMut<UiState>,
    mut exit: EventWriter<AppExit>,
    mut lights: Query<(&mut PointLight, Option<&Strobe>)>,
    blockers: Query<Entity, With<GateBlocker>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    finale.t += dt;
    match finale.stage {
        // five seconds of absolutely nothing
        1 => {
            if finale.t > 0.28 && !silence.0 {
                silence.0 = true;
            }
            if finale.t > 5.3 {
                silence.0 = false;
                for mut mat in led.iter_mut() {
                    mat.0 = palette.led_green.clone();
                }
                sfx.send(PlaySfx::flat(assets.snd.led_on.clone(), 0.7));
                sfx.send(PlaySfx::flat(assets.snd.buzz.clone(), 0.35));
                for mut slide in gate.iter_mut() {
                    slide.active = true;
                }
                for e in blockers.iter() {
                    commands.entity(e).despawn_recursive();
                }
                progress.gate_open = true;
                sub.say("TOR ENTSPERRT.", 4.0);
                finale.stage = 2;
                finale.t = 0.0;
            }
        }
        // the gate is rolling open
        2 => {
            if finale.t > 2.2 {
                sub.say(
                    "Das Gitter rollt zur Seite. Dahinter liegt ein Gang, so eng, dass \
die Schultern die Ziegel streifen.",
                    8.0,
                );
                finale.stage = 3;
                finale.t = 0.0;
            }
        }
        // strobe + screamer
        4 => {
            if finale.t > 0.42 {
                fade.alpha = 1.0;
                fade.target = 1.0;
                silence.0 = true;
                ui.locked = true;
                finale.stage = 5;
                finale.t = 0.0;
                for (mut light, strobe) in lights.iter_mut() {
                    if strobe.is_some() {
                        light.intensity = 0.0;
                    }
                }
            }
        }
        // black, then the game closes itself
        5 => {
            fade.alpha = 1.0;
            fade.target = 1.0;
            if finale.t > 3.4 {
                finale.stage = 6;
                exit.send(AppExit::Success);
            }
        }
        _ => {}
    }

    // keep the strobe flickering while it lasts
    if finale.stage == 4 {
        let flick = (finale.t * 47.0).sin().abs();
        for (mut light, strobe) in lights.iter_mut() {
            if strobe.is_some() {
                light.intensity = 90_000.0 * flick;
            }
        }
    }
}

#[derive(Component)]
pub struct Strobe;

#[allow(clippy::too_many_arguments)]
fn scare_trigger(
    mut commands: Commands,
    triggers: Query<&Transform, With<JumpscareTrigger>>,
    player: Query<&Transform, With<Player>>,
    eye: Query<&GlobalTransform, With<Eye>>,
    mut finale: ResMut<Finale>,
    mut progress: ResMut<Progress>,
    mut ui: ResMut<UiState>,
    mut silence: ResMut<Silence>,
    mut sub: ResMut<Subtitle>,
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
) {
    if finale.stage != 3 || progress.ended {
        return;
    }
    let Ok(pt) = player.get_single() else { return };
    let mut close = false;
    for t in triggers.iter() {
        let d = t.translation.distance(pt.translation);
        if d < 2.0 {
            close = true;
        }
    }
    if !close {
        return;
    }

    progress.ended = true;
    finale.stage = 4;
    finale.t = 0.0;
    ui.locked = true;
    silence.0 = false;
    sub.say("", 0.1);

    // the scream, right in the ear
    sfx.send(PlaySfx::flat(assets.snd.stinger.clone(), 2.0));
    sfx.send(PlaySfx::flat(assets.snd.thunder_near[0].clone(), 1.4));

    // and the light
    let mut at = pt.translation;
    if let Ok(e) = eye.get_single() {
        at = e.translation() + e.rotation() * Vec3::NEG_Z * 1.2;
    }
    commands.spawn((
        PointLight {
            color: Color::srgb(0.92, 0.95, 1.0),
            intensity: 90_000.0,
            range: 30.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(at),
        Strobe,
        Name::new("strobe"),
    ));
}
