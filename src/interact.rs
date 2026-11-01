//! Looking at things and pressing E.

use bevy::prelude::*;

use crate::collision::{ray_sphere, SolidBox};
use crate::core::{AppState, UiState};
use crate::player::PlayerCamera;

/// Everything the player can do with the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ReadDeskArticle,
    SafeKeypad,
    SafeTakeKey,
    CorridorDoor,
    ReadCorridorArticle,
    Lever(u8),
    ElevatorCall,
    ReadBallroomArticle,
    ReadChefDiary,
    ClockHour,
    ClockMinute,
    ClockLatch,
    Piano,
    EmergencySwitch,
    Gate,
    Flavor(u8),
}

#[derive(Component, Clone)]
pub struct Interactable {
    pub action: Action,
    /// Shown in the prompt, e.g. "Tresor öffnen".
    pub prompt: &'static str,
    /// Bounding sphere radius used for the hit test.
    pub radius: f32,
    pub enabled: bool,
}

impl Interactable {
    pub fn new(action: Action, prompt: &'static str, radius: f32) -> Self {
        Self {
            action,
            prompt,
            radius,
            enabled: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct Focus {
    pub entity: Option<Entity>,
    pub prompt: String,
    pub action: Option<Action>,
    pub distance: f32,
}

#[derive(Event, Clone, Copy)]
pub struct Interacted {
    pub action: Action,
    pub entity: Entity,
}

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Focus>()
            .add_event::<Interacted>()
            .add_systems(
                Update,
                (update_focus, interact_input)
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn update_focus(
    ui: Res<UiState>,
    mut focus: ResMut<Focus>,
    cam: Query<&GlobalTransform, With<PlayerCamera>>,
    mut items: Query<(Entity, &GlobalTransform, &Interactable)>,
    solids: Query<(&GlobalTransform, &SolidBox)>,
) {
    focus.entity = None;
    focus.prompt.clear();
    focus.action = None;
    focus.distance = f32::MAX;
    if !ui.can_play() {
        return;
    }
    let Ok(cam_t) = cam.get_single() else {
        return;
    };
    let origin = cam_t.translation();
    let dir = cam_t.rotation() * Vec3::NEG_Z;

    // Distance to the nearest wall along the ray, so you cannot read a note
    // through a closed door.
    let mut wall_dist = 3.6f32;
    for (t, s) in solids.iter() {
        let d = ray_sphere(
            origin,
            dir,
            t.translation(),
            s.half.length_squared().sqrt() * 0.55,
        );
        if let Some(d) = d {
            if d < wall_dist {
                wall_dist = d;
            }
        }
    }

    let mut best = f32::MAX;
    let mut found: Option<(Entity, Action, &'static str, f32)> = None;
    for (entity, t, item) in items.iter_mut() {
        if !item.enabled {
            continue;
        }
        let Some(d) = ray_sphere(origin, dir, t.translation(), item.radius) else {
            continue;
        };
        if d > 3.4 {
            continue;
        }
        if d < best {
            best = d;
            found = Some((entity, item.action, item.prompt, d));
        }
    }
    if let Some((entity, action, prompt, d)) = found {
        focus.entity = Some(entity);
        focus.prompt = prompt.to_string();
        focus.action = Some(action);
        focus.distance = d;
    }
}

fn interact_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    ui: Res<UiState>,
    focus: Res<Focus>,
    mut events: EventWriter<Interacted>,
) {
    if !ui.can_play() {
        return;
    }
    let pressed = keys.just_pressed(KeyCode::KeyE) || mouse.just_pressed(MouseButton::Left);
    if !pressed {
        return;
    }
    if let (Some(entity), Some(action)) = (focus.entity, focus.action) {
        events.send(Interacted { action, entity });
    }
}
