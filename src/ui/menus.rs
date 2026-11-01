//! Loading screen, title screen, pause menu, settings and the end card.

use bevy::app::AppExit;
use bevy::pbr::VolumetricFog;
use bevy::prelude::*;
use bevy::time::Virtual;

use crate::audio::PlaySfx;
use crate::core::{AppState, GameAssets, Overlay, Settings, UiState};
use crate::player::{Flashlight, PlayerCamera};

use super::{Cinematic, Fade, JournalOpen, COL_BRIGHT, COL_DIM, COL_FAINT, COL_GOLD, COL_PANEL};

#[derive(Component)]
pub struct TitleRoot;
#[derive(Component)]
pub struct TitleCamera;
#[derive(Component)]
pub struct LoadingRoot;
#[derive(Component)]
pub struct LoadingBar;
#[derive(Component)]
pub struct PauseRoot;
#[derive(Component)]
pub struct EndRoot;
#[derive(Component)]
pub struct MenuButton(pub MenuAction);
#[derive(Component)]
pub struct SettingsStep(pub usize, pub f32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    StartGame,
    OpenSettings,
    CloseSettings,
    Resume,
    BackToTitle,
    Quit,
    RestartFromEnd,
}

/// The camera the loading screen, the title menu and the ending are drawn on.
///
/// The 3D camera only exists while playing, so without this the UI in `Boot`,
/// `Title` and `Ending` would have no view to render into and stay invisible.
#[derive(Component)]
pub struct MenuCamera;

pub fn spawn_menu_camera(mut commands: Commands, existing: Query<&MenuCamera>) {
    if existing.iter().next().is_some() {
        return;
    }
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.008, 0.009, 0.011)),
            ..default()
        },
        IsDefaultUiCamera,
        MenuCamera,
        Name::new("MenuCamera"),
    ));
}

pub fn despawn_menu_camera(mut commands: Commands, existing: Query<Entity, With<MenuCamera>>) {
    for e in existing.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn spawn_loading(mut commands: Commands, assets: Res<GameAssets>) {
    let title = assets.fonts.title.clone();
    let ui = assets.fonts.ui.clone();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(26.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.008, 0.009, 0.011)),
            GlobalZIndex(80),
            LoadingRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("The Diplomat Peak"),
                TextFont {
                    font: title.clone(),
                    font_size: 46.0,
                    ..default()
                },
                TextColor(Color::srgba(0.80, 0.75, 0.64, 0.55)),
            ));
            root.spawn((
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.80, 0.75, 0.64, 0.15)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Node {
                        width: Val::Percent(2.0),
                        height: Val::Px(1.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.90, 0.80, 0.55, 0.75)),
                    LoadingBar,
                ));
            });
            root.spawn((
                Text::new("DAS HAUS SAMMELT SEINE GESCHICHTE"),
                TextFont {
                    font: ui.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(COL_FAINT),
            ));
        });
}

pub fn loading_progress(
    assets: Res<GameAssets>,
    images: Res<Assets<Image>>,
    audio: Res<Assets<bevy::audio::AudioSource>>,
    mut bar: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = bar.get_single_mut() {
        let p = (assets.progress(&images, &audio) * 100.0).max(2.0);
        node.width = Val::Percent(p);
    }
}

pub fn spawn_title(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut fade: ResMut<Fade>,
    mut cin: ResMut<Cinematic>,
    loading: Query<Entity, With<LoadingRoot>>,
) {
    for e in loading.iter() {
        commands.entity(e).despawn_recursive();
    }
    fade.alpha = 1.0;
    fade.target = 0.0;
    fade.speed = 0.55;
    cin.0 = false;

    let title = assets.fonts.title.clone();
    let ui = assets.fonts.ui.clone();
    let doc = assets.fonts.doc.clone();

    // Slow drifting camera in the lobby - the menu is a real place.
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.01, 0.012, 0.015)),
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 62f32.to_radians(),
            near: 0.05,
            far: 120.0,
            ..default()
        }),
        bevy::render::camera::Exposure { ev100: 6.0 },
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
        bevy::core_pipeline::bloom::Bloom {
            intensity: 0.12,
            ..bevy::core_pipeline::bloom::Bloom::NATURAL
        },
        Transform::from_xyz(-8.5, 1.75, 7.0).looking_at(Vec3::new(2.0, 2.4, -6.0), Vec3::Y),
        TitleCamera,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                padding: UiRect::left(Val::Px(120.0)),
                ..default()
            },
            GlobalZIndex(20),
            TitleRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("B A G U I O   C I T Y   ·   C O R D I L L E R A   ·   1 9 8 8"),
                TextFont {
                    font: ui.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COL_FAINT),
            ));
            root.spawn((
                Text::new("The Diplomat Peak"),
                TextFont {
                    font: title.clone(),
                    font_size: 84.0,
                    ..default()
                },
                TextColor(Color::srgba(0.92, 0.89, 0.82, 0.96)),
                Node {
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                },
            ));
            root.spawn((
                Text::new("Das letzte Kapitel"),
                TextFont {
                    font: doc.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(COL_GOLD),
                Node {
                    margin: UiRect::top(Val::Px(-6.0)),
                    ..default()
                },
            ));
            root.spawn((
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(26.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.86, 0.74, 0.48, 0.35)),
            ));

            for (label, action) in [
                ("NEUES SPIEL", MenuAction::StartGame),
                ("EINSTELLUNGEN", MenuAction::OpenSettings),
                ("BEENDEN", MenuAction::Quit),
            ] {
                root.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(9.0)),
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    MenuButton(action),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont {
                            font: ui.clone(),
                            font_size: 21.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.82, 0.79, 0.72, 0.80)),
                        MenuLabel,
                    ));
                });
            }

            root.spawn((
                Text::new("Kopfhörer empfohlen. Das Spiel enthält Stroboskoplicht."),
                TextFont {
                    font: ui.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.70, 0.66, 0.58, 0.32)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(42.0),
                    left: Val::Px(120.0),
                    ..default()
                },
            ));
        });
}

#[derive(Component)]
pub struct MenuLabel;

pub fn title_buttons(
    mut interactions: Query<(&Interaction, &MenuButton, &mut BackgroundColor), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    mut ui: ResMut<UiState>,
    mut fade: ResMut<Fade>,
    mut exit: EventWriter<AppExit>,
) {
    // Escape hatch: the game must start even if mouse picking on this machine
    // never delivers a click to the buttons.
    if keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::NumpadEnter)
        || keys.just_pressed(KeyCode::Space)
    {
        ui.overlay = Overlay::None;
        fade.to_black(1.1);
        next.set(AppState::Playing);
        return;
    }
    for (interaction, button, mut bg) in interactions.iter_mut() {
        bg.0 = match *interaction {
            Interaction::Hovered => Color::srgba(0.86, 0.74, 0.48, 0.07),
            Interaction::Pressed => Color::srgba(0.86, 0.74, 0.48, 0.14),
            Interaction::None => Color::NONE,
        };
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button.0 {
            MenuAction::StartGame => {
                ui.overlay = Overlay::None;
                fade.to_black(1.1);
                next.set(AppState::Playing);
            }
            MenuAction::OpenSettings => {
                ui.overlay = Overlay::Settings;
            }
            MenuAction::Quit => {
                exit.send(AppExit::Success);
            }
            _ => {}
        }
    }
}

pub fn spawn_ending(mut commands: Commands, assets: Res<GameAssets>) {
    let title = assets.fonts.title.clone();
    let ui = assets.fonts.ui.clone();
    let doc = assets.fonts.doc.clone();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(90),
            EndRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("The Diplomat Peak"),
                TextFont {
                    font: title.clone(),
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgba(0.85, 0.81, 0.72, 0.9)),
            ));
            root.spawn((
                Text::new("DAS LETZTE KAPITEL"),
                TextFont {
                    font: ui.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(COL_FAINT),
            ));
            root.spawn((
                Node {
                    width: Val::Px(240.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(18.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.86, 0.74, 0.48, 0.25)),
            ));
            root.spawn((
                Text::new(
                    "Gorgii hat das Resort verlassen.\nWas er auf der Terrasse gesehen hat, steht in keinem Bericht.",
                ),
                TextFont {
                    font: doc.clone(),
                    font_size: 17.0,
                    ..default()
                },
                TextColor(Color::srgba(0.72, 0.69, 0.63, 0.6)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));
            root.spawn((
                Text::new("ESC ODER EINGABETASTE ZUM BEENDEN"),
                TextFont {
                    font: ui.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(0.70, 0.66, 0.58, 0.30)),
                Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
            ));
        });
}

pub fn ending_tick(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut elapsed: Local<f32>,
    mut exit: EventWriter<AppExit>,
) {
    *elapsed += time.delta_secs();
    let quit = keys.just_pressed(KeyCode::Escape)
        || keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::Space);
    if quit || *elapsed > 26.0 {
        exit.send(AppExit::Success);
    }
}

/// Pause menu, also hosts the settings panel.
pub fn pause_menu(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    assets: Res<GameAssets>,
    mut ui: ResMut<UiState>,
    mut journal: ResMut<JournalOpen>,
    state: Res<State<AppState>>,
    mut time: ResMut<Time<Virtual>>,
    mut exit: EventWriter<AppExit>,
    existing: Query<Entity, With<PauseRoot>>,
    mut interactions: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next: ResMut<NextState<AppState>>,
    mut settings: ResMut<Settings>,
    mut last_page: Local<Overlay>,
) {
    if *state.get() != AppState::Playing {
        return;
    }

    // open / close
    if keys.just_pressed(KeyCode::Escape) {
        if ui.overlay == Overlay::None {
            journal.0 = false;
            ui.overlay = Overlay::Pause;
        } else if ui.overlay == Overlay::Pause {
            ui.overlay = Overlay::None;
        } else if ui.overlay == Overlay::Settings {
            ui.overlay = Overlay::Pause;
        }
    }

    let paused = ui.overlay == Overlay::Pause || ui.overlay == Overlay::Settings;
    if paused && !time.is_paused() {
        time.pause();
    } else if !paused && time.is_paused() {
        time.unpause();
    }

    // rebuild panel when the desired page changes
    let have = existing.iter().next().is_some();
    let want_settings = ui.overlay == Overlay::Settings;
    if *last_page != ui.overlay || (paused && !have) {
        for e in existing.iter() {
            commands.entity(e).despawn_recursive();
        }
        if paused {
            spawn_pause(&mut commands, &assets, &settings, want_settings);
        }
        *last_page = if paused { ui.overlay } else { Overlay::None };
    }

    for (interaction, button) in interactions.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button.0 {
            MenuAction::Resume => ui.overlay = Overlay::None,
            MenuAction::OpenSettings => ui.overlay = Overlay::Settings,
            MenuAction::CloseSettings => ui.overlay = Overlay::Pause,
            MenuAction::Quit => {
                exit.send(AppExit::Success);
            }
            MenuAction::BackToTitle => {
                ui.overlay = Overlay::None;
                time.unpause();
                next.set(AppState::Title);
            }
            _ => {}
        }
    }
    let _ = &mut settings;
}

fn spawn_pause(
    commands: &mut Commands,
    assets: &GameAssets,
    settings: &Settings,
    show_settings: bool,
) {
    let ui = assets.fonts.ui.clone();
    let ui_light = assets.fonts.ui_light.clone();
    let title = assets.fonts.title.clone();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.62)),
            GlobalZIndex(70),
            PauseRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(520.0),
                    padding: UiRect::all(Val::Px(44.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(COL_PANEL),
                BorderColor(Color::srgba(0.75, 0.70, 0.58, 0.20)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(if show_settings { "Einstellungen" } else { "Pause" }),
                    TextFont {
                        font: title.clone(),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(COL_GOLD),
                ));
                panel.spawn((
                    Node {
                        height: Val::Px(1.0),
                        width: Val::Percent(100.0),
                        margin: UiRect::vertical(Val::Px(14.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.75, 0.70, 0.58, 0.16)),
                ));

                if show_settings {
                    let rows: [(&str, String, usize); 6] = [
                        ("Maus-Empfindlichkeit", format!("{:.1}", settings.sensitivity * 1000.0), 0),
                        ("Lautstärke", format!("{}%", (settings.volume_master * 100.0) as i32), 1),
                        ("Blickfeld", format!("{}°", settings.fov as i32), 2),
                        ("Filmkorn", format!("{}%", (settings.grain * 100.0) as i32), 3),
                        ("Volumetrischer Nebel", if settings.volumetric { "AN".into() } else { "AUS".into() }, 4),
                        ("Schatten", if settings.shadows { "AN".into() } else { "AUS".into() }, 5),
                    ];
                    for (label, value, index) in rows {
                        panel
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                width: Val::Percent(100.0),
                                height: Val::Px(40.0),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    Text::new(label),
                                    TextFont {
                                        font: ui_light.clone(),
                                        font_size: 19.0,
                                        ..default()
                                    },
                                    TextColor(COL_BRIGHT),
                                    Node {
                                        width: Val::Px(280.0),
                                        ..default()
                                    },
                                ));
                                for (sym, dir) in [("‹", -1.0_f32), ("›", 1.0_f32)] {
                                    row.spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(34.0),
                                            height: Val::Px(30.0),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.12, 0.13, 0.14, 0.9)),
                                        BorderColor(Color::srgba(0.5, 0.47, 0.40, 0.3)),
                                        SettingsStep(index, dir),
                                    ))
                                    .with_children(|b| {
                                        b.spawn((
                                            Text::new(sym),
                                            TextFont {
                                                font: ui.clone(),
                                                font_size: 18.0,
                                                ..default()
                                            },
                                            TextColor(COL_DIM),
                                        ));
                                    });
                                }
                                row.spawn((
                                    Text::new(value),
                                    TextFont {
                                        font: ui.clone(),
                                        font_size: 17.0,
                                        ..default()
                                    },
                                    TextColor(COL_GOLD),
                                    Node {
                                        margin: UiRect::left(Val::Px(8.0)),
                                        ..default()
                                    },
                                ));
                            });
                    }
                    panel.spawn((
                        Node {
                            height: Val::Px(1.0),
                            width: Val::Percent(100.0),
                            margin: UiRect::vertical(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.75, 0.70, 0.58, 0.16)),
                    ));
                    menu_button(panel, &ui, "ZURÜCK", MenuAction::CloseSettings);
                } else {
                    menu_button(panel, &ui, "FORTSETZEN", MenuAction::Resume);
                    menu_button(panel, &ui, "EINSTELLUNGEN", MenuAction::OpenSettings);
                    menu_button(panel, &ui, "HAUPTMENÜ", MenuAction::BackToTitle);
                    menu_button(panel, &ui, "SPIEL BEENDEN", MenuAction::Quit);
                }
            });
        });
}

fn menu_button(parent: &mut ChildBuilder, font: &Handle<Font>, label: &str, action: MenuAction) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                width: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            MenuButton(action),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 21.0,
                    ..default()
                },
                TextColor(Color::srgba(0.84, 0.81, 0.74, 0.85)),
            ));
        });
}

/// Settings arrows (works both in the pause menu and on the title screen).
pub fn settings_cursor(
    mut q: Query<(&Interaction, &SettingsStep), Changed<Interaction>>,
    mut settings: ResMut<Settings>,
) {
    for (interaction, step) in q.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let d = step.1;
        match step.0 {
            0 => {
                settings.sensitivity =
                    (settings.sensitivity + 0.0002 * d).clamp(0.0006, 0.0055);
            }
            1 => {
                settings.volume_master = (settings.volume_master + 0.1 * d).clamp(0.0, 1.0);
                settings.volume_ambience = (settings.volume_ambience + 0.1 * d).clamp(0.0, 1.0);
            }
            2 => settings.fov = (settings.fov + 2.0 * d).clamp(60.0, 92.0),
            3 => settings.grain = (settings.grain + 0.1 * d).clamp(0.0, 1.0),
            4 => settings.volumetric = !settings.volumetric,
            5 => settings.shadows = !settings.shadows,
            _ => {}
        }
    }
}

/// Pushes live settings into the camera and lights.
pub fn apply_settings(
    settings: Res<Settings>,
    mut commands: Commands,
    mut cam: Query<(Entity, &mut Projection, Option<&VolumetricFog>), With<PlayerCamera>>,
    mut lights: Query<&mut SpotLight, With<Flashlight>>,
) {
    if !settings.is_changed() {
        return;
    }
    if let Ok((entity, mut projection, fog)) = cam.get_single_mut() {
        if let Projection::Perspective(p) = &mut *projection {
            p.fov = settings.fov.to_radians();
        }
        let has_fog = fog.is_some();
        if settings.volumetric && !has_fog {
            commands.entity(entity).insert(VolumetricFog {
                ambient_intensity: 0.06,
                jitter: 0.5,
                step_count: 32,
                ..default()
            });
        } else if !settings.volumetric && has_fog {
            commands.entity(entity).remove::<VolumetricFog>();
        }
    }
    for mut light in lights.iter_mut() {
        light.shadows_enabled = settings.shadows;
    }
}

pub fn hover_sound(
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
    q: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in q.iter() {
        if *interaction == Interaction::Hovered {
            sfx.send(PlaySfx::flat(assets.snd.ui_hover.clone(), 0.20));
        }
    }
}

pub fn press_sound(
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
    q: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in q.iter() {
        if *interaction == Interaction::Pressed {
            sfx.send(PlaySfx::flat(assets.snd.ui_click.clone(), 0.35));
        }
    }
}

/// Removes the menu camera and UI when play starts.
pub fn cleanup_title(mut commands: Commands, q: Query<Entity, Or<(With<TitleRoot>, With<TitleCamera>)>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
