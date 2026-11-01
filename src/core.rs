//! Core application state, loaded asset handles and shared resources.

use bevy::audio::AudioSource;
use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;

/// High level flow of the game.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    /// Loading assets and preparing materials.
    #[default]
    Boot,
    /// Main menu, camera drifts through the lobby.
    Title,
    /// The game itself.
    Playing,
    /// Post-jumpscare end card.
    Ending,
}

/// What kind of modal UI is currently covering the screen.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Overlay {
    #[default]
    None,
    Document,
    Keypad,
    Pause,
    Settings,
    Controls,
}

#[derive(Resource)]
pub struct UiState {
    pub overlay: Overlay,
    /// Document currently open in the reader, if any.
    pub open_doc: usize,
    /// True while a scripted sequence takes control away from the player.
    pub locked: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            overlay: Overlay::None,
            open_doc: 0,
            locked: false,
        }
    }
}

impl UiState {
    pub fn can_play(&self) -> bool {
        self.overlay == Overlay::None && !self.locked
    }
}

#[derive(Resource, Clone)]
pub struct Settings {
    pub sensitivity: f32,
    pub volume_master: f32,
    pub volume_ambience: f32,
    pub fov: f32,
    pub head_bob: f32,
    pub grain: f32,
    pub volumetric: bool,
    pub shadows: bool,
    pub invert_y: bool,
}

/// Where the settings menu stores its state. Relative to the working
/// directory, which is the project folder when started with `cargo run`.
pub const SETTINGS_PATH: &str = "settings.cfg";

impl Settings {
    /// Reads the last used settings, falling back to the defaults for every
    /// value that is missing or unreadable.
    pub fn load() -> Self {
        let mut s = Settings::default();
        let Ok(text) = std::fs::read_to_string(SETTINGS_PATH) else {
            return s;
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "sensitivity" => {
                    if let Ok(x) = value.parse() {
                        s.sensitivity = x.clamp(0.0002, 0.01);
                    }
                }
                "volume_master" => {
                    if let Ok(x) = value.parse() {
                        s.volume_master = x.clamp(0.0, 1.0);
                    }
                }
                "volume_ambience" => {
                    if let Ok(x) = value.parse() {
                        s.volume_ambience = x.clamp(0.0, 1.0);
                    }
                }
                "fov" => {
                    if let Ok(x) = value.parse() {
                        s.fov = x.clamp(50.0, 110.0);
                    }
                }
                "head_bob" => {
                    if let Ok(x) = value.parse() {
                        s.head_bob = x.clamp(0.0, 2.0);
                    }
                }
                "grain" => {
                    if let Ok(x) = value.parse() {
                        s.grain = x.clamp(0.0, 1.0);
                    }
                }
                "volumetric" => s.volumetric = value != "0",
                "shadows" => s.shadows = value != "0",
                "invert_y" => s.invert_y = value != "0",
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) {
        let text = format!(
            "sensitivity={}\nvolume_master={}\nvolume_ambience={}\nfov={}\nhead_bob={}\n\
             grain={}\nvolumetric={}\nshadows={}\ninvert_y={}\n",
            self.sensitivity,
            self.volume_master,
            self.volume_ambience,
            self.fov,
            self.head_bob,
            self.grain,
            self.volumetric as u8,
            self.shadows as u8,
            self.invert_y as u8,
        );
        let _ = std::fs::write(SETTINGS_PATH, text);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: 0.0022,
            volume_master: 0.85,
            volume_ambience: 0.75,
            fov: 74.0,
            head_bob: 1.0,
            grain: 0.55,
            volumetric: true,
            shadows: true,
            invert_y: false,
        }
    }
}

#[derive(Resource)]
pub struct Fonts {
    pub title: Handle<Font>,
    pub doc: Handle<Font>,
    pub ui: Handle<Font>,
    pub ui_light: Handle<Font>,
}

#[derive(Resource)]
pub struct Textures {
    pub marble: Handle<Image>,
    pub marble_n: Handle<Image>,
    pub wood_floor: Handle<Image>,
    pub wood_floor_n: Handle<Image>,
    pub carpet: Handle<Image>,
    pub carpet_n: Handle<Image>,
    pub stone: Handle<Image>,
    pub stone_n: Handle<Image>,
    pub concrete: Handle<Image>,
    pub concrete_n: Handle<Image>,
    pub wallpaper: Handle<Image>,
    pub wallpaper_n: Handle<Image>,
    pub plaster: Vec<Handle<Image>>,
    pub plaster_n: Vec<Handle<Image>>,
    pub ceiling: Handle<Image>,
    pub ceiling_n: Handle<Image>,
    pub brick: Handle<Image>,
    pub brick_n: Handle<Image>,
    pub wood: Handle<Image>,
    pub wood_n: Handle<Image>,
    pub wood_light: Handle<Image>,
    pub wood_light_n: Handle<Image>,
    pub velvet: Handle<Image>,
    pub velvet_n: Handle<Image>,
    pub velvet_green: Handle<Image>,
    pub metal: Handle<Image>,
    pub metal_n: Handle<Image>,
    pub brass: Handle<Image>,
    pub brass_n: Handle<Image>,
    pub paper: Handle<Image>,
    pub leaf: Vec<Handle<Image>>,
    pub vine: Handle<Image>,
    pub moss: Handle<Image>,
    pub glass_rain: Handle<Image>,
    pub noise: Handle<Image>,
    pub rain_streaks: Handle<Image>,
    pub clock_face: Handle<Image>,
    pub piano_keys: Handle<Image>,
}

#[derive(Resource)]
pub struct Sounds {
    pub rain: Handle<AudioSource>,
    pub wind: Handle<AudioSource>,
    pub tone: Handle<AudioSource>,
    pub gen_hum: Handle<AudioSource>,
    pub elev: Handle<AudioSource>,
    pub buzz: Handle<AudioSource>,
    pub thunder_far: Vec<Handle<AudioSource>>,
    pub thunder_near: Vec<Handle<AudioSource>>,
    pub drip: Vec<Handle<AudioSource>>,
    pub creak: Vec<Handle<AudioSource>>,
    pub step_wood: Vec<Handle<AudioSource>>,
    pub step_stone: Vec<Handle<AudioSource>>,
    pub door_slam: Handle<AudioSource>,
    pub door_creak: Handle<AudioSource>,
    pub clock_tick: Handle<AudioSource>,
    pub gear_clunk: Handle<AudioSource>,
    pub keypad_beep: Handle<AudioSource>,
    pub keypad_error: Handle<AudioSource>,
    pub keypad_ok: Handle<AudioSource>,
    pub lever_clunk: Handle<AudioSource>,
    pub spark: Handle<AudioSource>,
    pub gen_start: Handle<AudioSource>,
    pub piano: Handle<AudioSource>,
    pub stinger: Handle<AudioSource>,
    pub heartbeat: Handle<AudioSource>,
    pub paper_pickup: Handle<AudioSource>,
    pub paper_flip: Handle<AudioSource>,
    pub whisper: Handle<AudioSource>,
    pub distant_steps: Handle<AudioSource>,
    pub switch_click: Handle<AudioSource>,
    pub led_on: Handle<AudioSource>,
    pub power_down: Handle<AudioSource>,
    pub ui_click: Handle<AudioSource>,
    pub ui_hover: Handle<AudioSource>,
    pub rain_stop: Handle<AudioSource>,
    pub metal_ring: Handle<AudioSource>,
    pub chair_scrape: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct GameAssets {
    pub fonts: Fonts,
    pub tex: Textures,
    pub snd: Sounds,
    /// Every image handle, used for the loading screen progress bar.
    pub images: Vec<Handle<Image>>,
    /// Every audio handle, used for the loading screen progress bar.
    pub audio: Vec<Handle<AudioSource>>,
}

impl GameAssets {
    pub fn progress(&self, images: &Assets<Image>, audio: &Assets<AudioSource>) -> f32 {
        let total = self.images.len() + self.audio.len();
        if total == 0 {
            return 1.0;
        }
        let have = self
            .images
            .iter()
            .filter(|h| images.contains(h.id()))
            .count()
            + self
                .audio
                .iter()
                .filter(|h| audio.contains(h.id()))
                .count();
        have as f32 / total as f32
    }

    pub fn ready(&self, images: &Assets<Image>, audio: &Assets<AudioSource>) -> bool {
        self.progress(images, audio) >= 0.999
    }
}

/// Tiny deterministic RNG so we do not need an extra crate and so that
/// repeated scares feel varied but reproducible enough to debug.
#[derive(Resource)]
pub struct Rng {
    state: u32,
}

impl Default for Rng {
    fn default() -> Self {
        Self { state: 0x9E37_79B9 }
    }
}

impl Rng {
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Uniform float in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }

    /// Uniform integer index in [0, n).
    pub fn index(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u32() as usize) % n
        }
    }
}

/// Everything that reads `GameAssets` has to run after this set.
///
/// `StatesPlugin` inserts the `StateTransition` schedule *before* `PreStartup`
/// (see `insert_startup_before(PreStartup, StateTransition)`), so the very first
/// `OnEnter(AppState::Boot)` fires before `load_assets` ever ran. Screen building
/// therefore lives in `PreStartup`, ordered after this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadAssetsSet;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::load())
            .init_resource::<UiState>()
            .init_resource::<Rng>()
            .add_systems(PreStartup, load_assets.in_set(LoadAssetsSet))
            .add_systems(
                Update,
                (tick_loading, print_ready_hint).run_if(in_state(AppState::Boot)),
            );
    }
}

/// Loads every texture, sound and font up front: the hotel is small enough
/// that a single load pass avoids any hitch in the middle of a scare.
fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let _ = &mut images;

    let tex = |path: &str| asset_server.load::<Image>(path.to_string());
    // normal maps must not be treated as sRGB colour data
    let nrm = |path: &str| {
        asset_server.load_with_settings::<Image, ImageLoaderSettings>(path.to_string(), |s| {
            s.is_srgb = false;
        })
    };

    let leaf: Vec<Handle<Image>> = vec![tex("textures/leaf_card.png"), tex("textures/leaf_card2.png")];

    let textures = Textures {
        marble: tex("textures/floor_marble.png"),
        marble_n: nrm("textures/floor_marble_n.png"),
        wood_floor: tex("textures/floor_wood.png"),
        wood_floor_n: nrm("textures/floor_wood_n.png"),
        carpet: tex("textures/carpet_runner.png"),
        carpet_n: nrm("textures/carpet_runner_n.png"),
        stone: tex("textures/stone_wet.png"),
        stone_n: nrm("textures/stone_wet_n.png"),
        concrete: tex("textures/concrete.png"),
        concrete_n: nrm("textures/concrete_n.png"),
        wallpaper: tex("textures/wall_wallpaper.png"),
        wallpaper_n: nrm("textures/wall_wallpaper_n.png"),
        plaster: vec![
            tex("textures/wall_plaster.png"),
            tex("textures/wall_plaster1.png"),
            tex("textures/wall_plaster2.png"),
            tex("textures/wall_plaster3.png"),
        ],
        plaster_n: vec![
            nrm("textures/wall_plaster_n.png"),
            nrm("textures/wall_plaster1_n.png"),
            nrm("textures/wall_plaster2_n.png"),
            nrm("textures/wall_plaster3_n.png"),
        ],
        ceiling: tex("textures/ceiling_plaster.png"),
        ceiling_n: nrm("textures/ceiling_plaster_n.png"),
        brick: tex("textures/wall_brick.png"),
        brick_n: nrm("textures/wall_brick_n.png"),
        wood: tex("textures/wood_dark.png"),
        wood_n: nrm("textures/wood_dark_n.png"),
        wood_light: tex("textures/wood_light.png"),
        wood_light_n: nrm("textures/wood_light_n.png"),
        velvet: tex("textures/velvet_red.png"),
        velvet_n: nrm("textures/velvet_red_n.png"),
        velvet_green: tex("textures/velvet_green.png"),
        metal: tex("textures/metal_rust.png"),
        metal_n: nrm("textures/metal_rust_n.png"),
        brass: tex("textures/brass.png"),
        brass_n: nrm("textures/brass_n.png"),
        paper: tex("textures/paper.png"),
        leaf: leaf.clone(),
        vine: tex("textures/vine_card.png"),
        moss: tex("textures/moss_patch.png"),
        glass_rain: tex("textures/glass_rain.png"),
        noise: tex("textures/noise_grain.png"),
        rain_streaks: tex("textures/rain_streaks.png"),
        clock_face: tex("textures/clock_face.png"),
        piano_keys: tex("textures/piano_keys.png"),
    };

    let mut image_list: Vec<Handle<Image>> = vec![
        textures.marble.clone(),
        textures.marble_n.clone(),
        textures.wood_floor.clone(),
        textures.wood_floor_n.clone(),
        textures.carpet.clone(),
        textures.carpet_n.clone(),
        textures.stone.clone(),
        textures.stone_n.clone(),
        textures.concrete.clone(),
        textures.concrete_n.clone(),
        textures.wallpaper.clone(),
        textures.wallpaper_n.clone(),
        textures.ceiling.clone(),
        textures.ceiling_n.clone(),
        textures.brick.clone(),
        textures.brick_n.clone(),
        textures.wood.clone(),
        textures.wood_n.clone(),
        textures.wood_light.clone(),
        textures.wood_light_n.clone(),
        textures.velvet.clone(),
        textures.velvet_n.clone(),
        textures.velvet_green.clone(),
        textures.metal.clone(),
        textures.metal_n.clone(),
        textures.brass.clone(),
        textures.brass_n.clone(),
        textures.paper.clone(),
        textures.vine.clone(),
        textures.moss.clone(),
        textures.glass_rain.clone(),
        textures.noise.clone(),
        textures.rain_streaks.clone(),
        textures.clock_face.clone(),
        textures.piano_keys.clone(),
    ];
    image_list.extend(textures.plaster.iter().cloned());
    image_list.extend(textures.plaster_n.iter().cloned());
    image_list.extend(leaf.iter().cloned());

    let snd = |path: &str| asset_server.load::<AudioSource>(path.to_string());

    let sounds = Sounds {
        rain: snd("audio/rain_loop.wav"),
        wind: snd("audio/wind_loop.wav"),
        tone: snd("audio/tone_loop.wav"),
        gen_hum: snd("audio/gen_hum_loop.wav"),
        elev: snd("audio/elev_loop.wav"),
        buzz: snd("audio/buzz_loop.wav"),
        thunder_far: vec![
            snd("audio/thunder_far_1.wav"),
            snd("audio/thunder_far_2.wav"),
            snd("audio/thunder_far_3.wav"),
        ],
        thunder_near: vec![
            snd("audio/thunder_near_1.wav"),
            snd("audio/thunder_near_2.wav"),
        ],
        drip: vec![
            snd("audio/drip_1.wav"),
            snd("audio/drip_2.wav"),
            snd("audio/drip_3.wav"),
        ],
        creak: vec![
            snd("audio/creak_1.wav"),
            snd("audio/creak_2.wav"),
            snd("audio/creak_3.wav"),
            snd("audio/creak_4.wav"),
        ],
        step_wood: vec![
            snd("audio/step_wood_1.wav"),
            snd("audio/step_wood_2.wav"),
            snd("audio/step_wood_3.wav"),
            snd("audio/step_wood_4.wav"),
        ],
        step_stone: vec![
            snd("audio/step_stone_1.wav"),
            snd("audio/step_stone_2.wav"),
            snd("audio/step_stone_3.wav"),
        ],
        door_slam: snd("audio/door_slam.wav"),
        door_creak: snd("audio/door_creak.wav"),
        clock_tick: snd("audio/clock_tick.wav"),
        gear_clunk: snd("audio/gear_clunk.wav"),
        keypad_beep: snd("audio/keypad_beep.wav"),
        keypad_error: snd("audio/keypad_error.wav"),
        keypad_ok: snd("audio/keypad_ok.wav"),
        lever_clunk: snd("audio/lever_clunk.wav"),
        spark: snd("audio/spark.wav"),
        gen_start: snd("audio/gen_start.wav"),
        piano: snd("audio/piano_note.wav"),
        stinger: snd("audio/stinger.wav"),
        heartbeat: snd("audio/heartbeat.wav"),
        paper_pickup: snd("audio/paper_pickup.wav"),
        paper_flip: snd("audio/paper_flip.wav"),
        whisper: snd("audio/whisper.wav"),
        distant_steps: snd("audio/distant_steps.wav"),
        switch_click: snd("audio/switch_click.wav"),
        led_on: snd("audio/led_on.wav"),
        power_down: snd("audio/power_down.wav"),
        ui_click: snd("audio/ui_click.wav"),
        ui_hover: snd("audio/ui_hover.wav"),
        rain_stop: snd("audio/rain_stop.wav"),
        metal_ring: snd("audio/metal_ring.wav"),
        chair_scrape: snd("audio/chair_scrape.wav"),
    };

    let mut audio_list: Vec<Handle<AudioSource>> = vec![
        sounds.rain.clone(),
        sounds.wind.clone(),
        sounds.tone.clone(),
        sounds.gen_hum.clone(),
        sounds.elev.clone(),
        sounds.buzz.clone(),
        sounds.door_slam.clone(),
        sounds.door_creak.clone(),
        sounds.clock_tick.clone(),
        sounds.gear_clunk.clone(),
        sounds.keypad_beep.clone(),
        sounds.keypad_error.clone(),
        sounds.keypad_ok.clone(),
        sounds.lever_clunk.clone(),
        sounds.spark.clone(),
        sounds.gen_start.clone(),
        sounds.piano.clone(),
        sounds.stinger.clone(),
        sounds.heartbeat.clone(),
        sounds.paper_pickup.clone(),
        sounds.paper_flip.clone(),
        sounds.whisper.clone(),
        sounds.distant_steps.clone(),
        sounds.switch_click.clone(),
        sounds.led_on.clone(),
        sounds.power_down.clone(),
        sounds.ui_click.clone(),
        sounds.ui_hover.clone(),
        sounds.rain_stop.clone(),
        sounds.metal_ring.clone(),
        sounds.chair_scrape.clone(),
    ];
    audio_list.extend(sounds.thunder_far.iter().cloned());
    audio_list.extend(sounds.thunder_near.iter().cloned());
    audio_list.extend(sounds.drip.iter().cloned());
    audio_list.extend(sounds.creak.iter().cloned());
    audio_list.extend(sounds.step_wood.iter().cloned());
    audio_list.extend(sounds.step_stone.iter().cloned());

    let fonts = Fonts {
        title: asset_server.load("fonts/PlayfairDisplay.ttf"),
        doc: asset_server.load("fonts/SpecialElite.ttf"),
        ui: asset_server.load("fonts/BarlowCondensed-Medium.ttf"),
        ui_light: asset_server.load("fonts/BarlowCondensed-Light.ttf"),
    };

    // the material library is built from the textures, so it belongs here
    let palette = crate::palette::Palette::new(&textures, &mut materials);

    commands.insert_resource(GameAssets {
        fonts,
        tex: textures,
        snd: sounds,
        images: image_list,
        audio: audio_list,
    });
    commands.insert_resource(palette);
}

/// Boot state simply waits for the asset server, then hands over to the menu.
/// (The loading screen itself lives in `ui::menus`.)
fn tick_loading(
    assets: Res<GameAssets>,
    images: Res<Assets<Image>>,
    audio: Res<Assets<AudioSource>>,
    mut next: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut waited: Local<f32>,
) {
    *waited += time.delta_secs();
    let all_loaded = assets.ready(&images, &audio);
    // the timeout keeps one unreadable file from freezing the game on the
    // loading screen with no way out
    if (*waited > 0.25 && all_loaded) || *waited > 25.0 {
        next.set(AppState::Title);
    }
}

fn print_ready_hint() {}
