//! Interaction logic: the safe, the breaker panel, the clock and the elevator.

use bevy::prelude::*;

use crate::audio::{AmbCategory, Ambience, AmbienceBeds, DelayedSfx, PlaySfx};
use crate::core::{AppState, GameAssets, Overlay, UiState};
use crate::interact::{Action, Interactable, Interacted};
use crate::palette::Palette;
use crate::player::Player;
use crate::story::{Progress, Subtitle};
use crate::ui::keypad::SafeOpened;
use crate::world::{
    ClockHand, CorridorDoor, DoorBlocker, SafeDoor, SafeKey, SecretDoor, Slide, Swing, F2,
};

#[derive(Resource, Default)]
pub struct RideState {
    pub stage: u8,
    pub t: f32,
}

pub struct PuzzlesPlugin;

impl Plugin for PuzzlesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RideState>()
            .add_systems(
                Update,
                (handle_actions, safe_opened, tick_ride, sync_clock_hands)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// main interaction switch
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn handle_actions(
    mut events: EventReader<Interacted>,
    mut progress: ResMut<Progress>,
    mut sub: ResMut<Subtitle>,
    mut ui: ResMut<UiState>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    mut sfx: EventWriter<PlaySfx>,
    mut delayed: ResMut<DelayedSfx>,
    mut items: Query<(Entity, &mut Interactable)>,
    mut swings: Query<(
        &mut Swing,
        &mut Transform,
        Option<&CorridorDoor>,
        Option<&crate::world::LeverHandle>,
    )>,
    blockers: Query<Entity, With<DoorBlocker>>,
    mut ride: ResMut<RideState>,
) {
    for ev in events.read() {
        // Every document is just the reader with a different page.
        match ev.action {
            Action::ReadDeskArticle => open_doc(&mut ui, &mut progress, 0),
            Action::ReadCorridorArticle => open_doc(&mut ui, &mut progress, 1),
            Action::ReadBallroomArticle => open_doc(&mut ui, &mut progress, 2),
            Action::ReadChefDiary => open_doc(&mut ui, &mut progress, 3),
            Action::Flavor(1) => open_doc(&mut ui, &mut progress, 4),
            Action::Flavor(2) => {
                sfx.send(PlaySfx::flat(assets.snd.metal_ring.clone(), 0.35));
                sub.say("Die Glocke klingt, als stünde sie unter Wasser. Niemand kommt.", 5.0);
            }
            Action::Flavor(_) => {
                sub.say(
                    "Die Uhr an der Wand steht. 03:15 – seit achtunddreißig Jahren.",
                    5.0,
                );
            }
            Action::SafeKeypad => {
                if progress.safe_open {
                    sub.say("Der Tresen steht offen.", 3.0);
                } else {
                    ui.overlay = Overlay::Keypad;
                    sfx.send(PlaySfx::flat(assets.snd.keypad_beep.clone(), 0.4));
                }
            }
            Action::SafeTakeKey => {
                if progress.safe_open && !progress.has_key {
                    progress.has_key = true;
                    commands.entity(ev.entity).despawn_recursive();
                    sfx.send(PlaySfx::flat(assets.snd.metal_ring.clone(), 0.6));
                    sub.say(
                        "Ein eiserner Schlüssel, kalt wie ein Geländer im Winter. \
Er passt zu der Tür hinter der Rezeption.",
                        7.0,
                    );
                }
            }
            Action::CorridorDoor => {
                if progress.corridor_open {
                    // already open
                } else if progress.has_key {
                    progress.corridor_open = true;
                    for (mut swing, _, is_door, _) in swings.iter_mut() {
                        if is_door.is_some() {
                            swing.active = true;
                        }
                    }
                    // the door is no longer solid once it swings away
                    for e in blockers.iter() {
                        commands.entity(e).despawn_recursive();
                    }
                    sfx.send(PlaySfx::flat(assets.snd.door_creak.clone(), 0.8));
                    sub.say(
                        "Der Schlüssel dreht sich schwer, dann gibt das Schloss nach. \
Hinter der Tür liegt ein Treppenhaus, das nach Regen riecht.",
                        7.0,
                    );
                } else {
                    sfx.send(PlaySfx::flat(assets.snd.door_slam.clone(), 0.25));
                    sub.say(
                        "Verschlossen. Ein Eisenschloss, so groß wie eine Handfläche.",
                        4.5,
                    );
                }
            }
            Action::Lever(i) => {
                if progress.generator {
                    continue;
                }
                let i = i as usize;
                let expected = progress.levers.len();
                // the lever we touched goes down, whatever happens next
                for (mut swing, _, _, handle) in swings.iter_mut() {
                    if let Some(h) = handle {
                        if h.0 == i {
                            swing.active = true;
                        }
                    }
                }
                if i == expected {
                    progress.levers.push(i as u8);
                    // a lever that is down stays down: no second pull by accident
                    for (_, mut item) in items.iter_mut() {
                        if item.action == Action::Lever(i as u8) {
                            item.enabled = false;
                        }
                    }
                    sfx.send(PlaySfx::flat(assets.snd.lever_clunk.clone(), 0.7));
                    if progress.levers.len() == 4 {
                        progress.generator = true;
                        start_generator(&mut commands, &assets, &mut sfx, &mut sub);
                    } else {
                        sub.say(
                            match i {
                                0 => "Der Hauptschalter sitzt. Ein Relais zieht an.",
                                1 => "Die Sicherung hält. Das Brummen wird tiefer.",
                                2 => "Das Hilfsaggregat springt an.",
                                _ => "Der Erdschluss schließt den Kreis.",
                            },
                            3.5,
                        );
                    }
                } else {
                    // wrong order: everything springs back
                    progress.levers.clear();
                    for (mut swing, mut tf, _, handle) in swings.iter_mut() {
                        if handle.is_some() {
                            swing.active = false;
                            swing.t = 0.0;
                            tf.rotation = swing.from;
                        }
                    }
                    for (_, mut item) in items.iter_mut() {
                        if matches!(item.action, Action::Lever(_)) {
                            item.enabled = true;
                        }
                    }
                    sfx.send(PlaySfx::flat(assets.snd.spark.clone(), 0.7));
                    sfx.send(PlaySfx::flat(assets.snd.power_down.clone(), 0.45));
                    sub.say(
                        "Die Sicherungen springen zurück. Alle Hebel stehen wieder oben. \
Die Reihenfolge steht in der Zeitung – nicht im Zufall.",
                        8.0,
                    );
                }
            }
            Action::ElevatorCall => {
                if !progress.generator {
                    sfx.send(PlaySfx::flat(assets.snd.buzz.clone(), 0.4));
                    sub.say(
                        "Kein Strom. Der Aufzug hängt irgendwo zwischen den Stockwerken.",
                        4.5,
                    );
                } else if !progress.reached_penthouse && ride.stage == 0 {
                    ride.stage = 1;
                    ride.t = 0.0;
                    ui.locked = true;
                    sfx.send(PlaySfx::flat(assets.snd.elev.clone(), 0.7));
                    sfx.send(PlaySfx::flat(assets.snd.gear_clunk.clone(), 0.5));
                }
            }
            Action::Piano => {
                sfx.send(PlaySfx::flat(assets.snd.piano.clone(), 0.75));
                delayed.0.push((
                    2.6,
                    PlaySfx::flat(assets.snd.chair_scrape.clone(), 0.5),
                ));
                sub.say(
                    "Die Tasten sind kalt und ein wenig klebrig. Der letzte Ton bleibt \
zu lange im Raum stehen.",
                    6.0,
                );
            }
            Action::ClockHour => {
                progress.clock_hour = (progress.clock_hour + 1) % 24;
                sfx.send(PlaySfx::flat(assets.snd.clock_tick.clone(), 0.5));
            }
            Action::ClockMinute => {
                progress.clock_minute = (progress.clock_minute + 5) % 60;
                sfx.send(PlaySfx::flat(assets.snd.clock_tick.clone(), 0.5));
            }
            Action::ClockLatch => {
                if progress.clock_solved {
                    sub.say("Der Durchgang steht offen.", 3.0);
                } else {
                    sfx.send(PlaySfx::flat(assets.snd.gear_clunk.clone(), 0.25));
                    sub.say(
                        "Das Gehäuse ist verriegelt. Nur die Zeiger lassen sich drehen – \
und irgendwo im Werk klickt es bei jeder Bewegung.",
                        7.0,
                    );
                }
            }
            Action::EmergencySwitch | Action::Gate => {}
        }
    }
}

fn open_doc(ui: &mut UiState, progress: &mut Progress, index: usize) {
    ui.open_doc = index;
    ui.overlay = Overlay::Document;
    progress.note(index);
}

fn start_generator(
    commands: &mut Commands,
    assets: &GameAssets,
    sfx: &mut EventWriter<PlaySfx>,
    sub: &mut Subtitle,
) {
    sfx.send(PlaySfx::flat(assets.snd.gen_start.clone(), 0.9));
    commands.spawn((
        AudioPlayer::new(assets.snd.gen_hum.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.0),
            ..PlaybackSettings::ONCE
        },
        Ambience {
            base: 0.55,
            category: AmbCategory::Machine,
        },
    ));
    sub.say(
        "Das Aggregat erwacht. Unter dem Boden läuft ein Ruck durch das ganze Haus, \
und irgendwo über mir setzt sich etwas in Bewegung.",
        9.0,
    );
}

// ---------------------------------------------------------------------------
// the safe
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn safe_opened(
    mut events: EventReader<SafeOpened>,
    mut progress: ResMut<Progress>,
    mut sub: ResMut<Subtitle>,
    assets: Res<GameAssets>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut sfx: EventWriter<PlaySfx>,
    mut door: Query<&mut Slide, With<SafeDoor>>,
) {
    for _ in events.read() {
        if progress.safe_open {
            continue;
        }
        progress.safe_open = true;
        for mut slide in door.iter_mut() {
            slide.active = true;
        }
        // the key, lying in the open safe
        let key_mesh = meshes.add(Mesh::from(Cuboid::new(0.05, 0.02, 0.26)));
        commands.spawn((
            Mesh3d(key_mesh),
            MeshMaterial3d(palette.brass.clone()),
            Transform::from_translation(Vec3::new(-10.98, 1.02, -8.76))
                .with_rotation(Quat::from_rotation_y(0.42)),
            Interactable::new(Action::SafeTakeKey, "Eisenschlüssel nehmen", 0.4),
            SafeKey,
        ));
        sfx.send(PlaySfx::flat(assets.snd.gear_clunk.clone(), 0.6));
        sub.say(
            "10449. Die Tür schwingt auf, und im Tresen liegt ein einzelner Schlüssel.",
            7.0,
        );
    }
}

// ---------------------------------------------------------------------------
// elevator ride
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn tick_ride(
    time: Res<Time>,
    mut ride: ResMut<RideState>,
    mut fade: ResMut<crate::ui::Fade>,
    mut player: Query<(&mut Transform, &mut Player)>,
    mut progress: ResMut<Progress>,
    mut sub: ResMut<Subtitle>,
    mut ui: ResMut<UiState>,
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
) {
    if ride.stage == 0 {
        return;
    }
    let dt = time.delta_secs();
    ride.t += dt;
    match ride.stage {
        1 => {
            fade.to_black(1.3);
            if ride.t > 1.0 {
                if let Ok((mut tf, mut pl)) = player.get_single_mut() {
                    let dest = Vec3::new(-15.0, F2 + 0.05, -20.2);
                    tf.translation = dest;
                    pl.last_safe = dest;
                    pl.vel_xz = Vec2::ZERO;
                    pl.vel_y = 0.0;
                    pl.yaw = -std::f32::consts::FRAC_PI_2;
                    pl.pitch = -0.02;
                }
                progress.reached_penthouse = true;
                sfx.send(PlaySfx::flat(assets.snd.elev.clone(), 0.6));
                ride.stage = 2;
                ride.t = 0.0;
            }
        }
        2 => {
            if ride.t > 0.45 {
                fade.to_clear(0.9);
                ride.stage = 3;
                ride.t = 0.0;
            }
        }
        3 => {
            if ride.t > 1.4 {
                ride.stage = 0;
                ui.locked = false;
                sub.say(
                    "Das Gitter klappt auf. Der Ballsaal liegt vor mir – und alle Uhren \
in diesem Haus stehen.",
                    8.0,
                );
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// clock hands
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn sync_clock_hands(
    mut progress: ResMut<Progress>,
    mut hands: Query<(&ClockHand, &mut Transform)>,
    mut secret: Query<&mut Slide, With<SecretDoor>>,
    mut sub: ResMut<Subtitle>,
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
    blockers: Query<Entity, With<crate::world::SecretBlocker>>,
    mut commands: Commands,
    mut last_hour: Local<f32>,
    mut last_minute: Local<f32>,
) {
    let hour = progress.clock_angle_hour();
    let minute = progress.clock_angle_minute();
    if (hour - *last_hour).abs() > 0.001 || (minute - *last_minute).abs() > 0.001 {
        *last_hour = hour;
        *last_minute = minute;
        for (hand, mut tf) in hands.iter_mut() {
            let angle = if hand.hour { hour } else { minute };
            tf.rotation = hand.base * Quat::from_rotation_z(-angle.to_radians());
        }
    }

    if !progress.clock_solved && progress.clock_is_solution() {
        progress.clock_solved = true;
        for mut slide in secret.iter_mut() {
            slide.active = true;
        }
        for e in blockers.iter() {
            commands.entity(e).despawn_recursive();
        }
        sfx.send(PlaySfx::flat(assets.snd.gear_clunk.clone(), 0.9));
        sfx.send(PlaySfx::flat(assets.snd.metal_ring.clone(), 0.4));
        sub.say(
            "Etwas im Uhrwerk greift ineinander. Die Wand hinter der Standuhr schiebt \
sich zollweise zur Seite und gibt einen Gang aus Ziegeln frei.",
            9.0,
        );
    }
}
