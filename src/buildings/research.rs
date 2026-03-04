use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use super::components::*;
use crate::resources::components::PlayerResources;
use crate::units::types::UnitKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Technology {
    // Town Center
    Loom,
    Wheelbarrow,
    HandCart,
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
    // Unit line upgrades - Barracks
    ManAtArmsUpgrade,
    LongSwordsmanUpgrade,
    TwoHandedSwordsmanUpgrade,
    ChampionUpgrade,
    PikemanUpgrade,
    HalberdierUpgrade,
    // Unit line upgrades - Archery Range
    CrossbowmanUpgrade,
    ArbalesterUpgrade,
    EliteSkirmisherUpgrade,
    // Unit line upgrades - Stable
    LightCavalryUpgrade,
    HussarUpgrade,
    CavalierUpgrade,
    PaladinUpgrade,
    // Unit line upgrades - Castle (elite unique units)
    EliteLongbowmanUpgrade,
    EliteThrowingAxemanUpgrade,
    EliteTeutonicKnightUpgrade,
    EliteMangudaiUpgrade,
}

impl Technology {
    pub fn cost(self) -> (u32, u32, u32, u32) {
        match self {
            Technology::Loom => (0, 0, 50, 0),
            Technology::Wheelbarrow => (50, 50, 0, 0),
            Technology::HandCart => (300, 200, 0, 0),
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
            // Unit line upgrades
            Technology::ManAtArmsUpgrade => (100, 0, 40, 0),
            Technology::LongSwordsmanUpgrade => (200, 0, 65, 0),
            Technology::TwoHandedSwordsmanUpgrade => (300, 0, 100, 0),
            Technology::ChampionUpgrade => (750, 0, 350, 0),
            Technology::PikemanUpgrade => (215, 0, 90, 0),
            Technology::HalberdierUpgrade => (300, 0, 600, 0),
            Technology::CrossbowmanUpgrade => (125, 0, 75, 0),
            Technology::ArbalesterUpgrade => (350, 0, 300, 0),
            Technology::EliteSkirmisherUpgrade => (230, 0, 130, 0),
            Technology::LightCavalryUpgrade => (150, 0, 50, 0),
            Technology::HussarUpgrade => (500, 0, 600, 0),
            Technology::CavalierUpgrade => (300, 0, 300, 0),
            Technology::PaladinUpgrade => (1300, 0, 750, 0),
            Technology::EliteLongbowmanUpgrade => (0, 0, 850, 0),
            Technology::EliteThrowingAxemanUpgrade => (0, 0, 750, 0),
            Technology::EliteTeutonicKnightUpgrade => (0, 0, 800, 0),
            Technology::EliteMangudaiUpgrade => (0, 0, 900, 0),
        }
    }

    pub fn research_time(self) -> f32 {
        match self {
            Technology::Loom => 25.0,
            Technology::Wheelbarrow => 75.0,
            Technology::HandCart => 55.0,
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
            Technology::ManAtArmsUpgrade => 40.0,
            Technology::LongSwordsmanUpgrade => 45.0,
            Technology::TwoHandedSwordsmanUpgrade => 75.0,
            Technology::ChampionUpgrade => 100.0,
            Technology::PikemanUpgrade => 45.0,
            Technology::HalberdierUpgrade => 50.0,
            Technology::CrossbowmanUpgrade => 35.0,
            Technology::ArbalesterUpgrade => 50.0,
            Technology::EliteSkirmisherUpgrade => 50.0,
            Technology::LightCavalryUpgrade => 45.0,
            Technology::HussarUpgrade => 50.0,
            Technology::CavalierUpgrade => 100.0,
            Technology::PaladinUpgrade => 170.0,
            Technology::EliteLongbowmanUpgrade | Technology::EliteThrowingAxemanUpgrade
            | Technology::EliteTeutonicKnightUpgrade | Technology::EliteMangudaiUpgrade => 60.0,
        }
    }

    pub fn required_age(self) -> Age {
        match self {
            Technology::Loom => Age::Dark,
            Technology::Wheelbarrow | Technology::ManAtArmsUpgrade => Age::Feudal,
            Technology::HandCart => Age::Castle,
            Technology::Forging | Technology::ScaleMailArmor
            | Technology::Fletching | Technology::PaddedArcherArmor
            | Technology::DoubleBitAxe | Technology::GoldMining
            | Technology::StoneMining | Technology::HorseCollar => Age::Feudal,
            Technology::IronCasting | Technology::ChainMailArmor
            | Technology::BodkinArrow | Technology::LeatherArcherArmor
            | Technology::Ballistics | Technology::MurderHoles
            | Technology::Architecture | Technology::BowSaw
            | Technology::HeavyPlow => Age::Castle,
            Technology::LongSwordsmanUpgrade | Technology::PikemanUpgrade
            | Technology::CrossbowmanUpgrade | Technology::EliteSkirmisherUpgrade
            | Technology::LightCavalryUpgrade | Technology::CavalierUpgrade => Age::Castle,
            Technology::BlastFurnace | Technology::PlateMailArmor
            | Technology::Bracer | Technology::RingArcherArmor
            | Technology::Chemistry => Age::Imperial,
            Technology::TwoHandedSwordsmanUpgrade | Technology::ChampionUpgrade
            | Technology::HalberdierUpgrade | Technology::ArbalesterUpgrade
            | Technology::HussarUpgrade | Technology::PaladinUpgrade => Age::Imperial,
            Technology::EliteLongbowmanUpgrade | Technology::EliteThrowingAxemanUpgrade
            | Technology::EliteTeutonicKnightUpgrade | Technology::EliteMangudaiUpgrade => Age::Imperial,
        }
    }

    pub fn researched_at(self) -> BuildingKind {
        match self {
            Technology::Loom | Technology::Wheelbarrow | Technology::HandCart => BuildingKind::TownCenter,
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
            Technology::ManAtArmsUpgrade | Technology::LongSwordsmanUpgrade
            | Technology::TwoHandedSwordsmanUpgrade | Technology::ChampionUpgrade
            | Technology::PikemanUpgrade | Technology::HalberdierUpgrade => BuildingKind::Barracks,
            Technology::CrossbowmanUpgrade | Technology::ArbalesterUpgrade
            | Technology::EliteSkirmisherUpgrade => BuildingKind::ArcheryRange,
            Technology::LightCavalryUpgrade | Technology::HussarUpgrade
            | Technology::CavalierUpgrade | Technology::PaladinUpgrade => BuildingKind::Stable,
            Technology::EliteLongbowmanUpgrade | Technology::EliteThrowingAxemanUpgrade
            | Technology::EliteTeutonicKnightUpgrade | Technology::EliteMangudaiUpgrade => BuildingKind::Castle,
        }
    }

    /// If this technology is a unit line upgrade, returns (from_kind, to_kind).
    pub fn unit_upgrade(self) -> Option<(UnitKind, UnitKind)> {
        use crate::units::types::UnitKind;
        match self {
            Technology::ManAtArmsUpgrade => Some((UnitKind::Militia, UnitKind::ManAtArms)),
            Technology::LongSwordsmanUpgrade => Some((UnitKind::ManAtArms, UnitKind::LongSwordsman)),
            Technology::TwoHandedSwordsmanUpgrade => Some((UnitKind::LongSwordsman, UnitKind::TwoHandedSwordsman)),
            Technology::ChampionUpgrade => Some((UnitKind::TwoHandedSwordsman, UnitKind::Champion)),
            Technology::PikemanUpgrade => Some((UnitKind::Spearman, UnitKind::Pikeman)),
            Technology::HalberdierUpgrade => Some((UnitKind::Pikeman, UnitKind::Halberdier)),
            Technology::CrossbowmanUpgrade => Some((UnitKind::Archer, UnitKind::Crossbowman)),
            Technology::ArbalesterUpgrade => Some((UnitKind::Crossbowman, UnitKind::Arbalester)),
            Technology::EliteSkirmisherUpgrade => Some((UnitKind::Skirmisher, UnitKind::EliteSkirmisher)),
            Technology::LightCavalryUpgrade => Some((UnitKind::ScoutCavalry, UnitKind::LightCavalry)),
            Technology::HussarUpgrade => Some((UnitKind::LightCavalry, UnitKind::Hussar)),
            Technology::CavalierUpgrade => Some((UnitKind::Knight, UnitKind::Cavalier)),
            Technology::PaladinUpgrade => Some((UnitKind::Cavalier, UnitKind::Paladin)),
            Technology::EliteLongbowmanUpgrade => Some((UnitKind::Longbowman, UnitKind::EliteLongbowman)),
            Technology::EliteThrowingAxemanUpgrade => Some((UnitKind::ThrowingAxeman, UnitKind::EliteThrowingAxeman)),
            Technology::EliteTeutonicKnightUpgrade => Some((UnitKind::TeutonicKnight, UnitKind::EliteTeutonicKnight)),
            Technology::EliteMangudaiUpgrade => Some((UnitKind::Mangudai, UnitKind::EliteMangudai)),
            _ => None,
        }
    }

    /// If this is a unit line upgrade, return the prerequisite technology (the previous upgrade in the chain).
    pub fn prerequisite(self) -> Option<Technology> {
        match self {
            Technology::LongSwordsmanUpgrade => Some(Technology::ManAtArmsUpgrade),
            Technology::TwoHandedSwordsmanUpgrade => Some(Technology::LongSwordsmanUpgrade),
            Technology::ChampionUpgrade => Some(Technology::TwoHandedSwordsmanUpgrade),
            Technology::HalberdierUpgrade => Some(Technology::PikemanUpgrade),
            Technology::ArbalesterUpgrade => Some(Technology::CrossbowmanUpgrade),
            Technology::HussarUpgrade => Some(Technology::LightCavalryUpgrade),
            Technology::PaladinUpgrade => Some(Technology::CavalierUpgrade),
            _ => None,
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

    pub fn has_loom(&self) -> bool {
        self.techs.contains(&Technology::Loom)
    }

    pub fn villager_hp_bonus(&self) -> f32 {
        if self.has_loom() { 15.0 } else { 0.0 }
    }

    pub fn villager_pierce_armor_bonus(&self) -> f32 {
        if self.has_loom() { 1.0 } else { 0.0 }
    }

    pub fn villager_speed_multiplier(&self) -> f32 {
        let mut mult = 1.0;
        if self.techs.contains(&Technology::Wheelbarrow) { mult *= 1.10; }
        if self.techs.contains(&Technology::HandCart) { mult *= 1.10; }
        mult
    }

    pub fn villager_carry_bonus(&self) -> u32 {
        let mut bonus = 0;
        if self.techs.contains(&Technology::Wheelbarrow) { bonus += 3; }
        if self.techs.contains(&Technology::HandCart) { bonus += 3; }
        bonus
    }
}

/// Tracks the current upgraded version of each unit line.
/// Key = base unit of the line, Value = current upgraded version.
#[derive(Resource, Default)]
pub struct UnitLineUpgrades {
    pub upgrades: HashMap<UnitKind, UnitKind>,
}

impl UnitLineUpgrades {
    /// Given a base unit kind (as listed in can_train), return the current
    /// version that should actually be trained or displayed.
    pub fn current_version(&self, base: UnitKind) -> UnitKind {
        self.upgrades.get(&base.base_unit()).copied().unwrap_or(base)
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
    mut buildings: Query<(Entity, &Building, &mut ResearchQueue)>,
    mut researched: ResMut<ResearchedTechnologies>,
    mut line_upgrades: ResMut<UnitLineUpgrades>,
    time: Res<Time>,
    mut units: Query<(
        &mut UnitKind,
        &mut crate::units::components::Health,
        &mut crate::units::components::Armor,
        &mut crate::units::components::Speed,
        &mut crate::units::components::AttackStats,
        &crate::units::components::Team,
        &mut crate::map::fog::LineOfSight,
    ), With<crate::units::components::Unit>>,
) {
    for (_entity, _building, mut queue) in &mut buildings {
        if queue.queue.is_empty() {
            continue;
        }

        queue.queue[0].remaining.tick(time.delta());
        if queue.queue[0].remaining.just_finished() {
            let slot = queue.queue.remove(0);
            researched.techs.insert(slot.tech);

            if let Some((from_kind, to_kind)) = slot.tech.unit_upgrade() {
                line_upgrades.upgrades.insert(to_kind.base_unit(), to_kind);
                let new_stats = to_kind.stats();
                let new_los = to_kind.line_of_sight();

                for (mut kind, mut health, mut armor, mut speed, mut attack, team, mut los) in &mut units {
                    if team.0 != 0 { continue; }
                    if *kind != from_kind { continue; }

                    let was_full = health.current >= health.max - 0.01;
                    *kind = to_kind;
                    health.max = new_stats.hp;
                    if was_full { health.current = new_stats.hp; }
                    armor.melee = new_stats.melee_armor;
                    armor.pierce = new_stats.pierce_armor;
                    speed.0 = new_stats.speed;
                    attack.melee_damage = new_stats.melee_damage;
                    attack.pierce_damage = new_stats.pierce_damage;
                    attack.range = new_stats.range;
                    attack.bonuses = new_stats.bonuses.clone();
                    los.0 = new_los;
                }
            }
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

        let has_training = !building.kind.can_train().is_empty();
        let hotkeys: &[KeyCode] = if has_training {
            &[KeyCode::KeyZ, KeyCode::KeyX, KeyCode::KeyC, KeyCode::KeyV]
        } else {
            &[KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR]
        };

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

pub fn apply_villager_tech_bonuses(
    techs: Res<ResearchedTechnologies>,
    mut villagers: Query<(&mut crate::units::components::Health, &mut crate::units::components::Armor, &mut crate::units::components::Speed, &mut crate::resources::components::Carrying, &crate::units::types::UnitKind), With<crate::units::components::Unit>>,
) {
    let hp_bonus = techs.villager_hp_bonus();
    let pierce_bonus = techs.villager_pierce_armor_bonus();
    let speed_mult = techs.villager_speed_multiplier();
    let carry_bonus = techs.villager_carry_bonus();

    for (mut health, mut armor, mut speed, mut carrying, kind) in &mut villagers {
        if *kind != crate::units::types::UnitKind::Villager { continue; }

        let base_hp = 25.0;
        let new_max = base_hp + hp_bonus;
        if (health.max - new_max).abs() > 0.01 {
            let was_full = health.current >= health.max - 0.01;
            health.max = new_max;
            if was_full { health.current = new_max; }
        }

        armor.pierce = pierce_bonus;

        let base_speed = 0.8;
        speed.0 = base_speed * speed_mult;

        carrying.max_carry = crate::resources::components::Carrying::BASE_CARRY + carry_bonus;
    }
}

pub fn available_techs(kind: BuildingKind, researched: &ResearchedTechnologies, age: &CurrentAge) -> Vec<Technology> {
    let all = match kind {
        BuildingKind::TownCenter => vec![
            Technology::Loom, Technology::Wheelbarrow, Technology::HandCart,
        ],
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
        BuildingKind::Barracks => vec![
            Technology::ManAtArmsUpgrade, Technology::LongSwordsmanUpgrade,
            Technology::TwoHandedSwordsmanUpgrade, Technology::ChampionUpgrade,
            Technology::PikemanUpgrade, Technology::HalberdierUpgrade,
        ],
        BuildingKind::ArcheryRange => vec![
            Technology::CrossbowmanUpgrade, Technology::ArbalesterUpgrade,
            Technology::EliteSkirmisherUpgrade,
        ],
        BuildingKind::Stable => vec![
            Technology::LightCavalryUpgrade, Technology::HussarUpgrade,
            Technology::CavalierUpgrade, Technology::PaladinUpgrade,
        ],
        BuildingKind::Castle => vec![
            Technology::EliteLongbowmanUpgrade, Technology::EliteThrowingAxemanUpgrade,
            Technology::EliteTeutonicKnightUpgrade, Technology::EliteMangudaiUpgrade,
        ],
        _ => vec![],
    };

    all.into_iter()
        .filter(|t| !researched.techs.contains(t))
        .filter(|t| t.required_age() <= age.0)
        .filter(|t| match t.prerequisite() {
            Some(prereq) => researched.techs.contains(&prereq),
            None => true,
        })
        .collect()
}
