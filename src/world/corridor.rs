//! First floor: the long corridor, the elevator shaft and the machine room.

use bevy::prelude::*;

use crate::core::{GameAssets, Rng};
use crate::interact::{Action, Interactable};
use crate::palette::Palette;
use crate::world::{
    props, wall_x_win, wall_z_win, BuildCtx, LeverHandle, Swing, F1,
};

const CEIL: f32 = 7.9;

pub fn build(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    corridor_shell(commands, b, pal, meshes, assets, rng);
    elevator(commands, b, pal, meshes, assets);
    machine_room(commands, b, pal, meshes, assets, rng);
}

fn corridor_shell(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    // floor, runner, ceiling
    b.mat(&pal.wood_floor);
    b.slab_solid(-16.0, -22.0, 16.0, -18.0, F1, 0.45, 2.0);
    b.mat(&pal.carpet);
    b.floor(-15.2, -20.05, 15.2, -18.55, F1 + 0.008, 1.4);
    b.mat(&pal.plaster[0]);
    b.ceiling(-16.0, -22.0, 16.0, -18.0, CEIL, 2.2);

    // south wall: six guest doors and the stairwell opening
    let mut doors = Vec::new();
    for (i, x) in [-13.5f32, -10.0, -6.5, -3.0, 0.5, 3.4].iter().enumerate() {
        doors.push((*x - 0.62, *x + 0.62, F1, F1 + 2.15));
        let _ = i;
    }
    doors.push((4.5, 10.5, F1, CEIL));
    b.mat(&pal.wallpaper);
    wall_z_win(b, -16.0, 16.0, F1, CEIL, -18.0, 0.35, 2.0, &doors);

    // north wall: tall windows onto the storm
    b.mat(&pal.plaster[2]);
    let mut windows = Vec::new();
    for x in [-11.5f32, -5.5, 0.5, 6.5, 12.5] {
        windows.push((x - 1.15, x + 1.15, F1 + 0.95, F1 + 2.95));
    }
    wall_z_win(b, -16.0, 16.0, F1, CEIL, -22.0, 0.4, 2.0, &windows);

    // east wall towards the machine room
    b.mat(&pal.plaster[2]);
    wall_x_win(
        b,
        -22.0,
        -18.0,
        F1,
        CEIL,
        16.0,
        0.4,
        2.0,
        &[(-21.4, -18.8, F1, F1 + 2.3)],
    );

    // skirting, picture rail
    b.mat(&pal.trim);
    b.mb().box_all(
        Vec3::new(0.0, F1 + 0.07, -18.22),
        Vec3::new(31.4, 0.14, 0.07),
        1.0,
    );
    b.mb().box_all(
        Vec3::new(0.0, F1 + 2.55, -18.22),
        Vec3::new(31.4, 0.06, 0.05),
        1.0,
    );

    // guest doors, numbers facing the corridor
    for (i, x) in [-13.5f32, -10.0, -6.5, -3.0, 0.5, 3.4].iter().enumerate() {
        props::door(
            commands,
            meshes,
            pal,
            Vec3::new(*x - 0.55, F1, -18.0),
            1.1,
            2.1,
            0.0,
            None,
            None,
        );
        props::text3d(
            commands,
            assets.fonts.ui.clone(),
            Vec3::new(*x, F1 + 2.36, -18.22),
            std::f32::consts::PI,
            0.13,
            &format!("10{}", i + 1),
            Color::srgba(0.80, 0.76, 0.66, 0.85),
        );
        b.solid(
            Vec3::new(*x, F1 + 1.05, -18.0),
            Vec3::new(0.62, 1.08, 0.12),
        );
    }

    // windows
    for x in [-11.5f32, -5.5, 0.5, 6.5, 12.5] {
        props::window(
            commands,
            meshes,
            pal,
            Vec3::new(x, F1 + 1.95, -22.0),
            2.3,
            2.0,
            0.0,
        );
    }
    // jungle outside the glass
    for x in [-13.0f32, -6.0, 3.0, 11.0] {
        props::potted_plant(commands, meshes, pal, Vec3::new(x, F1, -24.2), 2.4, rng.index(2));
    }

    // light: five sconces, half of them dead
    for (i, x) in [-12.0f32, -6.0, 0.0, 6.0, 12.0].iter().enumerate() {
        props::sconce(
            commands,
            meshes,
            pal,
            Vec3::new(*x, F1 + 2.45, -18.28),
            std::f32::consts::PI,
            i % 3 != 1,
            i != 2,
        );
    }

    // ceiling beams + a collapsed pipe
    b.mat(&pal.trim);
    for x in [-8.0f32, 0.0, 8.0] {
        b.mb().box_all(
            Vec3::new(x, CEIL - 0.12, -20.0),
            Vec3::new(0.24, 0.24, 3.9),
            1.0,
        );
    }
    b.mat(&pal.metal);
    b.mb().cylinder(
        Vec3::new(-14.0, CEIL - 0.35, -19.9),
        0.12,
        3.2,
        10,
        1.0,
        false,
    );

    // vines coming in through the windows
    for i in 0..4 {
        props::hanging_vines(
            commands,
            meshes,
            pal,
            assets.tex.vine.clone(),
            Vec3::new(-13.0 + i as f32 * 7.6, CEIL - 0.1, -21.3),
            4,
        );
    }

    // notice board with the second article
    b.mat(&pal.wood);
    b.box_at(
        Vec3::new(-14.2, F1 + 1.85, -18.24),
        Vec3::new(1.5, 1.1, 0.09),
        0.8,
    );
    b.mat(&pal.paper);
    b.box_at(
        Vec3::new(-14.35, F1 + 1.85, -18.18),
        Vec3::new(0.42, 0.56, 0.02),
        0.5,
    );
    b.box_at(
        Vec3::new(-13.85, F1 + 1.95, -18.18),
        Vec3::new(0.34, 0.42, 0.02),
        0.5,
    );
    b.box_at(
        Vec3::new(-14.2, F1 + 1.45, -18.18),
        Vec3::new(0.6, 0.2, 0.02),
        0.5,
    );
    commands.spawn((
        Transform::from_translation(Vec3::new(-14.25, F1 + 1.85, -18.1)),
        Interactable::new(Action::ReadCorridorArticle, "Zeitung lesen", 0.55),
    ));

    // dressing: luggage cart, cases, a plant, a chair
    b.mat(&pal.brass_dark);
    b.box_at(
        Vec3::new(-9.0, F1 + 0.55, -19.0),
        Vec3::new(1.6, 0.06, 0.9),
        1.0,
    );
    b.box_at(
        Vec3::new(-9.0, F1 + 0.95, -19.4),
        Vec3::new(1.6, 0.9, 0.05),
        1.0,
    );
    for (dx, dz) in [(-0.72f32, -0.4f32), (0.72, -0.4), (-0.72, 0.4), (0.72, 0.4)] {
        b.mb().cylinder(
            Vec3::new(-9.0 + dx, F1 + 0.12, -19.0 - dz),
            0.08,
            0.24,
            8,
            1.0,
            false,
        );
    }
    b.solid(
        Vec3::new(-9.0, F1 + 0.6, -19.0),
        Vec3::new(0.85, 0.6, 0.5),
    );
    b.mat(&pal.leather);
    b.box_collide(
        Vec3::new(-7.9, F1 + 0.2, -19.1),
        Vec3::new(0.7, 0.4, 0.45),
        1.0,
    );
    b.box_collide(
        Vec3::new(-7.2, F1 + 0.15, -18.9),
        Vec3::new(0.5, 0.3, 0.4),
        1.0,
    );
    props::potted_plant(commands, meshes, pal, Vec3::new(14.2, F1, -21.0), 1.2, 1);
    props::fallen_chair(commands, meshes, pal, Vec3::new(-4.2, F1, -19.6), 0.7);

    let _ = rng.f32();
}

// ---------------------------------------------------------------------------
// the elevator
// ---------------------------------------------------------------------------

fn elevator(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
) {
    // west wall of the corridor with the shaft opening
    b.mat(&pal.plaster[1]);
    wall_x_win(
        b,
        -22.0,
        -18.0,
        F1,
        CEIL,
        -16.0,
        0.35,
        2.0,
        &[(-21.2, -19.2, F1, F1 + 2.2)],
    );

    // shaft
    // the shaft runs all the way up to the penthouse landing
    let top = crate::world::F2 + 2.6;
    b.mat(&pal.stone);
    b.slab_solid(-20.4, -22.0, -16.2, -18.0, F1, 0.4, 1.6);
    b.ceiling_solid(-20.4, -22.0, -16.2, -18.0, top, 0.3, 1.6);
    b.mat(&pal.brick);
    b.wall_x(-22.0, -18.0, F1, top, -20.4, 0.35, 1.6, None);
    b.wall_z(-20.4, -16.2, F1, top, -22.0, 0.35, 1.6, None);
    b.wall_z(-20.4, -16.2, F1, top, -18.0, 0.35, 1.6, None);

    // brass cage: uprights, top rail, and a folding grille
    b.mat(&pal.brass_dark);
    for x in [-20.2f32, -16.4] {
        for z in [-21.8f32, -18.2] {
            b.mb()
                .cylinder(Vec3::new(x, F1 + 1.3, z), 0.05, 2.6, 8, 1.0, true);
        }
    }
    b.mb().bar(
        Vec3::new(-20.2, F1 + 2.55, -21.8),
        Vec3::new(-16.4, F1 + 2.55, -21.8),
        0.06,
        1.0,
    );
    b.mb().bar(
        Vec3::new(-20.2, F1 + 2.55, -18.2),
        Vec3::new(-16.4, F1 + 2.55, -18.2),
        0.06,
        1.0,
    );
    // grille across the open side
    for i in 0..10 {
        let z = -21.0 + i as f32 * 0.2;
        b.mb().bar(
            Vec3::new(-16.35, F1 + 0.05, z),
            Vec3::new(-16.35, F1 + 2.35, z),
            0.022,
            1.0,
        );
    }
    for y in [F1 + 0.5f32, F1 + 1.2, F1 + 1.9] {
        b.mb().bar(
            Vec3::new(-16.35, y, -21.1),
            Vec3::new(-16.35, y, -19.0),
            0.03,
            1.0,
        );
    }

    // car interior: bench, handrail, mirror, brass floor lip
    b.mat(&pal.velvet_green);
    b.box_at(
        Vec3::new(-18.4, F1 + 0.42, -21.6),
        Vec3::new(3.0, 0.12, 0.5),
        0.8,
    );
    b.mat(&pal.brass);
    b.mb().bar(
        Vec3::new(-20.1, F1 + 1.05, -18.4),
        Vec3::new(-16.6, F1 + 1.05, -18.4),
        0.035,
        1.0,
    );
    b.mat(&pal.ivory);
    b.box_at(
        Vec3::new(-19.9, F1 + 1.55, -20.0),
        Vec3::new(0.06, 1.1, 1.6),
        1.0,
    );
    b.mat(&pal.brass_dark);
    b.box_at(
        Vec3::new(-16.5, F1 + 0.03, -20.2),
        Vec3::new(0.3, 0.06, 1.9),
        1.0,
    );

    // call button
    b.mat(&pal.brass);
    b.box_at(
        Vec3::new(-15.78, F1 + 1.3, -18.95),
        Vec3::new(0.1, 0.34, 0.2),
        0.4,
    );
    b.mat(&pal.led_off);
    b.mb().cylinder(
        Vec3::new(-15.72, F1 + 1.34, -18.95),
        0.05,
        0.05,
        10,
        1.0,
        true,
    );
    commands.spawn((
        Transform::from_translation(Vec3::new(-15.6, F1 + 1.3, -18.95)),
        Interactable::new(Action::ElevatorCall, "Aufzug rufen", 0.5),
    ));

    // floor indicator above the opening
    b.mat(&pal.black_metal);
    b.box_at(
        Vec3::new(-15.74, F1 + 2.42, -20.2),
        Vec3::new(0.08, 0.26, 0.9),
        0.4,
    );
    props::text3d(
        commands,
        assets.fonts.ui.clone(),
        Vec3::new(-15.66, F1 + 2.42, -20.2),
        -std::f32::consts::FRAC_PI_2,
        0.16,
        "2",
        Color::srgba(0.9, 0.55, 0.2, 0.55),
    );

    // one lamp inside the car
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.84, 0.6),
            intensity: 1_600.0,
            range: 7.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(Vec3::new(-18.3, F1 + 2.4, -20.0)),
        crate::world::Flicker {
            base: 1_600.0,
            amp: 0.3,
            speed: 1.1,
            phase: 0.4,
            dying: true,
        },
    ));
}

// ---------------------------------------------------------------------------
// machine room with the breaker panel
// ---------------------------------------------------------------------------

fn machine_room(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    let mc = F1 + 2.7;
    b.mat(&pal.concrete);
    b.slab_solid(16.0, -22.0, 23.0, -17.0, F1, 0.4, 2.0);
    b.ceiling(16.0, -22.0, 23.0, -17.0, mc, 2.2);
    b.wall_x(-22.0, -17.0, F1, mc, 23.0, 0.4, 2.0, None);
    b.wall_z(16.0, 23.0, F1, mc, -17.0, 0.4, 2.0, None);
    b.wall_z(16.0, 23.0, F1, mc, -22.0, 0.4, 2.0, None);
    b.wall_x(-18.0, -17.0, F1, mc, 16.0, 0.4, 2.0, None);

    // the generator itself
    b.mat(&pal.metal);
    b.box_collide(
        Vec3::new(19.2, F1 + 0.75, -19.6),
        Vec3::new(2.6, 1.5, 1.5),
        1.2,
    );
    b.box_at(
        Vec3::new(19.2, F1 + 1.62, -19.6),
        Vec3::new(2.9, 0.24, 1.7),
        1.0,
    );
    b.mat(&pal.paint_red);
    b.box_at(
        Vec3::new(19.2, F1 + 0.68, -18.82),
        Vec3::new(1.1, 0.3, 0.06),
        1.0,
    );
    b.mat(&pal.black_metal);
    b.mb().cylinder(
        Vec3::new(17.7, F1 + 0.7, -19.6),
        0.55,
        1.5,
        18,
        1.2,
        true,
    );
    b.mb().cylinder(
        Vec3::new(17.7, F1 + 0.7, -19.6),
        0.62,
        0.12,
        18,
        1.0,
        true,
    );
    // exhaust pipe up through the ceiling
    b.mat(&pal.metal);
    b.mb().cylinder(
        Vec3::new(20.4, F1 + 1.9, -20.2),
        0.16,
        2.4,
        10,
        1.0,
        false,
    );
    b.mb().cylinder(
        Vec3::new(20.4, mc - 0.1, -20.2),
        0.24,
        0.24,
        10,
        1.0,
        false,
    );

    // pipes along the east wall
    for (i, y) in [1.1f32, 1.45, 1.8].iter().enumerate() {
        let r = 0.07 + i as f32 * 0.015;
        b.mb().cylinder(
            Vec3::new(22.6, F1 + y, -19.5),
            r,
            4.6,
            8,
            1.0,
            false,
        );
    }

    // breaker panel
    b.mat(&pal.black_metal);
    b.box_collide(
        Vec3::new(22.7, F1 + 1.75, -19.7),
        Vec3::new(0.24, 1.5, 2.7),
        0.8,
    );
    b.mat(&pal.metal);
    b.box_at(
        Vec3::new(22.55, F1 + 1.75, -19.7),
        Vec3::new(0.08, 1.34, 2.5),
        0.8,
    );

    let cols = [
        (&pal.paint_red, "HAUPTSCHALTER"),
        (&pal.paint_blue, "SICHERUNG"),
        (&pal.paint_green, "HILFSAGGREGAT"),
        (&pal.paint_yellow, "ERDUNG"),
    ];
    for (i, (colour, label)) in cols.iter().enumerate() {
        let z = -20.75 + i as f32 * 0.7;
        b.mat(colour);
        b.box_at(
            Vec3::new(22.5, F1 + 2.24, z),
            Vec3::new(0.06, 0.26, 0.5),
            1.0,
        );
        b.mat(&pal.brass_dark);
        b.box_at(
            Vec3::new(22.49, F1 + 1.86, z),
            Vec3::new(0.05, 0.12, 0.28),
            0.6,
        );
        // the lever handle: pivots at the plate, points down
        let handle = props::mesh_of(meshes, |mb| {
            mb.box_all(Vec3::new(0.0, -0.16, 0.0), Vec3::new(0.05, 0.32, 0.05), 1.0);
            mb.cylinder(Vec3::new(0.0, -0.32, 0.0), 0.045, 0.10, 10, 1.0, true);
        });
        commands.spawn((
            Mesh3d(handle),
            MeshMaterial3d(if i % 2 == 0 {
                pal.brass.clone()
            } else {
                pal.black_metal.clone()
            }),
            Transform::from_translation(Vec3::new(22.44, F1 + 1.9, z)),
            Swing::new(
                Quat::IDENTITY,
                Quat::from_rotation_x(if i == 3 { 1.15 } else { -1.15 }),
                0.35,
            ),
            LeverHandle(i),
        ));
        commands.spawn((
            Transform::from_translation(Vec3::new(22.3, F1 + 1.9, z)),
            Interactable::new(Action::Lever(i as u8), "Hebel umlegen", 0.44),
        ));
        let rot = -std::f32::consts::FRAC_PI_2;
        props::text3d(
            commands,
            assets.fonts.ui.clone(),
            Vec3::new(22.46, F1 + 2.58, z),
            rot,
            0.1,
            label,
            Color::srgba(0.86, 0.84, 0.78, 0.8),
        );
    }

    // gauges
    b.mat(&pal.brass);
    for z in [-21.6f32, -18.0] {
        b.mb().cylinder(
            Vec3::new(22.62, F1 + 2.2, z),
            0.16,
            0.06,
            14,
            1.0,
            true,
        );
    }

    // workbench, drums, crates
    b.mat(&pal.wood);
    b.box_collide(
        Vec3::new(18.2, F1 + 0.45, -21.3),
        Vec3::new(2.6, 0.1, 0.8),
        1.2,
    );
    for x in [17.1f32, 19.3] {
        b.mb().cylinder(
            Vec3::new(x, F1 + 0.22, -21.3),
            0.05,
            0.44,
            8,
            1.0,
            false,
        );
    }
    b.mat(&pal.paint_green);
    for (x, z) in [(21.4f32, -21.2f32), (21.9, -20.4)] {
        b.box_collide(
            Vec3::new(x, F1 + 0.45, z),
            Vec3::new(0.6, 0.9, 0.6),
            1.0,
        );
    }
    b.mat(&pal.wood);
    b.box_collide(
        Vec3::new(16.9, F1 + 0.3, -17.7),
        Vec3::new(0.7, 0.6, 0.6),
        1.0,
    );

    // cables hanging from the ceiling
    b.mat(&pal.black);
    for i in 0..5 {
        let z = -21.4 + i as f32 * 0.9;
        b.mb().bar(
            Vec3::new(16.6, mc - 0.05, z),
            Vec3::new(17.2, mc - 1.1, z + 0.3),
            0.022,
            1.0,
        );
    }

    // two ugly fluorescent tubes
    props::fluor_tube(commands, meshes, pal, Vec3::new(18.6, mc - 0.16, -19.4), true);
    props::fluor_tube(commands, meshes, pal, Vec3::new(21.2, mc - 0.16, -20.6), false);

    // an open steel door leaning into the corridor
    b.mat(&pal.metal);
    b.box_at(
        Vec3::new(16.6, F1 + 1.1, -18.55),
        Vec3::new(0.08, 2.2, 1.05),
        1.0,
    );

    let _ = rng.f32();
}
