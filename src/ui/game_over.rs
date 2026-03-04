use bevy::prelude::*;
use crate::buildings::components::Building;
use crate::buildings::components::BuildingKind;
use crate::units::components::*;
use super::stats::{GameStats, format_time};

#[derive(Resource, Default)]
pub struct GameResult {
    pub decided: bool,
    pub player_won: bool,
    pub overlay_spawned: bool,
}

#[derive(Component)]
pub struct GameOverOverlay;

pub fn check_win_lose(
    mut result: ResMut<GameResult>,
    buildings: Query<(&Building, &Team, &Health)>,
) {
    if result.decided {
        return;
    }

    let player_tc_alive = buildings.iter().any(|(b, t, h)| {
        t.0 == 0 && b.kind == BuildingKind::TownCenter && h.current > 0.0
    });

    let ai_tc_alive = buildings.iter().any(|(b, t, h)| {
        t.0 == 1 && b.kind == BuildingKind::TownCenter && h.current > 0.0
    });

    if !player_tc_alive {
        result.decided = true;
        result.player_won = false;
    } else if !ai_tc_alive {
        result.decided = true;
        result.player_won = true;
    }
}

pub fn show_game_over(
    mut commands: Commands,
    mut result: ResMut<GameResult>,
    stats: Res<GameStats>,
) {
    if !result.decided || result.overlay_spawned {
        return;
    }

    result.overlay_spawned = true;

    let message = if result.player_won {
        "VICTORY!"
    } else {
        "DEFEAT"
    };

    let color = if result.player_won {
        Color::srgb(1.0, 0.85, 0.0)
    } else {
        Color::srgb(0.9, 0.15, 0.15)
    };

    commands.spawn((
        GameOverOverlay,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent| {
        parent.spawn((
            Text::new(message),
            TextFont { font_size: 72.0, ..default() },
            TextColor(color),
        ));

        let stat_color = Color::srgb(0.85, 0.85, 0.85);
        let header_color = Color::srgb(1.0, 0.85, 0.3);

        let mut lines: Vec<(String, String)> = vec![
            ("Game Time".into(), format_time(stats.game_time)),
            ("Score".into(), stats.total_score().to_string()),
            ("Military Score".into(), stats.military_score().to_string()),
            ("Economy Score".into(), stats.economy_score().to_string()),
            ("Units Created".into(), stats.units_created.to_string()),
            ("Units Lost".into(), stats.units_lost.to_string()),
            ("Enemy Units Killed".into(), stats.enemy_units_killed.to_string()),
            ("Buildings Built".into(), stats.buildings_built.to_string()),
            ("Buildings Lost".into(), stats.buildings_lost.to_string()),
            ("Enemy Buildings Destroyed".into(), stats.enemy_buildings_destroyed.to_string()),
            ("Food Gathered".into(), stats.food_gathered.to_string()),
            ("Wood Gathered".into(), stats.wood_gathered.to_string()),
            ("Gold Gathered".into(), stats.gold_gathered.to_string()),
            ("Stone Gathered".into(), stats.stone_gathered.to_string()),
        ];
        if stats.conversions > 0 {
            lines.push(("Conversions".into(), stats.conversions.to_string()));
        }

        let summary: String = lines.iter()
            .map(|(label, value)| format!("{label}: {value}"))
            .collect::<Vec<_>>()
            .join("\n");

        parent.spawn((
            Text::new(summary),
            TextFont { font_size: 16.0, ..default() },
            TextColor(stat_color),
        ));
    });
}
