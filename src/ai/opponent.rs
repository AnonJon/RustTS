use bevy::prelude::*;
use rand::Rng;
use crate::map::{GridPosition, TILE_SIZE, MAP_WIDTH, MAP_HEIGHT};
use crate::map::generation::MapConfig;
use crate::units::components::*;
use crate::units::types::*;
use crate::units;
use crate::buildings::components::*;
use crate::buildings::spawn_building;
use super::behaviors::*;

const AI_TEAM: u8 = 1;

#[derive(Resource)]
pub struct AiState {
    pub resources: AiResources,
    pub age: Age,
    pub gather_timer: Timer,
    pub build_timer: Timer,
    pub train_timer: Timer,
    pub attack_timer: Timer,
    pub attack_wave_size: usize,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            resources: AiResources {
                food: 200,
                wood: 200,
                gold: 100,
                stone: 50,
            },
            age: Age::Dark,
            gather_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            build_timer: Timer::from_seconds(15.0, TimerMode::Repeating),
            train_timer: Timer::from_seconds(8.0, TimerMode::Repeating),
            attack_timer: Timer::from_seconds(90.0, TimerMode::Repeating),
            attack_wave_size: 3,
        }
    }
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
) {
    let ai_base = config.ai_base;

    spawn_building(
        &mut commands,
        &mut images,
        BuildingKind::TownCenter,
        Team(AI_TEAM),
        ai_base,
    );

    let villager_texture = units::create_unit_texture(&mut images, [200, 100, 100, 255]);
    let positions = [
        (ai_base.x + 2, ai_base.y),
        (ai_base.x, ai_base.y + 2),
        (ai_base.x - 2, ai_base.y),
    ];
    for (x, y) in positions {
        let grid = GridPosition::new(x, y);
        let world = grid.to_world();
        spawn_unit(&mut commands, &villager_texture, UnitKind::Villager, Team(AI_TEAM), grid, world);
    }
}

pub fn ai_economy_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    ai_villagers: Query<&Team, With<Unit>>,
) {
    ai.gather_timer.tick(time.delta());
    if !ai.gather_timer.just_finished() {
        return;
    }

    let villager_count = ai_villagers.iter().filter(|t| t.0 == AI_TEAM).count() as u32;
    let income_per_villager_food = 8;
    let income_per_villager_wood = 5;
    let income_per_villager_gold = 2;

    ai.resources.food += villager_count * income_per_villager_food;
    ai.resources.wood += villager_count * income_per_villager_wood;
    ai.resources.gold += villager_count * income_per_villager_gold;
}

pub fn ai_build_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    ai_buildings: Query<(&Building, &Team)>,
    config: Res<MapConfig>,
) {
    ai.build_timer.tick(time.delta());
    if !ai.build_timer.just_finished() {
        return;
    }

    let ai_base = config.ai_base;
    let mut has_barracks = false;
    let mut has_range = false;
    let mut has_stable = false;
    let mut building_count = 0u32;

    for (building, team) in &ai_buildings {
        if team.0 != AI_TEAM { continue; }
        building_count += 1;
        match building.kind {
            BuildingKind::Barracks => has_barracks = true,
            BuildingKind::ArcheryRange => has_range = true,
            BuildingKind::Stable => has_stable = true,
            _ => {}
        }
    }

    let mut rng = rand::rng();
    let offset_x = rng.random_range(-4..=4);
    let offset_y = rng.random_range(-4..=4);
    let grid = GridPosition::new(
        (ai_base.x + offset_x).clamp(2, MAP_WIDTH as i32 - 4),
        (ai_base.y + offset_y).clamp(2, MAP_HEIGHT as i32 - 4),
    );

    if !has_barracks {
        let (f, w, g, s) = BuildingKind::Barracks.build_cost();
        if ai.resources.spend(f, w, g, s) {
            spawn_building(&mut commands, &mut images, BuildingKind::Barracks, Team(AI_TEAM), grid);
        }
        return;
    }

    if ai.age >= Age::Feudal && !has_range {
        let (f, w, g, s) = BuildingKind::ArcheryRange.build_cost();
        if ai.resources.spend(f, w, g, s) {
            spawn_building(&mut commands, &mut images, BuildingKind::ArcheryRange, Team(AI_TEAM), grid);
        }
        return;
    }

    if ai.age >= Age::Castle && !has_stable {
        let (f, w, g, s) = BuildingKind::Stable.build_cost();
        if ai.resources.spend(f, w, g, s) {
            spawn_building(&mut commands, &mut images, BuildingKind::Stable, Team(AI_TEAM), grid);
        }
        return;
    }

    if building_count > 2 && ai.age < Age::Imperial {
        if let Some((cost_f, cost_w, cost_g, cost_s)) = ai.age.advance_cost() {
            if ai.resources.spend(cost_f, cost_w, cost_g, cost_s) {
                if let Some(next) = ai.age.next() {
                    ai.age = next;
                }
            }
        }
    }
}

pub fn ai_train_system(
    mut ai: ResMut<AiState>,
    time: Res<Time>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    ai_buildings: Query<(&Building, &Transform, &Team)>,
    player_buildings: Query<(&Building, &Transform, &Team), Without<Unit>>,
) {
    ai.train_timer.tick(time.delta());
    if !ai.train_timer.just_finished() {
        return;
    }

    let player_tc_pos = find_player_tc(&player_buildings);

    let enemy_texture = units::create_unit_texture(&mut images, [200, 40, 40, 255]);
    let villager_texture = units::create_unit_texture(&mut images, [200, 100, 100, 255]);

    for (building, transform, team) in &ai_buildings {
        if team.0 != AI_TEAM { continue; }

        let spawn_pos = transform.translation.truncate() + Vec2::new(0.0, -TILE_SIZE * 2.0);
        let grid = GridPosition::from_world(spawn_pos);

        match building.kind {
            BuildingKind::TownCenter => {
                let (f, w, g, s) = UnitKind::Villager.train_cost();
                if ai.resources.spend(f, w, g, s) {
                    let e = spawn_unit(&mut commands, &villager_texture, UnitKind::Villager, Team(AI_TEAM), grid, spawn_pos);
                    commands.entity(e).insert(UnitState::Idle);
                }
            }
            BuildingKind::Barracks => {
                let (f, w, g, s) = UnitKind::Militia.train_cost();
                if ai.resources.spend(f, w, g, s) {
                    let center = player_tc_pos;
                    let e = spawn_unit(&mut commands, &enemy_texture, UnitKind::Militia, Team(AI_TEAM), grid, spawn_pos);
                    commands.entity(e).insert((
                        AiBehavior::Patrol,
                        PatrolPath {
                            waypoints: vec![spawn_pos, center],
                            current_index: 0,
                        },
                        DetectionRadius(10.0),
                    ));
                }
            }
            BuildingKind::ArcheryRange if ai.age >= Age::Feudal => {
                let (f, w, g, s) = UnitKind::Archer.train_cost();
                if ai.resources.spend(f, w, g, s) {
                    let center = player_tc_pos;
                    let e = spawn_unit(&mut commands, &enemy_texture, UnitKind::Archer, Team(AI_TEAM), grid, spawn_pos);
                    commands.entity(e).insert((
                        AiBehavior::Patrol,
                        PatrolPath {
                            waypoints: vec![spawn_pos, center],
                            current_index: 0,
                        },
                        DetectionRadius(12.0),
                    ));
                }
            }
            BuildingKind::Stable if ai.age >= Age::Castle => {
                let (f, w, g, s) = UnitKind::Knight.train_cost();
                if ai.resources.spend(f, w, g, s) {
                    let center = player_tc_pos;
                    let e = spawn_unit(&mut commands, &enemy_texture, UnitKind::Knight, Team(AI_TEAM), grid, spawn_pos);
                    commands.entity(e).insert((
                        AiBehavior::Patrol,
                        PatrolPath {
                            waypoints: vec![spawn_pos, center],
                            current_index: 0,
                        },
                        DetectionRadius(8.0),
                    ));
                }
            }
            _ => {}
        }
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

    let player_tc = player_buildings.iter()
        .find(|(_, _, t)| t.0 == 0)
        .map(|(_, tf, _)| tf.translation.truncate());

    let Some(target_pos) = player_tc else { return };

    let mut idle_soldiers: Vec<Entity> = Vec::new();
    for (entity, team, state) in &ai_military {
        if team.0 != AI_TEAM { continue; }
        if matches!(state, UnitState::Idle) || matches!(state, UnitState::Moving) {
            idle_soldiers.push(entity);
        }
    }

    let send_count = ai.attack_wave_size.min(idle_soldiers.len());
    for &entity in idle_soldiers.iter().take(send_count) {
        commands.entity(entity)
            .insert(MoveTarget(target_pos))
            .insert(UnitState::Moving);
    }

    ai.attack_wave_size = (ai.attack_wave_size + 1).min(10);
}

fn find_player_tc(buildings: &Query<(&Building, &Transform, &Team), Without<Unit>>) -> Vec2 {
    for (building, tf, team) in buildings.iter() {
        if team.0 == 0 && building.kind == BuildingKind::TownCenter {
            return tf.translation.truncate();
        }
    }
    GridPosition::new(24, 24).to_world()
}
