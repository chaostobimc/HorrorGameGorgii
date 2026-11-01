//! F3 diagnostics.
//!
//! Shows state, input and camera numbers on screen. It is the fastest way to
//! tell *why* the player cannot move: no state transition, a stuck overlay,
//! a refused cursor grab or keys that never reach the game all look different
//! here. Visible for the first seconds of play, then toggled with F3.

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::time::{Real, Virtual};

use crate::core::{AppState, UiState};
use crate::player::Player;

#[derive(Component)]
pub struct DebugRoot;

#[derive(Component)]
pub struct DebugText;

pub fn spawn_debug_hud(mut commands: Commands, assets: Res<crate::core::GameAssets>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            top: Val::Px(16.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.72)),
        GlobalZIndex(60),
        Visibility::Hidden,
        PickingBehavior::IGNORE,
        DebugRoot,
    ))
    .with_children(|c| {
        c.spawn((
            Text::new(""),
            TextFont {
                font: assets.fonts.ui.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.78, 0.92, 0.72)),
            DebugText,
        ));
    });
}

pub fn update_debug(
    time: Res<Time<Real>>,
    virt: Res<Time<Virtual>>,
    state: Res<State<AppState>>,
    ui: Res<UiState>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    player: Query<&Player>,
    mut motion: EventReader<MouseMotion>,
    mut shown_for: Local<f32>,
    mut toggle: Local<bool>,
    mut fps: Local<f32>,
    mut root: Query<&mut Visibility, With<DebugRoot>>,
    mut text: Query<&mut Text, With<DebugText>>,
) {
    let dt = time.delta_secs();
    *shown_for += dt;

    if keys.just_pressed(KeyCode::F3) {
        *toggle = !*toggle;
        // a press during the intro window closes it for good as well
        *shown_for = 100.0;
    }
    let visible = *toggle || *shown_for < 6.0;

    let Ok(mut vis) = root.get_single_mut() else {
        return;
    };
    *vis = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if !visible {
        return;
    }

    if dt > 0.0 {
        let inst = 1.0 / dt;
        *fps = if *fps == 0.0 {
            inst
        } else {
            *fps * 0.92 + inst * 0.08
        };
    }

    let mut moved = 0.0f32;
    for ev in motion.read() {
        moved += ev.delta.length();
    }

    let (grab, cursor_vis, focused) = match windows.get_single() {
        Ok(w) => (
            format!("{:?}", w.cursor_options.grab_mode),
            w.cursor_options.visible,
            w.focused,
        ),
        Err(_) => ("???".to_string(), false, false),
    };

    let (pos, vel, ground) = match player.get_single() {
        Ok(p) => {
            let pos = p.last_safe;
            (format!("{:.2} {:.2}", pos.x, pos.z), p.vel_xz.length(), p.on_ground)
        }
        Err(_) => ("NO PLAYER ENTITY".to_string(), 0.0, false),
    };

    let body = format!(
        "STATE {:?}   fps {:.0}   paused {}\n\
         ui: overlay {:?}  locked {}  can_play {}\n\
         cursor: {}  visible {}  focused {}\n\
         keys: W {} A {} S {} D {} shift {}   mouse-motion {:.1}\n\
         player: xz {}  speed {:.2}  ground {}  torch {}\n\
         F3 schliesst diese Anzeige",
        state.get(),
        *fps,
        virt.is_paused(),
        ui.overlay,
        ui.locked,
        ui.can_play(),
        grab,
        cursor_vis,
        focused,
        keys.pressed(KeyCode::KeyW),
        keys.pressed(KeyCode::KeyA),
        keys.pressed(KeyCode::KeyS),
        keys.pressed(KeyCode::KeyD),
        keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight),
        moved,
        pos,
        vel,
        ground,
        match player.get_single() {
            Ok(p) => p.flashlight,
            Err(_) => false,
        },
    );

    if let Ok(mut t) = text.get_single_mut() {
        if t.0 != body {
            t.0 = body;
        }
    }
}
