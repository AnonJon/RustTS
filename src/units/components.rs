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

#[derive(Component)]
pub struct AttackStats {
    pub damage: f32,
    pub range: f32,
    pub cooldown: Timer,
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
    Dead,
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct HealthBar;
