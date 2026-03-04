use bevy::prelude::*;
use crate::resources::components::PlayerResources;
use crate::units::components::*;
use crate::buildings::components::*;
use crate::resources::components::{Carrying, ResourceKind, ResourceNode};

#[derive(Component)]
pub struct ResourceDisplay;

#[derive(Component)]
pub struct FoodText;

#[derive(Component)]
pub struct WoodText;

#[derive(Component)]
pub struct GoldText;

#[derive(Component)]
pub struct StoneText;

#[derive(Component)]
pub struct UnitInfoPanel;

#[derive(Component)]
pub struct UnitInfoText;

#[derive(Component)]
pub struct AgeDisplay;

#[derive(Component)]
pub struct AgeProgressBar;

#[derive(Component)]
pub struct AgeProgressFill;

#[derive(Component)]
pub struct AgeProgressText;

pub fn setup_hud(mut commands: Commands) {
    // Top resource bar
    commands.spawn((
        ResourceDisplay,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            height: Val::Px(36.0),
            padding: UiRect::all(Val::Px(8.0)),
            column_gap: Val::Px(24.0),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)),
    )).with_children(|parent| {
        spawn_resource_label(parent, "Food: 0", FoodText);
        spawn_resource_label(parent, "Wood: 0", WoodText);
        spawn_resource_label(parent, "Gold: 0", GoldText);
        spawn_resource_label(parent, "Stone: 0", StoneText);
    });

    // Bottom-left info panel
    commands.spawn((
        UnitInfoPanel,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Px(420.0),
            height: Val::Px(120.0),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)),
    )).with_children(|parent| {
        parent.spawn((
            UnitInfoText,
            Text::new("No unit selected"),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });

    // Top-right age display + progress bar
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            right: Val::Px(10.0),
            padding: UiRect::all(Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            align_items: AlignItems::End,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)),
    )).with_children(|parent| {
        parent.spawn((
            AgeDisplay,
            Text::new("Dark Age"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(1.0, 0.8, 0.2)),
        ));

        // Progress bar container
        parent.spawn((
            AgeProgressBar,
            Node {
                width: Val::Px(140.0),
                height: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            Visibility::Hidden,
        )).with_children(|bar| {
            bar.spawn((
                AgeProgressFill,
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.6, 1.0)),
            ));
        });

        parent.spawn((
            AgeProgressText,
            Text::new(""),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.7, 0.85, 1.0)),
        ));
    });
}

fn spawn_resource_label(parent: &mut ChildSpawnerCommands, text: &str, marker: impl Component) {
    parent.spawn((
        marker,
        Text::new(text.to_string()),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::WHITE),
    ));
}

pub fn update_resource_display(
    resources: Res<PlayerResources>,
    mut food_q: Query<&mut Text, (With<FoodText>, Without<WoodText>, Without<GoldText>, Without<StoneText>)>,
    mut wood_q: Query<&mut Text, (With<WoodText>, Without<FoodText>, Without<GoldText>, Without<StoneText>)>,
    mut gold_q: Query<&mut Text, (With<GoldText>, Without<FoodText>, Without<WoodText>, Without<StoneText>)>,
    mut stone_q: Query<&mut Text, (With<StoneText>, Without<FoodText>, Without<WoodText>, Without<GoldText>)>,
) {
    if let Ok(mut text) = food_q.single_mut() {
        **text = format!("Food: {}", resources.food);
    }
    if let Ok(mut text) = wood_q.single_mut() {
        **text = format!("Wood: {}", resources.wood);
    }
    if let Ok(mut text) = gold_q.single_mut() {
        **text = format!("Gold: {}", resources.gold);
    }
    if let Ok(mut text) = stone_q.single_mut() {
        **text = format!("Stone: {}", resources.stone);
    }
}

pub fn update_unit_info_panel(
    selected_units: Query<(&Health, &Speed, &AttackStats, &Armor, &UnitState, &Team, Option<&Carrying>), (With<Unit>, With<Selected>)>,
    selected_buildings: Query<(&Building, &Health, Option<&TrainingQueue>), (With<Selected>, Without<Unit>)>,
    selected_resources: Query<&ResourceNode, (With<Selected>, Without<Unit>, Without<Building>)>,
    mut info_text: Query<&mut Text, With<UnitInfoText>>,
    age: Res<CurrentAge>,
) {
    let Ok(mut text) = info_text.single_mut() else { return };

    let resources: Vec<_> = selected_resources.iter().collect();
    if !resources.is_empty() {
        if resources.len() == 1 {
            let node = resources[0];
            let name = match node.kind {
                ResourceKind::Wood => "Tree",
                ResourceKind::Gold => "Gold Mine",
                ResourceKind::Stone => "Stone Mine",
                ResourceKind::Food => "Food",
            };
            **text = format!("{name}\nRemaining: {} / {}", node.remaining, node.max_amount);
        } else {
            **text = format!("{} resources selected", resources.len());
        }
        return;
    }

    let buildings: Vec<_> = selected_buildings.iter().collect();
    if !buildings.is_empty() {
        if buildings.len() == 1 {
            let (building, health, queue) = buildings[0];
            let mut info = format!(
                "{:?}  HP: {:.0}/{:.0}",
                building.kind, health.current, health.max
            );

            let trainable = building.kind.can_train();
            if !trainable.is_empty() {
                info += "\n";
                for (i, kind) in trainable.iter().enumerate() {
                    let key = if i == 0 { "Q" } else { "W" };
                    let (f, w, g, s) = kind.train_cost();
                    let mut cost = Vec::new();
                    if f > 0 { cost.push(format!("{f}F")); }
                    if w > 0 { cost.push(format!("{w}W")); }
                    if g > 0 { cost.push(format!("{g}G")); }
                    if s > 0 { cost.push(format!("{s}S")); }
                    let locked = kind.required_age() > age.0;
                    if locked {
                        info += &format!("[{key}] {:?} (needs {:?})  ", kind, kind.required_age());
                    } else {
                        info += &format!("[{key}] {:?} ({})  ", kind, cost.join(" "));
                    }
                }
            }

            if building.kind == BuildingKind::TownCenter {
                info += "\n[P] Age Up";
            }

            if let Some(q) = queue {
                if !q.queue.is_empty() {
                    info += &format!("\nQueue: ");
                    for (i, slot) in q.queue.iter().enumerate() {
                        if i == 0 {
                            info += &format!("[{:?} {:.0}s]", slot.kind, slot.remaining.remaining_secs());
                        } else {
                            info += &format!(" [{:?}]", slot.kind);
                        }
                    }
                    for _ in q.queue.len()..5 {
                        info += " [---]";
                    }
                    info += "  [Esc] Cancel";
                }
            }

            **text = info;
        } else {
            **text = format!("{} buildings selected", buildings.len());
        }
        return;
    }

    let units: Vec<_> = selected_units.iter().collect();
    if units.is_empty() {
        **text = "No unit selected".to_string();
        return;
    }

    if units.len() == 1 {
        let (health, _speed, attack, armor, state, _team, carrying) = units[0];
        let state_str = format_unit_state(state, carrying);
        let atk = if attack.pierce_damage > 0.0 {
            format!("{:.0}P", attack.pierce_damage)
        } else {
            format!("{:.0}M", attack.melee_damage)
        };
        **text = format!(
            "HP: {:.0}/{:.0}  Atk: {}  Arm: {:.0}/{:.0}\n{}  [B] Build",
            health.current, health.max, atk, armor.melee, armor.pierce, state_str
        );
    } else {
        let total_hp: f32 = units.iter().map(|(h, _, _, _, _, _, _)| h.current).sum();
        let gathering: usize = units.iter().filter(|(_, _, _, _, s, _, _)| {
            matches!(s, UnitState::Gathering { .. } | UnitState::Returning { .. } | UnitState::FarmingAt { .. })
        }).count();
        let mut info = format!("{} units selected  (Total HP: {:.0})", units.len(), total_hp);
        if gathering > 0 {
            info += &format!("  ({} gathering)", gathering);
        }
        info += "  [B] Build";
        **text = info;
    }
}

fn format_unit_state(state: &UnitState, carrying: Option<&Carrying>) -> String {
    let carry_str = if let Some(c) = carrying {
        if c.has_resources() {
            let kind_name = match c.kind {
                Some(ResourceKind::Food) => "Food",
                Some(ResourceKind::Wood) => "Wood",
                Some(ResourceKind::Gold) => "Gold",
                Some(ResourceKind::Stone) => "Stone",
                None => "",
            };
            format!("  Carrying: {}/{} {}", c.amount, Carrying::MAX_CARRY, kind_name)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    match state {
        UnitState::Idle => format!("Idle{carry_str}"),
        UnitState::Moving => format!("Moving{carry_str}"),
        UnitState::Attacking => "Attacking".to_string(),
        UnitState::Gathering { .. } => {
            let kind_name = carrying.and_then(|c| c.kind).map(|k| match k {
                ResourceKind::Food => "Food",
                ResourceKind::Wood => "Wood",
                ResourceKind::Gold => "Gold",
                ResourceKind::Stone => "Stone",
            }).unwrap_or("...");
            let amt = carrying.map(|c| c.amount).unwrap_or(0);
            format!("Gathering {kind_name} ({amt}/{})", Carrying::MAX_CARRY)
        }
        UnitState::Returning { .. } => {
            let amt = carrying.map(|c| c.amount).unwrap_or(0);
            format!("Returning to drop off ({amt} resources)")
        }
        UnitState::FarmingAt { .. } => {
            let amt = carrying.map(|c| c.amount).unwrap_or(0);
            format!("Farming ({amt}/{})", Carrying::MAX_CARRY)
        }
        UnitState::Constructing { .. } => "Building...".to_string(),
        UnitState::Dead => "Dead".to_string(),
    }
}

pub fn update_age_display(
    age: Res<CurrentAge>,
    progress: Res<AgeUpProgress>,
    mut age_texts: Query<&mut Text, (With<AgeDisplay>, Without<AgeProgressText>)>,
    mut bar_vis: Query<&mut Visibility, With<AgeProgressBar>>,
    mut bar_fill: Query<&mut Node, (With<AgeProgressFill>, Without<AgeProgressBar>)>,
    mut progress_text: Query<&mut Text, (With<AgeProgressText>, Without<AgeDisplay>)>,
) {
    for mut text in &mut age_texts {
        **text = format!("{:?} Age", age.0);
    }

    if progress.researching {
        if let Some(ref timer) = progress.timer {
            let pct = timer.fraction() * 100.0;
            let remaining = timer.remaining_secs();

            for mut vis in &mut bar_vis {
                *vis = Visibility::Inherited;
            }
            for mut node in &mut bar_fill {
                node.width = Val::Percent(pct);
            }
            if let Some(target) = progress.target_age {
                for mut text in &mut progress_text {
                    **text = format!("Advancing to {:?}... {:.0}s", target, remaining);
                }
            }
        }
    } else {
        for mut vis in &mut bar_vis {
            *vis = Visibility::Hidden;
        }
        for mut text in &mut progress_text {
            **text = String::new();
        }
    }
}

pub fn update_building_panel() {
    // Merged into update_unit_info_panel
}
