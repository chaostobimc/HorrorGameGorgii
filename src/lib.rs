//! The Diplomat Peak — shared game code.
//!
//! Two programs build on this library:
//!
//! * `src/main.rs` — the game itself.
//! * `src/bin/editor.rs` — the level editor. It renders the real hotel with
//!   the real palette and props and writes `assets/level/extras.lvl`, which
//!   the game loads on top of the hand built level.
//!
//! Keeping both on one library is what makes "place it in the editor, see it
//! in the game" true instead of a promise.

// A handful of helpers and markers are prepared for scenes that are not wired
// up yet. Keeping them silent makes the build output readable.
#![allow(dead_code)]

pub mod audio;
pub mod body;
pub mod collision;
pub mod core;
pub mod custom;
pub mod factory;
pub mod finale;
pub mod fx;
pub mod geom;
pub mod interact;
pub mod level;
pub mod lighting;
pub mod palette;
pub mod player;
pub mod puzzles;
pub mod story;
pub mod ui;
pub mod world;
