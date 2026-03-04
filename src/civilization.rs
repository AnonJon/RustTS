use bevy::prelude::*;
use crate::buildings::research::Technology;
use crate::units::types::UnitKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Civilization {
    #[default]
    Britons,
    Franks,
    Teutons,
    Mongols,
}

impl Civilization {
    pub const ALL: [Civilization; 4] = [
        Civilization::Britons,
        Civilization::Franks,
        Civilization::Teutons,
        Civilization::Mongols,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Civilization::Britons => "Britons",
            Civilization::Franks => "Franks",
            Civilization::Teutons => "Teutons",
            Civilization::Mongols => "Mongols",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Civilization::Britons => "Archer civilization. +1 range for foot archers. Unique unit: Longbowman.",
            Civilization::Franks => "Cavalry civilization. +20% cavalry HP. Unique unit: Throwing Axeman.",
            Civilization::Teutons => "Infantry civilization. +1 melee armor for infantry. Unique unit: Teutonic Knight.",
            Civilization::Mongols => "Cavalry archer civilization. +30% siege unit speed. Unique unit: Mangudai.",
        }
    }

    pub fn unique_unit(self) -> UnitKind {
        match self {
            Civilization::Britons => UnitKind::Longbowman,
            Civilization::Franks => UnitKind::ThrowingAxeman,
            Civilization::Teutons => UnitKind::TeutonicKnight,
            Civilization::Mongols => UnitKind::Mangudai,
        }
    }

    pub fn archer_range_bonus(self) -> f32 {
        match self {
            Civilization::Britons => 1.0,
            _ => 0.0,
        }
    }

    pub fn cavalry_hp_multiplier(self) -> f32 {
        match self {
            Civilization::Franks => 1.2,
            _ => 1.0,
        }
    }

    pub fn infantry_melee_armor_bonus(self) -> f32 {
        match self {
            Civilization::Teutons => 1.0,
            _ => 0.0,
        }
    }

    pub fn siege_speed_multiplier(self) -> f32 {
        match self {
            Civilization::Mongols => 1.3,
            _ => 1.0,
        }
    }

    pub fn disabled_techs(self) -> &'static [Technology] {
        match self {
            Civilization::Britons => &[Technology::BlastFurnace, Technology::PlateMailArmor],
            Civilization::Franks => &[Technology::Bracer, Technology::RingArcherArmor],
            Civilization::Teutons => &[Technology::Bracer, Technology::BodkinArrow],
            Civilization::Mongols => &[Technology::PlateMailArmor, Technology::Architecture],
        }
    }
}

#[derive(Resource)]
pub struct PlayerCivilization(pub Civilization);

impl Default for PlayerCivilization {
    fn default() -> Self {
        Self(Civilization::Britons)
    }
}
