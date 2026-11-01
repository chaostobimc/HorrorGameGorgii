//! Interface: HUD, documents, keypad, menus and the screen grade.

pub mod debug;
pub mod documents;
pub mod hud;
pub mod keypad;
pub mod menus;
pub mod screen_fx;

use bevy::prelude::*;

use crate::core::{AppState, GameAssets, UiState};
use screen_fx::{ScreenFxMaterial, ScreenGrade};

pub const COL_BRIGHT: Color = Color::srgba(0.93, 0.90, 0.83, 0.95);
pub const COL_TEXT: Color = Color::srgba(0.88, 0.85, 0.78, 0.90);
pub const COL_DIM: Color = Color::srgba(0.75, 0.71, 0.64, 0.55);
pub const COL_FAINT: Color = Color::srgba(0.70, 0.66, 0.58, 0.30);
pub const COL_GOLD: Color = Color::srgba(0.86, 0.74, 0.48, 0.95);
pub const COL_PANEL: Color = Color::srgba(0.045, 0.05, 0.055, 0.94);

/// Fade to/from black, used for area transitions and the ending.
#[derive(Resource)]
pub struct Fade {
    pub alpha: f32,
    pub target: f32,
    pub speed: f32,
}

impl Default for Fade {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            target: 0.0,
            speed: 0.9,
        }
    }
}

impl Fade {
    pub fn to_black(&mut self, speed: f32) {
        self.target = 1.0;
        self.speed = speed;
    }
    pub fn to_clear(&mut self, speed: f32) {
        self.target = 0.0;
        self.speed = speed;
    }
}

/// Cinematic black bars for scripted moments.
#[derive(Resource, Default)]
pub struct Cinematic(pub bool);

/// Modal UI opened with TAB: the notebook.
#[derive(Resource, Default)]
pub struct JournalOpen(pub bool);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<ScreenFxMaterial>::default())
            .init_resource::<ScreenGrade>()
            .init_resource::<Fade>()
            .init_resource::<Cinematic>()
            .init_resource::<JournalOpen>()
            .init_resource::<keypad::Keypad>()
            .add_event::<keypad::SafeOpened>()
            .add_systems(
                OnEnter(AppState::Playing),
                (hud::spawn_hud, spawn_grade_layer, debug::spawn_debug_hud),
            )
            .add_systems(
                Update,
                debug::update_debug.run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, tick_fade)
            .add_systems(
                Update,
                (
                    (documents::close_document, documents::close_journal, documents::sync_viewer)
                        .chain(),
                    documents::sync_journal,
                    keypad::sync_keypad,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    hud::update_prompt,
                    hud::update_objective,
                    hud::update_notes,
                    hud::update_subtitle,
                    hud::update_fade,
                    hud::update_letterbox,
                    hud::toggle_journal,
                    hud::hide_hud,
                    screen_fx::update_material,
                    menus::pause_menu,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    menus::settings_cursor,
                    menus::apply_settings,
                    menus::hover_sound,
                    menus::press_sound,
                    menus::loading_progress,
                ),
            )
            .add_systems(OnEnter(AppState::Title), menus::spawn_title)
            .add_systems(OnExit(AppState::Title), menus::cleanup_title)
            // the loading screen cannot use OnEnter(Boot): the first state
            // transition runs before PreStartup, i.e. before GameAssets exists
            .add_systems(
                PreStartup,
                (menus::spawn_loading, menus::spawn_menu_camera)
                    .chain()
                    .after(crate::core::LoadAssetsSet),
            )
            // In `Title` the menu is drawn on its own 3D camera inside the
            // lobby, so the 2D menu camera must go away - otherwise it would
            // claim the UI pass and the 3D camera would paint over the menu.
            .add_systems(OnEnter(AppState::Title), menus::despawn_menu_camera)
            .add_systems(OnEnter(AppState::Ending), menus::spawn_menu_camera)
            .add_systems(OnEnter(AppState::Playing), menus::despawn_menu_camera)
            .add_systems(OnEnter(AppState::Ending), menus::spawn_ending)
            .add_systems(
                Update,
                (
                    menus::title_buttons,
                    menus::title_overlay,
                    menus::title_camera_drift,
                )
                    .chain()
                    .run_if(in_state(AppState::Title)),
            )
            .add_systems(Update, menus::save_settings)
            .add_systems(
                Update,
                menus::ending_tick.run_if(in_state(AppState::Ending)),
            );
    }
}

fn spawn_grade_layer(
    mut commands: Commands,
    mut materials: ResMut<Assets<ScreenFxMaterial>>,
    assets: Res<GameAssets>,
) {
    screen_fx::spawn_layer(&mut commands, &mut materials, &assets);
}

fn tick_fade(time: Res<Time>, mut fade: ResMut<Fade>) {
    let dt = time.delta_secs();
    let diff = fade.target - fade.alpha;
    if diff.abs() < 0.001 {
        fade.alpha = fade.target;
        return;
    }
    let step = fade.speed * dt;
    fade.alpha += diff.signum() * step.min(diff.abs());
    fade.alpha = fade.alpha.clamp(0.0, 1.0);
}

/// Convenience: is the mouse free for menus?
pub fn ui_open(ui: &UiState, journal: &JournalOpen) -> bool {
    ui.overlay != crate::core::Overlay::None || journal.0
}
