//! Screen space grade: grain, vignette, rain streaks and lightning flash.

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;

use crate::core::{GameAssets, Settings};

const SHADER: &str = "shaders/screen_fx.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ScreenFxMaterial {
    #[uniform(0)]
    pub params: Vec4,
    #[uniform(1)]
    pub clock: Vec4,
    #[texture(2)]
    #[sampler(3)]
    pub noise: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub streaks: Handle<Image>,
}

impl UiMaterial for ScreenFxMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}

/// Values fed into the shader every frame by other modules.
#[derive(Resource)]
pub struct ScreenGrade {
    pub grain: f32,
    pub vignette: f32,
    pub flash: f32,
    pub rain: f32,
    pub time: f32,
}

impl Default for ScreenGrade {
    fn default() -> Self {
        Self {
            grain: 0.32,
            vignette: 0.55,
            flash: 0.0,
            rain: 0.0,
            time: 0.0,
        }
    }
}

#[derive(Component)]
pub struct ScreenFxLayer;

pub fn spawn_layer(
    commands: &mut Commands,
    materials: &mut Assets<ScreenFxMaterial>,
    assets: &GameAssets,
) {
    let handle = materials.add(ScreenFxMaterial {
        params: Vec4::new(0.32, 0.55, 0.0, 0.0),
        clock: Vec4::ZERO,
        noise: assets.tex.noise.clone(),
        streaks: assets.tex.rain_streaks.clone(),
    });
    commands.spawn((
        MaterialNode(handle),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        GlobalZIndex(1),
        ScreenFxLayer,
        PickingBehavior::IGNORE,
    ));
}

pub fn update_material(
    grade: Res<ScreenGrade>,
    settings: Res<Settings>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ScreenFxMaterial>>,
    q: Query<&MaterialNode<ScreenFxMaterial>>,
) {
    let _ = time;
    for handle in q.iter() {
        if let Some(mat) = materials.get_mut(&handle.0.clone()) {
            mat.params = Vec4::new(
                grade.grain * settings.grain.max(0.05) * 1.8,
                grade.vignette,
                grade.flash,
                grade.rain,
            );
            mat.clock = Vec4::new(grade.time, 0.0, 0.0, 0.0);
        }
    }
}
