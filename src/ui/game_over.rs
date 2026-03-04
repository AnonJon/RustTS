use bevy::prelude::*;
use crate::buildings::components::{Building, BuildingKind, RelicStorage};
use crate::units::components::*;
use crate::units::types::UnitKind;
use crate::GameMode;
use super::stats::{GameStats, format_time};

#[derive(Resource)]
pub struct GameResult {
    pub decided: bool,
    pub player_won: bool,
    pub overlay_spawned: bool,
    pub wonder_countdown: Option<WonderCountdown>,
    pub relic_countdown: Option<RelicCountdown>,
}

impl Default for GameResult {
    fn default() -> Self {
        Self {
            decided: false,
            player_won: false,
            overlay_spawned: false,
            wonder_countdown: None,
            relic_countdown: None,
        }
    }
}

pub struct WonderCountdown {
    pub team: u8,
    pub timer: Timer,
}

pub struct RelicCountdown {
    pub team: u8,
    pub timer: Timer,
}

const WONDER_COUNTDOWN_SECS: f32 = 200.0;
const RELIC_COUNTDOWN_SECS: f32 = 200.0;
const TOTAL_RELICS: usize = 5;

#[derive(Component)]
pub struct GameOverOverlay;

#[derive(Component)]
pub struct CountdownDisplay;

pub fn check_win_lose(
    mut result: ResMut<GameResult>,
    settings: Res<crate::GameSettings>,
    buildings: Query<(&Building, &Team, &Health)>,
    units: Query<(&Team, &UnitKind, &Health), With<Unit>>,
    time: Res<Time>,
    monasteries: Query<(&Team, &RelicStorage)>,
) {
    if result.decided {
        return;
    }

    match settings.game_mode {
        GameMode::Conquest => check_conquest(&mut result, &buildings, &units),
        GameMode::WonderRace => check_wonder_relic(&mut result, &buildings, &monasteries, &time),
        GameMode::Regicide => check_regicide(&mut result, &units),
    }
}

fn check_conquest(
    result: &mut GameResult,
    buildings: &Query<(&Building, &Team, &Health)>,
    units: &Query<(&Team, &UnitKind, &Health), With<Unit>>,
) {
    let player_alive = buildings.iter().any(|(_, t, h)| t.0 == 0 && h.current > 0.0)
        || units.iter().any(|(t, _, h)| t.0 == 0 && h.current > 0.0);

    let ai_alive = buildings.iter().any(|(_, t, h)| t.0 != 0 && h.current > 0.0)
        || units.iter().any(|(t, _, h)| t.0 != 0 && h.current > 0.0);

    if !player_alive {
        result.decided = true;
        result.player_won = false;
    } else if !ai_alive {
        result.decided = true;
        result.player_won = true;
    }
}

fn check_wonder_relic(
    result: &mut GameResult,
    buildings: &Query<(&Building, &Team, &Health)>,
    monasteries: &Query<(&Team, &RelicStorage)>,
    time: &Time,
) {
    // Wonder victory
    let player_has_wonder = buildings.iter()
        .any(|(b, t, h)| t.0 == 0 && b.kind == BuildingKind::Wonder && h.current > 0.0);
    let ai_has_wonder = buildings.iter()
        .any(|(b, t, h)| t.0 != 0 && b.kind == BuildingKind::Wonder && h.current > 0.0);

    match &mut result.wonder_countdown {
        Some(cd) => {
            let still_alive = if cd.team == 0 { player_has_wonder } else { ai_has_wonder };
            if !still_alive {
                result.wonder_countdown = None;
            } else {
                cd.timer.tick(time.delta());
                if cd.timer.just_finished() {
                    result.decided = true;
                    result.player_won = cd.team == 0;
                    return;
                }
            }
        }
        None => {
            if player_has_wonder {
                result.wonder_countdown = Some(WonderCountdown {
                    team: 0,
                    timer: Timer::from_seconds(WONDER_COUNTDOWN_SECS, TimerMode::Once),
                });
            } else if ai_has_wonder {
                result.wonder_countdown = Some(WonderCountdown {
                    team: 1,
                    timer: Timer::from_seconds(WONDER_COUNTDOWN_SECS, TimerMode::Once),
                });
            }
        }
    }

    // Relic victory
    let mut player_relics = 0usize;
    let mut ai_relics = 0usize;
    for (team, storage) in monasteries.iter() {
        if team.0 == 0 {
            player_relics += storage.relics.len();
        } else {
            ai_relics += storage.relics.len();
        }
    }

    match &mut result.relic_countdown {
        Some(cd) => {
            let still_holding = if cd.team == 0 {
                player_relics >= TOTAL_RELICS
            } else {
                ai_relics >= TOTAL_RELICS
            };
            if !still_holding {
                result.relic_countdown = None;
            } else {
                cd.timer.tick(time.delta());
                if cd.timer.just_finished() {
                    result.decided = true;
                    result.player_won = cd.team == 0;
                    return;
                }
            }
        }
        None => {
            if player_relics >= TOTAL_RELICS {
                result.relic_countdown = Some(RelicCountdown {
                    team: 0,
                    timer: Timer::from_seconds(RELIC_COUNTDOWN_SECS, TimerMode::Once),
                });
            } else if ai_relics >= TOTAL_RELICS {
                result.relic_countdown = Some(RelicCountdown {
                    team: 1,
                    timer: Timer::from_seconds(RELIC_COUNTDOWN_SECS, TimerMode::Once),
                });
            }
        }
    }

    // Fall back: if all buildings/units of a side are gone, they lose
    let player_alive = buildings.iter().any(|(_, t, h)| t.0 == 0 && h.current > 0.0);
    let ai_alive = buildings.iter().any(|(_, t, h)| t.0 != 0 && h.current > 0.0);
    if !player_alive {
        result.decided = true;
        result.player_won = false;
    } else if !ai_alive {
        result.decided = true;
        result.player_won = true;
    }
}

fn check_regicide(
    result: &mut GameResult,
    units: &Query<(&Team, &UnitKind, &Health), With<Unit>>,
) {
    let player_king_alive = units.iter()
        .any(|(t, k, h)| t.0 == 0 && *k == UnitKind::King && h.current > 0.0);
    let ai_king_alive = units.iter()
        .any(|(t, k, h)| t.0 != 0 && *k == UnitKind::King && h.current > 0.0);

    if !player_king_alive {
        result.decided = true;
        result.player_won = false;
    } else if !ai_king_alive {
        result.decided = true;
        result.player_won = true;
    }
}

pub fn update_countdown_display(
    result: Res<GameResult>,
    settings: Res<crate::GameSettings>,
    mut text_q: Query<&mut Text, With<CountdownDisplay>>,
) {
    if settings.game_mode != GameMode::WonderRace { return; }

    let mut msg = String::new();

    if let Some(cd) = &result.wonder_countdown {
        let remaining = cd.timer.duration().as_secs_f32() - cd.timer.elapsed_secs();
        let who = if cd.team == 0 { "Your" } else { "Enemy" };
        msg.push_str(&format!("{who} Wonder: {:.0}s remaining", remaining));
    }

    if let Some(cd) = &result.relic_countdown {
        let remaining = cd.timer.duration().as_secs_f32() - cd.timer.elapsed_secs();
        let who = if cd.team == 0 { "You hold" } else { "Enemy holds" };
        if !msg.is_empty() { msg.push_str("  |  "); }
        msg.push_str(&format!("{who} all relics: {:.0}s", remaining));
    }

    for mut text in &mut text_q {
        **text = msg.clone();
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
