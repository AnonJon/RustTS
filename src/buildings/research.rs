use bevy::prelude::*;
use std::collections::HashSet;
use super::components::*;
use crate::resources::components::PlayerResources;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Technology {
    // Blacksmith - melee attack
    Forging,
    IronCasting,
    BlastFurnace,
    // Blacksmith - melee armor
    ScaleMailArmor,
    ChainMailArmor,
    PlateMailArmor,
    // Blacksmith - pierce attack
    Fletching,
    BodkinArrow,
    Bracer,
    // Blacksmith - pierce armor
    PaddedArcherArmor,
    LeatherArcherArmor,
    RingArcherArmor,
    // University
    Ballistics,
    MurderHoles,
    Architecture,
    Chemistry,
    // Lumber Camp
    DoubleBitAxe,
    BowSaw,
    // Mining Camp
    GoldMining,
    StoneMining,
    // Mill
    HorseCollar,
    HeavyPlow,
}

impl Technology {
    pub fn cost(self) -> (u32, u32, u32, u32) {
        match self {
            Technology::Forging => (150, 0, 0, 0),
            Technology::IronCasting => (0, 0, 220, 0),
            Technology::BlastFurnace => (0, 0, 275, 0),
            Technology::ScaleMailArmor => (100, 0, 0, 0),
            Technology::ChainMailArmor => (0, 0, 250, 0),
            Technology::PlateMailArmor => (0, 0, 300, 0),
            Technology::Fletching => (100, 0, 50, 0),
            Technology::BodkinArrow => (0, 0, 200, 0),
            Technology::Bracer => (0, 0, 300, 0),
            Technology::PaddedArcherArmor => (100, 0, 0, 0),
            Technology::LeatherArcherArmor => (0, 0, 150, 0),
            Technology::RingArcherArmor => (0, 0, 250, 0),
            Technology::Ballistics => (0, 0, 300, 0),
            Technology::MurderHoles => (0, 0, 200, 0),
            Technology::Architecture => (0, 0, 200, 0),
            Technology::Chemistry => (0, 0, 300, 0),
            Technology::DoubleBitAxe => (100, 0, 50, 0),
            Technology::BowSaw => (150, 0, 100, 0),
            Technology::GoldMining => (100, 0, 75, 0),
            Technology::StoneMining => (100, 0, 75, 0),
            Technology::HorseCollar => (75, 0, 0, 0),
            Technology::HeavyPlow => (125, 0, 0, 0),
        }
    }

    pub fn research_time(self) -> f32 {
        match self {
            Technology::Forging | Technology::ScaleMailArmor
            | Technology::Fletching | Technology::PaddedArcherArmor => 25.0,
            Technology::IronCasting | Technology::ChainMailArmor
            | Technology::BodkinArrow | Technology::LeatherArcherArmor => 40.0,
            Technology::BlastFurnace | Technology::PlateMailArmor
            | Technology::Bracer | Technology::RingArcherArmor => 55.0,
            Technology::Ballistics | Technology::Chemistry => 60.0,
            Technology::MurderHoles | Technology::Architecture => 35.0,
            Technology::DoubleBitAxe | Technology::GoldMining
            | Technology::StoneMining | Technology::HorseCollar => 25.0,
            Technology::BowSaw | Technology::HeavyPlow => 35.0,
        }
    }

    pub fn required_age(self) -> Age {
        match self {
            Technology::Forging | Technology::ScaleMailArmor
            | Technology::Fletching | Technology::PaddedArcherArmor
            | Technology::DoubleBitAxe | Technology::GoldMining
            | Technology::StoneMining | Technology::HorseCollar => Age::Feudal,
            Technology::IronCasting | Technology::ChainMailArmor
            | Technology::BodkinArrow | Technology::LeatherArcherArmor
            | Technology::Ballistics | Technology::MurderHoles
            | Technology::Architecture | Technology::BowSaw
            | Technology::HeavyPlow => Age::Castle,
            Technology::BlastFurnace | Technology::PlateMailArmor
            | Technology::Bracer | Technology::RingArcherArmor
            | Technology::Chemistry => Age::Imperial,
        }
    }

    pub fn researched_at(self) -> BuildingKind {
        match self {
            Technology::Forging | Technology::IronCasting | Technology::BlastFurnace
            | Technology::ScaleMailArmor | Technology::ChainMailArmor | Technology::PlateMailArmor
            | Technology::Fletching | Technology::BodkinArrow | Technology::Bracer
            | Technology::PaddedArcherArmor | Technology::LeatherArcherArmor
            | Technology::RingArcherArmor => BuildingKind::Blacksmith,
            Technology::Ballistics | Technology::MurderHoles
            | Technology::Architecture | Technology::Chemistry => BuildingKind::University,
            Technology::DoubleBitAxe | Technology::BowSaw => BuildingKind::LumberCamp,
            Technology::GoldMining | Technology::StoneMining => BuildingKind::MiningCamp,
            Technology::HorseCollar | Technology::HeavyPlow => BuildingKind::Mill,
        }
    }
}

#[derive(Resource, Default)]
pub struct ResearchedTechnologies {
    pub techs: HashSet<Technology>,
}

impl ResearchedTechnologies {
    pub fn melee_attack_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.techs.contains(&Technology::Forging) { bonus += 1.0; }
        if self.techs.contains(&Technology::IronCasting) { bonus += 1.0; }
        if self.techs.contains(&Technology::BlastFurnace) { bonus += 2.0; }
        bonus
    }

    pub fn melee_armor_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.techs.contains(&Technology::ScaleMailArmor) { bonus += 1.0; }
        if self.techs.contains(&Technology::ChainMailArmor) { bonus += 1.0; }
        if self.techs.contains(&Technology::PlateMailArmor) { bonus += 1.0; }
        bonus
    }

    pub fn pierce_attack_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.techs.contains(&Technology::Fletching) { bonus += 1.0; }
        if self.techs.contains(&Technology::BodkinArrow) { bonus += 1.0; }
        if self.techs.contains(&Technology::Bracer) { bonus += 1.0; }
        bonus
    }

    pub fn pierce_armor_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.techs.contains(&Technology::PaddedArcherArmor) { bonus += 1.0; }
        if self.techs.contains(&Technology::LeatherArcherArmor) { bonus += 1.0; }
        if self.techs.contains(&Technology::RingArcherArmor) { bonus += 1.0; }
        bonus
    }
}

#[derive(Component)]
pub struct ResearchQueue {
    pub queue: Vec<ResearchSlot>,
}

pub struct ResearchSlot {
    pub tech: Technology,
    pub remaining: Timer,
}

pub fn research_system(
    mut commands: Commands,
    mut buildings: Query<(Entity, &Building, &mut ResearchQueue)>,
    mut researched: ResMut<ResearchedTechnologies>,
    time: Res<Time>,
) {
    for (_entity, _building, mut queue) in &mut buildings {
        if queue.queue.is_empty() {
            continue;
        }

        queue.queue[0].remaining.tick(time.delta());
        if queue.queue[0].remaining.just_finished() {
            let slot = queue.queue.remove(0);
            researched.techs.insert(slot.tech);
        }
    }
}

pub fn keyboard_research_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut buildings: Query<(&Building, &crate::units::components::Team, Option<&mut ResearchQueue>), With<crate::units::components::Selected>>,
    mut resources: ResMut<PlayerResources>,
    researched: Res<ResearchedTechnologies>,
    age: Res<CurrentAge>,
    player_civ: Res<crate::civilization::PlayerCivilization>,
) {
    for (building, team, mut queue) in &mut buildings {
        if team.0 != 0 {
            continue;
        }

        let disabled = player_civ.0.disabled_techs();
        let available: Vec<Technology> = available_techs(building.kind, &researched, &age)
            .into_iter()
            .filter(|t| !disabled.contains(t))
            .collect();
        if available.is_empty() {
            continue;
        }

        let hotkeys = [KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR];
        for (i, &key) in hotkeys.iter().enumerate() {
            if i >= available.len() { break; }
            if !keys.just_pressed(key) { continue; }

            let tech = available[i];
            let (f, w, g, s) = tech.cost();
            if !resources.spend(f, w, g, s) { continue; }

            if let Some(ref mut rq) = queue {
                rq.queue.push(ResearchSlot {
                    tech,
                    remaining: Timer::from_seconds(tech.research_time(), TimerMode::Once),
                });
            }
        }
    }
}

pub fn available_techs(kind: BuildingKind, researched: &ResearchedTechnologies, age: &CurrentAge) -> Vec<Technology> {
    let all = match kind {
        BuildingKind::Blacksmith => vec![
            Technology::Forging, Technology::IronCasting, Technology::BlastFurnace,
            Technology::ScaleMailArmor, Technology::ChainMailArmor, Technology::PlateMailArmor,
            Technology::Fletching, Technology::BodkinArrow, Technology::Bracer,
            Technology::PaddedArcherArmor, Technology::LeatherArcherArmor, Technology::RingArcherArmor,
        ],
        BuildingKind::University => vec![
            Technology::Ballistics, Technology::MurderHoles,
            Technology::Architecture, Technology::Chemistry,
        ],
        BuildingKind::LumberCamp => vec![Technology::DoubleBitAxe, Technology::BowSaw],
        BuildingKind::MiningCamp => vec![Technology::GoldMining, Technology::StoneMining],
        BuildingKind::Mill => vec![Technology::HorseCollar, Technology::HeavyPlow],
        _ => vec![],
    };

    all.into_iter()
        .filter(|t| !researched.techs.contains(t))
        .filter(|t| t.required_age() <= age.0)
        .collect()
}
