use bevy::prelude::*;
use crate::buildings::components::Building;
use crate::buildings::components::BuildingKind;
use crate::units::components::*;

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
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent| {
        parent.spawn((
            Text::new(message),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(color),
        ));
    });
}
