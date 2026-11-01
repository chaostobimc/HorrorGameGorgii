//! Sound direction.
//!
//! Bevy has no mixer, so this module keeps a small stack of looping ambience
//! beds plus a delayed one-shot queue. Everything is attenuated by hand from
//! the listener position, which is enough for a game played in headphones.

use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::audio::{PlaybackMode, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::core::{AppState, GameAssets, Rng, Settings, UiState};

/// Position the player hears from (updated by the player system).
#[derive(Resource, Default)]
pub struct Listener(pub Vec3);

/// Ask the audio system to play a one-shot.
#[derive(Event, Clone)]
pub struct PlaySfx {
    pub sound: Handle<AudioSource>,
    pub volume: f32,
    /// World position for distance attenuation. `None` plays "in the head".
    pub position: Option<Vec3>,
    pub speed: f32,
}

impl PlaySfx {
    pub fn flat(sound: Handle<AudioSource>, volume: f32) -> Self {
        Self {
            sound,
            volume,
            position: None,
            speed: 1.0,
        }
    }

    pub fn at(sound: Handle<AudioSource>, volume: f32, position: Vec3) -> Self {
        Self {
            sound,
            volume,
            position: Some(position),
            speed: 1.0,
        }
    }
}

/// A looping ambience bed.
#[derive(Component)]
pub struct Ambience {
    pub base: f32,
    pub category: AmbCategory,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AmbCategory {
    Rain,
    Wind,
    Room,
    Machine,
    Elevator,
    Buzz,
}

#[derive(Resource, Default)]
pub struct AmbienceBeds {
    pub rain: Option<Entity>,
    pub wind: Option<Entity>,
    pub tone: Option<Entity>,
    pub gen: Option<Entity>,
    pub elev: Option<Entity>,
    pub buzz: Option<Entity>,
}

/// Global switch used by the finale: everything goes quiet, the world with it.
#[derive(Resource, Default)]
pub struct Silence(pub bool);

/// Delayed sounds (thunder needs to arrive late to feel far away).
#[derive(Resource, Default)]
pub struct DelayedSfx(pub Vec<(f32, PlaySfx)>);

#[derive(Resource)]
pub struct Weather {
    pub next_strike: f32,
    pub pending_bolt: Option<(f32, bool)>,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            next_strike: 7.0,
            pending_bolt: None,
        }
    }
}

/// A lightning bolt is about to be drawn by the FX module; the audio system
/// schedules the thunder that goes with it.
#[derive(Event, Clone, Copy)]
pub struct LightningStrike {
    pub near: bool,
    pub delay: f32,
}

#[derive(Resource)]
pub struct AmbienceTimer {
    pub drip: f32,
    pub creak: f32,
    pub whisper: f32,
    pub heartbeat: f32,
}

impl Default for AmbienceTimer {
    fn default() -> Self {
        Self {
            drip: 4.0,
            creak: 9.0,
            whisper: 30.0,
            heartbeat: 1.2,
        }
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Listener>()
            .init_resource::<AmbienceBeds>()
            .init_resource::<Silence>()
            .init_resource::<DelayedSfx>()
            .init_resource::<Weather>()
            .init_resource::<AmbienceTimer>()
            .add_event::<PlaySfx>()
            .add_event::<LightningStrike>()
            .add_systems(OnEnter(AppState::Playing), start_ambience)
            .add_systems(
                Update,
                (
                    play_sfx.run_if(in_state(AppState::Playing)),
                    tick_delays,
                    apply_ambience_volume,
                    schedule_weather,
                    schedule_ambient_details,
                    react_to_lightning,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn start_ambience(mut commands: Commands, assets: Res<GameAssets>, mut beds: ResMut<AmbienceBeds>) {
    let spawn_bed =
        |commands: &mut Commands, handle: Handle<AudioSource>, base: f32, category: AmbCategory| -> Entity {
            commands
                .spawn((
                    AudioPlayer::new(handle),
                    PlaybackSettings {
                        mode: PlaybackMode::Loop,
                        volume: Volume::new(0.0),
                        ..PlaybackSettings::ONCE
                    },
                    Ambience { base, category },
                ))
                .id()
        };

    beds.rain = Some(spawn_bed(
        &mut commands,
        assets.snd.rain.clone(),
        0.62,
        AmbCategory::Rain,
    ));
    beds.wind = Some(spawn_bed(
        &mut commands,
        assets.snd.wind.clone(),
        0.30,
        AmbCategory::Wind,
    ));
    beds.tone = Some(spawn_bed(
        &mut commands,
        assets.snd.tone.clone(),
        0.16,
        AmbCategory::Room,
    ));
}

fn play_sfx(
    mut commands: Commands,
    mut events: EventReader<PlaySfx>,
    settings: Res<Settings>,
    listener: Res<Listener>,
    silence: Res<Silence>,
) {
    for ev in events.read() {
        if silence.0 {
            continue;
        }
        let mut vol = ev.volume * settings.volume_master;
        if let Some(pos) = ev.position {
            let d = pos.distance(listener.0);
            vol *= 1.0 / (1.0 + d * 0.14);
        }
        if vol <= 0.001 {
            continue;
        }
        commands.spawn((
            AudioPlayer::new(ev.sound.clone()),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(vol.clamp(0.0, 2.0)),
                speed: ev.speed,
                ..PlaybackSettings::ONCE
            },
        ));
    }
}

fn tick_delays(mut delayed: ResMut<DelayedSfx>, mut out: EventWriter<PlaySfx>, time: Res<Time>) {
    let dt = time.delta_secs();
    let mut ready: Vec<PlaySfx> = Vec::new();
    delayed.0.retain_mut(|(t, sfx)| {
        *t -= dt;
        if *t <= 0.0 {
            ready.push(sfx.clone());
            false
        } else {
            true
        }
    });
    for s in ready {
        out.send(s);
    }
}

fn apply_ambience_volume(
    settings: Res<Settings>,
    silence: Res<Silence>,
    mut q: Query<(&Ambience, &mut AudioSink)>,
) {
    for (amb, sink) in q.iter_mut() {
        let category = match amb.category {
            AmbCategory::Rain | AmbCategory::Wind => settings.volume_ambience,
            AmbCategory::Room => settings.volume_ambience * 0.9,
            AmbCategory::Machine => settings.volume_ambience,
            AmbCategory::Elevator => settings.volume_ambience,
            AmbCategory::Buzz => settings.volume_ambience * 0.7,
        };
        let target = if silence.0 {
            0.0
        } else {
            amb.base * category * settings.volume_master
        };
        // smooth towards the target so fades never click
        let current = sink.volume();
        let next = current + (target - current) * 0.08;
        sink.set_volume(next.clamp(0.0, 2.0));
    }
}

fn schedule_weather(
    mut weather: ResMut<Weather>,
    mut strikes: EventWriter<LightningStrike>,
    time: Res<Time>,
    silence: Res<Silence>,
    ui: Res<UiState>,
) {
    if silence.0 || !ui.can_play() {
        return;
    }
    weather.next_strike -= time.delta_secs();
    if weather.next_strike <= 0.0 {
        let near = false;
        strikes.send(LightningStrike {
            near,
            delay: 0.0,
        });
        weather.next_strike = 9.0;
    }
}

fn react_to_lightning(
    mut strikes: EventReader<LightningStrike>,
    mut delayed: ResMut<DelayedSfx>,
    assets: Res<GameAssets>,
    mut rng: ResMut<Rng>,
) {
    for strike in strikes.read() {
        let near = strike.near;
        let dist_delay = strike.delay + if near { 0.35 } else { 2.6 };
        let handle = if near {
            let i = rng.index(assets.snd.thunder_near.len());
            assets.snd.thunder_near[i].clone()
        } else {
            let i = rng.index(assets.snd.thunder_far.len());
            assets.snd.thunder_far[i].clone()
        };
        let vol = if near { 0.95 } else { 0.55 };
        delayed
            .0
            .push((dist_delay, PlaySfx::flat(handle, vol)));
    }
}

fn schedule_ambient_details(
    mut timer: ResMut<AmbienceTimer>,
    mut out: EventWriter<PlaySfx>,
    assets: Res<GameAssets>,
    listener: Res<Listener>,
    mut rng: ResMut<Rng>,
    ui: Res<UiState>,
    silence: Res<Silence>,
    time: Res<Time>,
) {
    if silence.0 || !ui.can_play() {
        return;
    }
    let dt = time.delta_secs().min(0.1);
    timer.drip -= dt;
    timer.creak -= dt;
    timer.whisper -= dt;

    if timer.drip <= 0.0 {
        timer.drip = rng.range(6.0, 17.0);
        let i = rng.index(assets.snd.drip.len());
        let pos = listener.0
            + Vec3::new(rng.range(-6.0, 6.0), rng.range(1.0, 4.0), rng.range(-6.0, 6.0));
        out.send(PlaySfx::at(assets.snd.drip[i].clone(), 0.5, pos));
    }
    if timer.creak <= 0.0 {
        timer.creak = rng.range(11.0, 26.0);
        let i = rng.index(assets.snd.creak.len());
        let pos = listener.0
            + Vec3::new(rng.range(-9.0, 9.0), rng.range(-1.0, 3.0), rng.range(-9.0, 9.0));
        out.send(PlaySfx::at(assets.snd.creak[i].clone(), 0.45, pos));
    }
    if timer.whisper <= 0.0 {
        timer.whisper = rng.range(38.0, 70.0);
        out.send(PlaySfx::flat(assets.snd.whisper.clone(), 0.22));
    }
}
