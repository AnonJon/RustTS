use bevy::prelude::*;

#[derive(Component)]
pub struct Unit;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Team(pub u8);

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn fraction(&self) -> f32 {
        self.current / self.max
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitClass {
    Infantry,
    Cavalry,
    Archer,
    Siege,
    Villager,
    Building,
}

#[derive(Clone, Debug)]
pub struct BonusDamage {
    pub vs_class: UnitClass,
    pub amount: f32,
}

#[derive(Component, Clone, Debug)]
pub struct Armor {
    pub melee: f32,
    pub pierce: f32,
}

impl Armor {
    pub fn new(melee: f32, pierce: f32) -> Self {
        Self { melee, pierce }
    }
}

impl Default for Armor {
    fn default() -> Self {
        Self { melee: 0.0, pierce: 0.0 }
    }
}

#[derive(Component)]
pub struct AttackStats {
    pub melee_damage: f32,
    pub pierce_damage: f32,
    pub bonuses: Vec<BonusDamage>,
    pub range: f32,
    pub cooldown: Timer,
    pub unit_class: UnitClass,
}

impl AttackStats {
    /// AoE2 damage formula: max(1, melee - melee_armor) + max(0, pierce - pierce_armor) + bonus
    pub fn calc_damage(&self, target_armor: &Armor, target_class: UnitClass) -> f32 {
        let melee = (self.melee_damage - target_armor.melee).max(0.0);
        let pierce = (self.pierce_damage - target_armor.pierce).max(0.0);
        let bonus: f32 = self.bonuses.iter()
            .filter(|b| b.vs_class == target_class)
            .map(|b| b.amount)
            .sum();
        (melee + pierce + bonus).max(1.0)
    }
}

#[derive(Component)]
pub struct MoveTarget(pub Vec2);

#[derive(Component)]
pub struct AttackTarget(pub Entity);

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub enum UnitState {
    #[default]
    Idle,
    Moving,
    Attacking,
    Gathering {
        resource: Entity,
    },
    Returning {
        drop_off: Entity,
        then_gather: Option<Entity>,
    },
    FarmingAt {
        farm: Entity,
    },
    Constructing {
        building: Entity,
    },
    Repairing {
        building: Entity,
    },
    Dead,
}

#[derive(Component)]
pub struct ConstructTarget(pub Entity);

#[derive(Component)]
pub struct AreaDamage {
    pub radius: f32,
}

#[derive(Component, Clone, Debug, PartialEq)]
pub enum MovementIntent {
    Move,
    AttackMove,
    Patrol { a: Vec2, b: Vec2, going_to_b: bool },
}

impl Default for MovementIntent {
    fn default() -> Self {
        Self::Move
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct MonkUnit;

#[derive(Component)]
pub struct NeedsCivBonus;

#[derive(Component)]
pub struct NavalUnit;

#[derive(Component)]
pub struct HealTarget(pub Entity);

#[derive(Component)]
pub struct ConvertTarget {
    pub entity: Entity,
    pub progress: Timer,
}

#[derive(Component)]
pub struct RelicCarrier(pub Entity);

#[derive(Component)]
pub struct Relic;

#[derive(Component)]
pub struct DeathTimer(pub Timer);

#[derive(Component)]
pub struct RepairTarget(pub Entity);
