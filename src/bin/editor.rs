//! The Diplomat Peak — Level Editor.
//!
//! A separate program (`cargo run --release --bin editor`) that shares the
//! whole game library: it renders the real hotel with the real palette, the
//! real props and the real lights. Everything you place is written to
//! `assets/level/extras.lvl`; the game reads that file when it builds the
//! level, so editor work is in the game, behaviour included.
//!
//! Controls
//!   WASD / QE        fly, hold shift to go faster
//!   right mouse      look around
//!   left mouse       place the selected tool, or pick an object
//!   arrows / U O     move the picked object, `,` `.` rotate it
//!   C / Del          duplicate / delete the picked object
//!   F5 / F9          save / reload the level file

use bevy::audio::AudioSource;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::WindowResolution;

use diplomat_peak::collision::{ray_sphere, SolidBox};
use diplomat_peak::core::{AppState, CorePlugin, GameAssets, Rng};
use diplomat_peak::factory::{self, KindClass};
use diplomat_peak::level::{LevelFile, LevelObject, LEVEL_PATH};
use diplomat_peak::palette::Palette;
use diplomat_peak::world::{self, BuildCtx, F1, F2};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The Diplomat Peak — Level Editor".to_string(),
                resolution: WindowResolution::new(1600.0, 900.0),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_plugins(CorePlugin)
        .init_resource::<Editor>()
        .add_systems(Startup, spawn_editor_camera)
        .add_systems(
            Update,
            (
                build_world_once,
                fly_camera,
                edit_world,
                ui_buttons,
                save_load_keys,
                update_markers,
                button_tint,
                update_status,
            )
                .chain(),
        )
        .run();
}

// ---------------------------------------------------------------------------
// state
// ---------------------------------------------------------------------------

#[derive(Component)]
struct EditorCam {
    yaw: f32,
    pitch: f32,
}

#[derive(Component)]
struct HitMarker;

#[derive(Component)]
struct SelectMarker;

#[derive(Component)]
struct EditorStatusText;

#[derive(Component)]
struct KindButton(usize);

#[derive(Component)]
struct AreaButton(usize);

#[derive(Component)]
struct SelectToolButton;

/// The colour a button returns to when the mouse leaves it.
#[derive(Component)]
struct BaseTint(Color);

struct Placed {
    object: LevelObject,
    root: Entity,
}

#[derive(Resource, Default)]
struct Editor {
    built: bool,
    /// Selected catalogue entry, `None` = pick objects instead of placing.
    tool: Option<usize>,
    selected: Option<usize>,
    status: String,
    hit: Option<Vec3>,
    placed: Vec<Placed>,
}

/// Viewpoints for every area of the hotel.
const AREAS: &[(&str, Vec3, f32)] = &[
    ("Eingang / Veranda", Vec3::new(0.0, 1.8, 17.0), 0.0),
    ("Lobby", Vec3::new(0.0, 1.9, 8.0), 0.0),
    ("Rezeption", Vec3::new(-7.0, 1.9, -5.0), 0.0),
    ("Tresor", Vec3::new(-9.5, 1.6, -6.2), 0.0),
    ("Treppenhaus", Vec3::new(7.5, 1.9, -10.5), 0.0),
    ("Korridor", Vec3::new(11.0, F1 + 1.7, -20.0), 1.57),
    ("Maschinenraum", Vec3::new(19.5, F1 + 1.7, -19.5), 0.0),
    ("Ballsaal", Vec3::new(-5.0, F2 + 1.7, 2.0), 0.0),
    ("Terrasse", Vec3::new(8.0, F2 + 1.7, -2.0), -1.57),
    ("Tor / Gang", Vec3::new(23.0, F2 + 1.7, -7.0), -1.57),
];

// ---------------------------------------------------------------------------
// setup
// ---------------------------------------------------------------------------

fn spawn_editor_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.025, 0.03)),
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 70f32.to_radians(),
            near: 0.04,
            far: 320.0,
            ..default()
        }),
        bevy::render::camera::Exposure { ev100: 6.2 },
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
        bevy::core_pipeline::bloom::Bloom {
            intensity: 0.14,
            ..bevy::core_pipeline::bloom::Bloom::NATURAL
        },
        Msaa::Sample4,
        IsDefaultUiCamera,
        Transform::from_xyz(0.0, 1.9, 8.0),
        EditorCam {
            yaw: 0.0,
            pitch: -0.05,
        },
    ));
}

/// Builds the real hotel once the generated assets are ready, then loads the
/// objects that are already in the level file.
fn build_world_once(
    mut commands: Commands,
    assets: Res<GameAssets>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rng: ResMut<Rng>,
    images: Res<Assets<Image>>,
    audio: Res<Assets<AudioSource>>,
    mut ambient: ResMut<AmbientLight>,
    mut state: ResMut<Editor>,
) {
    if state.built || !assets.ready(&images, &audio) {
        return;
    }

    ambient.color = Color::srgb(0.46, 0.51, 0.60);
    ambient.brightness = 60.0;
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.52, 0.62, 0.86),
            illuminance: 320.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(30.0, 60.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mut b = BuildCtx::new();
    world::foyer::build(
        &mut commands,
        &mut b,
        &palette,
        &mut meshes,
        &assets,
        &mut rng,
    );
    world::corridor::build(
        &mut commands,
        &mut b,
        &palette,
        &mut meshes,
        &assets,
        &mut rng,
    );
    world::penthouse::build(
        &mut commands,
        &mut b,
        &palette,
        &mut meshes,
        &assets,
        &mut rng,
    );
    world::detail::dress(&mut commands, &mut b, &palette, &mut meshes);
    b.finish(&mut commands, &mut meshes, "architecture");

    let file = LevelFile::load();
    let count = file.objects.len();
    for object in file.objects {
        let root = factory::spawn_object(&mut commands, &mut meshes, &palette, &assets, &object);
        state.placed.push(Placed { object, root });
    }

    // markers: where the cursor lands, and what is picked
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.16, 0.16, 0.16))),
        MeshMaterial3d(palette.bulb.clone()),
        Transform::from_xyz(0.0, -60.0, 0.0),
        HitMarker,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.34, 0.34, 0.34))),
        MeshMaterial3d(palette.led_green.clone()),
        Transform::from_xyz(0.0, -60.0, 0.0),
        SelectMarker,
    ));

    spawn_ui(&mut commands, &assets);

    state.built = true;
    state.status = format!("{} Objekte aus {} geladen", count, LEVEL_PATH);
}

// ---------------------------------------------------------------------------
// camera
// ---------------------------------------------------------------------------

fn forward(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    )
}

fn fly_camera(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut EditorCam)>,
) {
    let Ok((mut transform, mut cam)) = q.get_single_mut() else {
        return;
    };

    let looking = buttons.pressed(MouseButton::Right);
    let mut looked = Vec2::ZERO;
    for event in motion.read() {
        looked += event.delta;
    }
    if looking {
        cam.yaw -= looked.x * 0.0032;
        cam.pitch = (cam.pitch - looked.y * 0.0032).clamp(-1.5, 1.5);
    }

    let mut dir = Vec3::ZERO;
    let aim = forward(cam.yaw, cam.pitch);
    let right = Vec3::new(cam.yaw.cos(), 0.0, -cam.yaw.sin());
    if keys.pressed(KeyCode::KeyW) {
        dir += aim;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= aim;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir -= right;
    }
    if keys.pressed(KeyCode::KeyE) {
        dir += Vec3::Y;
    }
    if keys.pressed(KeyCode::KeyQ) {
        dir -= Vec3::Y;
    }

    let speed = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        20.0
    } else {
        6.5
    };
    if dir.length_squared() > 0.0 {
        transform.translation += dir.normalize() * speed * time.delta_secs();
    }
    transform.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
}

// ---------------------------------------------------------------------------
// editing
// ---------------------------------------------------------------------------

/// Slab test against a world aligned box.
fn ray_box(origin: Vec3, dir: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    let mut near = 0.0f32;
    let mut far = 400.0f32;
    for axis in 0..3 {
        let o = origin[axis];
        let d = dir[axis];
        let c = center[axis];
        let h = half[axis].abs().max(0.01);
        if d.abs() < 1e-5 {
            if o < c - h || o > c + h {
                return None;
            }
        } else {
            let inv = 1.0 / d;
            let mut t1 = (c - h - o) * inv;
            let mut t2 = (c + h - o) * inv;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            near = near.max(t1);
            far = far.min(t2);
            if near > far {
                return None;
            }
        }
    }
    Some(near)
}

#[allow(clippy::too_many_arguments)]
fn edit_world(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cam: Query<(&Transform, &EditorCam)>,
    solids: Query<(&GlobalTransform, &SolidBox)>,
    mut meshes: ResMut<Assets<Mesh>>,
    palette: Res<Palette>,
    assets: Res<GameAssets>,
    mut state: ResMut<Editor>,
) {
    if !state.built {
        return;
    }
    let Ok((cam_tf, editor_cam)) = cam.get_single() else {
        return;
    };

    let origin = cam_tf.translation;
    let dir = forward(editor_cam.yaw, editor_cam.pitch);

    // where does the crosshair hit the hotel?
    let mut best = f32::MAX;
    for (transform, solid) in solids.iter() {
        if let Some(t) = ray_box(origin, dir, transform.translation(), solid.half) {
            if t > 0.02 && t < best {
                best = t;
            }
        }
    }
    let hit = if best < f32::MAX {
        origin + dir * best
    } else {
        origin + dir * 9.0
    };
    state.hit = Some(hit);

    // clicks on the tool lists must not place anything
    let over_ui = windows
        .get_single()
        .ok()
        .map(|window| {
            let x = window.cursor_position().map(|c| c.x).unwrap_or(0.0);
            x < 260.0 || x > window.width() - 270.0
        })
        .unwrap_or(false);

    if buttons.just_pressed(MouseButton::Left) && !over_ui {
        match state.tool {
            Some(index) => {
                let Some(def) = factory::CATALOG.get(index) else {
                    return;
                };
                let mut object = factory::default_object(def.id, hit);
                object.yaw = editor_cam.yaw;
                let root =
                    factory::spawn_object(&mut commands, &mut meshes, &palette, &assets, &object);
                state.placed.push(Placed { object, root });
                state.selected = Some(state.placed.len().saturating_sub(1));
                state.status = format!("{} gesetzt", def.label);
            }
            None => {
                let mut found = None;
                let mut nearest = 4.5f32;
                for (index, placed) in state.placed.iter().enumerate() {
                    if let Some(t) = ray_sphere(origin, dir, placed.object.pos, 1.4) {
                        if t < nearest {
                            nearest = t;
                            found = Some(index);
                        }
                    }
                }
                state.selected = found;
            }
        }
    }

    // nudge / rotate / delete / duplicate the picked object
    let Some(index) = state.selected else {
        return;
    };
    if index >= state.placed.len() {
        state.selected = None;
        return;
    }

    let step = if keys.pressed(KeyCode::ShiftLeft) {
        0.5
    } else {
        0.1
    };
    let mut delta = Vec3::ZERO;
    if keys.just_pressed(KeyCode::ArrowLeft) {
        delta.x -= step;
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        delta.x += step;
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        delta.z -= step;
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        delta.z += step;
    }
    if keys.just_pressed(KeyCode::KeyU) {
        delta.y -= step;
    }
    if keys.just_pressed(KeyCode::KeyO) {
        delta.y += step;
    }
    let mut turn = 0.0f32;
    if keys.just_pressed(KeyCode::Comma) {
        turn += 0.12;
    }
    if keys.just_pressed(KeyCode::Period) {
        turn -= 0.12;
    }

    if delta != Vec3::ZERO || turn != 0.0 {
        // keep the borrow of `state` short: `ResMut` cannot split fields
        let (pos, yaw, kind, root) = {
            let placed = &mut state.placed[index];
            placed.object.pos += delta;
            placed.object.yaw += turn;
            (
                placed.object.pos,
                placed.object.yaw,
                placed.object.kind.clone(),
                placed.root,
            )
        };
        let transform =
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw));
        commands.entity(root).insert(transform);
        state.status = format!("{} bei {:.2} {:.2} {:.2}", kind, pos.x, pos.y, pos.z);
    }

    if keys.just_pressed(KeyCode::KeyC) {
        let mut copy = state.placed[index].object.clone();
        copy.pos += Vec3::new(0.9, 0.0, 0.0);
        let root = factory::spawn_object(&mut commands, &mut meshes, &palette, &assets, &copy);
        state.placed.push(Placed { object: copy, root });
        state.selected = Some(state.placed.len().saturating_sub(1));
        state.status = "dupliziert".to_string();
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        let placed = state.placed.remove(index);
        commands.entity(placed.root).despawn_recursive();
        state.selected = None;
        state.status = "gelöscht".to_string();
    }
}

// ---------------------------------------------------------------------------
// ui
// ---------------------------------------------------------------------------

fn ui_buttons(
    mut kinds: Query<(&Interaction, &KindButton), Changed<Interaction>>,
    mut areas: Query<(&Interaction, &AreaButton), Changed<Interaction>>,
    mut select_tool: Query<&Interaction, (Changed<Interaction>, With<SelectToolButton>)>,
    mut cam: Query<(&mut Transform, &mut EditorCam)>,
    mut state: ResMut<Editor>,
) {
    for (interaction, button) in kinds.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(def) = factory::CATALOG.get(button.0) {
            state.tool = Some(button.0);
            state.status = format!("Werkzeug: {} — Linksklick setzt", def.label);
        }
    }

    for interaction in select_tool.iter_mut() {
        if *interaction == Interaction::Pressed {
            state.tool = None;
            state.status = "Werkzeug: Objekt wählen".to_string();
        }
    }

    for (interaction, button) in areas.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some((name, pos, yaw)) = AREAS.get(button.0) {
            if let Ok((mut transform, mut editor_cam)) = cam.get_single_mut() {
                transform.translation = *pos;
                editor_cam.yaw = *yaw;
                editor_cam.pitch = -0.05;
            }
            state.status = format!("Bereich: {}", name);
        }
    }
}

fn save_load_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    palette: Res<Palette>,
    assets: Res<GameAssets>,
    mut state: ResMut<Editor>,
) {
    if !state.built {
        return;
    }

    if keys.just_pressed(KeyCode::F5) {
        let file = LevelFile {
            objects: state
                .placed
                .iter()
                .map(|placed| placed.object.clone())
                .collect(),
        };
        state.status = match file.save() {
            Ok(()) => format!("gespeichert: {} Objekte → {}", file.objects.len(), LEVEL_PATH),
            Err(error) => format!("Speichern fehlgeschlagen: {}", error),
        };
    }

    if keys.just_pressed(KeyCode::F9) {
        for placed in state.placed.drain(..) {
            commands.entity(placed.root).despawn_recursive();
        }
        let file = LevelFile::load();
        let count = file.objects.len();
        for object in file.objects {
            let root =
                factory::spawn_object(&mut commands, &mut meshes, &palette, &assets, &object);
            state.placed.push(Placed { object, root });
        }
        state.selected = None;
        state.status = format!("neu geladen: {} Objekte", count);
    }
}

fn update_markers(
    mut hit: Query<&mut Transform, With<HitMarker>>,
    mut select: Query<&mut Transform, (With<SelectMarker>, Without<HitMarker>)>,
    state: Res<Editor>,
) {
    if let Ok(mut transform) = hit.get_single_mut() {
        transform.translation = state.hit.unwrap_or(Vec3::new(0.0, -60.0, 0.0));
    }
    let picked = state
        .selected
        .and_then(|index| state.placed.get(index))
        .map(|placed| placed.object.pos)
        .unwrap_or(Vec3::new(0.0, -60.0, 0.0));
    if let Ok(mut transform) = select.get_single_mut() {
        transform.translation = picked;
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha,
    )
}

fn button_tint(
    mut buttons: Query<(&Interaction, &BaseTint, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, base, mut color) in buttons.iter_mut() {
        color.0 = match interaction {
            Interaction::Pressed => mix(base.0, Color::WHITE, 0.35),
            Interaction::Hovered => mix(base.0, Color::WHITE, 0.14),
            Interaction::None => base.0,
        };
    }
}

fn update_status(
    state: Res<Editor>,
    mut text: Query<&mut Text, With<EditorStatusText>>,
) {
    let Ok(mut text) = text.get_single_mut() else {
        return;
    };
    let tool = state
        .tool
        .and_then(|index| factory::CATALOG.get(index))
        .map(|def| def.label)
        .unwrap_or("Objekt wählen");
    let picked = state
        .selected
        .and_then(|index| state.placed.get(index))
        .map(|placed| {
            format!(
                "{} bei {:.2} {:.2} {:.2}",
                placed.object.kind, placed.object.pos.x, placed.object.pos.y, placed.object.pos.z
            )
        })
        .unwrap_or_else(|| "—".to_string());
    let body = format!(
        "Werkzeug: {}   |   {} Objekte   |   Auswahl: {}\n{}\n\n\
         LMB setzen/wählen · RMB umsehen · WASD+QE fliegen (Shift schnell) · \
         Pfeile bewegen · U/O hoch/runter · , . drehen\n\
         C duplizieren · Entf löschen · F5 speichern · F9 neu laden · {}",
        tool,
        state.placed.len(),
        picked,
        state.status,
        LEVEL_PATH
    );
    if text.0 != body {
        text.0 = body;
    }
}

/// The tool list on the left, the area list on the right, the status line.
fn spawn_ui(commands: &mut Commands, assets: &GameAssets) {
    let ui = assets.fonts.ui.clone();
    let ui_light = assets.fonts.ui_light.clone();
    let title = assets.fonts.title.clone();

    // ---- tool list ------------------------------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(250.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.035, 0.04, 0.94)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Diplomat Peak"),
                TextFont {
                    font: title.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.74, 0.48)),
            ));
            panel.spawn((
                Text::new("LEVEL EDITOR"),
                TextFont {
                    font: ui.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.66, 0.58, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            panel
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.16, 0.18, 0.20, 0.95)),
                    BaseTint(Color::srgba(0.16, 0.18, 0.20, 0.95)),
                    SelectToolButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Objekt wählen"),
                        TextFont {
                            font: ui_light.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.85)),
                    ));
                });

            for (index, kind) in factory::CATALOG.iter().enumerate() {
                let tint = match kind.class {
                    KindClass::Solid => Color::srgba(0.20, 0.22, 0.26, 0.92),
                    KindClass::Decor => Color::srgba(0.14, 0.16, 0.15, 0.92),
                    KindClass::Interactive => Color::srgba(0.24, 0.21, 0.12, 0.92),
                    KindClass::Light => Color::srgba(0.24, 0.19, 0.13, 0.92),
                };
                panel
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(tint),
                        BaseTint(tint),
                        KindButton(index),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(kind.label),
                            TextFont {
                                font: ui.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.84, 0.84, 0.80)),
                        ));
                    });
            }
        });

    // ---- area list ------------------------------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(250.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.035, 0.04, 0.94)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("BEREICHE"),
                TextFont {
                    font: ui.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.66, 0.58, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));
            for (index, area) in AREAS.iter().enumerate() {
                panel
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.13, 0.15, 0.92)),
                        BaseTint(Color::srgba(0.12, 0.13, 0.15, 0.92)),
                        AreaButton(index),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(area.0),
                            TextFont {
                                font: ui.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.80, 0.82, 0.88)),
                        ));
                    });
            }
        });

    // ---- status line ----------------------------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(262.0),
                bottom: Val::Px(14.0),
                right: Val::Px(262.0),
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .with_children(|line| {
            line.spawn((
                Text::new(""),
                TextFont {
                    font: ui.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.88, 0.82)),
                EditorStatusText,
            ));
        });
}
