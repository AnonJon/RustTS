pub mod components;
pub mod movement;
pub mod selection;
pub mod combat;
pub mod types;
pub mod animation;
pub mod pathfinding;

use bevy::prelude::*;
use components::*;
use movement::*;
use selection::*;
use combat::*;
use animation::*;
use pathfinding::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_initial_units)
            .add_systems(Update, (
                handle_selection_click,
                handle_drag_selection,
                draw_selection_box,
                draw_selection_indicators,
                handle_right_click_command,
                pathfinding_system,
                path_following_system,
                movement_system,
                attack_damage_system,
                chase_system,
                death_system,
                health_bar_system,
                carry_indicator_system,
                gather_visual_system,
                animation_system,
                facing_system,
                separation_system,
            ));
    }
}

fn spawn_initial_units(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<crate::map::generation::MapConfig>,
) {
    let militia_texture = create_unit_texture(&mut images, [40, 80, 220, 255]);
    let villager_texture = create_unit_texture(&mut images, [200, 160, 60, 255]);

    let bx = config.player_base.x;
    let by = config.player_base.y;

    let militia_offsets = [(2, 0), (3, 1), (0, 3)];
    for (dx, dy) in militia_offsets {
        let grid = crate::map::GridPosition::new(bx + dx, by + dy);
        let world = grid.to_world();
        types::spawn_unit(
            &mut commands,
            &militia_texture,
            types::UnitKind::Militia,
            Team(0),
            grid,
            world,
        );
    }

    let villager_offsets = [(0, 0), (1, 2)];
    for (dx, dy) in villager_offsets {
        let grid = crate::map::GridPosition::new(bx + dx, by + dy);
        let world = grid.to_world();
        types::spawn_unit(
            &mut commands,
            &villager_texture,
            types::UnitKind::Villager,
            Team(0),
            grid,
            world,
        );
    }
}

pub fn create_unit_texture(images: &mut Assets<Image>, color: [u8; 4]) -> Handle<Image> {
    let size = 24u32;
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = size as f32 / 2.0 - 1.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center + 0.5;
            let dy = y as f32 - center + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * size + x) * 4) as usize;
            if dist <= radius {
                data[idx] = color[0];
                data[idx + 1] = color[1];
                data[idx + 2] = color[2];
                data[idx + 3] = color[3];
            }
        }
    }

    let mut image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        mag_filter: bevy::image::ImageFilterMode::Nearest,
        min_filter: bevy::image::ImageFilterMode::Nearest,
        ..default()
    });

    images.add(image)
}
