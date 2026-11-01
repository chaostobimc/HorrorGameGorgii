//! The material library of the hotel.
//!
//! Every surface in the game is one of these materials, which keeps the whole
//! place coherent: same dust, same faded palette, same dampness.

use bevy::prelude::*;

use crate::core::Textures;

#[derive(Resource, Clone)]
pub struct Palette {
    // architecture
    pub marble: Handle<StandardMaterial>,
    pub wood_floor: Handle<StandardMaterial>,
    pub carpet: Handle<StandardMaterial>,
    pub stone: Handle<StandardMaterial>,
    pub concrete: Handle<StandardMaterial>,
    pub wallpaper: Handle<StandardMaterial>,
    pub wallpaper_b: Handle<StandardMaterial>,
    pub plaster: Vec<Handle<StandardMaterial>>,
    pub ceiling: Handle<StandardMaterial>,
    pub brick: Handle<StandardMaterial>,
    pub trim: Handle<StandardMaterial>,
    // furniture & props
    pub wood: Handle<StandardMaterial>,
    pub wood_h: Handle<StandardMaterial>,
    pub velvet: Handle<StandardMaterial>,
    pub velvet_green: Handle<StandardMaterial>,
    pub cloth: Handle<StandardMaterial>,
    pub leather: Handle<StandardMaterial>,
    pub brass: Handle<StandardMaterial>,
    pub brass_dark: Handle<StandardMaterial>,
    pub metal: Handle<StandardMaterial>,
    pub black_metal: Handle<StandardMaterial>,
    pub paint_red: Handle<StandardMaterial>,
    pub paint_green: Handle<StandardMaterial>,
    pub paint_yellow: Handle<StandardMaterial>,
    pub paint_blue: Handle<StandardMaterial>,
    pub paper: Handle<StandardMaterial>,
    pub clock_face: Handle<StandardMaterial>,
    pub piano_keys: Handle<StandardMaterial>,
    pub ivory: Handle<StandardMaterial>,
    pub terracotta: Handle<StandardMaterial>,
    pub painting: Handle<StandardMaterial>,
    // light & nature
    pub bulb: Handle<StandardMaterial>,
    pub bulb_dim: Handle<StandardMaterial>,
    pub led_green: Handle<StandardMaterial>,
    pub led_off: Handle<StandardMaterial>,
    pub leaf: Handle<StandardMaterial>,
    pub leaf2: Handle<StandardMaterial>,
    pub vine: Handle<StandardMaterial>,
    pub moss: Handle<StandardMaterial>,
    pub glass: Handle<StandardMaterial>,
    pub rain_glass: Handle<StandardMaterial>,
    pub black: Handle<StandardMaterial>,
    // the player's own body (first person view model)
    pub skin: Handle<StandardMaterial>,
    pub skin_dark: Handle<StandardMaterial>,
    pub trouser: Handle<StandardMaterial>,
    pub shirt: Handle<StandardMaterial>,
    pub sleeve: Handle<StandardMaterial>,
}

impl Palette {
    pub fn new(tex: &Textures, mats: &mut Assets<StandardMaterial>) -> Self {
        let mut add = |m: StandardMaterial| mats.add(m);

        let arch = |base: Handle<Image>, normal: Handle<Image>, rough: f32, metallic: f32| {
            StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(base),
                normal_map_texture: Some(normal),
                perceptual_roughness: rough,
                metallic,
                double_sided: true,
                cull_mode: None,
                ..default()
            }
        };

        let marble = add(arch(tex.marble.clone(), tex.marble_n.clone(), 0.16, 0.0));
        let wood_floor = add(arch(tex.wood_floor.clone(), tex.wood_floor_n.clone(), 0.52, 0.0));
        let carpet = add(arch(tex.carpet.clone(), tex.carpet_n.clone(), 1.0, 0.0));
        let stone = add(arch(tex.stone.clone(), tex.stone_n.clone(), 0.30, 0.0));
        let concrete = add(arch(tex.concrete.clone(), tex.concrete_n.clone(), 0.92, 0.0));
        let wallpaper = add(arch(tex.wallpaper.clone(), tex.wallpaper_n.clone(), 0.95, 0.0));
        let brick = add(arch(tex.brick.clone(), tex.brick_n.clone(), 0.9, 0.0));

        let mut wallpaper_b_mat = arch(tex.wallpaper.clone(), tex.wallpaper_n.clone(), 0.95, 0.0);
        wallpaper_b_mat.base_color = Color::srgb(0.86, 0.88, 0.82);
        let wallpaper_b = add(wallpaper_b_mat);

        let mut plaster_mats = Vec::new();
        for i in 0..tex.plaster.len() {
            plaster_mats.push(add(arch(
                tex.plaster[i].clone(),
                tex.plaster_n[i].clone(),
                0.94,
                0.0,
            )));
        }

        let ceiling = add(arch(tex.ceiling.clone(), tex.ceiling_n.clone(), 0.97, 0.0));

        let trim = add(StandardMaterial {
            base_color: Color::srgb(0.30, 0.24, 0.17),
            perceptual_roughness: 0.55,
            ..default()
        });

        let wood = add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.82, 0.80),
            base_color_texture: Some(tex.wood.clone()),
            normal_map_texture: Some(tex.wood_n.clone()),
            perceptual_roughness: 0.34,
            ..default()
        });
        let wood_h = add(StandardMaterial {
            base_color: Color::srgb(0.90, 0.86, 0.80),
            base_color_texture: Some(tex.wood_light.clone()),
            normal_map_texture: Some(tex.wood_light_n.clone()),
            perceptual_roughness: 0.45,
            ..default()
        });
        let velvet = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.velvet.clone()),
            normal_map_texture: Some(tex.velvet_n.clone()),
            perceptual_roughness: 0.92,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let velvet_green = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.velvet_green.clone()),
            normal_map_texture: Some(tex.velvet_n.clone()),
            perceptual_roughness: 0.92,
            ..default()
        });
        let cloth = add(StandardMaterial {
            base_color: Color::srgb(0.62, 0.60, 0.54),
            perceptual_roughness: 1.0,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let leather = add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.12, 0.09),
            perceptual_roughness: 0.55,
            ..default()
        });
        let brass = add(StandardMaterial {
            base_color: Color::srgb(0.90, 0.82, 0.62),
            base_color_texture: Some(tex.brass.clone()),
            normal_map_texture: Some(tex.brass_n.clone()),
            metallic: 0.95,
            perceptual_roughness: 0.32,
            ..default()
        });
        let mut brass_dark_mat = StandardMaterial {
            base_color: Color::srgb(0.62, 0.56, 0.42),
            base_color_texture: Some(tex.brass.clone()),
            normal_map_texture: Some(tex.brass_n.clone()),
            metallic: 0.9,
            perceptual_roughness: 0.55,
            ..default()
        };
        brass_dark_mat.base_color = Color::srgb(0.55, 0.48, 0.36);
        let brass_dark = add(brass_dark_mat);

        let metal = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.metal.clone()),
            normal_map_texture: Some(tex.metal_n.clone()),
            metallic: 0.85,
            perceptual_roughness: 0.55,
            ..default()
        });
        let black_metal = add(StandardMaterial {
            base_color: Color::srgb(0.045, 0.048, 0.052),
            metallic: 0.7,
            perceptual_roughness: 0.42,
            ..default()
        });
        let paint_red = add(StandardMaterial {
            base_color: Color::srgb(0.42, 0.045, 0.045),
            metallic: 0.4,
            perceptual_roughness: 0.45,
            ..default()
        });
        let paint_green = add(StandardMaterial {
            base_color: Color::srgb(0.06, 0.24, 0.10),
            metallic: 0.4,
            perceptual_roughness: 0.45,
            ..default()
        });
        let paint_yellow = add(StandardMaterial {
            base_color: Color::srgb(0.55, 0.42, 0.03),
            metallic: 0.4,
            perceptual_roughness: 0.45,
            ..default()
        });
        let paint_blue = add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.13, 0.34),
            metallic: 0.4,
            perceptual_roughness: 0.45,
            ..default()
        });
        let paper = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.paper.clone()),
            perceptual_roughness: 0.95,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let clock_face = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.clock_face.clone()),
            perceptual_roughness: 0.55,
            ..default()
        });
        let piano_keys = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.piano_keys.clone()),
            perceptual_roughness: 0.18,
            ..default()
        });
        let ivory = add(StandardMaterial {
            base_color: Color::srgb(0.86, 0.83, 0.72),
            perceptual_roughness: 0.3,
            ..default()
        });

        let terracotta = add(StandardMaterial {
            base_color: Color::srgb(0.29, 0.16, 0.11),
            perceptual_roughness: 0.85,
            ..default()
        });
        let painting = add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.13, 0.11),
            perceptual_roughness: 0.7,
            ..default()
        });
        let bulb = add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.80, 0.55),
            emissive: LinearRgba::rgb(9.0, 6.0, 3.1),
            perceptual_roughness: 0.4,
            ..default()
        });
        let bulb_dim = add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.40, 0.30),
            emissive: LinearRgba::rgb(1.1, 0.75, 0.38),
            perceptual_roughness: 0.5,
            ..default()
        });
        let led_green = add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.9, 0.2),
            emissive: LinearRgba::rgb(0.4, 9.0, 1.2),
            ..default()
        });
        let led_off = add(StandardMaterial {
            base_color: Color::srgb(0.06, 0.10, 0.07),
            perceptual_roughness: 0.3,
            ..default()
        });
        let leaf = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.leaf[0].clone()),
            alpha_mode: AlphaMode::Mask(0.42),
            perceptual_roughness: 0.72,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let leaf2 = add(StandardMaterial {
            base_color: Color::srgb(0.86, 0.92, 0.84),
            base_color_texture: Some(tex.leaf[1].clone()),
            alpha_mode: AlphaMode::Mask(0.42),
            perceptual_roughness: 0.72,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let vine = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.vine.clone()),
            alpha_mode: AlphaMode::Mask(0.4),
            perceptual_roughness: 0.8,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let moss = add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex.moss.clone()),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 1.0,
            unlit: false,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let glass = add(StandardMaterial {
            base_color: Color::srgba(0.42, 0.50, 0.58, 0.10),
            metallic: 0.0,
            perceptual_roughness: 0.08,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: true,
            ..default()
        });
        let rain_glass = add(StandardMaterial {
            base_color: Color::srgba(0.62, 0.68, 0.74, 0.55),
            base_color_texture: Some(tex.glass_rain.clone()),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.15,
            double_sided: true,
            cull_mode: None,
            unlit: true,
            ..default()
        });
        let black = add(StandardMaterial {
            base_color: Color::srgb(0.01, 0.01, 0.012),
            perceptual_roughness: 1.0,
            ..default()
        });

        // ---- first person body -------------------------------------------------
        // flat colours, no textures: a view model never wants tiling noise
        let skin = add(StandardMaterial {
            base_color: Color::srgb(0.60, 0.45, 0.35),
            perceptual_roughness: 0.62,
            ..default()
        });
        let skin_dark = add(StandardMaterial {
            base_color: Color::srgb(0.44, 0.32, 0.25),
            perceptual_roughness: 0.70,
            ..default()
        });
        let trouser = add(StandardMaterial {
            base_color: Color::srgb(0.17, 0.19, 0.22),
            perceptual_roughness: 0.88,
            ..default()
        });
        let shirt = add(StandardMaterial {
            base_color: Color::srgb(0.21, 0.23, 0.20),
            perceptual_roughness: 0.90,
            ..default()
        });
        let sleeve = add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.20, 0.18),
            perceptual_roughness: 0.92,
            ..default()
        });

        Self {
            marble,
            wood_floor,
            carpet,
            stone,
            concrete,
            wallpaper,
            wallpaper_b,
            plaster: plaster_mats,
            ceiling,
            brick,
            trim,
            wood,
            wood_h,
            velvet,
            velvet_green,
            cloth,
            leather,
            brass,
            brass_dark,
            metal,
            black_metal,
            paint_red,
            paint_green,
            paint_yellow,
            paint_blue,
            paper,
            clock_face,
            piano_keys,
            ivory,
            terracotta,
            painting,
            bulb,
            bulb_dim,
            led_green,
            led_off,
            leaf,
            leaf2,
            vine,
            moss,
            glass,
            rain_glass,
            black,
            skin,
            skin_dark,
            trouser,
            shirt,
            sleeve,
        }
    }
}

/// Standard rough metal for the outside of the building, gates etc.
pub fn gate_material(mats: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    mats.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.17, 0.18),
        metallic: 0.8,
        perceptual_roughness: 0.6,
        ..default()
    })
}
