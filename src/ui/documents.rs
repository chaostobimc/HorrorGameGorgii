//! Reading the things Gorgii finds: newspaper clippings, the chef's diary and
//! the notebook that collects them.

use bevy::prelude::*;

use crate::core::{GameAssets, Overlay, UiState};
use crate::story::{Progress, DOCUMENTS};

use super::{JournalOpen, COL_GOLD, COL_PANEL, COL_TEXT};

#[derive(Component)]
pub struct DocumentRoot;

#[derive(Component)]
pub struct DocumentCloseHint;

#[derive(Component)]
pub struct JournalRoot;

#[derive(Component)]
pub struct JournalEntry(pub usize);

pub fn sync_viewer(
    mut commands: Commands,
    assets: Res<GameAssets>,
    ui: Res<UiState>,
    existing: Query<Entity, With<DocumentRoot>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut journal: ResMut<JournalOpen>,
    mut sfx: EventWriter<crate::audio::PlaySfx>,
) {
    let open = ui.overlay == Overlay::Document;
    if open {
        if existing.iter().next().is_some() {
            return;
        }
        sfx.send(crate::audio::PlaySfx::flat(
            assets.snd.paper_flip.clone(),
            0.5,
        ));
        spawn_document(&mut commands, &assets, ui.open_doc);
    } else {
        for e in existing.iter() {
            commands.entity(e).despawn_recursive();
        }
    }
    let _ = (&keys, &mouse, &mut journal);
}

fn spawn_document(commands: &mut Commands, assets: &GameAssets, index: usize) {
    let doc = DOCUMENTS.get(index.min(DOCUMENTS.len() - 1)).unwrap();
    let title_font = assets.fonts.title.clone();
    let doc_font = assets.fonts.doc.clone();
    let ui_font = assets.fonts.ui.clone();
    let paper = assets.tex.paper.clone();

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
            GlobalZIndex(60),
            DocumentRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(720.0),
                    padding: UiRect::all(Val::Px(58.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(18.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.79, 0.75, 0.63)),
                BorderColor(Color::srgba(0.15, 0.13, 0.10, 0.55)),
                BorderRadius::all(Val::Px(2.0)),
                ImageNode {
                    image: paper,
                    ..default()
                },
                BoxShadow::default(),
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new(doc.source.to_uppercase()),
                    TextFont {
                        font: ui_font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.20, 0.17, 0.13, 0.75)),
                ));
                card.spawn((
                    Text::new(doc.title),
                    TextFont {
                        font: title_font.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.09, 0.08, 0.07)),
                ));
                card.spawn((
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(1.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.18, 0.15, 0.11, 0.45)),
                ));
                card.spawn((
                    Text::new(doc.body),
                    TextFont {
                        font: doc_font.clone(),
                        font_size: 15.5,
                        ..default()
                    },
                    TextColor(Color::srgb(0.11, 0.10, 0.09)),
                    TextLayout::new_with_justify(JustifyText::Left),
                    Node {
                        width: Val::Px(604.0),
                        ..default()
                    },
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(38.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::new(Val::Px(-180.0), Val::Px(0.0), Val::Px(0.0), Val::Px(0.0)),
                    width: Val::Px(360.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                DocumentCloseHint,
            ))
            .with_children(|hint| {
                hint.spawn((
                    Text::new("E ODER ESC ZUM SCHLIESSEN"),
                    TextFont {
                        font: ui_font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(COL_TEXT),
                ));
            });
        });
}

/// Closes the document again. Runs before `sync_viewer` despawns it.
pub fn close_document(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut ui: ResMut<UiState>,
) {
    if ui.overlay != Overlay::Document {
        return;
    }
    if keys.just_pressed(KeyCode::KeyE)
        || keys.just_pressed(KeyCode::Escape)
        || keys.just_pressed(KeyCode::Space)
        || mouse.just_pressed(MouseButton::Left)
    {
        ui.overlay = Overlay::None;
    }
}

/// The notebook (TAB). Re-read what you have found so far.
#[derive(Component)]
pub struct JournalHint;

pub fn sync_journal(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut journal: ResMut<JournalOpen>,
    progress: Res<Progress>,
    existing: Query<Entity, With<JournalRoot>>,
    mut entries: Query<(&Interaction, &JournalEntry), Changed<Interaction>>,
    mut ui_state: ResMut<UiState>,
) {
    for (interaction, entry) in entries.iter_mut() {
        if *interaction == Interaction::Pressed {
            ui_state.open_doc = entry.0;
            ui_state.overlay = Overlay::Document;
            journal.0 = false;
        }
    }

    let want = journal.0 && ui_state.overlay == Overlay::None;
    let have = existing.iter().next().is_some();
    if want && !have {
        spawn_journal(&mut commands, &assets, &progress);
    } else if !want && have {
        for e in existing.iter() {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn spawn_journal(commands: &mut Commands, assets: &GameAssets, progress: &Progress) {
    let ui_font = assets.fonts.ui.clone();
    let ui_light = assets.fonts.ui_light.clone();
    let title_font = assets.fonts.title.clone();

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
            GlobalZIndex(58),
            JournalRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(760.0),
                    padding: UiRect::all(Val::Px(48.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(14.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(COL_PANEL),
                BorderColor(Color::srgba(0.75, 0.70, 0.58, 0.22)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Notizbuch"),
                    TextFont {
                        font: title_font.clone(),
                        font_size: 34.0,
                        ..default()
                    },
                    TextColor(COL_GOLD),
                ));
                panel.spawn((
                    Text::new(progress.objective()),
                    TextFont {
                        font: ui_light.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(COL_TEXT),
                ));
                panel.spawn((
                    Node {
                        height: Val::Px(1.0),
                        width: Val::Percent(100.0),
                        margin: UiRect::vertical(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.75, 0.70, 0.58, 0.18)),
                ));

                for (i, doc) in DOCUMENTS.iter().enumerate() {
                    let found = progress.notes.contains(&i);
                    let label = if found {
                        doc.title.to_string()
                    } else {
                        "— — —".to_string()
                    };
                    let color = if found {
                        Color::srgba(0.88, 0.85, 0.78, 0.92)
                    } else {
                        Color::srgba(0.60, 0.57, 0.50, 0.35)
                    };
                    let mut entity = panel.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ));
                    if found {
                        entity.insert((Button, JournalEntry(i)));
                    }
                    entity.with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("{}", label)),
                            TextFont {
                                font: ui_font.clone(),
                                font_size: 19.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    });
                }

                panel.spawn((
                    Node {
                        height: Val::Px(1.0),
                        width: Val::Percent(100.0),
                        margin: UiRect::vertical(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.75, 0.70, 0.58, 0.18)),
                ));
                panel.spawn((
                    Text::new("TAB SCHLIESSEN   ·   EINTRAG ANKLICKEN ZUM LESEN"),
                    TextFont {
                        font: ui_font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.70, 0.66, 0.58, 0.45)),
                ));
            });
        });
}

/// ESC also closes the notebook.
pub fn close_journal(keys: Res<ButtonInput<KeyCode>>, mut journal: ResMut<JournalOpen>) {
    if journal.0 && keys.just_pressed(KeyCode::Escape) {
        journal.0 = false;
    }
}
