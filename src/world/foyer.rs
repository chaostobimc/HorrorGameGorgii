//! Ground floor: the rain porch, the lobby, the reception, the stair hall.

use bevy::prelude::*;

use crate::core::{GameAssets, Rng};
use crate::interact::{Action, Interactable};
use crate::palette::Palette;
use crate::world::{
    props, wall_x_win, wall_z_win, BuildCtx, CorridorDoor, DoorBlocker, EntryDoor, SafeDoor,
    Swing, F1, GROUND,
};

pub fn build(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    porch(commands, b, pal, meshes, assets, rng);
    lobby(commands, b, pal, meshes, assets, rng);
    reception(commands, b, pal, meshes, assets, rng);
    stair_hall(commands, b, pal, meshes, assets, rng);
}

// ---------------------------------------------------------------------------
// the porch: where the night starts
// ---------------------------------------------------------------------------

fn porch(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    // stone deck
    b.mat(&pal.stone);
    b.slab_solid(-9.0, 10.0, 9.0, 20.4, GROUND, 0.6, 2.0);
    b.ceiling_solid(-9.0, 10.0, 9.0, 20.4, 4.75, 0.45, 2.0);

    // side walls, open to the jungle
    wall_x_win(
        b,
        10.0,
        20.4,
        GROUND,
        4.75,
        -9.2,
        0.45,
        2.0,
        &[(13.6, 18.4, 0.0, 3.5)],
    );
    wall_x_win(
        b,
        10.0,
        20.4,
        GROUND,
        4.75,
        9.2,
        0.45,
        2.0,
        &[(13.6, 18.4, 0.0, 3.5)],
    );

    // roof beams and a deep shadow under the canopy
    b.mat(&pal.wood);
    for z in [11.4, 14.0, 16.6, 19.2] {
        b.mb().box_all(Vec3::new(0.0, 4.44, z), Vec3::new(18.4, 0.26, 0.22), 1.2);
    }
    b.mat(&pal.trim);
    b.mb().box_all(Vec3::new(0.0, 4.6, 10.4), Vec3::new(18.6, 0.3, 0.35), 1.2);

    // balustrade at the jungle edge
    b.mat(&pal.stone);
    b.mb().box_all(Vec3::new(0.0, 0.55, 20.2), Vec3::new(18.4, 1.1, 0.35), 1.6);
    b.solid(Vec3::new(0.0, 0.55, 20.2), Vec3::new(18.4, 1.1, 0.35));
    b.mat(&pal.stone);
    b.slab_solid(-9.0, 20.4, 9.0, 20.9, 1.1, 0.2, 1.4);

    // columns
    for x in [-7.6f32, 7.6] {
        for z in [12.2f32, 18.6] {
            b.mat(&pal.stone);
            b.mb().box_all(Vec3::new(x, 0.22, z), Vec3::new(1.0, 0.44, 1.0), 1.2);
            b.mb()
                .cylinder(Vec3::new(x, 2.4, z), 0.4, 3.9, 10, 1.6, false);
            b.mb().box_all(Vec3::new(x, 4.5, z), Vec3::new(0.92, 0.3, 0.92), 1.2);
            b.solid(Vec3::new(x, 2.4, z), Vec3::new(0.8, 4.8, 0.8));
        }
    }

    // the entrance: frame, two leaves that will slam shut behind us
    b.mat(&pal.trim);
    b.box_at(Vec3::new(-2.35, 1.9, 9.9), Vec3::new(0.36, 3.9, 0.62), 1.2);
    b.box_at(Vec3::new(2.35, 1.9, 9.9), Vec3::new(0.36, 3.9, 0.62), 1.2);
    b.box_at(Vec3::new(0.0, 3.62, 9.9), Vec3::new(5.1, 0.42, 0.62), 1.2);
    b.mat(&pal.wood_h);
    b.box_at(Vec3::new(0.0, 0.06, 9.9), Vec3::new(4.6, 0.06, 0.7), 1.2);

    let leaf = props::mesh_of(meshes, |mb| {
        mb.box_all(Vec3::new(1.1, 1.7, 0.0), Vec3::new(2.2, 3.4, 0.09), 1.4);
        for y in [1.0f32, 2.5] {
            mb.box_all(
                Vec3::new(1.1, y, 0.06),
                Vec3::new(1.7, 1.25, 0.015),
                1.0,
            );
        }
        mb.cylinder(Vec3::new(2.0, 1.62, 0.09), 0.03, 0.09, 10, 1.0, true);
    });
    // left leaf: mesh runs from its hinge (+X), swings out to the porch
    let left = commands
        .spawn((
            Mesh3d(leaf.clone()),
            MeshMaterial3d(pal.wood_h.clone()),
            Transform::from_translation(Vec3::new(-2.2, 0.0, 9.9))
                .with_rotation(Quat::from_rotation_y(-1.85)),
            Swing::new(
                Quat::from_rotation_y(-1.85),
                Quat::IDENTITY,
                0.85,
            ),
            EntryDoor { left: true },
        ))
        .id();
    let _ = left;
    // right leaf: mirrored, mesh runs from its hinge towards -X
    let leaf_r = props::mesh_of(meshes, |mb| {
        mb.box_all(Vec3::new(-1.1, 1.7, 0.0), Vec3::new(2.2, 3.4, 0.09), 1.4);
        for y in [1.0f32, 2.5] {
            mb.box_all(
                Vec3::new(-1.1, y, 0.06),
                Vec3::new(1.7, 1.25, 0.015),
                1.0,
            );
        }
        mb.cylinder(Vec3::new(-2.0, 1.62, 0.09), 0.03, 0.09, 10, 1.0, true);
    });
    commands.spawn((
        Mesh3d(leaf_r),
        MeshMaterial3d(pal.wood_h.clone()),
        Transform::from_translation(Vec3::new(2.2, 0.0, 9.9))
            .with_rotation(Quat::from_rotation_y(1.85)),
        Swing::new(Quat::from_rotation_y(1.85), Quat::IDENTITY, 0.85),
        EntryDoor { left: false },
    ));

    // brass name plate over the door
    props::text3d(
        commands,
        assets.fonts.title.clone(),
        Vec3::new(0.0, 3.98, 10.12),
        0.0,
        0.44,
        "THE DIPLOMAT PEAK",
        Color::srgb(0.86, 0.74, 0.44),
    );
    props::text3d(
        commands,
        assets.fonts.ui.clone(),
        Vec3::new(0.0, 3.52, 10.12),
        0.0,
        0.15,
        "BAGUIO  ·  EST. 1961",
        Color::srgba(0.78, 0.72, 0.58, 0.85),
    );

    // jungle creeping onto the deck
    props::potted_plant(commands, meshes, pal, Vec3::new(-4.4, 0.0, 11.0), 1.35, 2);
    props::potted_plant(commands, meshes, pal, Vec3::new(4.4, 0.0, 11.0), 1.25, 1);
    props::potted_plant(commands, meshes, pal, Vec3::new(-8.2, 0.0, 16.4), 1.6, 0);
    props::potted_plant(commands, meshes, pal, Vec3::new(8.2, 0.0, 16.8), 1.45, 2);
    props::potted_plant(commands, meshes, pal, Vec3::new(-7.4, 0.0, 19.4), 1.2, 1);
    for i in 0..4 {
        let x = -7.0 + i as f32 * 4.6;
        props::hanging_vines(
            commands,
            meshes,
            pal,
            assets.tex.vine.clone(),
            Vec3::new(x, 4.55, 10.6 + (i as f32 * 1.3).sin().abs() * 6.0),
            4,
        );
    }

    // one dying lamp above the door - the only light out here
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.72, 0.42),
            intensity: 1_400.0,
            range: 9.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 4.1, 10.9)),
        crate::world::Flicker {
            base: 1_400.0,
            amp: 0.5,
            speed: 3.3,
            phase: 0.9,
            dying: true,
        },
    ));

    let _ = rng.f32();
}

// ---------------------------------------------------------------------------
// the lobby
// ---------------------------------------------------------------------------

fn lobby(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    // floor + ceiling
    b.mat(&pal.marble);
    b.slab_solid(-12.0, -10.0, 12.0, 10.0, GROUND, 0.5, 2.6);
    b.mat(&pal.carpet);
    b.floor(-1.7, -9.6, 1.7, 9.9, 0.008, 1.5);
    b.mat(&pal.ceiling);
    b.ceiling_solid(-12.0, -10.0, 12.0, 10.0, 7.0, 0.4, 2.4);

    // shell
    b.mat(&pal.wallpaper_b);
    wall_z_win(
        b,
        -12.0,
        12.0,
        GROUND,
        7.0,
        -10.0,
        0.5,
        2.2,
        &[(5.5, 8.5, 0.0, 2.65)],
    );
    wall_z_win(
        b,
        -12.0,
        12.0,
        GROUND,
        7.0,
        10.0,
        0.5,
        2.2,
        &[
            (-2.3, 2.3, 0.0, 3.45),
            (-9.6, -6.4, 1.0, 4.45),
            (6.4, 9.6, 1.0, 4.45),
        ],
    );
    wall_x_win(
        b,
        -10.0,
        10.0,
        GROUND,
        7.0,
        12.0,
        0.5,
        2.2,
        &[(-6.0, -3.4, 1.0, 4.45), (3.4, 6.0, 1.0, 4.45)],
    );
    b.wall_x(-10.0, 10.0, GROUND, 7.0, -12.0, 0.5, 2.2, None);

    // wainscot along the west and east walls
    b.mat(&pal.wood_h);
    for (x, w) in [(-11.72f32, 0.06f32), (11.72, 0.06)] {
        b.mb()
            .box_all(Vec3::new(x, 0.62, 0.0), Vec3::new(w, 1.24, 19.4), 1.2);
        b.mb()
            .box_all(Vec3::new(x, 1.29, 0.0), Vec3::new(w + 0.04, 0.09, 19.6), 1.0);
    }
    // skirting
    b.mat(&pal.trim);
    b.mb()
        .box_all(Vec3::new(0.0, 0.09, -9.72), Vec3::new(23.4, 0.18, 0.10), 1.0);
    b.mb()
        .box_all(Vec3::new(-11.72, 0.09, 0.0), Vec3::new(0.10, 0.18, 19.4), 1.0);
    b.mb()
        .box_all(Vec3::new(11.72, 0.09, 0.0), Vec3::new(0.10, 0.18, 19.4), 1.0);
    // cornice
    b.mb()
        .box_all(Vec3::new(0.0, 6.62, -9.70), Vec3::new(23.4, 0.34, 0.28), 1.0);
    b.mb()
        .box_all(Vec3::new(0.0, 6.62, 9.70), Vec3::new(23.4, 0.34, 0.28), 1.0);
    b.mb()
        .box_all(Vec3::new(-11.70, 6.62, 0.0), Vec3::new(0.28, 0.34, 19.4), 1.0);
    b.mb()
        .box_all(Vec3::new(11.70, 6.62, 0.0), Vec3::new(0.28, 0.34, 19.4), 1.0);

    // ceiling beams
    b.mat(&pal.wood);
    for x in [-7.5f32, 0.0, 7.5] {
        b.mb()
            .box_all(Vec3::new(x, 6.84, 0.0), Vec3::new(0.34, 0.3, 19.4), 1.2);
    }

    // columns
    for x in [-6.6f32, 6.6] {
        for z in [-6.2f32, 0.0, 6.2] {
            b.mat(&pal.marble);
            b.mb().box_all(Vec3::new(x, 0.18, z), Vec3::new(0.98, 0.36, 0.98), 1.0);
            b.mb()
                .cylinder(Vec3::new(x, 3.4, z), 0.42, 6.1, 12, 1.6, false);
            b.mb().box_all(Vec3::new(x, 6.5, z), Vec3::new(0.94, 0.28, 0.94), 1.0);
            b.solid(Vec3::new(x, 3.0, z), Vec3::new(0.78, 6.0, 0.78));
        }
    }

    // windows and the storm outside
    for (x, w) in [(-8.0f32, 3.2f32), (8.0, 3.2)] {
        props::window(commands, meshes, pal, Vec3::new(x, 2.72, 10.0), w, 3.4, 0.0);
    }
    for z in [-4.7f32, 4.7] {
        props::window(
            commands,
            meshes,
            pal,
            Vec3::new(12.0, 2.72, z),
            2.6,
            3.4,
            std::f32::consts::FRAC_PI_2,
        );
    }
    // jungle pressing against the east glass
    for i in 0..5 {
        let z = -7.5 + i as f32 * 3.6;
        props::potted_plant(commands, meshes, pal, Vec3::new(15.0, 0.0, z), 2.2, rng.index(2));
    }

    // the great chandelier, dying
    props::chandelier(commands, meshes, pal, Vec3::new(0.0, 5.35, 0.0), 1.7, true);

    // sconces
    for z in [-6.0f32, 4.0] {
        props::sconce(
            commands,
            meshes,
            pal,
            Vec3::new(-11.66, 3.35, z),
            std::f32::consts::FRAC_PI_2,
            true,
            true,
        );
    }
    for z in [-7.5f32, 1.5] {
        props::sconce(
            commands,
            meshes,
            pal,
            Vec3::new(11.66, 3.35, z),
            -std::f32::consts::FRAC_PI_2,
            z > 0.0,
            true,
        );
    }
    props::sconce(
        commands,
        meshes,
        pal,
        Vec3::new(-3.4, 3.35, -9.62),
        std::f32::consts::PI,
        true,
        false,
    );

    // lounge corner
    props::rug(
        commands,
        meshes,
        pal,
        Vec3::new(7.4, 0.0, 1.6),
        Vec2::new(6.2, 4.4),
        0.0,
    );
    props::sofa(
        commands,
        meshes,
        pal,
        Vec3::new(9.2, 0.0, 1.6),
        -std::f32::consts::FRAC_PI_2,
    );
    b.solid(Vec3::new(9.2, 0.5, 1.6), Vec3::new(0.95, 1.0, 2.2));
    props::armchair(
        commands,
        meshes,
        pal,
        Vec3::new(6.1, 0.0, 4.4),
        std::f32::consts::PI * 0.78,
    );
    b.solid(Vec3::new(6.1, 0.5, 4.4), Vec3::new(0.95, 1.0, 0.95));
    props::armchair(
        commands,
        meshes,
        pal,
        Vec3::new(6.1, 0.0, -1.0),
        std::f32::consts::PI * 0.22,
    );
    b.solid(Vec3::new(6.1, 0.5, -1.0), Vec3::new(0.95, 1.0, 0.95));
    props::coffee_table(commands, meshes, pal, Vec3::new(7.3, 0.0, 1.6), 0.0);
    b.solid(Vec3::new(7.3, 0.45, 1.6), Vec3::new(1.4, 0.45, 0.75));

    // a few plants in the corners
    props::potted_plant(commands, meshes, pal, Vec3::new(-11.0, 0.0, 8.6), 1.5, 1);
    props::potted_plant(commands, meshes, pal, Vec3::new(11.0, 0.0, 8.6), 1.4, 2);
    props::potted_plant(commands, meshes, pal, Vec3::new(-11.2, 0.0, -2.4), 1.25, 0);
    props::potted_plant(commands, meshes, pal, Vec3::new(11.2, 0.0, -8.6), 1.5, 1);

    // hanging vines from the beams
    props::hanging_vines(
        commands,
        meshes,
        pal,
        assets.tex.vine.clone(),
        Vec3::new(-9.6, 6.7, 7.5),
        5,
    );
    props::hanging_vines(
        commands,
        meshes,
        pal,
        assets.tex.vine.clone(),
        Vec3::new(9.4, 6.7, -6.5),
        4,
    );
    props::hanging_vines(
        commands,
        meshes,
        pal,
        assets.tex.vine.clone(),
        Vec3::new(3.0, 6.7, 8.8),
        3,
    );

    // two paintings, both ruined
    for (x, z, yaw) in [(-11.68f32, -7.0f32, std::f32::consts::FRAC_PI_2)] {
        b.mat(&pal.painting);
        b.box_at(Vec3::new(x, 3.6, z), Vec3::new(0.06, 1.6, 1.2), 1.0);
        b.mat(&pal.brass_dark);
        b.box_at(Vec3::new(x + 0.02, 3.6, z), Vec3::new(0.04, 1.72, 1.32), 1.0);
        let _ = yaw;
    }

    // the locked corridor door
    let door = props::door(
        commands,
        meshes,
        pal,
        Vec3::new(5.5, 0.0, -9.85),
        3.0,
        2.6,
        0.0,
        None,
        Some(assets.fonts.ui.clone()),
    );
    commands.entity(door).insert((
        CorridorDoor,
        Swing::new(Quat::IDENTITY, Quat::from_rotation_y(-1.62), 1.1),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(7.0, 1.3, -9.9)),
        Interactable::new(Action::CorridorDoor, "Tür aufschließen", 1.0),
    ));
    // invisible blocker so the closed door is solid
    commands.spawn((
        Transform::from_translation(Vec3::new(7.0, 1.3, -9.9)),
        crate::collision::SolidBox {
            half: Vec3::new(1.55, 1.35, 0.18),
        },
        DoorBlocker,
        Name::new("door_blocker_corridor"),
    ));

    // signage over the reception
    props::text3d(
        commands,
        assets.fonts.title.clone(),
        Vec3::new(-7.6, 3.55, -9.62),
        0.0,
        0.34,
        "REZEPTION",
        Color::srgb(0.80, 0.66, 0.38),
    );
    // stairs sign
    props::text3d(
        commands,
        assets.fonts.ui.clone(),
        Vec3::new(10.6, 3.2, -9.6),
        0.0,
        0.16,
        "ZIMMER 101 – 106",
        Color::srgba(0.72, 0.68, 0.58, 0.7),
    );
}

// ---------------------------------------------------------------------------
// reception desk, safe and the key cabinet
// ---------------------------------------------------------------------------

fn reception(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    // counter, top and the low cabinet against the wall behind it
    b.mat(&pal.wood);
    b.box_collide(Vec3::new(-7.6, 0.55, -8.0), Vec3::new(6.8, 1.1, 0.8), 1.6);
    b.mat(&pal.marble);
    b.box_at(Vec3::new(-7.6, 1.14, -8.0), Vec3::new(7.2, 0.09, 1.05), 1.6);
    b.mat(&pal.wood);
    b.box_collide(Vec3::new(-7.6, 0.45, -9.72), Vec3::new(6.6, 0.9, 0.3), 1.4);

    // key cabinet on the wall behind
    b.mat(&pal.wood_h);
    b.box_collide(Vec3::new(-7.6, 2.35, -9.66), Vec3::new(4.6, 1.7, 0.26), 1.2);
    b.mat(&pal.black);
    for i in 0..6 {
        let x = -9.7 + i as f32 * 0.84;
        b.mb()
            .box_all(Vec3::new(x, 2.35, -9.51), Vec3::new(0.03, 1.62, 0.05), 1.0);
    }
    for i in 0..4 {
        let y = 1.62 + i as f32 * 0.42;
        b.mb()
            .box_all(Vec3::new(-7.6, y, -9.51), Vec3::new(4.5, 0.025, 0.05), 1.0);
    }
    b.mat(&pal.brass_dark);
    for i in 0..18 {
        let x = -9.6 + (i % 6) as f32 * 0.84;
        let y = 1.72 + (i / 6) as f32 * 0.42;
        b.mb().box_all(Vec3::new(x, y, -9.48), Vec3::new(0.05, 0.14, 0.02), 1.0);
    }

    // wall clock above the cabinet
    b.mat(&pal.wood);
    b.box_at(Vec3::new(-7.6, 4.55, -9.62), Vec3::new(1.1, 1.1, 0.14), 1.0);
    let face = props::mesh_of(meshes, |mb| {
        mb.quad_uv(
            Vec3::new(-0.5, -0.5, 0.0),
            Vec3::new(0.5, -0.5, 0.0),
            Vec3::new(0.5, 0.5, 0.0),
            Vec3::new(-0.5, 0.5, 0.0),
            [
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 0.0),
            ],
        )
    });
    props::spawn_prop(
        commands,
        face,
        pal.clock_face.clone(),
        Transform::from_translation(Vec3::new(-7.6, 4.55, -9.54)),
    );
    commands.spawn((
        Transform::from_translation(Vec3::new(-7.6, 4.55, -9.35)),
        Interactable::new(Action::Flavor(3), "Uhr ansehen", 0.5),
    ));

    // the safe, in the corner where nobody walks
    b.mat(&pal.metal);
    b.box_collide(Vec3::new(-11.0, 0.55, -9.0), Vec3::new(1.1, 1.1, 0.72), 0.7);
    b.mat(&pal.brass_dark);
    b.box_at(Vec3::new(-11.0, 0.55, -8.66), Vec3::new(1.16, 1.16, 0.04), 0.6);
    // the door leaf slides sideways into the corner when the code is right
    let safe_leaf = props::mesh_of(meshes, |mb| {
        mb.box_all(Vec3::new(-11.0, 0.55, -8.63), Vec3::new(1.0, 1.0, 0.08), 0.6);
        mb.cylinder(Vec3::new(-10.62, 0.55, -8.55), 0.09, 0.06, 12, 1.0, true);
        mb.box_all(Vec3::new(-11.22, 0.92, -8.55), Vec3::new(0.36, 0.13, 0.02), 1.0);
    });
    commands.spawn((
        Mesh3d(safe_leaf),
        MeshMaterial3d(pal.metal.clone()),
        Transform::IDENTITY,
        crate::world::Slide::new(Vec3::ZERO, Vec3::new(-1.06, 0.0, 0.0), 1.7),
        SafeDoor,
    ));
    // keypad on the front of the safe
    b.mat(&pal.black_metal);
    b.box_at(
        Vec3::new(-10.55, 1.0, -8.56),
        Vec3::new(0.30, 0.36, 0.06),
        0.4,
    );
    b.mat(&pal.led_off);
    b.mb()
        .box_all(Vec3::new(-10.44, 1.13, -8.52), Vec3::new(0.05, 0.05, 0.02), 1.0);
    commands.spawn((
        Transform::from_translation(Vec3::new(-10.55, 1.0, -8.44)),
        Interactable::new(Action::SafeKeypad, "Tastatur benutzen", 0.4),
    ));

    // desk lamp
    b.mat(&pal.brass_dark);
    b.mb()
        .cylinder(Vec3::new(-5.05, 1.34, -8.15), 0.14, 0.06, 14, 1.0, true);
    b.mb()
        .cylinder(Vec3::new(-5.05, 1.62, -8.15), 0.025, 0.55, 8, 1.0, false);
    b.mb()
        .cylinder(Vec3::new(-5.05, 1.92, -8.15), 0.17, 0.22, 14, 1.0, false);
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.74, 0.46),
            intensity: 1_100.0,
            range: 6.5,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::new(-5.05, 1.86, -8.1)),
        crate::world::Flicker {
            base: 1_100.0,
            amp: 0.06,
            speed: 0.9,
            phase: 0.2,
            dying: false,
        },
    ));

    // guest book
    b.mat(&pal.paper);
    b.box_at(Vec3::new(-8.75, 1.22, -7.9), Vec3::new(0.52, 0.05, 0.38), 0.5);
    b.mat(&pal.leather);
    b.box_at(Vec3::new(-8.75, 1.19, -7.9), Vec3::new(0.56, 0.03, 0.42), 0.5);
    commands.spawn((
        Transform::from_translation(Vec3::new(-8.75, 1.3, -7.9)),
        Interactable::new(Action::Flavor(1), "Gästebuch lesen", 0.45),
    ));

    // newspaper, the first real clue
    b.mat(&pal.paper);
    b.box_at(Vec3::new(-6.5, 1.2, -7.95), Vec3::new(0.40, 0.03, 0.55), 0.5);
    commands.spawn((
        Transform::from_translation(Vec3::new(-6.5, 1.28, -7.95)),
        Interactable::new(Action::ReadDeskArticle, "Zeitung lesen", 0.45),
    ));

    // brass bell
    b.mat(&pal.brass);
    b.mb()
        .cylinder(Vec3::new(-9.55, 1.24, -8.0), 0.11, 0.14, 14, 1.0, true);
    b.mb()
        .cylinder(Vec3::new(-9.55, 1.34, -8.0), 0.03, 0.1, 8, 1.0, false);
    commands.spawn((
        Transform::from_translation(Vec3::new(-9.55, 1.3, -8.0)),
        Interactable::new(Action::Flavor(2), "Glocke läuten", 0.4),
    ));

    // a chair, a ledger and the old telephone
    props::chair(commands, meshes, pal, Vec3::new(-3.9, 0.0, -8.9), 1.9);
    b.mat(&pal.leather);
    b.box_at(Vec3::new(-9.2, 1.25, -8.1), Vec3::new(0.42, 0.14, 0.32), 0.5);
    b.mat(&pal.black_metal);
    b.box_at(Vec3::new(-4.85, 1.2, -8.1), Vec3::new(0.26, 0.14, 0.2), 0.4);
    b.mb().bar(
        Vec3::new(-4.85, 1.27, -8.1),
        Vec3::new(-4.72, 1.36, -8.1),
        0.05,
        1.0,
    );

    let _ = rng.f32();
    let _ = assets;
}

// ---------------------------------------------------------------------------
// stair hall behind the locked door
// ---------------------------------------------------------------------------

fn stair_hall(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    _rng: &mut Rng,
) {
    b.mat(&pal.wood_floor);
    b.slab_solid(4.0, -18.0, 11.0, -10.0, GROUND, 0.45, 2.0);
    b.mat(&pal.plaster[1]);
    b.ceiling_solid(4.0, -18.0, 11.0, -10.0, 7.0, 0.35, 2.2);

    b.mat(&pal.plaster[0]);
    b.wall_x(-18.0, -10.0, GROUND, 7.0, 4.0, 0.45, 2.2, None);
    b.wall_x(-18.0, -10.0, GROUND, 7.0, 11.0, 0.45, 2.2, None);
    // below the landing; above it the corridor wall takes over
    b.wall_z(4.0, 11.0, GROUND, F1, -18.0, 0.45, 2.2, None);

    // the stair itself
    b.mat(&pal.wood_floor);
    b.stairs(4.6, 10.4, -11.0, -17.0, GROUND, F1, 24, 1.8);
    b.slab_solid(4.5, -18.0, 10.5, -17.0, F1, 0.45, 1.8);

    // handrail, following the flight
    b.mat(&pal.brass_dark);
    let steps = 12;
    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let p0 = Vec3::new(4.78, GROUND + (F1 - GROUND) * t0 + 0.98, -11.0 - 6.0 * t0);
        let p1 = Vec3::new(4.78, GROUND + (F1 - GROUND) * t1 + 0.98, -11.0 - 6.0 * t1);
        b.mb().bar(p0, p1, 0.035, 1.0);
        b.mb().cylinder(
            Vec3::new(4.78, p0.y - 0.5, p0.z),
            0.02,
            1.0,
            6,
            1.0,
            false,
        );
    }
    b.mb().box_all(Vec3::new(4.78, 0.55, -10.7), Vec3::new(0.14, 1.1, 0.14), 1.0);

    // a bare bulb in the stairwell, one sconce at the bottom
    props::sconce(
        commands,
        meshes,
        pal,
        Vec3::new(4.24, 3.1, -12.6),
        std::f32::consts::FRAC_PI_2,
        true,
        true,
    );
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.8, 0.58),
            intensity: 900.0,
            range: 8.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(Vec3::new(7.5, 6.2, -16.0)),
        crate::world::Flicker {
            base: 900.0,
            amp: 0.55,
            speed: 4.1,
            phase: 2.2,
            dying: true,
        },
    ));

    // crates and dust in the corner
    b.mat(&pal.wood);
    b.box_collide(Vec3::new(5.2, 0.4, -17.3), Vec3::new(1.1, 0.8, 0.9), 1.0);
    b.box_collide(Vec3::new(5.9, 0.3, -16.4), Vec3::new(0.8, 0.6, 0.7), 1.0);
    props::potted_plant(commands, meshes, pal, Vec3::new(10.3, 0.0, -10.7), 1.1, 2);

    // "1. STOCK" on the landing wall
    props::text3d(
        commands,
        assets.fonts.ui.clone(),
        Vec3::new(7.5, 5.3, -17.72),
        0.0,
        0.24,
        "1. STOCK",
        Color::srgba(0.76, 0.70, 0.56, 0.75),
    );
}
