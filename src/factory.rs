//! The object library shared by the level editor and the game.
//!
//! Every entry of [`CATALOG`] can be placed anywhere in the hotel. The editor
//! shows the list, the game rebuilds the placed objects from the level file —
//! both call [`spawn_object`], so what you place is exactly what you get,
//! including behaviour: a keypad opens the code panel wherever it stands, a
//! door swings, a lamp lights its corner.

use bevy::prelude::*;

use crate::collision::SolidBox;
use crate::core::GameAssets;
use crate::custom::CustomDoor;
use crate::interact::{Action, Interactable};
use crate::level::LevelObject;
use crate::models;
use crate::palette::Palette;
use crate::world::{props, Flicker, Swing};

/// How an object behaves, used for sorting the editor list.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KindClass {
    Decor,
    Solid,
    Interactive,
    Light,
}

pub struct KindDef {
    pub id: &'static str,
    pub label: &'static str,
    pub class: KindClass,
    /// Rough half size in metres: editor picking and the selection box.
    pub half: Vec3,
}

/// Everything the editor can place.
pub const CATALOG: &[KindDef] = &[
    KindDef { id: "block", label: "Block (massiv)", class: KindClass::Solid, half: Vec3::new(0.5, 0.5, 0.5) },
    KindDef { id: "plank", label: "Platte (Boden)", class: KindClass::Solid, half: Vec3::new(1.5, 0.2, 1.5) },
    KindDef { id: "crate", label: "Kiste", class: KindClass::Solid, half: Vec3::new(0.4, 0.4, 0.4) },
    KindDef { id: "barrel", label: "Fass", class: KindClass::Solid, half: Vec3::new(0.35, 0.45, 0.35) },
    KindDef { id: "chair", label: "Stuhl", class: KindClass::Decor, half: Vec3::new(0.3, 0.5, 0.3) },
    KindDef { id: "fallen_chair", label: "Stuhl (umgefallen)", class: KindClass::Decor, half: Vec3::new(0.5, 0.4, 0.5) },
    KindDef { id: "armchair", label: "Sessel", class: KindClass::Decor, half: Vec3::new(0.5, 0.5, 0.5) },
    KindDef { id: "sofa", label: "Sofa", class: KindClass::Decor, half: Vec3::new(1.1, 0.5, 0.5) },
    KindDef { id: "coffee_table", label: "Couchtisch", class: KindClass::Decor, half: Vec3::new(0.7, 0.3, 0.4) },
    KindDef { id: "round_table", label: "Runder Tisch", class: KindClass::Decor, half: Vec3::new(0.7, 0.5, 0.7) },
    KindDef { id: "rug", label: "Teppich", class: KindClass::Decor, half: Vec3::new(1.5, 0.1, 1.0) },
    KindDef { id: "plant", label: "Pflanze", class: KindClass::Decor, half: Vec3::new(0.5, 0.8, 0.5) },
    KindDef { id: "curtain", label: "Vorhang", class: KindClass::Decor, half: Vec3::new(1.0, 1.2, 0.2) },
    KindDef { id: "piano", label: "Klavier", class: KindClass::Decor, half: Vec3::new(1.4, 0.6, 0.9) },
    KindDef { id: "sconce", label: "Wandleuchte", class: KindClass::Light, half: Vec3::new(0.2, 0.3, 0.2) },
    KindDef { id: "light", label: "Licht (frei)", class: KindClass::Light, half: Vec3::new(0.15, 0.15, 0.15) },
    KindDef { id: "note", label: "Notiz (lesbar)", class: KindClass::Interactive, half: Vec3::new(0.3, 0.6, 0.3) },
    KindDef { id: "keypad", label: "Eingabepad", class: KindClass::Interactive, half: Vec3::new(0.2, 0.3, 0.1) },
    KindDef { id: "door", label: "Tuer", class: KindClass::Interactive, half: Vec3::new(1.5, 1.2, 0.3) },
    KindDef { id: "model", label: "3D-Modell (Blender)", class: KindClass::Solid, half: Vec3::new(0.7, 0.7, 0.7) },
];

/// One adjustable number of a placed object, shown in the editor panel.
pub struct FieldDef {
    pub key: &'static str,
    pub label: &'static str,
    pub step: f32,
    pub min: f32,
    pub max: f32,
}

const F_W: FieldDef = FieldDef { key: "w", label: "Breite", step: 0.25, min: 0.1, max: 24.0 };
const F_H: FieldDef = FieldDef { key: "h", label: "Hoehe", step: 0.25, min: 0.05, max: 14.0 };
const F_D: FieldDef = FieldDef { key: "d", label: "Tiefe", step: 0.25, min: 0.1, max: 24.0 };
const F_S: FieldDef = FieldDef { key: "s", label: "Groesse", step: 0.1, min: 0.2, max: 6.0 };
const F_I: FieldDef = FieldDef { key: "i", label: "Helligkeit", step: 200.0, min: 0.0, max: 20000.0 };
const F_RANGE: FieldDef = FieldDef { key: "range", label: "Reichweite", step: 1.0, min: 1.0, max: 40.0 };
const F_VOXEL: FieldDef = FieldDef { key: "voxel", label: "Kollision (m)", step: 0.05, min: 0.1, max: 1.0 };
const F_SCALE: FieldDef = FieldDef { key: "scale", label: "Massstab", step: 0.1, min: 0.1, max: 12.0 };
const F_FLICKER: FieldDef = FieldDef { key: "flicker", label: "Flackern", step: 1.0, min: 0.0, max: 1.0 };
const F_CLOTH: FieldDef = FieldDef { key: "cloth", label: "Tischtuch", step: 1.0, min: 0.0, max: 1.0 };

static FIELDS_WH: [FieldDef; 2] = [F_W, F_H];
static FIELDS_WD: [FieldDef; 2] = [F_W, F_D];
static FIELDS_WH_D: [FieldDef; 3] = [F_W, F_H, F_D];
static FIELDS_S: [FieldDef; 1] = [F_S];
static FIELDS_I_FLICKER: [FieldDef; 2] = [F_I, F_FLICKER];
static FIELDS_LIGHT: [FieldDef; 3] = [F_I, F_RANGE, F_FLICKER];
static FIELDS_MODEL: [FieldDef; 2] = [F_SCALE, F_VOXEL];
static FIELDS_CLOTH: [FieldDef; 1] = [F_CLOTH];
static FIELDS_NONE: [FieldDef; 0] = [];

/// The numbers the editor lets you change for `kind`.
pub fn fields(kind: &str) -> &'static [FieldDef] {
    match kind {
        "block" | "plank" => &FIELDS_WH_D,
        "crate" | "barrel" | "plant" => &FIELDS_S,
        "rug" => &FIELDS_WD,
        "curtain" | "door" => &FIELDS_WH,
        "sconce" => &FIELDS_I_FLICKER,
        "light" => &FIELDS_LIGHT,
        "model" => &FIELDS_MODEL,
        "round_table" => &FIELDS_CLOTH,
        _ => &FIELDS_NONE,
    }
}

pub fn find(id: &str) -> Option<&'static KindDef> {
    CATALOG.iter().find(|kind| kind.id == id)
}

/// A fresh object of `kind` with the settings the editor starts from.
pub fn default_object(id: &str, pos: Vec3) -> LevelObject {
    let mut object = LevelObject::new(id, pos);
    match id {
        "block" => {
            object.set("w", "1.0");
            object.set("h", "1.0");
            object.set("d", "1.0");
        }
        "plank" => {
            object.set("w", "3.0");
            object.set("h", "0.2");
            object.set("d", "3.0");
        }
        "crate" => {
            object.set("s", "0.8");
        }
        "barrel" => {
            object.set("s", "0.9");
        }
        "rug" => {
            object.set("w", "3.0");
            object.set("d", "2.0");
        }
        "plant" => {
            object.set("s", "1.4");
            object.set("v", "0");
        }
        "curtain" => {
            object.set("w", "1.8");
            object.set("h", "2.4");
        }
        "round_table" => {
            object.set("cloth", "1");
        }
        "sconce" => {
            object.set("i", "2600");
            object.set("flicker", "1");
        }
        "light" => {
            object.set("i", "2200");
            object.set("range", "9");
            object.set("flicker", "0");
        }
        "door" => {
            object.set("w", "1.0");
            object.set("h", "2.1");
        }
        "model" => {
            object.set("file", "");
            object.set("scale", "1.0");
            object.set("voxel", "0.25");
            object.set("solid", "1");
        }
        _ => {}
    }
    object
}

/// Builds one object into the world and returns its root entity.
///
/// The root carries the transform; every piece the prop spawns becomes its
/// child, so the whole object can be moved, rotated and deleted as one thing.
pub fn spawn_object(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    _assets: &GameAssets,
    object: &LevelObject,
) -> Entity {
    let root = commands
        .spawn((
            Transform::from_translation(object.pos)
                .with_rotation(Quat::from_rotation_y(object.yaw)),
            Visibility::default(),
            Name::new(format!("level:{}", object.kind)),
        ))
        .id();

    let mut door_leaf: Option<Entity> = None;

    props::with_parent(Some(root), || {
        match object.kind.as_str() {
            "chair" => props::chair(commands, meshes, pal, Vec3::ZERO, 0.0),
            "fallen_chair" => props::fallen_chair(commands, meshes, pal, Vec3::ZERO, 0.0),
            "armchair" => props::armchair(commands, meshes, pal, Vec3::ZERO, 0.0),
            "sofa" => props::sofa(commands, meshes, pal, Vec3::ZERO, 0.0),
            "coffee_table" => props::coffee_table(commands, meshes, pal, Vec3::ZERO, 0.0),
            "round_table" => props::round_table(
                commands,
                meshes,
                pal,
                Vec3::ZERO,
                object.flag("cloth", true),
            ),
            "rug" => props::rug(
                commands,
                meshes,
                pal,
                Vec3::ZERO,
                Vec2::new(object.number("w", 3.0), object.number("d", 2.0)),
                0.0,
            ),
            "plant" => props::potted_plant(
                commands,
                meshes,
                pal,
                Vec3::ZERO,
                object.number("s", 1.4),
                object.integer("v", 0),
            ),
            "curtain" => props::curtain(
                commands,
                meshes,
                pal,
                Vec3::ZERO,
                object.number("w", 1.8),
                object.number("h", 2.4),
                0.0,
            ),
            "piano" => props::grand_piano(commands, meshes, pal, Vec3::ZERO, 0.0),
            "door" => {
                let leaf = props::door(
                    commands,
                    meshes,
                    pal,
                    // `door` builds from the left jamb, so shift by half a
                    // door to keep the object centred on its origin
                    Vec3::new(object.number("w", 1.0) * -0.5, 0.0, 0.0),
                    object.number("w", 1.0),
                    object.number("h", 2.1),
                    0.0,
                    None,
                    None,
                );
                door_leaf = Some(leaf);
            }
            _ => {}
        }
    });

    match object.kind.as_str() {
        "block" | "plank" => {
            let size = Vec3::new(
                object.number("w", 1.0).max(0.05),
                object.number("h", 1.0).max(0.05),
                object.number("d", 1.0).max(0.05),
            );
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
                    MeshMaterial3d(pal.concrete.clone()),
                    Transform::from_xyz(0.0, size.y * 0.5, 0.0),
                    SolidBox::new(size * 0.5),
                ));
            });
        }
        "crate" => {
            let s = object.number("s", 0.8);
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(s, s, s))),
                    MeshMaterial3d(pal.wood_h.clone()),
                    Transform::from_xyz(0.0, s * 0.5, 0.0),
                    SolidBox::new(Vec3::splat(s * 0.5)),
                ));
            });
        }
        "barrel" => {
            let s = object.number("s", 0.9);
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Cylinder::new(s * 0.38, s))),
                    MeshMaterial3d(pal.brass_dark.clone()),
                    Transform::from_xyz(0.0, s * 0.5, 0.0),
                    SolidBox::new(Vec3::new(s * 0.38, s * 0.5, s * 0.38)),
                ));
            });
        }
        "sconce" => {
            let intensity = object.number("i", 2600.0);
            let flicker = object.flag("flicker", true);
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.09, 0.24, 0.03))),
                    MeshMaterial3d(pal.brass_dark.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.04),
                ));
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.18, 0.16, 0.18))),
                    MeshMaterial3d(pal.bulb.clone()),
                    Transform::from_xyz(0.0, 0.14, 0.16),
                ));
                let mut light = parent.spawn((
                    PointLight {
                        color: Color::srgb(1.0, 0.80, 0.55),
                        intensity,
                        range: 9.0,
                        shadows_enabled: false,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.16, 0.20),
                ));
                if flicker {
                    light.insert(Flicker {
                        base: intensity,
                        amp: 0.18,
                        speed: 1.4,
                        phase: object.pos.x * 0.7 + object.pos.z * 1.3,
                        dying: false,
                    });
                }
            });
        }
        "light" => {
            let intensity = object.number("i", 2200.0);
            let range = object.number("range", 9.0);
            let flicker = object.flag("flicker", false);
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.10, 0.10, 0.10))),
                    MeshMaterial3d(pal.bulb.clone()),
                    Transform::IDENTITY,
                ));
                let mut light = parent.spawn((
                    PointLight {
                        color: Color::srgb(1.0, 0.84, 0.62),
                        intensity,
                        range,
                        shadows_enabled: false,
                        ..default()
                    },
                    Transform::IDENTITY,
                ));
                if flicker {
                    light.insert(Flicker {
                        base: intensity,
                        amp: 0.2,
                        speed: 1.1,
                        phase: object.pos.x,
                        dying: false,
                    });
                }
            });
        }
        "note" => {
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.028, 0.95))),
                    MeshMaterial3d(pal.brass_dark.clone()),
                    Transform::from_xyz(0.0, 0.475, 0.0),
                ));
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.42, 0.025, 0.30))),
                    MeshMaterial3d(pal.wood_h.clone()),
                    Transform::from_xyz(0.0, 0.96, 0.0),
                ));
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.30, 0.008, 0.21))),
                    MeshMaterial3d(pal.paper.clone()),
                    Transform::from_xyz(0.0, 0.978, 0.0)
                        .with_rotation(Quat::from_rotation_y(0.22)),
                ));
            });
            commands.entity(root).insert(Interactable::new(
                Action::Flavor(1),
                "Notiz lesen",
                1.2,
            ));
        }
        "keypad" => {
            commands.entity(root).with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.26, 0.36, 0.07))),
                    MeshMaterial3d(pal.black_metal.clone()),
                    Transform::IDENTITY,
                ));
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.19, 0.10, 0.02))),
                    MeshMaterial3d(pal.led_green.clone()),
                    Transform::from_xyz(0.0, 0.10, 0.045),
                ));
            });
            commands.entity(root).insert(Interactable::new(
                Action::SafeKeypad,
                "Code eingeben",
                1.1,
            ));
        }
        "model" => {
            let file = object.get("file").unwrap_or_default().to_string();
            let scale = object.number("scale", 1.0).max(0.01);
            let voxel = object.number("voxel", 0.25);
            let solid = object.flag("solid", true);
            if !file.is_empty() {
                match models::scene(&file) {
                    Some(scene) => {
                        let hull = if solid {
                            models::hull(&file, &*meshes, voxel)
                        } else {
                            Vec::new()
                        };
                        commands.entity(root).with_children(|parent| {
                            parent.spawn((
                                SceneRoot(scene),
                                Transform::from_scale(Vec3::splat(scale)),
                            ));
                            // one collider per filled voxel run: only where the
                            // model really has geometry, so a hollow room is
                            // walkable inside
                            for (center, half) in hull.iter() {
                                parent.spawn((
                                    Transform::from_translation(*center * scale),
                                    SolidBox::new(*half * scale),
                                ));
                            }
                        });
                    }
                    None => {
                        // not loaded yet: show where it will be
                        commands.entity(root).with_children(|parent| {
                            parent.spawn((
                                Mesh3d(meshes.add(Cuboid::new(0.6, 0.6, 0.6))),
                                MeshMaterial3d(pal.bulb_dim.clone()),
                                Transform::from_xyz(0.0, 0.3, 0.0),
                            ));
                        });
                    }
                }
            }
        }
        "door" => {
            if let Some(leaf) = door_leaf {
                let width = object.number("w", 1.0);
                let height = object.number("h", 2.1);
                commands.entity(leaf).insert(Swing::new(
                    Quat::IDENTITY,
                    Quat::from_rotation_y(-1.55),
                    0.9,
                ));
                let mut blocker = None;
                let local = Vec3::new(0.0, height * 0.5, 0.0);
                commands.entity(root).with_children(|parent| {
                    blocker = Some(
                        parent
                            .spawn((
                                Transform::from_translation(local),
                                SolidBox::new(Vec3::new(
                                    width * 0.5 + 0.08,
                                    height * 0.5,
                                    0.14,
                                )),
                            ))
                            .id(),
                    );
                });
                commands.entity(root).insert(Interactable::new(
                    Action::Custom(0),
                    "Tuer oeffnen",
                    1.3,
                ));
                if let Some(blocker) = blocker {
                    commands.entity(root).insert(CustomDoor {
                        leaf,
                        blocker,
                        blocker_at: local,
                        open: false,
                        duration: 0.9,
                    });
                }
            }
        }
        _ => {}
    }

    root
}
