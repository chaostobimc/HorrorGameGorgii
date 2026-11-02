//! The custom layer: everything the level editor did to the hotel.
//!
//! Two halves, both written by the editor into `assets/level/extras.lvl`:
//!
//! * objects that do not exist in the hand built hotel (furniture, lights,
//!   doors, imported models),
//! * changes to the hotel itself (a piece replaced by a custom model, a prop
//!   removed or moved).
//!
//! The game rebuilds both when it builds the level, so the editor output is
//! the game — nothing is exported twice.

use bevy::gltf::{Gltf, GltfMesh};
use bevy::prelude::*;

use crate::audio::PlaySfx;
use crate::core::{AppState, GameAssets};
use crate::factory;
use crate::interact::{Action, Interacted};
use crate::level::{EditKind, EditTarget, LevelFile};
use crate::models;
use crate::palette::Palette;
use crate::world::props::{HotelGroup, HotelPart};
use crate::world::Swing;

/// A door placed with the editor: it swings open and closes again.
#[derive(Component)]
pub struct CustomDoor {
    /// The swinging leaf (`props::door`).
    pub leaf: Entity,
    /// The invisible collider that keeps the closed door solid.
    pub blocker: Entity,
    /// Local position of the blocker while the door is closed.
    pub blocker_at: Vec3,
    pub open: bool,
    pub duration: f32,
}

/// True once every model of the level (and of the edits) can be spawned.
///
/// The level must not be built before that, otherwise imported models would
/// come in without their collision.
#[derive(Resource, Default)]
pub struct ModelGate {
    pub ready: bool,
    pub waited: f32,
}

pub fn models_ready(gate: Res<ModelGate>) -> bool {
    gate.ready
}

#[derive(Resource, Default)]
struct PartEditState {
    done: bool,
    waited: f32,
}

/// Builds every object of the level file into the running hotel.
pub fn spawn_level_objects(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    assets: &GameAssets,
    level: &LevelFile,
) {
    for object in level.objects.iter() {
        factory::spawn_object(commands, meshes, pal, assets, object);
    }
}

/// Starts loading `file` and registers it as soon as Bevy has it.
///
/// Keeps its own handle cache, so call it once per file per program.
pub fn ensure_model(
    file: &str,
    asset_server: &AssetServer,
    gltfs: &Assets<Gltf>,
    gltf_meshes: &Assets<GltfMesh>,
    cache: &mut Vec<(String, Handle<Gltf>)>,
) -> bool {
    if models::is_loaded(file) {
        return true;
    }
    let index = match cache.iter().position(|(name, _)| name == file) {
        Some(index) => index,
        None => {
            cache.push((file.to_string(), asset_server.load::<Gltf>(file.to_string())));
            cache.len() - 1
        }
    };
    let handle = cache[index].1.clone();
    let Some(gltf) = gltfs.get(&handle) else {
        return false;
    };
    let scene = gltf
        .default_scene
        .clone()
        .or_else(|| gltf.scenes.first().cloned());
    let Some(scene) = scene else {
        println!("[models] {} has no scene, skipped", file);
        return true;
    };
    // the primitives carry the meshes; without them there is nothing to build
    // collision from, so wait another frame
    if !gltf
        .meshes
        .iter()
        .all(|mesh| gltf_meshes.get(mesh).is_some())
    {
        return false;
    }
    let meshes: Vec<Handle<Mesh>> = gltf
        .meshes
        .iter()
        .filter_map(|mesh| gltf_meshes.get(mesh))
        .flat_map(|mesh| {
            mesh.primitives
                .iter()
                .map(|primitive| primitive.mesh.clone())
        })
        .collect();
    models::register(file, scene, meshes);
    println!(
        "[models] {} geladen ({} Meshes)",
        file,
        models::mesh_count(file)
    );
    true
}

fn preload_models(
    level: Res<LevelFile>,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut gate: ResMut<ModelGate>,
    mut cache: Local<Vec<(String, Handle<Gltf>)>>,
    time: Res<Time>,
) {
    if gate.ready {
        return;
    }
    gate.waited += time.delta_secs();

    let needed = level.model_files();
    if needed.is_empty() {
        gate.ready = true;
        return;
    }

    let mut all = true;
    for file in needed.iter() {
        if !ensure_model(file, &asset_server, &gltfs, &gltf_meshes, &mut cache) {
            all = false;
        }
    }

    if all {
        gate.ready = true;
    } else if gate.waited > 20.0 {
        // a broken file must not keep the hotel from being built
        println!("[models] Zeitlimit erreicht, baue trotzdem");
        gate.ready = true;
    }
}

/// Applies the changes the editor made to the hand built hotel.
fn apply_part_edits(
    mut commands: Commands,
    level: Res<LevelFile>,
    parts: Query<(Entity, &HotelPart)>,
    groups: Query<(Entity, &HotelGroup)>,
    transforms: Query<&Transform>,
    mut state: ResMut<PartEditState>,
    time: Res<Time>,
) {
    if state.done {
        return;
    }
    // wait until the hotel exists and its spawn commands have run
    if parts.is_empty() {
        state.waited = 0.0;
        return;
    }
    state.waited += time.delta_secs();
    if state.waited < 0.15 {
        return;
    }
    state.done = true;
    if level.edits.is_empty() {
        return;
    }

    let part_entity = |id: u32| -> Option<Entity> {
        parts
            .iter()
            .find(|(_, part)| part.id == id)
            .map(|(entity, _)| entity)
    };
    let group_root = |id: u32| -> Option<Entity> {
        groups
            .iter()
            .find(|(_, group)| group.0 == id)
            .map(|(entity, _)| entity)
    };
    let moved = |entity: Entity, delta: Vec3| -> Option<Transform> {
        let transform = transforms.get(entity).ok()?;
        Some(
            Transform::from_translation(transform.translation + delta)
                .with_rotation(transform.rotation)
                .with_scale(transform.scale),
        )
    };

    for edit in level.edits.iter() {
        match (&edit.target, &edit.kind) {
            (EditTarget::Part(id), EditKind::Remove) => {
                if let Some(entity) = part_entity(*id) {
                    commands.entity(entity).despawn_recursive();
                }
            }
            (EditTarget::Part(id), EditKind::Hide) => {
                if let Some(entity) = part_entity(*id) {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
            (EditTarget::Part(id), EditKind::Move(delta)) => {
                if let Some(entity) = part_entity(*id) {
                    if let Some(transform) = moved(entity, *delta) {
                        commands.entity(entity).insert(transform);
                    }
                }
            }
            (EditTarget::Part(id), EditKind::Swap { file, scale }) => {
                if let Some(entity) = part_entity(*id) {
                    match models::scene(file) {
                        Some(scene) => {
                            // keep the entity: it still carries the swing, the
                            // prompt and the collider, we only swap the look
                            commands.entity(entity).remove::<Mesh3d>();
                            commands.entity(entity).with_children(|parent| {
                                parent.spawn((
                                    SceneRoot(scene),
                                    Transform::from_scale(Vec3::splat(*scale)),
                                ));
                            });
                            println!("[models] Teil {} ersetzt durch {}", id, file);
                        }
                        None => println!("[models] {} fehlt, Teil {} bleibt", file, id),
                    }
                }
            }
            (EditTarget::Group(id), EditKind::Remove) => {
                if let Some(entity) = group_root(*id) {
                    commands.entity(entity).despawn_recursive();
                }
            }
            (EditTarget::Group(id), EditKind::Hide) => {
                if let Some(entity) = group_root(*id) {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
            (EditTarget::Group(id), EditKind::Move(delta)) => {
                if let Some(entity) = group_root(*id) {
                    if let Some(transform) = moved(entity, *delta) {
                        commands.entity(entity).insert(transform);
                    }
                }
            }
            (EditTarget::Group(id), EditKind::Swap { file, scale }) => {
                let Some(entity) = group_root(*id) else {
                    continue;
                };
                let Some(scene) = models::scene(file) else {
                    continue;
                };
                // put the model where the prop stands: average of its pieces
                let mut sum = Vec3::ZERO;
                let mut count = 0.0;
                for (part_entity, part) in parts.iter() {
                    if part.group != *id {
                        continue;
                    }
                    if let Ok(transform) = transforms.get(part_entity) {
                        sum += transform.translation;
                        count += 1.0;
                    }
                    commands.entity(part_entity).insert(Visibility::Hidden);
                }
                let center = if count > 0.0 {
                    sum / count
                } else {
                    Vec3::ZERO
                };
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        SceneRoot(scene),
                        Transform::from_translation(center).with_scale(Vec3::splat(*scale)),
                    ));
                });
            }
        }
    }
}

fn custom_doors(
    mut events: EventReader<Interacted>,
    mut doors: Query<&mut CustomDoor>,
    mut swings: Query<&mut Swing>,
    mut transforms: Query<&mut Transform>,
    assets: Res<GameAssets>,
    mut sfx: EventWriter<PlaySfx>,
) {
    for event in events.read() {
        if !matches!(event.action, Action::Custom(_)) {
            continue;
        }
        let Ok(mut door) = doors.get_mut(event.entity) else {
            continue;
        };
        let Ok(mut swing) = swings.get_mut(door.leaf) else {
            continue;
        };

        let opened = Quat::from_rotation_y(-1.55);
        door.open = !door.open;
        if door.open {
            swing.from = Quat::IDENTITY;
            swing.to = opened;
        } else {
            swing.from = opened;
            swing.to = Quat::IDENTITY;
        }
        swing.t = 0.0;
        swing.duration = door.duration.max(0.1);
        swing.active = true;

        // the collider steps aside while the door is open
        if let Ok(mut transform) = transforms.get_mut(door.blocker) {
            transform.translation = if door.open {
                door.blocker_at + Vec3::Y * 6.0
            } else {
                door.blocker_at
            };
        }

        sfx.send(PlaySfx::flat(assets.snd.door_creak.clone(), 0.45));
    }
}

pub struct CustomLevelPlugin;

impl Plugin for CustomLevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelFile::load())
            .init_resource::<ModelGate>()
            .init_resource::<PartEditState>()
            .add_systems(Update, preload_models)
            .add_systems(Update, apply_part_edits)
            .add_systems(Update, custom_doors.run_if(in_state(AppState::Playing)));
    }
}
