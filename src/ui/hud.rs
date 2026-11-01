//! In-game HUD: objective, interaction prompt, subtitles, fades.

use bevy::prelude::*;

use crate::core::{GameAssets, Overlay, UiState};
use crate::interact::Focus;
use crate::story::{Progress, Subtitle, DOCUMENTS};

use super::{Cinematic, Fade, JournalOpen, COL_BRIGHT, COL_DIM, COL_FAINT, COL_TEXT};

#[derive(Component)]
pub struct HudRoot;
#[derive(Component)]
pub struct ObjectiveText;
#[derive(Component)]
pub struct NotesText;
#[derive(Component)]
pub struct PromptRow;
#[derive(Component)]
pub struct PromptKey;
#[derive(Component)]
pub struct PromptText;
#[derive(Component)]
pub struct SubtitleText;
#[derive(Component)]
pub struct CrosshairRing;
#[derive(Component)]
pub struct CrosshairDot;
#[derive(Component)]
pub struct FadeLayer;
#[derive(Component)]
pub struct LetterboxTop;
#[derive(Component)]
pub struct LetterboxBottom;
#[derive(Component)]
pub struct HintText;

pub fn spawn_hud(mut commands: Commands, assets: Res<GameAssets>) {
    let ui = assets.fonts.ui.clone();
    let ui_light = assets.fonts.ui_light.clone();
    let doc = assets.fonts.doc.clone();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(10),
            PickingBehavior::IGNORE,
            HudRoot,
        ))
        .with_children(|root| {
            // ---- objective -------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(46.0),
                    top: Val::Px(38.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    max_width: Val::Px(430.0),
                    ..default()
                },
                PickingBehavior::IGNORE,
            ))
            .with_children(|c| {
                c.spawn((
                    Text::new("A U F G A B E"),
                    TextFont {
                        font: ui.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(COL_FAINT),
                ));
                c.spawn((
                    Text::new(""),
                    TextFont {
                        font: ui_light.clone(),
                        font_size: 19.0,
                        ..default()
                    },
                    TextColor(COL_TEXT),
                    ObjectiveText,
                ));
            });

            // ---- notes counter ---------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(46.0),
                    top: Val::Px(40.0),
                    ..default()
                },
                PickingBehavior::IGNORE,
            ))
            .with_children(|c| {
                c.spawn((
                    Text::new(""),
                    TextFont {
                        font: ui.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(COL_FAINT),
                    NotesText,
                ));
            });

            // ---- crosshair -------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(3.0),
                    height: Val::Px(3.0),
                    margin: UiRect::new(Val::Px(-1.5), Val::Px(0.0), Val::Px(-1.5), Val::Px(0.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.92, 0.90, 0.84, 0.45)),
                PickingBehavior::IGNORE,
                CrosshairDot,
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(26.0),
                    height: Val::Px(26.0),
                    margin: UiRect::new(Val::Px(-13.0), Val::Px(0.0), Val::Px(-13.0), Val::Px(0.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(13.0)),
                BorderColor(Color::srgba(0.92, 0.90, 0.84, 0.10)),
                PickingBehavior::IGNORE,
                CrosshairRing,
            ));

            // ---- interaction prompt ---------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Percent(16.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    display: Display::None,
                    ..default()
                },
                PickingBehavior::IGNORE,
                PromptRow,
            ))
            .with_children(|c| {
                c.spawn((
                    Node {
                        width: Val::Px(26.0),
                        height: Val::Px(26.0),
                        border: UiRect::all(Val::Px(1.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderColor(Color::srgba(0.86, 0.84, 0.78, 0.55)),
                    BorderRadius::all(Val::Px(3.0)),
                    PickingBehavior::IGNORE,
                ))
                .with_children(|k| {
                    k.spawn((
                        Text::new("E"),
                        TextFont {
                            font: ui.clone(),
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(COL_BRIGHT),
                        PromptKey,
                    ));
                });
                c.spawn((
                    Text::new(""),
                    TextFont {
                        font: ui_light.clone(),
                        font_size: 21.0,
                        ..default()
                    },
                    TextColor(COL_BRIGHT),
                    PromptText,
                ));
            });

            // ---- subtitle --------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(20.0),
                    right: Val::Percent(20.0),
                    bottom: Val::Percent(27.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                PickingBehavior::IGNORE,
            ))
            .with_children(|c| {
                c.spawn((
                    Text::new(""),
                    TextFont {
                        font: doc.clone(),
                        font_size: 19.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.90, 0.87, 0.80, 0.0)),
                    TextLayout::new_with_justify(JustifyText::Center),
                    SubtitleText,
                ));
            });

            // ---- onboarding hint ------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(44.0),
                    margin: UiRect::new(Val::Px(-330.0), Val::Px(0.0), Val::Px(0.0), Val::Px(0.0)),
                    width: Val::Px(660.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                PickingBehavior::IGNORE,
            ))
            .with_children(|c| {
                c.spawn((
                    Text::new("WASD BEWEGEN   ·   MAUS UMSEHEN   ·   E INTERAGIEREN   ·   F STIRNLAMPE   ·   TAB NOTIZEN   ·   ESC MENÜ"),
                    TextFont {
                        font: ui.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(COL_FAINT),
                    TextLayout::new_with_justify(JustifyText::Center),
                    HintText,
                ));
            });

            // ---- letterbox bars -------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                PickingBehavior::IGNORE,
                LetterboxTop,
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                PickingBehavior::IGNORE,
                LetterboxBottom,
            ));

            // ---- fade ------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 1.0)),
                GlobalZIndex(30),
                PickingBehavior::IGNORE,
                FadeLayer,
            ));
        });
}

pub fn update_prompt(
    focus: Res<Focus>,
    mut row: Query<&mut Node, With<PromptRow>>,
    mut text: Query<&mut Text, With<PromptText>>,
    mut ring: Query<&mut BorderColor, With<CrosshairRing>>,
) {
    let focused = focus.entity.is_some();
    if let Ok(mut node) = row.get_single_mut() {
        let want = if focused { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
    }
    if let Ok(mut t) = text.get_single_mut() {
        if t.0 != focus.prompt {
            t.0 = focus.prompt.clone();
        }
    }
    if let Ok(mut b) = ring.get_single_mut() {
        let target = if focused {
            Color::srgba(0.92, 0.88, 0.78, 0.45)
        } else {
            Color::srgba(0.92, 0.90, 0.84, 0.10)
        };
        if b.0 != target {
            b.0 = target;
        }
    }
}

pub fn update_objective(
    progress: Res<Progress>,
    mut q: Query<&mut Text, With<ObjectiveText>>,
    mut last: Local<String>,
) {
    let text = progress.objective().to_string();
    if *last == text {
        return;
    }
    *last = text.clone();
    if let Ok(mut t) = q.get_single_mut() {
        t.0 = text;
    }
}

pub fn update_notes(
    progress: Res<Progress>,
    mut q: Query<&mut Text, With<NotesText>>,
    mut last: Local<usize>,
) {
    if progress.notes.len() == *last {
        return;
    }
    *last = progress.notes.len();
    if let Ok(mut t) = q.get_single_mut() {
        t.0 = format!(
            "N O T I Z E N   {} / {}",
            progress.notes.len(),
            DOCUMENTS.len()
        );
    }
}

pub fn update_subtitle(
    subtitle: Res<Subtitle>,
    mut q: Query<(&mut Text, &mut TextColor), With<SubtitleText>>,
) {
    if let Ok((mut text, mut color)) = q.get_single_mut() {
        if text.0 != subtitle.text {
            text.0 = subtitle.text.clone();
        }
        let alpha = if subtitle.timer <= 0.0 {
            0.0
        } else {
            let fade_in = (1.0 - subtitle.timer / subtitle.total.max(0.001)).min(1.0);
            let fade_out = (subtitle.timer / 0.8).min(1.0);
            (fade_in * 3.0).min(1.0) * fade_out * 0.95
        };
        color.0 = Color::srgba(0.90, 0.87, 0.80, alpha);
    }
}

pub fn update_fade(fade: Res<Fade>, mut q: Query<&mut BackgroundColor, With<FadeLayer>>) {
    if let Ok(mut bg) = q.get_single_mut() {
        let a = fade.alpha.clamp(0.0, 1.0);
        let c = Color::srgba(0.0, 0.0, 0.0, a);
        if bg.0 != c {
            bg.0 = c;
        }
    }
}

pub fn update_letterbox(
    cinematic: Res<Cinematic>,
    time: Res<Time>,
    mut q_top: Query<&mut Node, (With<LetterboxTop>, Without<LetterboxBottom>)>,
    mut q_bottom: Query<&mut Node, (With<LetterboxBottom>, Without<LetterboxTop>)>,
    mut current: Local<f32>,
) {
    let target = if cinematic.0 { 74.0 } else { 0.0 };
    let dt = time.delta_secs();
    let diff = target - *current;
    *current += diff.signum() * (90.0 * dt).min(diff.abs());
    if let Ok(mut n) = q_top.get_single_mut() {
        n.height = Val::Px(*current);
    }
    if let Ok(mut n) = q_bottom.get_single_mut() {
        n.height = Val::Px(*current);
    }
}

pub fn toggle_journal(
    keys: Res<ButtonInput<KeyCode>>,
    mut journal: ResMut<JournalOpen>,
    ui: Res<UiState>,
    mut hint: Query<&mut TextColor, With<HintText>>,
    time: Res<Time>,
    mut hint_timer: Local<f32>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        // slowly retire the onboarding hint
        *hint_timer += time.delta_secs();
        if *hint_timer > 22.0 {
            if let Ok(mut c) = hint.get_single_mut() {
                let a = (c.0.alpha() - time.delta_secs() * 0.10).max(0.0);
                c.0 = Color::srgba(0.70, 0.66, 0.58, a);
            }
        }
        return;
    }
    if ui.overlay != Overlay::None || ui.locked {
        return;
    }
    journal.0 = !journal.0;
}

/// Blanks the HUD when a modal panel wants the whole screen.
pub fn hide_hud(mut q: Query<&mut Visibility, With<HudRoot>>, ui: Res<UiState>) {
    if let Ok(mut v) = q.get_single_mut() {
        let want = if ui.overlay == Overlay::Document {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *v != want {
            *v = want;
        }
    }
}

pub const _KEEP: Color = COL_DIM;
