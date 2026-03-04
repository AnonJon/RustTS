use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceKind {
    Food,
    Wood,
    Gold,
    Stone,
}

#[derive(Component)]
pub struct ResourceNode {
    pub kind: ResourceKind,
    pub remaining: u32,
    pub max_amount: u32,
}

#[derive(Component)]
pub struct FloatingText {
    pub lifetime: Timer,
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct Carrying {
    pub kind: Option<ResourceKind>,
    pub amount: u32,
    pub max_carry: u32,
}

impl Default for Carrying {
    fn default() -> Self {
        Self { kind: None, amount: 0, max_carry: Self::BASE_CARRY }
    }
}

impl Carrying {
    pub const BASE_CARRY: u32 = 10;

    pub fn is_full(&self) -> bool {
        self.amount >= self.max_carry
    }

    pub fn has_resources(&self) -> bool {
        self.amount > 0 && self.kind.is_some()
    }

    pub fn take_all(&mut self) -> (ResourceKind, u32) {
        let kind = self.kind.take().unwrap_or(ResourceKind::Food);
        let amount = self.amount;
        self.amount = 0;
        (kind, amount)
    }
}

#[derive(Component)]
pub struct DropOff {
    pub accepts: Vec<ResourceKind>,
}

impl DropOff {
    pub fn all() -> Self {
        Self {
            accepts: vec![ResourceKind::Food, ResourceKind::Wood, ResourceKind::Gold, ResourceKind::Stone],
        }
    }

    pub fn wood() -> Self {
        Self { accepts: vec![ResourceKind::Wood] }
    }

    pub fn mining() -> Self {
        Self { accepts: vec![ResourceKind::Gold, ResourceKind::Stone] }
    }

    pub fn food() -> Self {
        Self { accepts: vec![ResourceKind::Food] }
    }

    pub fn accepts(&self, kind: ResourceKind) -> bool {
        self.accepts.contains(&kind)
    }
}

#[derive(Component)]
pub struct FarmWorker;

#[derive(Component)]
pub struct FarmFood {
    pub remaining: u32,
    pub max: u32,
}

impl FarmFood {
    pub fn new() -> Self {
        Self { remaining: 300, max: 300 }
    }
}

#[derive(Component)]
pub struct AutoReseed(pub bool);

#[derive(Resource)]
pub struct Population {
    pub current: u32,
    pub cap: u32,
}

impl Population {
    pub const MAX_POP: u32 = 200;

    pub fn has_room(&self, cost: u32) -> bool {
        self.current + cost <= self.cap.min(Self::MAX_POP)
    }
}

impl Default for Population {
    fn default() -> Self {
        Self { current: 0, cap: 5 }
    }
}

#[derive(Resource, Default, Debug)]
pub struct PlayerResources {
    pub food: u32,
    pub wood: u32,
    pub gold: u32,
    pub stone: u32,
}

impl PlayerResources {
    pub fn add(&mut self, kind: ResourceKind, amount: u32) {
        match kind {
            ResourceKind::Food => self.food += amount,
            ResourceKind::Wood => self.wood += amount,
            ResourceKind::Gold => self.gold += amount,
            ResourceKind::Stone => self.stone += amount,
        }
    }

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
