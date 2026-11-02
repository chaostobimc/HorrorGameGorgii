//! The Diplomat Peak — Level Editor.
//!
//! A separate program (`cargo run --release --bin editor`) that shares the
//! whole game library: it renders the real hotel with the real palette, the
//! real props and the real lights. Everything you do is written to
//! `assets/level/extras.lvl`; the game reads that file when it builds the
//! level, so editor work is in the game, behaviour included.
//!
//! Three things can be edited:
//!   * place new objects from the library (furniture, lights, doors, keypads)
//!   * place imported 3D models from `assets/models` (Blender glTF) — they
//!     bring their own collision, generated from their triangles
//!   * change the hand built hotel: pick a piece, replace it with a model,
//!     move the prop or delete it
//!
//! Controls
//!   WASD / QE        fly, hold shift to go faster
//!   right mouse      look around
//!   left mouse       place / pick
//!   arrows / U O     move what is picked, `,` `.` turn it
//!   C / Del          duplicate / delete
//!   F5 / F9          save / reload the level file

use bevy::audio::AudioSource;
use bevy::gltf::{Gltf, GltfMesh};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use std::collections::HashMap;

use diplomat_peak::collision::{ray_sphere, SolidBox};
use diplomat_peak::core::{AppState, CorePlugin, GameAssets, Rng};
use diplomat_peak::custom::{ensure_model, CustomLevelPlugin};
use diplomat_peak::factory::{self, KindClass};
use diplomat_peak::level::{EditKind, EditTarget, LevelFile, LevelObject, PartEdit, LEVEL_PATH};
use diplomat_peak::models;
use diplomat_peak::palette::Palette;
use diplomat_peak::world::props::{HotelGroup, HotelPart};
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
        .add_plugins((CorePlugin, CustomLevelPlugin))
        .init_resource::<Editor>()
        .add_systems(Startup, spawn_editor_camera)
        .add_systems(
            Update,
            (
                build_world_once,
                preload_models,
                fly_camera,
                edit_world,
                pick_hotel,
                ui_tools,
                ui_fields,
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
struct StatusText;

#[derive(Component)]
struct KindButton(usize);

#[derive(Component)]
struct ModelButton(usize);

#[derive(Component)]
struct AreaButton(usize);

#[derive(Component)]
struct SelectToolButton;

#[derive(Component)]
struct HotelModeButton;

#[derive(Component)]
struct HotelSwapButton;

#[derive(Component)]
struct HotelRemoveButton;

#[derive(Component)]
struct FieldLabel(usize);

#[derive(Component)]
struct FieldValue(usize);

#[derive(Component)]
struct FieldRow(usize);

#[derive(Component)]
struct FieldStep {
    row: usize,
    direction: f32,
}

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
    /// Selected catalogue entry, `None` = pick instead of placing.
    tool: Option<usize>,
    /// Selected model file: placing puts that model down.
    model: Option<String>,
    /// Every model found in `assets/models`.
    models: Vec<String>,
    models_loaded: bool,
    /// Clicking picks hotel pieces instead of placing.
    hotel_mode: bool,
    selected: Option<usize>,
    picked_part: Option<u32>,
    picked_group: Option<u32>,
    status: String,
    hit: Option<Vec3>,
    placed: Vec<Placed>,
    edits: Vec<PartEdit>,
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
/// level file: its objects and its changes to the hotel.
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

    // same order as the game, so the piece numbers line up
    world::props::reset_parts();

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
    for object in file.objects.iter() {
        let root = factory::spawn_object(&mut commands, &mut meshes, &palette, &assets, object);
        state.placed.push(Placed {
            object: object.clone(),
            root,
        });
    }
    state.edits = file.edits.clone();

    state.models = models::scan_dir();

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

    spawn_ui(&mut commands, &assets, &state.models);

    state.built = true;
    state.status = format!(
        "{} Objekte, {} Aenderungen, {} Modelle",
        state.placed.len(),
        state.edits.len(),
        state.models.len()
    );
}

/// Loads every model in `assets/models` so the palette can place them.
fn preload_models(
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut cache: Local<Vec<(String, Handle<Gltf>)>>,
    mut state: ResMut<Editor>,
) {
    if !state.built || state.models_loaded {
        return;
    }
    let mut all = true;
    for file in state.models.clone() {
        if !ensure_model(&file, &asset_server, &gltfs, &gltf_meshes, &mut cache) {
            all = false;
        }
    }
    state.models_loaded = all;
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
// picking helpers
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

// ---------------------------------------------------------------------------
// editing: placing, moving, deleting objects
// ---------------------------------------------------------------------------

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

    // clicks on the side panels must not place anything
    let over_ui = windows
        .get_single()
        .ok()
        .map(|window| {
            let x = window.cursor_position().map(|c| c.x).unwrap_or(0.0);
            x < 292.0 || x > window.width() - 312.0
        })
        .unwrap_or(false);

    if buttons.just_pressed(MouseButton::Left) && !over_ui && !state.hotel_mode {
        let mut created = None;
        if let Some(file) = state.model.clone() {
            let mut object = factory::default_object("model", hit);
            object.set("file", file.clone());
            object.yaw = editor_cam.yaw;
            created = Some(object);
        } else if let Some(index) = state.tool {
            if let Some(def) = factory::CATALOG.get(index) {
                let mut object = factory::default_object(def.id, hit);
                object.yaw = editor_cam.yaw;
                created = Some(object);
            }
        }

        match created {
            Some(object) => {
                let root =
                    factory::spawn_object(&mut commands, &mut meshes, &palette, &assets, &object);
                state.status = format!("{} gesetzt", object.kind);
                state.placed.push(Placed { object, root });
                state.selected = Some(state.placed.len().saturating_sub(1));
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
                state.status = match found {
                    Some(_) => "Objekt gewaehlt".to_string(),
                    None => "nichts getroffen".to_string(),
                };
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
        let transform = Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw));
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
        state.status = "geloescht".to_string();
    }
}

// ---------------------------------------------------------------------------
// editing: the hand built hotel
// ---------------------------------------------------------------------------

fn record_edit(state: &mut Editor, edit: PartEdit) {
    // keep one entry per target and kind, so nudging does not pile up
    let same_target = |existing: &PartEdit| {
        existing.target == edit.target
            && std::mem::discriminant(&existing.kind) == std::mem::discriminant(&edit.kind)
    };
    if let EditKind::Move(delta) = &edit.kind {
        if let Some(slot) = state
            .edits
            .iter_mut()
            .find(|existing| same_target(existing))
        {
            if let EditKind::Move(previous) = &mut slot.kind {
                *previous += *delta;
                return;
            }
        }
    }
    state.edits.retain(|existing| !same_target(existing));
    state.edits.push(edit);
}

#[allow(clippy::too_many_arguments)]
fn pick_hotel(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cam: Query<(&Transform, &EditorCam)>,
    mesh_assets: Res<Assets<Mesh>>,
    parts: Query<(&GlobalTransform, &Mesh3d, &HotelPart)>,
    groups: Query<(Entity, &HotelGroup)>,
    transforms: Query<&Transform>,
    mut bounds_cache: Local<HashMap<AssetId<Mesh>, Option<(Vec3, Vec3)>>>,
    mut state: ResMut<Editor>,
) {
    if !state.built || !state.hotel_mode {
        return;
    }
    let Ok((cam_tf, editor_cam)) = cam.get_single() else {
        return;
    };
    let origin = cam_tf.translation;
    let dir = forward(editor_cam.yaw, editor_cam.pitch);

    let over_ui = windows
        .get_single()
        .ok()
        .map(|window| {
            let x = window.cursor_position().map(|c| c.x).unwrap_or(0.0);
            x < 292.0 || x > window.width() - 312.0
        })
        .unwrap_or(false);

    if buttons.just_pressed(MouseButton::Left) && !over_ui {
        let mut best = 12.0f32;
        let mut found = None;
        for (transform, mesh3d, part) in parts.iter() {
            let Some(mesh) = mesh_assets.get(&mesh3d.0) else {
                continue;
            };
            let cached = bounds_cache
                .entry(mesh3d.0.id())
                .or_insert_with(|| models::mesh_bounds(mesh));
            let Some((min, max)) = cached else {
                continue;
            };
            let center = transform.transform_point((*min + *max) * 0.5);
            let half = (*max - *min) * 0.5 * transform.scale().abs();
            let radius = half.length().max(0.15);
            if let Some(t) = ray_sphere(origin, dir, center, radius) {
                if t < best {
                    best = t;
                    found = Some((part.id, part.group));
                }
            }
        }
        state.picked_part = found.map(|(id, _)| id);
        state.picked_group = found.map(|(_, group)| group);
        state.status = match found {
            Some((id, group)) => format!("Hotelteile: Teil {} / Gruppe {}", id, group),
            None => "kein Hotelteil getroffen".to_string(),
        };
    }

    // move the picked prop with the arrow keys
    let (Some(group), true) = (state.picked_group, state.hotel_mode) else {
        return;
    };
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
    if delta == Vec3::ZERO {
        return;
    }
    let Some((root, _)) = groups.iter().find(|(_, marker)| marker.0 == group) else {
        return;
    };
    if let Ok(transform) = transforms.get(root) {
        let moved = Transform::from_translation(transform.translation + delta)
            .with_rotation(transform.rotation)
            .with_scale(transform.scale);
        commands.entity(root).insert(moved);
        record_edit(
            &mut state,
            PartEdit {
                target: EditTarget::Group(group),
                kind: EditKind::Move(delta),
                label: format!("Gruppe {} verschoben", group),
            },
        );
        state.status = format!("Gruppe {} verschoben", group);
    }
}

// ---------------------------------------------------------------------------
// ui
// ---------------------------------------------------------------------------

fn ui_tools(
    mut kinds: Query<(&Interaction, &KindButton), Changed<Interaction>>,
    mut models_q: Query<(&Interaction, &ModelButton), Changed<Interaction>>,
    mut areas: Query<(&Interaction, &AreaButton), Changed<Interaction>>,
    mut select_tool: Query<&Interaction, (Changed<Interaction>, With<SelectToolButton>)>,
    mut hotel_mode: Query<&Interaction, (Changed<Interaction>, With<HotelModeButton>)>,
    mut cam: Query<(&mut Transform, &mut EditorCam)>,
    mut state: ResMut<Editor>,
) {
    for (interaction, button) in kinds.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(def) = factory::CATALOG.get(button.0) {
            state.tool = Some(button.0);
            state.model = None;
            state.hotel_mode = false;
            state.status = format!("Werkzeug: {}", def.label);
        }
    }

    for (interaction, button) in models_q.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(file) = state.models.get(button.0).cloned() {
            let short = file.rsplit('/').next().unwrap_or(&file).to_string();
            state.model = Some(file);
            state.tool = None;
            state.hotel_mode = false;
            state.status = format!("Modell: {} — Linksklick setzt", short);
        }
    }

    for interaction in select_tool.iter_mut() {
        if *interaction == Interaction::Pressed {
            state.tool = None;
            state.model = None;
            state.hotel_mode = false;
            state.status = "Auswahl: Objekt anklicken".to_string();
        }
    }

    for interaction in hotel_mode.iter_mut() {
        if *interaction == Interaction::Pressed {
            state.hotel_mode = !state.hotel_mode;
            state.tool = None;
            state.model = None;
            state.selected = None;
            state.status = if state.hotel_mode {
                "Hotelteile: anklicken, dann ersetzen / entfernen / verschieben".to_string()
            } else {
                "Hotelteile: aus".to_string()
            };
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

fn change_field(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &Palette,
    assets: &GameAssets,
    state: &mut Editor,
    row: usize,
    direction: f32,
) {
    let Some(index) = state.selected else {
        return;
    };
    if index >= state.placed.len() {
        return;
    }
    let message;
    {
        let placed = &mut state.placed[index];
        let kind = placed.object.kind.clone();
        let Some(field) = factory::fields(&kind).get(row) else {
            return;
        };
        let current = placed.object.number(field.key, field.min);
        let next = (current + field.step * direction).clamp(field.min, field.max);
        placed.object.set(field.key, format!("{:.3}", next));
        message = format!("{} = {:.2}", field.label, next);

        commands.entity(placed.root).despawn_recursive();
        placed.root = factory::spawn_object(commands, meshes, palette, assets, &placed.object);
    }
    state.status = message;
}

#[allow(clippy::too_many_arguments)]
fn ui_fields(
    mut steps: Query<(&Interaction, &FieldStep), Changed<Interaction>>,
    mut swap: Query<&Interaction, (Changed<Interaction>, With<HotelSwapButton>)>,
    mut remove: Query<&Interaction, (Changed<Interaction>, With<HotelRemoveButton>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    palette: Res<Palette>,
    assets: Res<GameAssets>,
    groups: Query<(Entity, &HotelGroup)>,
    parts: Query<(Entity, &HotelPart)>,
    mut state: ResMut<Editor>,
) {
    for (interaction, step) in steps.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        change_field(
            &mut commands,
            &mut meshes,
            &palette,
            &assets,
            &mut state,
            step.row,
            step.direction,
        );
    }

    for interaction in swap.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let (Some(part), Some(file)) = (state.picked_part, state.model.clone()) else {
            state.status = "erst ein Hotelteil und ein Modell waehlen".to_string();
            continue;
        };
        if !models::is_loaded(&file) {
            state.status = format!("{} ist noch nicht geladen", file);
            continue;
        }
        let Some((entity, _)) = parts.iter().find(|(_, marker)| marker.id == part) else {
            continue;
        };
        commands.entity(entity).remove::<Mesh3d>();
        if let Some(scene) = models::scene(&file) {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((SceneRoot(scene), Transform::from_scale(Vec3::ONE)));
            });
        }
        record_edit(
            &mut state,
            PartEdit {
                target: EditTarget::Part(part),
                kind: EditKind::Swap {
                    file: file.clone(),
                    scale: 1.0,
                },
                label: format!("Teil {} ersetzt", part),
            },
        );
        state.status = format!("Teil {} ersetzt durch {}", part, file);
    }
    for interaction in remove.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(group) = state.picked_group else {
            state.status = "erst ein Hotelteil anklicken".to_string();
            continue;
        };
        let Some((root, _)) = groups.iter().find(|(_, marker)| marker.0 == group) else {
            continue;
        };
        commands.entity(root).despawn_recursive();
        record_edit(
            &mut state,
            PartEdit {
                target: EditTarget::Group(group),
                kind: EditKind::Remove,
                label: format!("Gruppe {} entfernt", group),
            },
        );
        state.picked_part = None;
        state.picked_group = None;
        state.status = format!("Gruppe {} entfernt", group);
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
            edits: state.edits.clone(),
        };
        state.status = match file.save() {
            Ok(()) => format!(
                "gespeichert: {} Objekte, {} Aenderungen -> {}",
                file.objects.len(),
                file.edits.len(),
                LEVEL_PATH
            ),
            Err(error) => format!("Speichern fehlgeschlagen: {}", error),
        };
    }

    if keys.just_pressed(KeyCode::F9) {
        for placed in state.placed.drain(..) {
            commands.entity(placed.root).despawn_recursive();
        }
        let file = LevelFile::load();
        for object in file.objects.iter() {
            let root = factory::spawn_object(&mut commands, &mut meshes, &palette, &assets, object);
            state.placed.push(Placed {
                object: object.clone(),
                root,
            });
        }
        state.selected = None;
        state.status = format!(
            "neu geladen: {} Objekte (Hotelaenderungen bleiben bis zum Neustart)",
            state.placed.len()
        );
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
    mut text: Query<&mut Text, With<StatusText>>,
    mut labels: Query<(&mut Text, &FieldLabel), Without<StatusText>>,
    mut values: Query<(&mut Text, &FieldValue), (Without<StatusText>, Without<FieldLabel>)>,
    mut rows: Query<(&mut Node, &FieldRow), (Without<StatusText>, Without<FieldLabel>, Without<FieldValue>)>,
) {
    // property panel ------------------------------------------------------
    let selected = state.selected.and_then(|index| state.placed.get(index));
    let fields = selected
        .map(|placed| factory::fields(&placed.object.kind))
        .unwrap_or(&[]);
    for (mut node, row) in rows.iter_mut() {
        node.display = if row.0 < fields.len() {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (mut text, label) in labels.iter_mut() {
        let name = fields
            .get(label.0)
            .map(|field| field.label)
            .unwrap_or("—");
        if text.0 != name {
            text.0 = name.to_string();
        }
    }
    for (mut text, value) in values.iter_mut() {
        let shown = match (selected, fields.get(value.0)) {
            (Some(placed), Some(field)) => format!("{:.2}", placed.object.number(field.key, 0.0)),
            _ => "—".to_string(),
        };
        if text.0 != shown {
            text.0 = shown;
        }
    }

    // status line ---------------------------------------------------------
    let Ok(mut text) = text.get_single_mut() else {
        return;
    };
    let tool = if let Some(file) = state.model.as_ref() {
        format!("Modell {}", file.rsplit('/').next().unwrap_or(file))
    } else if let Some(index) = state.tool {
        factory::CATALOG
            .get(index)
            .map(|def| def.label.to_string())
            .unwrap_or_else(|| "—".to_string())
    } else if state.hotel_mode {
        "Hotelteile bearbeiten".to_string()
    } else {
        "Auswahl".to_string()
    };
    let picked = match (state.picked_part, state.picked_group) {
        (Some(part), Some(group)) => format!("Teil {} / Gruppe {}", part, group),
        _ => "—".to_string(),
    };
    let body = format!(
        "Werkzeug: {tool}   |   {} Objekte   |   {} Aenderungen   |   Hotelteil: {picked}\n\
         {}\n\n\
         LMB setzen/waehlen · RMB umsehen · WASD+QE fliegen (Shift schnell) · \
         Pfeile bewegen · U/O hoch/runter · , . drehen · C duplizieren · Entf loeschen\n\
         F5 speichern · F9 laden · Datei: {}",
        state.placed.len(),
        state.edits.len(),
        state.status,
        LEVEL_PATH
    );
    if text.0 != body {
        text.0 = body;
    }
}

// ---------------------------------------------------------------------------
// ui layout
// ---------------------------------------------------------------------------

fn button<M: Component>(
    parent: &mut ChildBuilder,
    fonts: &diplomat_peak::core::Fonts,
    label: &str,
    tint: Color,
    size: f32,
    marker: M,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(7.0), Val::Px(2.0)),
                width: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(tint),
            BaseTint(tint),
            marker,
        ))
        .with_children(|text| {
            text.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font: fonts.ui.clone(),
                    font_size: size,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.86, 0.82)),
            ));
        });
}

fn heading(parent: &mut ChildBuilder, fonts: &diplomat_peak::core::Fonts, label: &str) {
    parent.spawn((
        Text::new(label),
        TextFont {
            font: fonts.ui.clone(),
            font_size: 11.0,
            ..default()
        },
        TextColor(Color::srgba(0.72, 0.68, 0.58, 0.8)),
        Node {
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(8.0), Val::Px(3.0)),
            ..default()
        },
    ));
}

fn spawn_ui(commands: &mut Commands, assets: &GameAssets, models_found: &[String]) {
    let fonts_ref = &assets.fonts;

    // ---- left: tools and models ----------------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(282.0),
                padding: UiRect::all(Val::Px(9.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.035, 0.04, 0.95)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Diplomat Peak"),
                TextFont {
                    font: fonts_ref.title.clone(),
                    font_size: 21.0,
                    ..default()
                },
                TextColor(Color::srgb(0.87, 0.75, 0.49)),
            ));
            panel.spawn((
                Text::new("LEVEL EDITOR"),
                TextFont {
                    font: fonts_ref.ui.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.66, 0.58, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            heading(panel, fonts_ref, "AUSWAHL");
            button(
                panel,
                fonts_ref,
                "Objekt waehlen / verschieben",
                Color::srgba(0.16, 0.18, 0.21, 0.95),
                12.0,
                SelectToolButton,
            );

            heading(panel, fonts_ref, "HOTEL BEARBEITEN");
            button(
                panel,
                fonts_ref,
                "Hotelteile (an/aus)",
                Color::srgba(0.24, 0.17, 0.12, 0.95),
                12.0,
                HotelModeButton,
            );

            heading(panel, fonts_ref, "MODELLE (assets/models)");
            if models_found.is_empty() {
                panel.spawn((
                    Text::new("keine Datei gefunden — .glb nach assets/models legen"),
                    TextFont {
                        font: fonts_ref.ui.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.85, 0.55, 0.4, 0.9)),
                ));
            }
            for (index, file) in models_found.iter().enumerate() {
                let short = file.rsplit('/').next().unwrap_or(file);
                button(
                    panel,
                    fonts_ref,
                    short,
                    Color::srgba(0.13, 0.18, 0.16, 0.95),
                    12.0,
                    ModelButton(index),
                );
            }

            heading(panel, fonts_ref, "BIBLIOTHEK");
            for (index, kind) in factory::CATALOG.iter().enumerate() {
                let tint = match kind.class {
                    KindClass::Solid => Color::srgba(0.20, 0.22, 0.26, 0.92),
                    KindClass::Decor => Color::srgba(0.14, 0.16, 0.15, 0.92),
                    KindClass::Interactive => Color::srgba(0.24, 0.21, 0.12, 0.92),
                    KindClass::Light => Color::srgba(0.24, 0.19, 0.13, 0.92),
                };
                button(panel, fonts_ref, kind.label, tint, 12.0, KindButton(index));
            }
        });

    // ---- right: areas, properties, hotel actions ------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(302.0),
                padding: UiRect::all(Val::Px(9.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.035, 0.04, 0.95)),
        ))
        .with_children(|panel| {
            heading(panel, fonts_ref, "BEREICHE");
            for (index, area) in AREAS.iter().enumerate() {
                button(
                    panel,
                    fonts_ref,
                    area.0,
                    Color::srgba(0.12, 0.13, 0.16, 0.92),
                    12.0,
                    AreaButton(index),
                );
            }

            heading(panel, fonts_ref, "EIGENSCHAFTEN");
            for row in 0..4usize {
                panel
                    .spawn((
                        Node {
                            display: Display::None,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.0),
                            align_items: AlignItems::Center,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        FieldRow(row),
                    ))
                    .with_children(|line| {
                        line.spawn((
                            Text::new("—"),
                            TextFont {
                                font: fonts_ref.ui.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.8, 0.76)),
                            Node {
                                width: Val::Px(96.0),
                                ..default()
                            },
                            FieldLabel(row),
                        ));
                        button(
                            line,
                            fonts_ref,
                            "-",
                            Color::srgba(0.18, 0.2, 0.22, 0.95),
                            13.0,
                            FieldStep {
                                row,
                                direction: -1.0,
                            },
                        );
                        line.spawn((
                            Text::new("0"),
                            TextFont {
                                font: fonts_ref.ui.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.95, 0.92, 0.8)),
                            Node {
                                width: Val::Px(64.0),
                                ..default()
                            },
                            FieldValue(row),
                        ));
                        button(
                            line,
                            fonts_ref,
                            "+",
                            Color::srgba(0.18, 0.2, 0.22, 0.95),
                            13.0,
                            FieldStep {
                                row,
                                direction: 1.0,
                            },
                        );
                    });
            }

            heading(panel, fonts_ref, "HOTELTEIL");
            button(
                panel,
                fonts_ref,
                "Mit gewaehltem Modell ersetzen",
                Color::srgba(0.15, 0.22, 0.19, 0.95),
                12.0,
                HotelSwapButton,
            );
            button(
                panel,
                fonts_ref,
                "Ganzes Teil entfernen",
                Color::srgba(0.26, 0.14, 0.14, 0.95),
                12.0,
                HotelRemoveButton,
            );
        });

    // ---- crosshair ------------------------------------------------------
    for (width, height) in [(11.0, 2.0), (2.0, 11.0)] {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(width),
                height: Val::Px(height),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
            PickingBehavior::IGNORE,
        ));
    }

    // ---- status line ----------------------------------------------------
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(292.0),
                bottom: Val::Px(12.0),
                right: Val::Px(312.0),
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .with_children(|line| {
            line.spawn((
                Text::new(""),
                TextFont {
                    font: fonts_ref.ui.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.88, 0.82)),
                StatusText,
            ));
        });
}
