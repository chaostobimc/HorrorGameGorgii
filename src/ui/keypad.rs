//! The five digit combination lock on the reception safe.

use bevy::prelude::*;

use crate::core::{GameAssets, Overlay, UiState};

use super::{COL_BRIGHT, COL_DIM, COL_PANEL, COL_TEXT};

pub const SAFE_CODE: &str = "10449";

#[derive(Resource)]
pub struct Keypad {
    pub digits: String,
    pub error: f32,
    pub success: f32,
    pub solved: bool,
}

impl Default for Keypad {
    fn default() -> Self {
        Self {
            digits: String::new(),
            error: 0.0,
            success: 0.0,
            solved: false,
        }
    }
}

#[derive(Event)]
pub struct SafeOpened;

#[derive(Component)]
pub struct KeypadRoot;

#[derive(Component)]
pub struct KeypadDisplay;

#[derive(Component)]
pub struct KeypadKey(pub char);

/// Applies one key press. Returns true when the player asked to submit.
fn press_key(
    ch: char,
    state: &mut Keypad,
    sfx: &mut EventWriter<crate::audio::PlaySfx>,
    assets: &GameAssets,
) -> bool {
    if state.success > 0.0 {
        return false;
    }
    match ch {
        'C' => {
            state.digits.clear();
            sfx.send(crate::audio::PlaySfx::flat(assets.snd.keypad_beep.clone(), 0.4));
            false
        }
        'K' => true,
        d if d.is_ascii_digit() => {
            if state.digits.len() < 5 {
                state.digits.push(d);
                sfx.send(crate::audio::PlaySfx::flat(assets.snd.keypad_beep.clone(), 0.42));
            }
            false
        }
        _ => false,
    }
}

pub fn sync_keypad(
    mut commands: Commands,
    assets: Res<GameAssets>,
    existing: Query<Entity, With<KeypadRoot>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut buttons: Query<(&Interaction, &KeypadKey), Changed<Interaction>>,
    mut sfx: EventWriter<crate::audio::PlaySfx>,
    mut opened: EventWriter<SafeOpened>,
    mut state: ResMut<Keypad>,
    mut ui_state: ResMut<UiState>,
    time: Res<Time>,
    mut display: Query<&mut Text, With<KeypadDisplay>>,
) {
    let dt = time.delta_secs();
    state.error = (state.error - dt).max(0.0);
    if state.success > 0.0 {
        state.success -= dt;
        if state.success <= 0.0 && ui_state.overlay == Overlay::Keypad {
            ui_state.overlay = Overlay::None;
        }
    }

    let open = ui_state.overlay == Overlay::Keypad;
    let have = existing.iter().next().is_some();
    if open && !have {
        spawn_keypad(&mut commands, &assets);
    } else if !open && have {
        for e in existing.iter() {
            commands.entity(e).despawn_recursive();
        }
        state.digits.clear();
    }
    if !open {
        return;
    }

    let mut submit = false;
    for (interaction, key) in buttons.iter_mut() {
        if *interaction == Interaction::Pressed && press_key(key.0, &mut state, &mut sfx, &assets) {
            submit = true;
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        ui_state.overlay = Overlay::None;
        return;
    }
    if keys.just_pressed(KeyCode::Backspace) {
        state.digits.pop();
        sfx.send(crate::audio::PlaySfx::flat(assets.snd.keypad_beep.clone(), 0.3));
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        submit = true;
    }
    for (key, ch) in [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
        (KeyCode::Numpad0, '0'),
        (KeyCode::Numpad1, '1'),
        (KeyCode::Numpad2, '2'),
        (KeyCode::Numpad3, '3'),
        (KeyCode::Numpad4, '4'),
        (KeyCode::Numpad5, '5'),
        (KeyCode::Numpad6, '6'),
        (KeyCode::Numpad7, '7'),
        (KeyCode::Numpad8, '8'),
        (KeyCode::Numpad9, '9'),
    ] {
        if keys.just_pressed(key) && press_key(ch, &mut state, &mut sfx, &assets) {
            submit = true;
        }
    }

    if submit && !state.solved && state.digits.len() == 5 {
        if state.digits == SAFE_CODE {
            state.solved = true;
            state.success = 1.1;
            sfx.send(crate::audio::PlaySfx::flat(assets.snd.keypad_ok.clone(), 0.75));
            sfx.send(crate::audio::PlaySfx::flat(assets.snd.gear_clunk.clone(), 0.6));
            opened.send(SafeOpened);
        } else {
            state.error = 1.0;
            state.digits.clear();
            sfx.send(crate::audio::PlaySfx::flat(assets.snd.keypad_error.clone(), 0.6));
        }
    }

    // ---- display ---------------------------------------------------------
    if let Ok(mut text) = display.get_single_mut() {
        let mut shown = String::new();
        for i in 0..5 {
            shown.push(if i >= state.digits.len() { '·' } else { '●' });
            if i < 4 {
                shown.push(' ');
            }
        }
        if text.0 != shown {
            text.0 = shown;
        }
    }
}

fn spawn_keypad(commands: &mut Commands, assets: &GameAssets) {
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
            GlobalZIndex(60),
            KeypadRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(392.0),
                    padding: UiRect::all(Val::Px(28.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(COL_PANEL),
                BorderColor(Color::srgba(0.55, 0.52, 0.45, 0.35)),
                BorderRadius::all(Val::Px(4.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("SCHLOSS · MODELL 88"),
                    TextFont {
                        font: ui.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(COL_DIM),
                ));

                // display
                panel.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(72.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.02, 0.035, 0.03)),
                    BorderColor(Color::srgba(0.35, 0.5, 0.4, 0.4)),
                ))
                .with_children(|d| {
                    d.spawn((
                        Text::new("· · · · ·"),
                        TextFont {
                            font: title.clone(),
                            font_size: 34.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.95, 0.65)),
                        KeypadDisplay,
                    ));
                });

                let rows: [[char; 3]; 4] = [
                    ['1', '2', '3'],
                    ['4', '5', '6'],
                    ['7', '8', '9'],
                    ['C', '0', 'K'],
                ];
                for row in rows.iter() {
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|r| {
                            for ch in row.iter() {
                                let label = match ch {
                                    'C' => "LÖSCH".to_string(),
                                    'K' => "OK".to_string(),
                                    d => d.to_string(),
                                };
                                r.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(104.0),
                                        height: Val::Px(58.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.10, 0.11, 0.115, 0.95)),
                                    BorderColor(Color::srgba(0.45, 0.43, 0.38, 0.35)),
                                    BorderRadius::all(Val::Px(3.0)),
                                    KeypadKey(*ch),
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        TextFont {
                                            font: ui_light.clone(),
                                            font_size: 22.0,
                                            ..default()
                                        },
                                        TextColor(COL_BRIGHT),
                                    ));
                                });
                            }
                        });
                }

                panel.spawn((
                    Text::new("ESC  ABBRECHEN"),
                    TextFont {
                        font: ui.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(COL_TEXT),
                ));
            });
        });
}
