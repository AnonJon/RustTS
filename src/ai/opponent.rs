use bevy::prelude::*;
use rand::Rng;
use crate::map::{GridPosition, TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::map::generation::MapConfig;
use crate::units::components::*;
use crate::units::types::*;
use crate::buildings::components::*;
use crate::buildings::spawn_building;
use super::behaviors::*;

pub const AI_TEAM: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiDifficulty {
    Easy,
    Medium,
    Hard,
}

impl Default for AiDifficulty {
    fn default() -> Self { Self::Medium }
}

#[derive(Resource)]
pub struct AiState {
    pub difficulty: AiDifficulty,
    pub resources: AiResources,
    pub age: Age,
    pub gather_timer: Timer,
    pub build_timer: Timer,
    pub train_timer: Timer,
    pub attack_timer: Timer,
    pub scout_timer: Timer,
    pub defense_timer: Timer,
    pub attack_wave_size: usize,
    pub scouted: bool,
    pub last_player_composition: PlayerComposition,
}

impl Default for AiState {
    fn default() -> Self {
        Self::with_difficulty(AiDifficulty::Medium)
    }
}

impl AiState {
    pub fn with_difficulty(difficulty: AiDifficulty) -> Self {
        let (gather_s, build_s, train_s, attack_s, wave) = match difficulty {
            AiDifficulty::Easy => (4.0, 25.0, 14.0, 150.0, 2),
            AiDifficulty::Medium => (3.0, 15.0, 8.0, 90.0, 3),
            AiDifficulty::Hard => (2.0, 10.0, 5.0, 60.0, 5),
        };
        Self {
            difficulty,
            resources: AiResources { food: 200, wood: 200, gold: 100, stone: 50 },
            age: Age::Dark,
            gather_timer: Timer::from_seconds(gather_s, TimerMode::Repeating),
            build_timer: Timer::from_seconds(build_s, TimerMode::Repeating),
            train_timer: Timer::from_seconds(train_s, TimerMode::Repeating),
            attack_timer: Timer::from_seconds(attack_s, TimerMode::Repeating),
            scout_timer: Timer::from_seconds(20.0, TimerMode::Once),
            defense_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            attack_wave_size: wave,
            scouted: false,
            last_player_composition: PlayerComposition::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct PlayerComposition {
    pub infantry: u32,
    pub archers: u32,
    pub cavalry: u32,
    pub siege: u32,
}

#[derive(Debug, Clone)]
pub struct AiResources {
    pub food: u32,
    pub wood: u32,
    pub gold: u32,
    pub stone: u32,
}

impl AiResources {
    pub fn can_afford(&self, food: u32, wood: u32, gold: u32, stone: u32) -> bool {
        self.food >= food && self.wood >= wood && self.gold >= gold && self.stone >= stone
    }

    pub fn spend(&mut self, food: u32, wood: u32, gold: u32, stone: u32) -> bool {
        if self.can_afford(food, wood, gold, stone) {
            self.food -= food;
            self.wood -= wood;
            self.gold -= gold;
            self.stone -= stone;
            true
        } else {
            false
        }
    }
}

pub fn ai_startup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<MapConfig>,
    sprites: Res<UnitSprites>,
    settings: Res<crate::GameSettings>,
) {
    let ai_base = config.ai_base();
    let (tw, th) = BuildingKind::TownCenter.tile_size();
    let tc_pos = crate::map::generation::nudge_building_onto_land(ai_base, tw, th, &config.terrain_grid);

    spawn_building(
        &mut commands,
        &mut images,
        BuildingKind::TownCenter,
        Team(AI_TEAM),
        tc_pos,
        false,
    );

    let offsets = [
        (ai_base.x - 1, ai_base.y - 1),
        (ai_base.x + 4, ai_base.y),
        (ai_base.x, ai_base.y + 4),
    ];
    for (x, y) in offsets {
        let (lx, ly) = crate::map::generation::find_nearest_land(&config.terrain_grid, x, y);
        let grid = GridPosition::new(lx, ly);
        let world = grid.to_world();
        spawn_unit(&mut commands, sprites.get(UnitKind::Villager), UnitKind::Villager, Team(AI_TEAM), grid, world);
    }

    if settings.game_mode == crate::GameMode::Regicide {
        let (lx, ly) = crate::map::generation::find_nearest_land(&config.terrain_grid, ai_base.x + 2, ai_base.y + 2);
        let grid = GridPosition::new(lx, ly);
        let world = grid.to_world();
        spawn_unit(&mut commands, sprites.get(UnitKind::King), UnitKind::King, Team(AI_TEAM), grid, world);
    }
}

pub fn ai_economy_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    ai_villagers: Query<(&Team, &UnitKind), With<Unit>>,
) {
    ai.gather_timer.tick(time.delta());
    if !ai.gather_timer.just_finished() {
        return;
    }

    let villager_count = ai_villagers.iter()
        .filter(|(t, k)| t.0 == AI_TEAM && **k == UnitKind::Villager)
        .count() as u32;

    let age_mult = match ai.age {
        Age::Dark => 1.0_f32,
        Age::Feudal => 1.3,
        Age::Castle => 1.6,
        Age::Imperial => 2.0,
    };
    let difficulty_mult = match ai.difficulty {
        AiDifficulty::Easy => 0.7,
        AiDifficulty::Medium => 1.0,
        AiDifficulty::Hard => 1.5,
    };

    let base_food = 8.0 * age_mult * difficulty_mult;
    let base_wood = 5.0 * age_mult * difficulty_mult;
    let base_gold = 2.0 * age_mult * difficulty_mult;
    let base_stone = 0.5 * age_mult * difficulty_mult;

    ai.resources.food += (villager_count as f32 * base_food) as u32;
    ai.resources.wood += (villager_count as f32 * base_wood) as u32;
    ai.resources.gold += (villager_count as f32 * base_gold) as u32;
    ai.resources.stone += (villager_count as f32 * base_stone) as u32;
}

pub fn ai_build_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    ai_buildings: Query<(&Building, &Team)>,
    config: Res<MapConfig>,
    ai_villagers: Query<(Entity, &Transform, &Team, &UnitState), (With<Unit>, With<crate::resources::components::Carrying>)>,
) {
    ai.build_timer.tick(time.delta());
    if !ai.build_timer.just_finished() {
        return;
    }

    let ai_base = config.ai_base();

    let mut counts: std::collections::HashMap<BuildingKind, u32> = std::collections::HashMap::new();
    for (building, team) in &ai_buildings {
        if team.0 != AI_TEAM { continue; }
        *counts.entry(building.kind).or_insert(0) += 1;
    }

    let has = |k: BuildingKind| counts.get(&k).copied().unwrap_or(0) > 0;
    let count_of = |k: BuildingKind| counts.get(&k).copied().unwrap_or(0);
    let total_buildings: u32 = counts.values().sum();
    let house_count = count_of(BuildingKind::House);

    let kind_to_build = if house_count < 2 {
        Some(BuildingKind::House)
    } else if !has(BuildingKind::LumberCamp) {
        Some(BuildingKind::LumberCamp)
    } else if !has(BuildingKind::Mill) {
        Some(BuildingKind::Mill)
    } else if !has(BuildingKind::Barracks) {
        Some(BuildingKind::Barracks)
    } else if !has(BuildingKind::MiningCamp) {
        Some(BuildingKind::MiningCamp)
    } else if ai.age >= Age::Feudal && !has(BuildingKind::ArcheryRange) {
        Some(BuildingKind::ArcheryRange)
    } else if ai.age >= Age::Feudal && !has(BuildingKind::Blacksmith) {
        Some(BuildingKind::Blacksmith)
    } else if ai.age >= Age::Castle && !has(BuildingKind::Stable) {
        Some(BuildingKind::Stable)
    } else if ai.age >= Age::Feudal && house_count < 4 {
        Some(BuildingKind::House)
    } else if ai.age >= Age::Castle && !has(BuildingKind::University) {
        Some(BuildingKind::University)
    } else if ai.age >= Age::Castle && house_count < 6 {
        Some(BuildingKind::House)
    } else if ai.age >= Age::Imperial && !has(BuildingKind::Castle) {
        Some(BuildingKind::Castle)
    } else if ai.age >= Age::Castle && count_of(BuildingKind::Farm) < 6 {
        Some(BuildingKind::Farm)
    } else if ai.age >= Age::Imperial && house_count < 10 {
        Some(BuildingKind::House)
    } else {
        None
    };

    if let Some(kind) = kind_to_build {
        try_ai_build(&mut ai, &mut commands, &mut images, kind, ai_base, &config, &ai_villagers);
        return;
    }

    if total_buildings > 3 && ai.age < Age::Imperial {
        if let Some((cost_f, cost_w, cost_g, cost_s)) = ai.age.advance_cost() {
            if ai.resources.spend(cost_f, cost_w, cost_g, cost_s) {
                if let Some(next) = ai.age.next() {
                    ai.age = next;
                }
            }
        }
    }
}

fn try_ai_build(
    ai: &mut AiState,
    commands: &mut Commands,
    images: &mut Assets<Image>,
    kind: BuildingKind,
    ai_base: GridPosition,
    config: &MapConfig,
    ai_villagers: &Query<(Entity, &Transform, &Team, &UnitState), (With<Unit>, With<crate::resources::components::Carrying>)>,
) {
    let mut rng = rand::rng();
    let (tw, th) = kind.tile_size();
    let (f, w, g, s) = kind.build_cost();

    if !ai.resources.can_afford(f, w, g, s) { return; }

    for _ in 0..40 {
        let offset_x = rng.random_range(-8..=8);
        let offset_y = rng.random_range(-8..=8);
        let grid = GridPosition::new(
            (ai_base.x + offset_x).clamp(2, MAP_WIDTH as i32 - 4),
            (ai_base.y + offset_y).clamp(2, MAP_HEIGHT as i32 - 4),
        );

        if crate::map::generation::building_footprint_has_water(grid, tw, th, &config.terrain_grid) {
            continue;
        }

        if ai.resources.spend(f, w, g, s) {
            let building_entity = spawn_building(commands, images, kind, Team(AI_TEAM), grid, true);
            let build_site = grid.to_world();

            let nearest_villager = ai_villagers
                .iter()
                .filter(|(_, _, t, state)| {
                    t.0 == AI_TEAM && !matches!(state, UnitState::Constructing { .. })
                })
                .min_by(|(_, a_tf, _, _), (_, b_tf, _, _)| {
                    let da = a_tf.translation.truncate().distance_squared(build_site);
                    let db = b_tf.translation.truncate().distance_squared(build_site);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(e, _, _, _)| e);

            if let Some(villager_e) = nearest_villager {
                commands.entity(villager_e).insert((
                    ConstructTarget(building_entity),
                    MoveTarget(build_site),
                    UnitState::Constructing { building: building_entity },
                ));
            }
        }
        return;
    }
}

pub fn ai_scout_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    mut commands: Commands,
    sprites: Res<UnitSprites>,
    config: Res<MapConfig>,
    ai_buildings: Query<(&Building, &Transform, &Team)>,
) {
    ai.scout_timer.tick(time.delta());
    if !ai.scout_timer.just_finished() || ai.scouted {
        return;
    }
    ai.scouted = true;

    let ai_base = config.ai_base();

    let Some(tc_tf) = ai_buildings.iter()
        .find(|(b, _, t)| t.0 == AI_TEAM && b.kind == BuildingKind::TownCenter)
        .map(|(_, tf, _)| tf.translation.truncate()) else { return };

    let spawn_pos = tc_tf + Vec2::new(0.0, -TILE_SIZE * 2.0);
    let grid = GridPosition::from_world(spawn_pos);

    let (f, w, g, s) = UnitKind::ScoutCavalry.train_cost();
    if !ai.resources.spend(f, w, g, s) { return; }

    let scout = spawn_unit(&mut commands, sprites.get(UnitKind::ScoutCavalry), UnitKind::ScoutCavalry, Team(AI_TEAM), grid, spawn_pos);

    let map_center = GridPosition::new(MAP_WIDTH as i32 / 2, MAP_HEIGHT as i32 / 2).to_world();
    let player_region = GridPosition::new(
        (ai_base.x as i32).max(MAP_WIDTH as i32 - ai_base.x) / 2,
        (ai_base.y as i32).max(MAP_HEIGHT as i32 - ai_base.y) / 2,
    ).to_world();

    commands.entity(scout).insert((
        AiBehavior::Patrol,
        PatrolPath {
            waypoints: vec![spawn_pos, map_center, player_region, spawn_pos],
            current_index: 0,
        },
        DetectionRadius(14.0),
    ));
}

pub fn ai_analyze_player_system(
    mut ai: ResMut<AiState>,
    player_units: Query<(&Team, &UnitKind, &AttackStats), With<Unit>>,
) {
    let mut comp = PlayerComposition::default();
    for (team, kind, _stats) in &player_units {
        if team.0 != 0 { continue; }
        match kind {
            UnitKind::Militia | UnitKind::ManAtArms | UnitKind::LongSwordsman
            | UnitKind::TwoHandedSwordsman | UnitKind::Champion
            | UnitKind::Spearman | UnitKind::Pikeman | UnitKind::Halberdier => {
                comp.infantry += 1;
            }
            UnitKind::Archer | UnitKind::Crossbowman | UnitKind::Arbalester
            | UnitKind::Skirmisher | UnitKind::EliteSkirmisher
            | UnitKind::Longbowman | UnitKind::EliteLongbowman => {
                comp.archers += 1;
            }
            UnitKind::ScoutCavalry | UnitKind::LightCavalry | UnitKind::Hussar
            | UnitKind::Knight | UnitKind::Cavalier | UnitKind::Paladin
            | UnitKind::Mangudai | UnitKind::EliteMangudai => {
                comp.cavalry += 1;
            }
            UnitKind::BatteringRam | UnitKind::Mangonel | UnitKind::Scorpion => {
                comp.siege += 1;
            }
            _ => {}
        }
    }
    ai.last_player_composition = comp;
}

fn pick_counter_unit(ai: &AiState) -> UnitKind {
    let comp = &ai.last_player_composition;
    let total = comp.infantry + comp.archers + comp.cavalry + comp.siege;

    if total == 0 {
        return match ai.age {
            Age::Dark => UnitKind::Militia,
            Age::Feudal => UnitKind::ManAtArms,
            _ => UnitKind::Knight,
        };
    }

    if comp.archers > comp.infantry && comp.archers > comp.cavalry {
        if ai.age >= Age::Castle { return UnitKind::Knight; }
        return UnitKind::Skirmisher;
    }

    if comp.cavalry > comp.infantry && comp.cavalry > comp.archers {
        if ai.age >= Age::Castle { return UnitKind::Pikeman; }
        return UnitKind::Spearman;
    }

    if comp.infantry >= comp.archers && comp.infantry >= comp.cavalry {
        if ai.age >= Age::Feudal { return UnitKind::Archer; }
        return UnitKind::Militia;
    }

    UnitKind::Militia
}

fn unit_building(kind: UnitKind) -> BuildingKind {
    match kind {
        UnitKind::Villager => BuildingKind::TownCenter,
        UnitKind::Militia | UnitKind::ManAtArms | UnitKind::Spearman
        | UnitKind::Pikeman | UnitKind::LongSwordsman => BuildingKind::Barracks,
        UnitKind::Archer | UnitKind::Skirmisher | UnitKind::Crossbowman => BuildingKind::ArcheryRange,
        UnitKind::ScoutCavalry | UnitKind::Knight | UnitKind::LightCavalry
        | UnitKind::Cavalier | UnitKind::Paladin => BuildingKind::Stable,
        _ => BuildingKind::Barracks,
    }
}

pub fn ai_train_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    mut commands: Commands,
    sprites: Res<UnitSprites>,
    ai_buildings: Query<(&Building, &Transform, &Team)>,
    ai_units: Query<(&Team, &UnitKind), With<Unit>>,
    player_buildings: Query<(&Building, &Transform, &Team), Without<Unit>>,
) {
    ai.train_timer.tick(time.delta());
    if !ai.train_timer.just_finished() {
        return;
    }

    let player_tc_pos = find_player_tc(&player_buildings);
    let villager_count = ai_units.iter()
        .filter(|(t, k)| t.0 == AI_TEAM && **k == UnitKind::Villager)
        .count();

    let target_villagers = match ai.difficulty {
        AiDifficulty::Easy => 5,
        AiDifficulty::Medium => 8,
        AiDifficulty::Hard => 12,
    };

    // Train villagers if below target
    if villager_count < target_villagers {
        for (building, transform, team) in &ai_buildings {
            if team.0 != AI_TEAM || building.kind != BuildingKind::TownCenter { continue; }
            let (f, w, g, s) = UnitKind::Villager.train_cost();
            if ai.resources.spend(f, w, g, s) {
                let spawn_pos = transform.translation.truncate() + Vec2::new(0.0, -TILE_SIZE * 2.0);
                let grid = GridPosition::from_world(spawn_pos);
                spawn_unit(&mut commands, sprites.get(UnitKind::Villager), UnitKind::Villager, Team(AI_TEAM), grid, spawn_pos);
            }
            break;
        }
    }

    // Train military - pick counter unit
    let counter = pick_counter_unit(&ai);
    let needed_building = unit_building(counter);

    for (building, transform, team) in &ai_buildings {
        if team.0 != AI_TEAM { continue; }

        let spawn_pos = transform.translation.truncate() + Vec2::new(0.0, -TILE_SIZE * 2.0);
        let grid = GridPosition::from_world(spawn_pos);

        if building.kind == needed_building {
            let (f, w, g, s) = counter.train_cost();
            if ai.resources.spend(f, w, g, s) {
                let e = spawn_unit(&mut commands, sprites.get(counter), counter, Team(AI_TEAM), grid, spawn_pos);
                commands.entity(e).insert((
                    AiBehavior::Patrol,
                    PatrolPath {
                        waypoints: vec![spawn_pos, player_tc_pos],
                        current_index: 0,
                    },
                    DetectionRadius(10.0),
                ));
            }
            continue;
        }

        // Also train from other military buildings if we can
        let trainable = match building.kind {
            BuildingKind::Barracks if ai.age >= Age::Feudal => Some(UnitKind::ManAtArms),
            BuildingKind::Barracks => Some(UnitKind::Militia),
            BuildingKind::ArcheryRange if ai.age >= Age::Feudal => Some(UnitKind::Archer),
            BuildingKind::Stable if ai.age >= Age::Castle => Some(UnitKind::Knight),
            _ => None,
        };

        if let Some(kind) = trainable {
            let (f, w, g, s) = kind.train_cost();
            if ai.resources.spend(f, w, g, s) {
                let e = spawn_unit(&mut commands, sprites.get(kind), kind, Team(AI_TEAM), grid, spawn_pos);
                commands.entity(e).insert((
                    AiBehavior::Patrol,
                    PatrolPath {
                        waypoints: vec![spawn_pos, player_tc_pos],
                        current_index: 0,
                    },
                    DetectionRadius(10.0),
                ));
            }
        }
    }
}

pub fn ai_defense_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    mut commands: Commands,
    ai_buildings: Query<(&Building, &Transform, &Team, &Health)>,
    ai_military: Query<(Entity, &Transform, &Team, &UnitState), (With<Unit>, With<AiBehavior>)>,
) {
    ai.defense_timer.tick(time.delta());
    if !ai.defense_timer.just_finished() {
        return;
    }

    // Find damaged AI buildings (being attacked)
    let mut threatened_pos: Option<Vec2> = None;
    for (_building, transform, team, health) in &ai_buildings {
        if team.0 != AI_TEAM { continue; }
        if health.fraction() < 0.9 && health.fraction() > 0.0 {
            threatened_pos = Some(transform.translation.truncate());
            break;
        }
    }

    let Some(target) = threatened_pos else { return };

    let mut defenders_sent = 0;
    let max_defenders = match ai.difficulty {
        AiDifficulty::Easy => 2,
        AiDifficulty::Medium => 4,
        AiDifficulty::Hard => 8,
    };

    for (entity, _transform, team, state) in &ai_military {
        if team.0 != AI_TEAM { continue; }
        if defenders_sent >= max_defenders { break; }
        if matches!(state, UnitState::Idle | UnitState::Moving) {
            commands.entity(entity)
                .insert(MoveTarget(target))
                .insert(UnitState::Moving);
            defenders_sent += 1;
        }
    }
}

pub fn ai_rebuild_system(
    mut ai: ResMut<AiState>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    ai_buildings: Query<(&Building, &Team)>,
    config: Res<MapConfig>,
    ai_villagers: Query<(Entity, &Transform, &Team, &UnitState), (With<Unit>, With<crate::resources::components::Carrying>)>,
) {
    let ai_base = config.ai_base();

    let mut has_tc = false;
    let mut has_barracks = false;
    for (building, team) in &ai_buildings {
        if team.0 != AI_TEAM { continue; }
        match building.kind {
            BuildingKind::TownCenter => has_tc = true,
            BuildingKind::Barracks => has_barracks = true,
            _ => {}
        }
    }

    // Rebuild critical buildings
    if !has_tc {
        try_ai_build(&mut ai, &mut commands, &mut images, BuildingKind::TownCenter, ai_base, &config, &ai_villagers);
    } else if !has_barracks {
        try_ai_build(&mut ai, &mut commands, &mut images, BuildingKind::Barracks, ai_base, &config, &ai_villagers);
    }
}

pub fn ai_military_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    mut commands: Commands,
    ai_military: Query<(Entity, &Team, &UnitState), (With<Unit>, With<AiBehavior>)>,
    player_buildings: Query<(Entity, &Transform, &Team), With<Building>>,
) {
    ai.attack_timer.tick(time.delta());
    if !ai.attack_timer.just_finished() {
        return;
    }

    // Target priority: military buildings > production > TC
    let mut target_pos: Option<Vec2> = None;
    let mut target_priority = 0u8;

    for (_entity, transform, team) in &player_buildings {
        if team.0 != 0 { continue; }
        let pos = transform.translation.truncate();
        if target_pos.is_none() {
            target_pos = Some(pos);
        }
    }

    // Prefer TC as primary target
    for (_entity, transform, team) in &player_buildings {
        if team.0 != 0 { continue; }
        target_pos = Some(transform.translation.truncate());
        break;
    }

    let Some(target) = target_pos else { return };

    let mut idle_soldiers: Vec<Entity> = Vec::new();
    for (entity, team, state) in &ai_military {
        if team.0 != AI_TEAM { continue; }
        if matches!(state, UnitState::Idle | UnitState::Moving) {
            idle_soldiers.push(entity);
        }
    }

    let send_count = ai.attack_wave_size.min(idle_soldiers.len());
    for &entity in idle_soldiers.iter().take(send_count) {
        commands.entity(entity)
            .insert(MoveTarget(target))
            .insert(UnitState::Moving);
    }

    let max_wave = match ai.difficulty {
        AiDifficulty::Easy => 5,
        AiDifficulty::Medium => 10,
        AiDifficulty::Hard => 20,
    };
    ai.attack_wave_size = (ai.attack_wave_size + 1).min(max_wave);
}

fn find_player_tc(buildings: &Query<(&Building, &Transform, &Team), Without<Unit>>) -> Vec2 {
    for (building, tf, team) in buildings.iter() {
        if team.0 == 0 && building.kind == BuildingKind::TownCenter {
            return tf.translation.truncate();
        }
    }
    GridPosition::new(24, 24).to_world()
}
