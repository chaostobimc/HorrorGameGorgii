//! Weather and air: rain falling through the beams, dust drifting in the
//! flashlight, and the occasional bolt over the mountains.

use bevy::prelude::*;

use crate::audio::LightningStrike;
use crate::core::{AppState, GameAssets, Rng};
use crate::geom::MeshBuilder;
use crate::player::Player;
use crate::ui::screen_fx::ScreenGrade;
use crate::world::{F1, F2};

/// A block of falling rain streaks that loops downwards forever.
#[derive(Component)]
pub struct RainField {
    pub base: Vec3,
    pub span: f32,
    pub speed: f32,
    pub offset: f32,
}

#[derive(Component)]
pub struct DustField {
    pub base: Vec3,
    pub phase: f32,
}

#[derive(Component)]
pub struct Bolt {
    pub life: f32,
}

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), spawn_weather)
            .add_systems(
                Update,
                (fall_rain, drift_dust, bolt_life, draw_bolts, rain_grade),
            );
    }
}

fn rain_material(assets: &GameAssets, mats: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    mats.add(StandardMaterial {
        base_color: Color::srgba(0.78, 0.86, 1.0, 0.42),
        base_color_texture: Some(assets.tex.rain_streaks.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    })
}

fn dust_material(assets: &GameAssets, mats: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    mats.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.93, 0.86, 0.10),
        base_color_texture: Some(assets.tex.noise.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    })
}

fn spawn_weather(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    existing: Query<&RainField>,
    mut rng: ResMut<Rng>,
) {
    if existing.iter().next().is_some() {
        return;
    }
    let rain = rain_material(&assets, &mut mats);
    let dust_mat = dust_material(&assets, &mut mats);

    // (centre, half extents, amount)
    let fields: [(Vec3, Vec3, usize); 3] = [
        (Vec3::new(0.0, 3.0, 20.0), Vec3::new(22.0, 7.0, 12.0), 460),
        (Vec3::new(12.5, F2 + 3.0, -5.0), Vec3::new(8.5, 7.0, 10.5), 380),
        (Vec3::new(0.0, F1 + 3.0, -27.0), Vec3::new(18.0, 7.0, 5.0), 300),
    ];
    for (c, half, count) in fields.iter() {
        for layer in 0..2 {
            let mut mb = MeshBuilder::new();
            for _ in 0..*count {
                let x = c.x + rng.range(-half.x, half.x);
                let y = rng.range(0.0, half.y * 2.0);
                let z = c.z + rng.range(-half.z, half.z);
                let w = 0.016 + rng.f32() * 0.012;
                let h = 0.34 + rng.f32() * 0.34;
                let tilt = rng.range(-0.05, 0.05);
                let a = Vec3::new(x - w, y, z);
                let b = Vec3::new(x + w, y, z);
                let c2 = Vec3::new(x + w + tilt, y + h, z + 0.02);
                let d = Vec3::new(x - w + tilt, y + h, z + 0.02);
                mb.quad_uv(
                    a,
                    b,
                    c2,
                    d,
                    [
                        Vec2::new(0.0, 0.0),
                        Vec2::new(1.0, 0.0),
                        Vec2::new(1.0, 1.0),
                        Vec2::new(0.0, 1.0),
                    ],
                );
            }
            let base = Vec3::new(c.x, c.y - half.y, c.z);
            commands.spawn((
                Mesh3d(meshes.add(mb.into_mesh())),
                MeshMaterial3d(rain.clone()),
                Transform::from_translation(base),
                RainField {
                    base,
                    span: half.y * 2.0,
                    speed: if layer == 0 { 13.0 } else { 8.5 },
                    offset: rng.range(0.0, half.y * 2.0),
                },
            ));
        }
    }

    // dust in the two big rooms
    let rooms: [(Vec3, Vec3, usize); 2] = [
        (Vec3::new(0.0, 3.2, 0.0), Vec3::new(11.0, 3.0, 9.0), 260),
        (Vec3::new(-5.0, F2 + 3.0, -9.0), Vec3::new(10.0, 3.0, 12.0), 300),
    ];
    for (c, half, count) in rooms.iter() {
        let mut mb = MeshBuilder::new();
        for _ in 0..*count {
            let p = Vec3::new(
                c.x + rng.range(-half.x, half.x),
                c.y + rng.range(-half.y, half.y),
                c.z + rng.range(-half.z, half.z),
            );
            let s = 0.018 + rng.f32() * 0.026;
            mb.quad_uv(
                p + Vec3::new(-s, -s, 0.0),
                p + Vec3::new(s, -s, 0.0),
                p + Vec3::new(s, s, 0.0),
                p + Vec3::new(-s, s, 0.0),
                [
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                ],
            );
        }
        commands.spawn((
            Mesh3d(meshes.add(mb.into_mesh())),
            MeshMaterial3d(dust_mat.clone()),
            Transform::from_translation(*c),
            DustField {
                base: *c,
                phase: rng.range(0.0, 6.0),
            },
        ));
    }
}

fn fall_rain(time: Res<Time>, mut q: Query<(&mut Transform, &RainField)>) {
    let t = time.elapsed_secs();
    for (mut tf, field) in q.iter_mut() {
        let looped = (field.base.y - (t * field.speed + field.offset)) % field.span;
        tf.translation.y = field.base.y + looped;
        tf.translation.x = field.base.x + (t * 0.6).sin() * 0.25;
    }
}

fn drift_dust(time: Res<Time>, mut q: Query<(&mut Transform, &DustField)>) {
    let t = time.elapsed_secs();
    for (mut tf, field) in q.iter_mut() {
        tf.translation = field.base
            + Vec3::new(
                (t * 0.11 + field.phase).sin() * 0.5,
                (t * 0.07 + field.phase * 1.7).cos() * 0.3,
                (t * 0.09 + field.phase * 0.6).sin() * 0.5,
            );
        let s = 1.0 + (t * 0.2 + field.phase).sin() * 0.06;
        tf.scale = Vec3::splat(s);
    }
}

fn bolt_life(time: Res<Time>, mut q: Query<(Entity, &mut Bolt)>, mut commands: Commands) {
    for (e, mut bolt) in q.iter_mut() {
        bolt.life -= time.delta_secs();
        if bolt.life <= 0.0 {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn draw_bolts(
    mut strikes: EventReader<LightningStrike>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut rng: ResMut<Rng>,
    existing: Query<&Bolt>,
) {
    for s in strikes.read() {
        if !s.near || existing.iter().next().is_some() {
            continue;
        }
        let x = rng.range(-70.0, 70.0);
        let z = rng.range(-90.0, -30.0);
        let mut mb = MeshBuilder::new();
        let mut p = Vec3::new(x, 78.0, z);
        let mut w = 0.9;
        while p.y > 12.0 {
            let next = p + Vec3::new(rng.range(-4.0, 4.0), -rng.range(6.0, 11.0), rng.range(-2.0, 2.0));
            let dir = (next - p).normalize_or_zero();
            let side = Vec3::new(-dir.y, dir.x, 0.0);
            let a = p - side * w;
            let b = p + side * w;
            let c = next + side * w * 0.7;
            let d = next - side * w * 0.7;
            mb.quad(a, b, c, d, 1.0);
            p = next;
            w *= 0.72;
        }
        let mat = mats.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.92, 1.0),
            emissive: LinearRgba::new(9.0, 10.0, 13.0, 1.0),
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(mb.into_mesh())),
            MeshMaterial3d(mat),
            Transform::IDENTITY,
            Bolt { life: 0.16 },
            Name::new("bolt"),
        ));
    }
}

/// Rain creeps onto the lens as soon as Gorgii steps out into the storm.
fn rain_grade(player: Query<&Transform, With<Player>>, mut grade: ResMut<ScreenGrade>) {
    let Ok(t) = player.get_single() else { return };
    let p = t.translation;
    let outdoor = (p.y > 8.4 && p.x > 6.5 && p.x < 23.0) || (p.y < 1.5 && p.z > 8.5);
    let want = if outdoor { 0.8 } else { 0.0 };
    if (grade.rain - want).abs() > 0.001 {
        grade.rain = want;
    }
}
