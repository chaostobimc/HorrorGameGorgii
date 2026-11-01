//! The last beauty pass over the shared spaces.
//!
//! The lobby is twenty metres wide and seven metres high, so it needs a
//! ceiling that reads as a ceiling rather than as a void: a coffered wood
//! grid, a cornice, and a stone inlay on the marble floor that leads the eye
//! from the entrance to the reception. The west half of the room also gets its
//! own lounge island, mirrored against the one that already exists.

use bevy::prelude::*;

use crate::palette::Palette;
use crate::world::{props, BuildCtx, F1, GROUND};

/// Ceiling height of the first floor corridor (see `corridor.rs`).
const CORRIDOR_CEIL: f32 = 7.9;
/// Inner faces of the two long corridor walls.
const CORRIDOR_DOOR_WALL: f32 = -18.175;
const CORRIDOR_WINDOW_WALL: f32 = -21.8;

pub fn dress(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
) {
    coffered_ceiling(b, pal);
    floor_inlay(b, pal);
    corridor_beams(b, pal);
    west_lounge(commands, meshes, pal);
}

/// Wooden beams under the lobby ceiling plus a run of cornice.
fn coffered_ceiling(b: &mut BuildCtx, pal: &Palette) {
    b.mat(&pal.wood);
    for x in [-8.0f32, -4.0, 4.0, 8.0] {
        b.mb().box_all(
            Vec3::new(x, 6.80, 0.0),
            Vec3::new(0.36, 0.40, 19.4),
            1.4,
        );
    }
    for z in [-5.0f32, 5.0] {
        b.mb().box_all(
            Vec3::new(0.0, 6.84, z),
            Vec3::new(23.4, 0.32, 0.34),
            1.4,
        );
    }

    // cornice around the whole room
    b.mat(&pal.trim);
    b.mb()
        .box_all(Vec3::new(0.0, 6.90, -9.70), Vec3::new(23.4, 0.20, 0.32), 1.0);
    b.mb()
        .box_all(Vec3::new(0.0, 6.90, 9.70), Vec3::new(23.4, 0.20, 0.32), 1.0);
    b.mb()
        .box_all(Vec3::new(-11.70, 6.90, 0.0), Vec3::new(0.32, 0.20, 19.4), 1.0);
    b.mb()
        .box_all(Vec3::new(11.70, 6.90, 0.0), Vec3::new(0.32, 0.20, 19.4), 1.0);
}

/// Dark stone bands inlaid into the marble across the entrance axis. They
/// stay north and south of the lounge islands so nothing overlaps a carpet.
fn floor_inlay(b: &mut BuildCtx, pal: &Palette) {
    b.mat(&pal.stone);
    for z in [-8.9f32, 8.9] {
        b.mb().box_all(
            Vec3::new(0.0, 0.006, z),
            Vec3::new(22.2, 0.012, 0.9),
            1.4,
        );
    }
    // the carpet runner meets the inlay here, so give it an end piece
    b.mat(&pal.wood_h);
    b.mb()
        .box_all(Vec3::new(0.0, 0.007, 5.2), Vec3::new(4.0, 0.014, 0.22), 1.0);
}

/// The corridor is four metres wide and three high: beams and a skirting give
/// it the rhythm of a hotel floor instead of the look of a tunnel.
fn corridor_beams(b: &mut BuildCtx, pal: &Palette) {
    b.mat(&pal.wood);
    let mut x = -15.0f32;
    while x <= 15.0 {
        b.mb().box_all(
            Vec3::new(x, CORRIDOR_CEIL - 0.20, -20.0),
            Vec3::new(0.26, 0.34, 3.7),
            1.4,
        );
        x += 3.0;
    }

    b.mat(&pal.wood_h);
    // skirting boards, only on the door side: the window side has sills
    b.mb().box_all(
        Vec3::new(0.0, F1 + 0.11, CORRIDOR_DOOR_WALL - 0.06),
        Vec3::new(31.0, 0.22, 0.12),
        1.6,
    );
    b.mb().box_all(
        Vec3::new(0.0, F1 + 0.11, CORRIDOR_WINDOW_WALL + 0.06),
        Vec3::new(31.0, 0.22, 0.12),
        1.6,
    );
    // crown
    b.mb().box_all(
        Vec3::new(0.0, CORRIDOR_CEIL - 0.62, CORRIDOR_DOOR_WALL - 0.09),
        Vec3::new(31.0, 0.20, 0.18),
        1.2,
    );

    // pilasters between the room doors, leaving the real door at x 4.5..10.5
    b.mat(&pal.wallpaper_b);
    for px in [
        -14.2f32, -11.6, -9.0, -6.4, -3.8, -1.2, 1.4, 3.8, 11.6, 14.2,
    ] {
        b.mb().box_all(
            Vec3::new(px, F1 + 1.85, CORRIDOR_DOOR_WALL - 0.08),
            Vec3::new(0.34, 3.7, 0.16),
            1.0,
        );
    }

    // framed pictures between the pilasters
    for (px, big) in [(-12.9f32, false), (-7.7, true), (-2.5, false), (0.1, true), (12.9, false)] {
        let (w, h) = if big { (1.10, 1.45) } else { (0.80, 1.05) };
        b.mat(&pal.wood_h);
        b.mb().box_all(
            Vec3::new(px, F1 + 2.05, CORRIDOR_DOOR_WALL - 0.05),
            Vec3::new(w, h, 0.10),
            1.0,
        );
        b.mat(&pal.painting);
        b.mb().box_all(
            Vec3::new(px, F1 + 2.05, CORRIDOR_DOOR_WALL - 0.12),
            Vec3::new(w - 0.16, h - 0.16, 0.03),
            1.0,
        );
    }
}

/// Mirrored lounge island on the empty west side of the lobby.
fn west_lounge(commands: &mut Commands, meshes: &mut Assets<Mesh>, pal: &Palette) {
    use std::f32::consts::FRAC_PI_2;

    props::rug(
        commands,
        meshes,
        pal,
        Vec3::new(-8.8, 0.004, 2.0),
        Vec2::new(5.2, 4.4),
        0.0,
    );
    props::sofa(commands, meshes, pal, Vec3::new(-11.1, GROUND, 2.0), FRAC_PI_2);
    props::armchair(commands, meshes, pal, Vec3::new(-6.1, GROUND, 0.1), -FRAC_PI_2);
    props::armchair(commands, meshes, pal, Vec3::new(-6.1, GROUND, 3.9), -FRAC_PI_2);
    props::coffee_table(commands, meshes, pal, Vec3::new(-8.7, GROUND, 2.0), FRAC_PI_2);
    props::fallen_chair(commands, meshes, pal, Vec3::new(-9.4, GROUND, 4.6), 1.1);

    // two more lamps so the corner is not a black hole
    for (z, dying) in [(0.4f32, false), (4.6, true)] {
        props::sconce(
            commands,
            meshes,
            pal,
            Vec3::new(-11.66, 3.35, z),
            FRAC_PI_2,
            true,
            dying,
        );
    }

    // and a plant where the wall turns
    props::potted_plant(commands, meshes, pal, Vec3::new(-11.0, GROUND, -6.4), 1.55, 1);
}
