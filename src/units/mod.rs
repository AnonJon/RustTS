pub mod components;
pub mod movement;
pub mod selection;
pub mod combat;
pub mod types;
pub mod animation;
pub mod pathfinding;
pub mod monk;
pub mod civ_bonus;
pub mod naval;

use bevy::prelude::*;
use components::*;
use movement::*;
use selection::*;
use combat::*;
use animation::*;
use pathfinding::*;
use crate::GameState;
use crate::map::generation::generate_map_config;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<selection::ControlGroups>()
            .add_systems(PreStartup, load_unit_sprites)
            .add_systems(OnEnter(GameState::InGame), spawn_initial_units.after(generate_map_config))
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
                aoe_damage_system,
                attack_move_scan_system,
                patrol_system,
            ).run_if(in_state(GameState::InGame)))
            .add_systems(Update, (
                chase_system,
                death_system,
                health_bar_system,
                selection_health_bar_system,
                carry_indicator_system,
                gather_visual_system,
                animation_system,
                facing_system,
                separation_system,
                selection::control_group_system,
            ).run_if(in_state(GameState::InGame)))
            .add_systems(Update, (
                monk::monk_heal_system,
                monk::monk_convert_system,
                monk::monk_auto_heal_system,
                monk::relic_pickup_system,
                monk::relic_deposit_system,
                monk::relic_income_system,
                monk::relic_drop_on_death_system,
                civ_bonus::apply_civ_bonuses,
                naval::fishing_ship_system,
            ).run_if(in_state(GameState::InGame)));
    }
}

fn load_unit_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let villager = make_unit_sprite_sheet(&asset_server, &mut atlas_layouts, types::UnitKind::Villager);
    let militia = make_unit_sprite_sheet(&asset_server, &mut atlas_layouts, types::UnitKind::Militia);
    commands.insert_resource(types::UnitSprites { villager, militia });
}

fn spawn_initial_units(
    mut commands: Commands,
    sprites: Res<types::UnitSprites>,
    config: Res<crate::map::generation::MapConfig>,
    settings: Res<crate::GameSettings>,
) {
    let bx = config.player_base().x;
    let by = config.player_base().y;

    let militia_offsets = [(-1, -1), (4, 0), (0, 4)];
    for (dx, dy) in militia_offsets {
        let (lx, ly) = crate::map::generation::find_nearest_land(&config.terrain_grid, bx + dx, by + dy);
        let grid = crate::map::GridPosition::new(lx, ly);
        let world = grid.to_world();
        types::spawn_unit(
            &mut commands,
            sprites.get(types::UnitKind::Militia),
            types::UnitKind::Militia,
            Team(0),
            grid,
            world,
        );
    }

    let villager_offsets = [(-1, 1), (2, -1)];
    for (dx, dy) in villager_offsets {
        let (lx, ly) = crate::map::generation::find_nearest_land(&config.terrain_grid, bx + dx, by + dy);
        let grid = crate::map::GridPosition::new(lx, ly);
        let world = grid.to_world();
        types::spawn_unit(
            &mut commands,
            sprites.get(types::UnitKind::Villager),
            types::UnitKind::Villager,
            Team(0),
            grid,
            world,
        );
    }

    if settings.game_mode == crate::GameMode::Regicide {
        let (lx, ly) = crate::map::generation::find_nearest_land(&config.terrain_grid, bx + 2, by + 2);
        let grid = crate::map::GridPosition::new(lx, ly);
        let world = grid.to_world();
        types::spawn_unit(
            &mut commands,
            sprites.get(types::UnitKind::King),
            types::UnitKind::King,
            Team(0),
            grid,
            world,
        );
    }
}

fn make_unit_sprite_sheet(
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    kind: types::UnitKind,
) -> types::UnitSpriteSheet {
    let path = kind.sprite_path().expect("unit kind must have a sprite path");
    let texture: Handle<Image> = asset_server.load(path);

    let frame_count = kind.frame_count();
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(48, 48),
        frame_count as u32,
        1,
        None,
        None,
    );
    let atlas_layout = atlas_layouts.add(layout);

    types::UnitSpriteSheet {
        texture,
        atlas_layout,
    }
}
