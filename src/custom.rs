//! The custom layer: everything that was placed in the level editor.
//!
//! The game rebuilds it from `assets/level/extras.lvl` when the hotel is built,
//! so editor work shows up in the real thing without touching any code.

use bevy::prelude::*;

use crate::audio::PlaySfx;
use crate::core::{AppState, GameAssets};
use crate::factory;
use crate::interact::{Action, Interacted};
use crate::level::LevelFile;
use crate::palette::Palette;
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

/// Builds every object of the level file into the running hotel.
pub fn spawn_level_objects(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    pal: &Palette,
    assets: &GameAssets,
) {
    let level = LevelFile::load();
    for object in level.objects.iter() {
        factory::spawn_object(commands, meshes, pal, assets, object);
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
        app.add_systems(Update, custom_doors.run_if(in_state(AppState::Playing)));
    }
}
