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
pub struct PopText;

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

#[derive(Component)]
pub struct ActionPanel;

#[derive(Component)]
pub struct TrainButton(pub crate::units::types::UnitKind);

#[derive(Component)]
pub struct ResearchButton(pub crate::buildings::research::Technology);

#[derive(Component)]
pub struct BuildButton(pub crate::buildings::components::BuildingKind);

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
        spawn_resource_label(parent, "Pop: 0/5", PopText);

        // Idle villager button
        parent.spawn((
            IdleVillagerButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                margin: UiRect::left(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.6, 0.5, 0.1, 0.9)),
        )).with_child((
            IdleVillagerText,
            Text::new("Idle: 0"),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::WHITE),
        ));

        // Game speed display
        parent.spawn((
            GameSpeedDisplay,
            Text::new("Speed: 1.0x"),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node { margin: UiRect::left(Val::Px(16.0)), ..default() },
        ));
    });

    // Countdown display (Wonder/Relic victory timer)
    commands.spawn((
        super::game_over::CountdownDisplay,
        Text::new(""),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(1.0, 0.85, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(38.0),
            left: Val::Percent(30.0),
            right: Val::Percent(30.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));

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
    population: Res<crate::resources::components::Population>,
    mut food_q: Query<&mut Text, (With<FoodText>, Without<WoodText>, Without<GoldText>, Without<StoneText>, Without<PopText>)>,
    mut wood_q: Query<&mut Text, (With<WoodText>, Without<FoodText>, Without<GoldText>, Without<StoneText>, Without<PopText>)>,
    mut gold_q: Query<&mut Text, (With<GoldText>, Without<FoodText>, Without<WoodText>, Without<StoneText>, Without<PopText>)>,
    mut stone_q: Query<&mut Text, (With<StoneText>, Without<FoodText>, Without<WoodText>, Without<GoldText>, Without<PopText>)>,
    mut pop_q: Query<&mut Text, (With<PopText>, Without<FoodText>, Without<WoodText>, Without<GoldText>, Without<StoneText>)>,
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
    if let Ok(mut text) = pop_q.single_mut() {
        **text = format!("Pop: {}/{}", population.current, population.cap);
    }
}

pub fn update_unit_info_panel(
    selected_units: Query<(&Health, &Speed, &AttackStats, &Armor, &UnitState, &Team, Option<&Carrying>), (With<Unit>, With<Selected>)>,
    selected_buildings: Query<(&Building, &Health, Option<&TrainingQueue>, Option<&crate::buildings::research::ResearchQueue>), (With<Selected>, Without<Unit>)>,
    selected_resources: Query<&ResourceNode, (With<Selected>, Without<Unit>, Without<Building>)>,
    mut info_text: Query<&mut Text, With<UnitInfoText>>,
    age: Res<CurrentAge>,
    line_upgrades: Res<crate::buildings::research::UnitLineUpgrades>,
    researched: Res<crate::buildings::research::ResearchedTechnologies>,
    player_civ: Res<crate::civilization::PlayerCivilization>,
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
            let (building, health, queue, rq) = buildings[0];
            let mut info = format!(
                "{:?}  HP: {:.0}/{:.0}",
                building.kind, health.current, health.max
            );

            let trainable = building.kind.can_train();
            let train_keys = ["Q", "W", "E"];
            let available_trainable: Vec<_> = trainable.iter()
                .map(|base| line_upgrades.current_version(*base))
                .filter(|kind| kind.required_age() <= age.0)
                .collect();
            if !available_trainable.is_empty() {
                info += "\n";
                for (i, kind) in available_trainable.iter().enumerate() {
                    let key = train_keys.get(i).unwrap_or(&"?");
                    let (f, w, g, s) = kind.train_cost();
                    let mut cost = Vec::new();
                    if f > 0 { cost.push(format!("{f}F")); }
                    if w > 0 { cost.push(format!("{w}W")); }
                    if g > 0 { cost.push(format!("{g}G")); }
                    if s > 0 { cost.push(format!("{s}S")); }
                    info += &format!("[{key}] {:?} ({})  ", kind, cost.join(" "));
                }
            }

            let disabled = player_civ.0.disabled_techs();
            let avail_techs: Vec<_> = crate::buildings::research::available_techs(building.kind, &researched, &age)
                .into_iter()
                .filter(|t| !disabled.contains(t))
                .collect();
            if !avail_techs.is_empty() {
                let upgrade_keys = if !trainable.is_empty() {
                    &["Z", "X", "C", "V"][..]
                } else {
                    &["Q", "W", "E", "R"][..]
                };
                info += "\n";
                for (i, tech) in avail_techs.iter().enumerate() {
                    let key = upgrade_keys.get(i).unwrap_or(&"?");
                    let (f, w, g, s) = tech.cost();
                    let mut cost = Vec::new();
                    if f > 0 { cost.push(format!("{f}F")); }
                    if w > 0 { cost.push(format!("{w}W")); }
                    if g > 0 { cost.push(format!("{g}G")); }
                    if s > 0 { cost.push(format!("{s}S")); }
                    info += &format!("[{key}] {:?} ({})  ", tech, cost.join(" "));
                }
            }

            if building.kind == BuildingKind::TownCenter {
                info += "\n[P] Age Up";
            }

            if let Some(q) = queue {
                if !q.queue.is_empty() {
                    info += "\nTrain: ";
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

            if let Some(rqueue) = rq {
                if !rqueue.queue.is_empty() {
                    info += "\nResearch: ";
                    for (i, slot) in rqueue.queue.iter().enumerate() {
                        if i == 0 {
                            info += &format!("[{:?} {:.0}s]", slot.tech, slot.remaining.remaining_secs());
                        } else {
                            info += &format!(" [{:?}]", slot.tech);
                        }
                    }
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
            "HP: {:.0}/{:.0}  Atk: {}  Arm: {:.0}/{:.0}\n{}",
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
            format!("  Carrying: {}/{} {}", c.amount, c.max_carry, kind_name)
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
            format!("Gathering {kind_name} ({amt}/{})", Carrying::BASE_CARRY)
        }
        UnitState::Returning { .. } => {
            let amt = carrying.map(|c| c.amount).unwrap_or(0);
            format!("Returning to drop off ({amt} resources)")
        }
        UnitState::FarmingAt { .. } => {
            let amt = carrying.map(|c| c.amount).unwrap_or(0);
            format!("Farming ({amt}/{})", Carrying::BASE_CARRY)
        }
        UnitState::Constructing { .. } => "Building...".to_string(),
        UnitState::Repairing { .. } => "Repairing...".to_string(),
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

pub fn button_hover_system(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, Or<(With<TrainButton>, With<ResearchButton>, With<BuildButton>)>)>,
) {
    for (interaction, mut bg) in &mut buttons {
        *bg = match *interaction {
            Interaction::Hovered => BackgroundColor(Color::srgba(0.35, 0.35, 0.5, 0.95)),
            Interaction::Pressed => BackgroundColor(Color::srgba(0.5, 0.5, 0.7, 1.0)),
            Interaction::None => BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.9)),
        };
    }
}

pub fn rebuild_action_panel(
    mut commands: Commands,
    existing_panels: Query<Entity, With<ActionPanel>>,
    selected_buildings: Query<&Building, (With<Selected>, Without<Unit>)>,
    selected_units: Query<&crate::units::types::UnitKind, (With<Unit>, With<Selected>)>,
    age: Res<CurrentAge>,
    resources: Res<PlayerResources>,
    line_upgrades: Res<crate::buildings::research::UnitLineUpgrades>,
    researched: Res<crate::buildings::research::ResearchedTechnologies>,
    player_civ: Res<crate::civilization::PlayerCivilization>,
) {
    for entity in &existing_panels {
        commands.entity(entity).despawn();
    }

    let has_villager = selected_units.iter().any(|k| *k == crate::units::types::UnitKind::Villager);

    let buildings: Vec<_> = selected_buildings.iter().collect();
    let has_building = buildings.len() == 1;

    if !has_villager && !has_building { return; }

    commands.spawn((
        ActionPanel,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(430.0),
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            max_height: Val::Vh(40.0),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.92)),
    )).with_children(|parent| {
        if has_villager {
            spawn_build_buttons(parent, &age, &resources);
        }

        if let Some(building) = buildings.first() {
            let trainable: Vec<crate::units::types::UnitKind> = building.kind.can_train().iter()
                .map(|base| line_upgrades.current_version(*base))
                .filter(|kind| kind.required_age() <= age.0)
                .collect();

            if !trainable.is_empty() {
                parent.spawn((
                    Text::new("Train:"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                ));

                let train_keys = ["Q", "W", "E"];
                for (i, kind) in trainable.iter().enumerate() {
                    let key = train_keys.get(i).unwrap_or(&"?");
                    let (f, w, g, s) = kind.train_cost();
                    let mut cost = Vec::new();
                    if f > 0 { cost.push(format!("{f}F")); }
                    if w > 0 { cost.push(format!("{w}W")); }
                    if g > 0 { cost.push(format!("{g}G")); }
                    if s > 0 { cost.push(format!("{s}S")); }

                    parent.spawn((
                        TrainButton(*kind),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            margin: UiRect::top(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.9)),
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("[{key}] {:?}  ({})", kind, cost.join(" "))),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(Color::srgb(0.85, 0.85, 0.85)),
                        ));
                    });
                }
            }

            let disabled = player_civ.0.disabled_techs();
            let avail_techs: Vec<crate::buildings::research::Technology> =
                crate::buildings::research::available_techs(building.kind, &researched, &age)
                    .into_iter()
                    .filter(|t| !disabled.contains(t))
                    .collect();

            if !avail_techs.is_empty() {
                parent.spawn((
                    Text::new("Research:"),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                ));

                let has_training = !building.kind.can_train().is_empty();
                let upgrade_keys: &[&str] = if has_training {
                    &["Z", "X", "C", "V"]
                } else {
                    &["Q", "W", "E", "R"]
                };

                for (i, tech) in avail_techs.iter().enumerate() {
                    let key = upgrade_keys.get(i).unwrap_or(&"?");
                    let (f, w, g, s) = tech.cost();
                    let mut cost = Vec::new();
                    if f > 0 { cost.push(format!("{f}F")); }
                    if w > 0 { cost.push(format!("{w}W")); }
                    if g > 0 { cost.push(format!("{g}G")); }
                    if s > 0 { cost.push(format!("{s}S")); }

                    parent.spawn((
                        ResearchButton(*tech),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            margin: UiRect::top(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.25, 0.2, 0.9)),
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("[{key}] {:?}  ({})", tech, cost.join(" "))),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(Color::srgb(0.85, 0.85, 0.85)),
                        ));
                    });
                }
            }
        }
    });
}

fn spawn_build_buttons(
    parent: &mut ChildSpawnerCommands,
    age: &CurrentAge,
    resources: &PlayerResources,
) {
    use crate::buildings::placement::BUILDABLE_KINDS;

    parent.spawn((
        Text::new("Build:"),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgb(1.0, 0.85, 0.3)),
    ));

    let labels = [
        ("1", "House"),
        ("2", "Barracks"),
        ("3", "Archery Range"),
        ("4", "Stable"),
        ("5", "Lumber Camp"),
        ("6", "Mining Camp"),
        ("7", "Farm"),
        ("8", "Watch Tower"),
        ("9", "Palisade Wall"),
        ("0", "Stone Wall"),
        ("-", "Gate"),
        ("=", "Siege Workshop"),
        ("Bksp", "Blacksmith"),
        ("[", "University"),
        ("]", "Market"),
        ("\\", "Monastery"),
        (";", "Castle"),
        ("'", "Dock"),
        ("T", "Town Center"),
        ("Y", "Wonder"),
    ];

    for (i, &(key_label, name)) in labels.iter().enumerate() {
        let kind = BUILDABLE_KINDS[i].1;
        if kind.required_age() > age.0 {
            continue;
        }

        let (f, w, g, s) = kind.build_cost();
        let too_expensive = !resources.can_afford(f, w, g, s);

        let mut cost_parts: Vec<String> = Vec::new();
        if f > 0 { cost_parts.push(format!("{f}F")); }
        if w > 0 { cost_parts.push(format!("{w}W")); }
        if g > 0 { cost_parts.push(format!("{g}G")); }
        if s > 0 { cost_parts.push(format!("{s}S")); }

        let text_color = if too_expensive {
            Color::srgb(0.9, 0.3, 0.3)
        } else {
            Color::srgb(0.85, 0.85, 0.85)
        };

        parent.spawn((
            BuildButton(kind),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                margin: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.9)),
        )).with_children(|btn| {
            btn.spawn((
                Text::new(format!("[{key_label}] {name}  ({})", cost_parts.join(" "))),
                TextFont { font_size: 13.0, ..default() },
                TextColor(text_color),
            ));
        });
    }
}

pub fn handle_train_button_clicks(
    interactions: Query<(&Interaction, &TrainButton), Changed<Interaction>>,
    mut buildings_selected: Query<(&Building, &mut TrainingQueue, &Team), With<Selected>>,
    mut resources: ResMut<PlayerResources>,
    age: Res<CurrentAge>,
    population: Res<crate::resources::components::Population>,
) {
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }

        for (_building, mut queue, team) in &mut buildings_selected {
            if team.0 != 0 { continue; }
            let kind = btn.0;
            if queue.queue.len() >= 5 { break; }
            if age.0 < kind.required_age() { break; }
            let queued_pop: u32 = queue.queue.iter().map(|s| s.kind.population_cost()).sum();
            if !population.has_room(queued_pop + kind.population_cost()) { break; }
            let (food, wood, gold, stone) = kind.train_cost();
            if !resources.spend(food, wood, gold, stone) { break; }
            queue.queue.push(TrainingSlot {
                kind,
                remaining: Timer::from_seconds(kind.train_time(), TimerMode::Once),
            });
            break;
        }
    }
}

pub fn handle_research_button_clicks(
    interactions: Query<(&Interaction, &ResearchButton), Changed<Interaction>>,
    mut buildings_selected: Query<(&Building, &Team, Option<&mut crate::buildings::research::ResearchQueue>), With<Selected>>,
    mut resources: ResMut<PlayerResources>,
) {
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }

        let tech = btn.0;
        let (f, w, g, s) = tech.cost();

        for (_building, team, queue) in &mut buildings_selected {
            if team.0 != 0 { continue; }
            if !resources.spend(f, w, g, s) { continue; }

            if let Some(mut rq) = queue {
                rq.queue.push(crate::buildings::research::ResearchSlot {
                    tech,
                    remaining: Timer::from_seconds(tech.research_time(), TimerMode::Once),
                });
            }
            break;
        }
    }
}

pub fn handle_build_button_clicks(
    interactions: Query<(&Interaction, &BuildButton), Changed<Interaction>>,
    mut placement: ResMut<crate::buildings::placement::PlacementMode>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    age: Res<CurrentAge>,
) {
    use crate::buildings::placement::GhostBuilding;
    use crate::buildings::load_building_texture;
    use crate::map::TILE_SIZE;

    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed { continue; }

        let kind = btn.0;
        if kind.required_age() > age.0 { continue; }

        if let Some(old_ghost) = placement.ghost {
            commands.entity(old_ghost).despawn();
        }
        if let Some(menu) = placement.menu_entity {
            commands.entity(menu).despawn();
            placement.menu_entity = None;
        }

        placement.active = true;
        placement.just_activated = true;
        placement.kind = Some(kind);
        let (tw, th) = kind.tile_size();
        let iso_tile_h = TILE_SIZE / 2.0;
        let pixel_w = (tw as f32 * TILE_SIZE) as u32;
        let pixel_h = (th as f32 * iso_tile_h) as u32;

        let (texture, actual_dims) = load_building_texture(&mut images, kind, pixel_w, pixel_h);
        let display_size = if let Some((sw, sh)) = actual_dims {
            let aspect = sh as f32 / sw as f32;
            Vec2::new(pixel_w as f32, pixel_w as f32 * aspect)
        } else {
            Vec2::new(pixel_w as f32, pixel_h as f32)
        };

        let ghost = commands.spawn((
            GhostBuilding,
            Sprite {
                image: texture,
                custom_size: Some(display_size),
                color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 15.0),
        )).id();
        placement.ghost = Some(ghost);
    }
}

// --- Game Speed Controls ---

#[derive(Component)]
pub struct GameSpeedDisplay;

pub fn game_speed_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut speed: ResMut<crate::GameSpeed>,
    mut time_settings: ResMut<Time<Virtual>>,
) {
    let mut changed = false;
    if keys.just_pressed(KeyCode::NumpadAdd) || keys.just_pressed(KeyCode::BracketRight) {
        speed.0 = (speed.0 + crate::GameSpeed::STEP).min(crate::GameSpeed::MAX);
        changed = true;
    }
    if keys.just_pressed(KeyCode::NumpadSubtract) || keys.just_pressed(KeyCode::BracketLeft) {
        speed.0 = (speed.0 - crate::GameSpeed::STEP).max(crate::GameSpeed::MIN);
        changed = true;
    }
    if keys.just_pressed(KeyCode::Pause) || keys.just_pressed(KeyCode::F3) {
        if speed.0 > 0.0 {
            speed.0 = 0.0;
        } else {
            speed.0 = 1.0;
        }
        changed = true;
    }
    if changed {
        time_settings.set_relative_speed(speed.0);
    }
}

pub fn update_game_speed_display(
    speed: Res<crate::GameSpeed>,
    mut text_q: Query<&mut Text, With<GameSpeedDisplay>>,
) {
    if !speed.is_changed() { return; }
    for mut text in &mut text_q {
        if speed.0 == 0.0 {
            **text = "PAUSED".to_string();
        } else {
            **text = format!("Speed: {:.1}x", speed.0);
        }
    }
}

// --- Idle Villager Alert ---

#[derive(Component)]
pub struct IdleVillagerButton;

#[derive(Component)]
pub struct IdleVillagerText;

#[derive(Resource, Default)]
pub struct IdleVillagerCycle {
    pub last_index: usize,
}

pub fn idle_villager_system(
    keys: Res<ButtonInput<KeyCode>>,
    btn_q: Query<&Interaction, (With<IdleVillagerButton>, Changed<Interaction>)>,
    mut commands: Commands,
    mut cycle: ResMut<IdleVillagerCycle>,
    idle_villagers: Query<(Entity, &Transform, &Team, &UnitState, &crate::units::types::UnitKind), With<Unit>>,
    mut camera_q: Query<&mut Transform, (With<crate::camera::MainCamera>, Without<Unit>)>,
) {
    let mut clicked = false;
    for interaction in &btn_q {
        if *interaction == Interaction::Pressed {
            clicked = true;
        }
    }
    if keys.just_pressed(KeyCode::Period) {
        clicked = true;
    }
    if !clicked { return; }

    let mut idle: Vec<Entity> = idle_villagers.iter()
        .filter(|(_, _, t, state, kind)| {
            t.0 == 0
            && **kind == crate::units::types::UnitKind::Villager
            && matches!(state, UnitState::Idle)
        })
        .map(|(e, _, _, _, _)| e)
        .collect();

    if idle.is_empty() { return; }
    idle.sort();

    let idx = cycle.last_index % idle.len();
    let target = idle[idx];
    cycle.last_index = idx + 1;

    if let Ok((_, tf, _, _, _)) = idle_villagers.get(target) {
        commands.entity(target).insert(Selected);
        if let Ok(mut cam_tf) = camera_q.single_mut() {
            cam_tf.translation.x = tf.translation.x;
            cam_tf.translation.y = tf.translation.y;
        }
    }
}

pub fn update_idle_villager_count(
    mut text_q: Query<&mut Text, With<IdleVillagerText>>,
    villagers: Query<(&Team, &UnitState, &crate::units::types::UnitKind), With<Unit>>,
) {
    let count = villagers.iter()
        .filter(|(t, state, kind)| {
            t.0 == 0
            && **kind == crate::units::types::UnitKind::Villager
            && matches!(state, UnitState::Idle)
        })
        .count();

    for mut text in &mut text_q {
        **text = format!("Idle: {count}");
    }
}
