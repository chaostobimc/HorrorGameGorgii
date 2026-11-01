//! Second floor: the ballroom, the grandfather clock, the roof terrace and
//! the gate corridor where the night ends.

use bevy::prelude::*;

use crate::core::{GameAssets, Rng};
use crate::interact::{Action, Interactable};
use crate::palette::Palette;
use crate::world::{
    props, wall_x_win, wall_z_win, BuildCtx, ClockHand, ExitGate, Flicker, GateBlocker,
    GateLed, JumpscareTrigger, SecretBlocker, SecretDoor, Slide, F2,
};

const CEIL: f32 = F2 + 6.0;
const CLOCK_POS: Vec3 = Vec3::new(5.05, F2, 0.9);

pub fn build(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    ballroom(commands, b, pal, meshes, assets, rng);
    clock_and_secret(commands, b, pal, meshes, assets);
    terrace(commands, b, pal, meshes, assets, rng);
    gate_corridor(commands, b, pal, meshes, assets);
}

// ---------------------------------------------------------------------------
// ballroom
// ---------------------------------------------------------------------------

fn ballroom(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    // shell
    b.mat(&pal.wood_floor);
    b.slab_solid(-16.0, -22.0, 6.0, 4.0, F2, 0.5, 2.2);
    b.mat(&pal.carpet);
    b.floor(-11.5, -16.0, 1.5, -2.0, F2 + 0.008, 1.8);
    b.mat(&pal.ceiling);
    b.ceiling(-16.0, -22.0, 6.0, 4.0, CEIL, 2.6);
    // landing inside the elevator shaft at penthouse level
    b.mat(&pal.stone);
    b.slab_solid(-20.4, -22.0, -16.2, -18.0, F2, 0.4, 1.6);

    // walls
    b.mat(&pal.wallpaper);
    b.wall_z(-16.0, 6.0, F2, CEIL, -22.0, 0.45, 2.2, None);
    wall_z_win(
        b,
        -16.0,
        6.0,
        F2,
        CEIL,
        4.0,
        0.45,
        2.2,
        &[
            (-12.5, -8.5, F2 + 1.0, F2 + 4.6),
            (-4.5, -0.5, F2 + 1.0, F2 + 4.6),
        ],
    );
    // east wall: two windows, the secret panel, and the way to the terrace
    wall_x_win(
        b,
        -22.0,
        4.0,
        F2,
        CEIL,
        6.0,
        0.45,
        2.2,
        &[
            (-12.0, -8.5, F2 + 1.0, F2 + 5.0),
            (-6.0, -2.5, F2 + 1.0, F2 + 5.0),
            (0.2, 1.6, F2, F2 + 2.6),
        ],
    );
    // west wall with the elevator opening
    wall_x_win(
        b,
        -22.0,
        4.0,
        F2,
        CEIL,
        -16.0,
        0.45,
        2.2,
        &[(-21.2, -19.2, F2, F2 + 2.2)],
    );

    // wainscot + cornice + beams
    b.mat(&pal.wood_h);
    b.mb().box_all(
        Vec3::new(-5.0, F2 + 0.72, -21.72),
        Vec3::new(21.6, 1.44, 0.09),
        1.2,
    );
    b.mb().box_all(
        Vec3::new(-15.72, F2 + 0.72, -9.0),
        Vec3::new(0.09, 1.44, 25.6),
        1.2,
    );
    b.mat(&pal.trim);
    b.mb().box_all(
        Vec3::new(-5.0, F2 + 1.48, -21.72),
        Vec3::new(21.6, 0.08, 0.12),
        1.0,
    );
    b.mb().box_all(
        Vec3::new(-5.0, CEIL - 0.4, -21.7),
        Vec3::new(21.8, 0.42, 0.3),
        1.0,
    );
    b.mb().box_all(
        Vec3::new(-15.70, CEIL - 0.4, -9.0),
        Vec3::new(0.3, 0.42, 25.8),
        1.0,
    );
    b.mb().box_all(
        Vec3::new(0.0, CEIL - 0.4, 3.7),
        Vec3::new(32.0, 0.42, 0.3),
        1.0,
    );
    b.mat(&pal.wood);
    for z in [-18.0f32, -11.0, -4.0] {
        b.mb().box_all(
            Vec3::new(-5.0, CEIL - 0.22, z),
            Vec3::new(21.4, 0.3, 0.3),
            1.2,
        );
    }

    // windows
    for x in [-10.5f32, -2.5] {
        props::window(commands, meshes, pal, Vec3::new(x, F2 + 2.8, 4.0), 3.9, 3.6, 0.0);
        props::curtain(
            commands,
            meshes,
            pal,
            Vec3::new(x, F2 + 2.9, 3.7),
            4.6,
            4.6,
            0.0,
        );
    }
    for z in [-10.25f32, -4.25] {
        props::window(
            commands,
            meshes,
            pal,
            Vec3::new(6.0, F2 + 3.0, z),
            3.4,
            4.0,
            std::f32::consts::FRAC_PI_2,
        );
        props::curtain(
            commands,
            meshes,
            pal,
            Vec3::new(5.72, F2 + 3.05, z),
            4.2,
            4.8,
            std::f32::consts::FRAC_PI_2,
        );
    }

    // the stage and the piano
    b.mat(&pal.wood);
    b.box_collide(
        Vec3::new(-5.0, F2 + 0.18, -20.7),
        Vec3::new(15.0, 0.36, 2.6),
        1.6,
    );
    b.box_collide(
        Vec3::new(-5.0, F2 + 0.09, -19.15),
        Vec3::new(15.0, 0.18, 0.5),
        1.6,
    );
    props::curtain(
        commands,
        meshes,
        pal,
        Vec3::new(-5.0, F2 + 2.9, -21.6),
        13.6,
        5.4,
        0.0,
    );
    props::grand_piano(commands, meshes, pal, Vec3::new(-5.0, F2 + 0.36, -20.6), 0.0);
    commands.spawn((
        Transform::from_translation(Vec3::new(-5.0, F2 + 1.2, -19.1)),
        Interactable::new(Action::Piano, "Klavier anspielen", 0.9),
    ));

    // two chandeliers, both alive but sick
    props::chandelier(commands, meshes, pal, Vec3::new(-11.0, CEIL - 1.1, -13.0), 1.8, true);
    props::chandelier(commands, meshes, pal, Vec3::new(-1.0, CEIL - 1.1, -5.0), 1.5, true);

    // tables and chairs
    let tables = [
        (-13.0f32, -13.0f32, true),
        (-9.5, -6.5, false),
        (-13.5, -1.0, true),
        (-2.5, -13.5, false),
        (-2.0, -2.0, true),
    ];
    for (i, (x, z, cloth)) in tables.iter().enumerate() {
        props::round_table(commands, meshes, pal, Vec3::new(*x, F2, *z), *cloth);
        b.solid(
            Vec3::new(*x, F2 + 0.4, *z),
            Vec3::new(1.2, 0.8, 1.2),
        );
        for k in 0..3 {
            let a = i as f32 * 1.3 + k as f32 * 2.1;
            let pos = Vec3::new(*x + a.cos() * 1.15, F2, *z + a.sin() * 1.15);
            if (i + k) % 5 == 0 {
                props::fallen_chair(commands, meshes, pal, pos, a);
            } else {
                props::chair(commands, meshes, pal, pos, a + std::f32::consts::PI);
            }
        }
    }
    props::fallen_chair(commands, meshes, pal, Vec3::new(2.6, F2, -17.0), 0.9);
    props::fallen_chair(commands, meshes, pal, Vec3::new(-7.4, F2, 1.2), 2.2);
    props::fallen_chair(commands, meshes, pal, Vec3::new(0.8, F2, -8.4), 1.1);

    // sconces
    for x in [-14.0f32, 3.0] {
        props::sconce(commands, meshes, pal, Vec3::new(x, F2 + 3.1, -21.7), 0.0, true, true);
    }
    for x in [-14.0f32, 3.0] {
        props::sconce(
            commands,
            meshes,
            pal,
            Vec3::new(x, F2 + 3.1, 3.7),
            std::f32::consts::PI,
            x > 0.0,
            true,
        );
    }

    // plants pushing in through the corners
    props::potted_plant(commands, meshes, pal, Vec3::new(-15.0, F2, -20.6), 1.7, 1);
    props::potted_plant(commands, meshes, pal, Vec3::new(-15.0, F2, 3.0), 1.5, 2);
    props::potted_plant(commands, meshes, pal, Vec3::new(4.6, F2, -20.4), 1.4, 0);
    props::hanging_vines(
        commands,
        meshes,
        pal,
        assets.tex.vine.clone(),
        Vec3::new(-13.6, CEIL - 0.15, 1.6),
        5,
    );
    props::hanging_vines(
        commands,
        meshes,
        pal,
        assets.tex.vine.clone(),
        Vec3::new(3.4, CEIL - 0.15, -19.0),
        4,
    );

    // elevator gate on the west wall
    b.mat(&pal.brass_dark);
    b.box_at(
        Vec3::new(-15.72, F2 + 1.15, -20.2),
        Vec3::new(0.12, 2.3, 0.16),
        1.0,
    );
    b.box_at(
        Vec3::new(-15.72, F2 + 1.15, -20.2),
        Vec3::new(0.12, 2.3, 0.16),
        1.0,
    );
    for z in [-21.1f32, -19.3] {
        b.mb().box_all(
            Vec3::new(-15.72, F2 + 1.15, z),
            Vec3::new(0.12, 2.3, 0.14),
            1.0,
        );
    }
    props::text3d(
        commands,
        assets.fonts.ui.clone(),
        Vec3::new(-15.65, F2 + 2.42, -20.2),
        std::f32::consts::FRAC_PI_2,
        0.2,
        "PENTHOUSE",
        Color::srgba(0.86, 0.72, 0.42, 0.8),
    );

    // the newspaper on the piano lid
    b.mat(&pal.paper);
    b.box_at(
        Vec3::new(-5.0, F2 + 1.09, -20.1),
        Vec3::new(0.42, 0.03, 0.56),
        0.5,
    );
    commands.spawn((
        Transform::from_translation(Vec3::new(-5.0, F2 + 1.2, -20.1)),
        Interactable::new(Action::ReadBallroomArticle, "Zeitung lesen", 0.5),
    ));
    // the chef's diary, left open on a table
    b.mat(&pal.paper);
    b.box_at(
        Vec3::new(-2.0, F2 + 0.79, -2.0),
        Vec3::new(0.36, 0.04, 0.28),
        0.5,
    );
    commands.spawn((
        Transform::from_translation(Vec3::new(-2.0, F2 + 0.9, -2.0)),
        Interactable::new(Action::ReadChefDiary, "Tagebuch lesen", 0.45),
    ));

    let _ = rng.index(3);
}

// ---------------------------------------------------------------------------
// grandfather clock + the secret panel
// ---------------------------------------------------------------------------

fn clock_and_secret(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
) {
    let rot = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);

    // case, crown, plinth
    let case = props::mesh_of(meshes, |mb| {
        mb.box_all(Vec3::new(0.0, 1.58, 0.0), Vec3::new(1.0, 3.16, 0.62), 1.2);
        mb.box_all(Vec3::new(0.0, 0.10, 0.0), Vec3::new(1.16, 0.20, 0.72), 1.0);
        mb.box_all(Vec3::new(0.0, 3.20, 0.0), Vec3::new(1.16, 0.26, 0.70), 1.0);
        // iron straps
        for y in [0.9f32, 2.1, 2.9] {
            mb.box_all(Vec3::new(0.0, y, 0.315), Vec3::new(1.02, 0.05, 0.02), 1.0);
        }
        mb.box_all(Vec3::new(0.0, 2.62, 0.315), Vec3::new(0.72, 0.05, 0.02), 1.0);
        // two setting knobs, one for each hand
        for x in [-0.42f32, 0.42] {
            mb.cylinder(Vec3::new(x, 2.48, 0.33), 0.055, 0.06, 12, 1.0, true);
            mb.box_all(Vec3::new(x, 2.48, 0.36), Vec3::new(0.11, 0.02, 0.02), 1.0);
        }
    });
    props::spawn_prop(
        commands,
        case,
        pal.wood.clone(),
        Transform::from_translation(CLOCK_POS).with_rotation(rot),
    );
    b.solid(
        CLOCK_POS + rot * Vec3::new(0.0, 1.6, 0.0),
        Vec3::new(0.66, 3.2, 1.16),
    );

    // clock face
    let face = props::mesh_of(meshes, |mb| {
        mb.quad_uv(
            Vec3::new(-0.33, 2.34, 0.325),
            Vec3::new(0.33, 2.34, 0.325),
            Vec3::new(0.33, 3.0, 0.325),
            Vec3::new(-0.33, 3.0, 0.325),
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
        Transform::from_translation(CLOCK_POS).with_rotation(rot),
    );

    // glass over the pendulum
    let glass = props::mesh_of(meshes, |mb| {
        mb.quad_uv(
            Vec3::new(-0.3, 0.6, 0.33),
            Vec3::new(0.3, 0.6, 0.33),
            Vec3::new(0.3, 2.15, 0.33),
            Vec3::new(-0.3, 2.15, 0.33),
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
        glass,
        pal.glass.clone(),
        Transform::from_translation(CLOCK_POS).with_rotation(rot),
    );
    // pendulum behind the glass
    let pend = props::mesh_of(meshes, |mb| {
        mb.box_all(Vec3::new(0.0, 1.35, 0.12), Vec3::new(0.03, 1.4, 0.03), 1.0);
        mb.cylinder(Vec3::new(0.0, 0.72, 0.12), 0.13, 0.05, 14, 1.0, true);
    });
    props::spawn_prop(
        commands,
        pend,
        pal.brass_dark.clone(),
        Transform::from_translation(CLOCK_POS).with_rotation(rot),
    );

    // hands: pivot at the centre of the face
    let hub = CLOCK_POS + rot * Vec3::new(0.0, 2.67, 0.345);
    let minute = props::mesh_of(meshes, |mb| {
        mb.box_all(Vec3::new(0.0, 0.13, 0.0), Vec3::new(0.024, 0.28, 0.012), 1.0);
    });
    commands.spawn((
        Mesh3d(minute),
        MeshMaterial3d(pal.black_metal.clone()),
        Transform::from_translation(hub).with_rotation(rot),
        ClockHand {
            hour: false,
            base: rot,
        },
    ));
    let hour = props::mesh_of(meshes, |mb| {
        mb.box_all(Vec3::new(0.0, 0.09, 0.012), Vec3::new(0.032, 0.19, 0.014), 1.0);
    });
    commands.spawn((
        Mesh3d(hour),
        MeshMaterial3d(pal.black_metal.clone()),
        Transform::from_translation(hub).with_rotation(rot),
        ClockHand { hour: true, base: rot },
    ));

    // the latch on the side of the case
    commands.spawn((
        Transform::from_translation(CLOCK_POS + rot * Vec3::new(0.42, 1.4, 0.36)),
        Interactable::new(Action::ClockLatch, "Geheimmechanismus prüfen", 0.45),
    ));
    // the two knobs that set the hands
    commands.spawn((
        Transform::from_translation(CLOCK_POS + rot * Vec3::new(-0.42, 2.48, 0.40)),
        Interactable::new(Action::ClockHour, "Stundenzeiger drehen", 0.28),
    ));
    commands.spawn((
        Transform::from_translation(CLOCK_POS + rot * Vec3::new(0.42, 2.48, 0.40)),
        Interactable::new(Action::ClockMinute, "Minutenzeiger drehen", 0.28),
    ));

    // the panel in the east wall, hidden behind the clock
    let panel = props::mesh_of(meshes, |mb| {
        mb.box_all(Vec3::new(6.0, F2 + 1.3, 0.9), Vec3::new(0.3, 2.6, 1.4), 1.2);
        // iron studs so it reads as a service hatch
        for y in [0.35f32, 1.3, 2.25] {
            mb.box_all(Vec3::new(5.84, F2 + y, 0.9), Vec3::new(0.02, 0.05, 1.3), 1.0);
        }
    });
    let panel_entity = props::spawn_prop(
        commands,
        panel,
        pal.painting.clone(),
        Transform::IDENTITY,
    );
    // Slide is expressed in world space, so animate the whole entity upwards
    commands.entity(panel_entity).insert((
        SecretDoor,
        Slide::new(Vec3::ZERO, Vec3::new(0.0, 2.75, 0.0), 2.4),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(5.6, F2 + 1.2, 0.9)),
        Interactable::new(Action::ClockLatch, "Geheimtür öffnen", 0.9),
    ));

    // a solid while it is closed
    commands.spawn((
        Transform::from_translation(Vec3::new(6.0, F2 + 1.3, 0.9)),
        crate::collision::SolidBox {
            half: Vec3::new(0.2, 1.3, 0.72),
        },
        SecretBlocker,
        Name::new("secret_blocker"),
    ));

    // plaque beside the panel
    props::text3d(
        commands,
        assets.fonts.ui.clone(),
        Vec3::new(5.72, F2 + 2.62, 2.1),
        -std::f32::consts::FRAC_PI_2,
        0.13,
        "DIENSTGANG",
        Color::srgba(0.72, 0.66, 0.52, 0.5),
    );
}

// ---------------------------------------------------------------------------
// roof terrace
// ---------------------------------------------------------------------------

fn terrace(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
    rng: &mut Rng,
) {
    // deck
    b.mat(&pal.stone);
    b.slab_solid(6.0, -14.0, 22.0, 4.0, F2, 0.5, 2.2);
    // wet patches
    b.mat(&pal.rain_glass);
    b.floor(9.0, -11.0, 13.5, -6.0, F2 + 0.012, 2.0);
    b.floor(15.0, -3.0, 19.0, 1.5, F2 + 0.012, 2.0);

    // parapet
    b.mat(&pal.stone);
    b.box_collide(
        Vec3::new(14.0, F2 + 0.58, 3.8),
        Vec3::new(16.2, 1.16, 0.45),
        1.4,
    );
    b.box_collide(
        Vec3::new(14.0, F2 + 0.58, -13.8),
        Vec3::new(16.2, 1.16, 0.45),
        1.4,
    );
    // the east side is a parapet, broken by a tall gate portal
    wall_x_win(b, -14.0, -13.4, F2, F2 + 1.16, 21.8, 0.45, 1.4, &[]);
    wall_x_win(b, -4.5, 4.0, F2, F2 + 1.16, 21.8, 0.45, 1.4, &[]);
    b.box_collide(Vec3::new(21.8, F2 + 1.45, -11.1), Vec3::new(0.5, 2.9, 4.6), 1.4);
    b.box_collide(Vec3::new(21.8, F2 + 1.45, -4.65), Vec3::new(0.5, 2.9, 0.3), 1.4);
    b.box_collide(Vec3::new(21.8, F2 + 2.55, -7.0), Vec3::new(0.5, 0.7, 4.0), 1.4);
    // pillars at the corners
    for (x, z) in [(21.8f32, 3.8f32), (21.8, -13.8), (6.2, 3.8), (6.2, -13.8)] {
        b.box_collide(Vec3::new(x, F2 + 0.7, z), Vec3::new(0.7, 1.4, 0.7), 1.2);
        b.mat(&pal.moss);
        b.box_at(Vec3::new(x, F2 + 1.5, z), Vec3::new(0.55, 0.3, 0.55), 1.0);
        b.mat(&pal.stone);
    }

    // iron gate posts at the portal
    b.mat(&pal.black_metal);
    for z in [-9.0f32, -5.0] {
        b.box_at(Vec3::new(21.7, F2 + 1.1, z), Vec3::new(0.22, 2.2, 0.18), 1.0);
    }
    b.box_at(Vec3::new(21.7, F2 + 2.3, -7.0), Vec3::new(0.22, 0.18, 4.2), 1.0);

    // the gate itself: slides up when the switch is thrown
    let gate = props::mesh_of(meshes, |mb| {
        for i in 0..17 {
            let z = -7.15 + i as f32 * 0.24;
            mb.bar(
                Vec3::new(21.8, F2 + 0.05, z),
                Vec3::new(21.8, F2 + 2.2, z),
                0.035,
                1.0,
            );
        }
        for y in [0.15f32, 1.1, 2.15] {
            mb.bar(
                Vec3::new(21.8, F2 + y, -9.05),
                Vec3::new(21.8, F2 + y, -4.95),
                0.05,
                1.0,
            );
        }
    });
    let gate_entity = props::spawn_prop(
        commands,
        gate,
        pal.black_metal.clone(),
        Transform::IDENTITY,
    );
    commands
        .entity(gate_entity)
        .insert((ExitGate, Slide::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -4.2), 3.4)));

    // keep the gate solid until it rolls open
    commands.spawn((
        Transform::from_translation(Vec3::new(21.8, F2 + 1.1, -7.0)),
        crate::collision::SolidBox {
            half: Vec3::new(0.16, 1.1, 2.0),
        },
        GateBlocker,
        Name::new("gate_blocker"),
    ));

    // emergency switch box on the parapet
    b.mat(&pal.paint_red);
    b.box_at(
        Vec3::new(21.5, F2 + 1.15, -10.2),
        Vec3::new(0.24, 0.4, 0.34),
        0.5,
    );
    b.mat(&pal.black_metal);
    b.box_at(
        Vec3::new(21.38, F2 + 1.15, -10.2),
        Vec3::new(0.08, 0.3, 0.24),
        0.5,
    );
    b.mat(&pal.brass);
    b.mb().cylinder(
        Vec3::new(21.3, F2 + 1.15, -10.2),
        0.05,
        0.16,
        10,
        1.0,
        true,
    );
    commands.spawn((
        Transform::from_translation(Vec3::new(21.15, F2 + 1.15, -10.2)),
        Interactable::new(Action::EmergencySwitch, "Notstromschalter drücken", 0.5),
    ));
    // status LED
    b.mat(&pal.black_metal);
    b.box_at(
        Vec3::new(21.52, F2 + 1.55, -9.55),
        Vec3::new(0.2, 0.22, 0.3),
        0.5,
    );
    let led = props::mesh_of(meshes, |mb| {
        mb.cylinder(Vec3::new(0.0, 0.0, 0.0), 0.045, 0.03, 10, 1.0, true);
    });
    let led_entity = props::spawn_prop(
        commands,
        led,
        pal.led_off.clone(),
        Transform::from_translation(Vec3::new(21.42, F2 + 1.55, -9.55))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
    );
    commands.entity(led_entity).insert(GateLed);
    props::text3d(
        commands,
        assets.fonts.ui.clone(),
        Vec3::new(21.4, F2 + 1.35, -9.55),
        -std::f32::consts::FRAC_PI_2,
        0.09,
        "TOR",
        Color::srgba(0.8, 0.76, 0.66, 0.7),
    );

    // terrace furniture, all rotten
    props::potted_plant(commands, meshes, pal, Vec3::new(7.6, F2, -12.6), 1.7, 0);
    props::potted_plant(commands, meshes, pal, Vec3::new(7.4, F2, 2.6), 1.5, 2);
    props::potted_plant(commands, meshes, pal, Vec3::new(20.8, F2, 3.0), 1.3, 1);
    props::potted_plant(commands, meshes, pal, Vec3::new(20.6, F2, -12.4), 1.6, 2);
    props::round_table(commands, meshes, pal, Vec3::new(13.0, F2, -2.0), false);
    b.solid(
        Vec3::new(13.0, F2 + 0.4, -2.0),
        Vec3::new(1.0, 0.8, 1.0),
    );
    props::fallen_chair(commands, meshes, pal, Vec3::new(14.2, F2, -3.1), 1.4);
    props::fallen_chair(commands, meshes, pal, Vec3::new(11.9, F2, -3.4), 2.6);
    // a dead fountain
    b.mat(&pal.stone);
    b.mb()
        .cylinder(Vec3::new(17.0, F2 + 0.22, -9.0), 1.35, 0.44, 20, 1.6, true);
    b.mat(&pal.rain_glass);
    b.floor(15.7, -10.3, 18.3, -7.7, F2 + 0.4, 1.6);
    b.mat(&pal.stone);
    b.mb()
        .cylinder(Vec3::new(17.0, F2 + 0.9, -9.0), 0.18, 1.0, 10, 1.0, false);
    b.solid(
        Vec3::new(17.0, F2 + 0.3, -9.0),
        Vec3::new(2.7, 0.6, 2.7),
    );

    // jungle spilling over the parapet
    for i in 0..7 {
        let x = 8.0 + i as f32 * 2.2;
        props::hanging_vines(
            commands,
            meshes,
            pal,
            assets.tex.vine.clone(),
            Vec3::new(x, F2 + 1.2, 3.6),
            3,
        );
        props::hanging_vines(
            commands,
            meshes,
            pal,
            assets.tex.vine.clone(),
            Vec3::new(x, F2 + 1.2, -13.6),
            3,
        );
    }

    // a few cold lamps so the terrace is not pitch black
    for (x, z) in [(8.0f32, -6.0f32), (20.0, -6.0)] {
        commands.spawn((
            PointLight {
                color: Color::srgb(0.72, 0.82, 1.0),
                intensity: 1_200.0,
                range: 12.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(Vec3::new(x, F2 + 2.6, z)),
            Flicker {
                base: 1_200.0,
                amp: 0.4,
                speed: 1.7,
                phase: x,
                dying: true,
            },
        ));
        b.mat(&pal.brass_dark);
        b.box_at(Vec3::new(x, F2 + 2.7, z), Vec3::new(0.18, 0.3, 0.18), 0.5);
    }

    let _ = rng.f32();
}

// ---------------------------------------------------------------------------
// the gate corridor: narrow, dark, and the last place you will ever stand
// ---------------------------------------------------------------------------

fn gate_corridor(
    commands: &mut Commands,
    b: &mut BuildCtx,
    pal: &Palette,
    _meshes: &mut Assets<Mesh>,
    assets: &GameAssets,
) {
    let h = F2 + 3.4;

    // floors
    b.mat(&pal.stone);
    b.slab_solid(22.0, -9.0, 34.0, -5.0, F2, 0.5, 1.6);
    b.slab_solid(30.0, -22.0, 34.0, -5.0, F2, 0.5, 1.6);

    b.mat(&pal.brick);
    // corridor A walls
    b.wall_z(22.0, 30.0, F2, h, -9.0, 0.4, 1.4, None);
    b.wall_z(22.0, 34.0, F2, h, -5.0, 0.4, 1.4, None);
    // corridor B walls
    b.wall_x(-22.0, -9.0, F2, h, 30.0, 0.4, 1.4, None);
    b.wall_x(-22.0, -5.0, F2, h, 34.0, 0.4, 1.4, None);
    b.wall_z(30.0, 34.0, F2, h, -22.0, 0.4, 1.4, None);

    // ceilings
    b.mat(&pal.concrete);
    b.ceiling_solid(22.0, -9.0, 34.0, -5.0, h, 0.35, 1.6);
    b.ceiling_solid(30.0, -22.0, 34.0, -9.0, h, 0.35, 1.6);

    // pipes and cables running along the ceiling
    b.mat(&pal.metal);
    for (z, y) in [(-8.6f32, F2 + 3.05f32), (-8.2, F2 + 2.9)] {
        b.mb().cylinder(
            Vec3::new(28.0, y, z),
            0.09,
            12.0,
            8,
            1.0,
            false,
        );
    }
    for x in [31.0f32, 31.6] {
        b.mb().cylinder(
            Vec3::new(x, F2 + 3.0, -15.0),
            0.08,
            12.0,
            8,
            1.0,
            false,
        );
    }

    // three dying bulbs
    for (x, z) in [(25.0f32, -7.0f32), (31.0, -7.0), (32.0, -14.0)] {
        b.mat(&pal.brass_dark);
        b.mb().bar(
            Vec3::new(x, h - 0.05, z),
            Vec3::new(x, h - 0.55, z),
            0.02,
            1.0,
        );
        b.mat(&pal.bulb_dim);
        b.mb().cylinder(Vec3::new(x, h - 0.65, z), 0.06, 0.14, 8, 1.0, true);
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.74, 0.45),
                intensity: 700.0,
                range: 9.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(Vec3::new(x, h - 0.7, z)),
            Flicker {
                base: 700.0,
                amp: 0.8,
                speed: 5.5,
                phase: x * 1.7,
                dying: true,
            },
        ));
    }

    // a leaning door and some crates so the corridor is not bare
    b.mat(&pal.metal);
    b.box_at(
        Vec3::new(29.3, F2 + 1.1, -5.5),
        Vec3::new(0.08, 2.2, 1.0),
        1.0,
    );
    b.mat(&pal.wood);
    b.box_collide(
        Vec3::new(23.4, F2 + 0.4, -8.3),
        Vec3::new(0.8, 0.8, 0.7),
        1.0,
    );
    b.box_collide(
        Vec3::new(33.2, F2 + 0.3, -20.6),
        Vec3::new(0.9, 0.6, 0.9),
        1.0,
    );

    // EXIT sign over the far corner
    props::text3d(
        commands,
        assets.fonts.ui.clone(),
        Vec3::new(33.3, F2 + 2.35, -20.6),
        -std::f32::consts::FRAC_PI_2,
        0.22,
        "AUSGANG",
        Color::srgba(0.62, 0.86, 0.6, 0.55),
    );

    // the trigger: just past the last corner
    commands.spawn((
        Transform::from_translation(Vec3::new(32.0, F2 + 1.0, -12.5)),
        JumpscareTrigger,
        Name::new("jumpscare_trigger"),
    ));
}
