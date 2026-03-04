use crate::units::types::UnitKind;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Age {
    Dark,
    Feudal,
    Castle,
    Imperial,
}

impl Age {
    pub fn next(self) -> Option<Age> {
        match self {
            Age::Dark => Some(Age::Feudal),
            Age::Feudal => Some(Age::Castle),
            Age::Castle => Some(Age::Imperial),
            Age::Imperial => None,
        }
    }

    pub fn advance_cost(self) -> Option<(u32, u32, u32, u32)> {
        match self {
            Age::Dark => Some((500, 0, 0, 0)),
            Age::Feudal => Some((800, 0, 200, 0)),
            Age::Castle => Some((1000, 0, 800, 0)),
            Age::Imperial => None,
        }
    }
}

#[derive(Resource)]
pub struct CurrentAge(pub Age);

impl Default for CurrentAge {
    fn default() -> Self {
        Self(Age::Dark)
    }
}

#[derive(Resource, Default)]
pub struct AgeUpProgress {
    pub researching: bool,
    pub timer: Option<Timer>,
    pub target_age: Option<Age>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingKind {
    TownCenter,
    House,
    Barracks,
    ArcheryRange,
    Stable,
    Farm,
    LumberCamp,
    MiningCamp,
    Mill,
    WatchTower,
    PalisadeWall,
    StoneWall,
    Gate,
    SiegeWorkshop,
    Blacksmith,
    University,
    Market,
    Monastery,
    Castle,
    Dock,
}

impl BuildingKind {
    pub fn population_support(self) -> u32 {
        match self {
            BuildingKind::House => 5,
            BuildingKind::TownCenter => 5,
            BuildingKind::Castle => 20,
            _ => 0,
        }
    }

    pub fn build_cost(self) -> (u32, u32, u32, u32) {
        match self {
            BuildingKind::TownCenter => (0, 275, 0, 100),
            BuildingKind::House => (0, 25, 0, 0),
            BuildingKind::Barracks => (0, 175, 0, 0),
            BuildingKind::ArcheryRange => (0, 175, 0, 0),
            BuildingKind::Stable => (0, 175, 0, 0),
            BuildingKind::Farm => (60, 0, 0, 0),
            BuildingKind::LumberCamp => (0, 100, 0, 0),
            BuildingKind::MiningCamp => (0, 100, 0, 0),
            BuildingKind::Mill => (0, 100, 0, 0),
            BuildingKind::WatchTower => (0, 0, 0, 125),
            BuildingKind::PalisadeWall => (0, 5, 0, 0),
            BuildingKind::StoneWall => (0, 0, 0, 5),
            BuildingKind::Gate => (0, 0, 0, 30),
            BuildingKind::SiegeWorkshop => (0, 200, 0, 0),
            BuildingKind::Blacksmith => (0, 150, 0, 0),
            BuildingKind::University => (0, 200, 0, 0),
            BuildingKind::Market => (0, 175, 0, 0),
            BuildingKind::Monastery => (0, 175, 0, 0),
            BuildingKind::Castle => (0, 0, 0, 650),
            BuildingKind::Dock => (0, 150, 0, 0),
        }
    }

    pub fn max_hp(self) -> f32 {
        match self {
            BuildingKind::TownCenter => 2400.0,
            BuildingKind::House => 550.0,
            BuildingKind::Barracks => 1200.0,
            BuildingKind::ArcheryRange => 1200.0,
            BuildingKind::Stable => 1200.0,
            BuildingKind::Farm => 480.0,
            BuildingKind::LumberCamp => 600.0,
            BuildingKind::MiningCamp => 600.0,
            BuildingKind::Mill => 600.0,
            BuildingKind::WatchTower => 1020.0,
            BuildingKind::PalisadeWall => 250.0,
            BuildingKind::StoneWall => 900.0,
            BuildingKind::Gate => 1500.0,
            BuildingKind::SiegeWorkshop => 1500.0,
            BuildingKind::Blacksmith => 1200.0,
            BuildingKind::University => 1200.0,
            BuildingKind::Market => 1200.0,
            BuildingKind::Monastery => 1200.0,
            BuildingKind::Castle => 4800.0,
            BuildingKind::Dock => 800.0,
        }
    }

    pub fn tile_size(self) -> (u32, u32) {
        match self {
            BuildingKind::TownCenter => (4, 4),
            BuildingKind::House => (2, 2),
            BuildingKind::Castle => (4, 4),
            BuildingKind::Dock => (3, 3),
            BuildingKind::Farm => (3, 3),
            BuildingKind::WatchTower => (1, 1),
            BuildingKind::PalisadeWall | BuildingKind::StoneWall => (1, 1),
            BuildingKind::Gate => (1, 1),
            BuildingKind::LumberCamp | BuildingKind::MiningCamp | BuildingKind::Mill => (1, 1),
            _ => (3, 3),
        }
    }

    pub fn can_train(self) -> &'static [UnitKind] {
        match self {
            BuildingKind::TownCenter => &[UnitKind::Villager],
            BuildingKind::Barracks => &[UnitKind::Militia, UnitKind::ManAtArms, UnitKind::Spearman],
            BuildingKind::ArcheryRange => &[UnitKind::Archer, UnitKind::Skirmisher],
            BuildingKind::Stable => &[UnitKind::ScoutCavalry, UnitKind::Knight],
            BuildingKind::SiegeWorkshop => &[UnitKind::BatteringRam, UnitKind::Mangonel],
            BuildingKind::Market => &[UnitKind::TradeCart],
            BuildingKind::Monastery => &[UnitKind::Monk],
            BuildingKind::Castle => &[UnitKind::Longbowman, UnitKind::ThrowingAxeman, UnitKind::TeutonicKnight, UnitKind::Mangudai],
            BuildingKind::Dock => &[UnitKind::FishingShip, UnitKind::Galley],
            _ => &[],
        }
    }

    pub fn required_age(self) -> Age {
        match self {
            BuildingKind::TownCenter
            | BuildingKind::House
            | BuildingKind::Barracks
            | BuildingKind::Farm
            | BuildingKind::LumberCamp
            | BuildingKind::MiningCamp
            | BuildingKind::Mill
            | BuildingKind::PalisadeWall => Age::Dark,
            BuildingKind::ArcheryRange | BuildingKind::Stable
            | BuildingKind::Blacksmith | BuildingKind::Market => Age::Feudal,
            BuildingKind::WatchTower | BuildingKind::StoneWall | BuildingKind::Gate
            | BuildingKind::SiegeWorkshop | BuildingKind::University
            | BuildingKind::Monastery | BuildingKind::Castle => Age::Castle,
            BuildingKind::Dock => Age::Feudal,
        }
    }

    /// Display size for sprite-based buildings. For buildings with actual sprites,
    /// we use their native aspect ratio scaled to fit the tile footprint width.
    /// `fallback_w` / `fallback_h` are used for non-sprite buildings.
    pub fn sprite_display_size(self, fallback_w: f32, fallback_h: f32) -> Vec2 {
        match self {
            // Castle sprite is 2178×1516; scale to 4-tile width (512px), keep aspect ratio
            BuildingKind::TownCenter => {
                let w = fallback_w; // 512
                let aspect = 1516.0 / 2178.0;
                Vec2::new(w, w * aspect)
            }
            _ => Vec2::new(fallback_w, fallback_h),
        }
    }

    pub fn armor(self) -> (f32, f32) {
        match self {
            BuildingKind::TownCenter => (3.0, 5.0),
            BuildingKind::House => (0.0, 7.0),
            BuildingKind::Barracks | BuildingKind::ArcheryRange | BuildingKind::Stable => (1.0, 7.0),
            BuildingKind::WatchTower => (1.0, 8.0),
            BuildingKind::PalisadeWall => (0.0, 2.0),
            BuildingKind::StoneWall | BuildingKind::Gate => (6.0, 8.0),
            BuildingKind::SiegeWorkshop => (0.0, 7.0),
            BuildingKind::Blacksmith | BuildingKind::University | BuildingKind::Market
            | BuildingKind::Monastery => (1.0, 7.0),
            BuildingKind::Castle => (8.0, 11.0),
            BuildingKind::Dock => (0.0, 6.0),
            BuildingKind::Farm => (0.0, 0.0),
            BuildingKind::LumberCamp | BuildingKind::MiningCamp | BuildingKind::Mill => (0.0, 5.0),
        }
    }

    pub fn build_time(self) -> f32 {
        match self {
            BuildingKind::TownCenter => 150.0,
            BuildingKind::House => 25.0,
            BuildingKind::Barracks => 50.0,
            BuildingKind::ArcheryRange => 50.0,
            BuildingKind::Stable => 50.0,
            BuildingKind::Farm => 15.0,
            BuildingKind::LumberCamp => 25.0,
            BuildingKind::MiningCamp => 25.0,
            BuildingKind::Mill => 35.0,
            BuildingKind::WatchTower => 80.0,
            BuildingKind::PalisadeWall => 5.0,
            BuildingKind::StoneWall => 8.0,
            BuildingKind::Gate => 40.0,
            BuildingKind::SiegeWorkshop => 60.0,
            BuildingKind::Blacksmith => 40.0,
            BuildingKind::University => 60.0,
            BuildingKind::Market => 50.0,
            BuildingKind::Monastery => 60.0,
            BuildingKind::Castle => 200.0,
            BuildingKind::Dock => 35.0,
        }
    }

    pub fn is_wall(self) -> bool {
        matches!(self, BuildingKind::PalisadeWall | BuildingKind::StoneWall | BuildingKind::Gate)
    }

    pub fn color(self) -> [u8; 4] {
        match self {
            BuildingKind::TownCenter => [180, 140, 80, 255],
            BuildingKind::House => [160, 130, 90, 255],
            BuildingKind::Barracks => [140, 60, 60, 255],
            BuildingKind::ArcheryRange => [60, 120, 60, 255],
            BuildingKind::Stable => [100, 80, 140, 255],
            BuildingKind::Farm => [160, 180, 60, 255],
            BuildingKind::LumberCamp => [100, 70, 40, 255],
            BuildingKind::MiningCamp => [130, 130, 130, 255],
            BuildingKind::Mill => [170, 150, 100, 255],
            BuildingKind::WatchTower => [120, 100, 80, 255],
            BuildingKind::PalisadeWall => [140, 100, 50, 255],
            BuildingKind::StoneWall => [160, 160, 160, 255],
            BuildingKind::Gate => [140, 140, 180, 255],
            BuildingKind::SiegeWorkshop => [110, 90, 60, 255],
            BuildingKind::Blacksmith => [80, 80, 100, 255],
            BuildingKind::University => [60, 80, 120, 255],
            BuildingKind::Market => [180, 160, 80, 255],
            BuildingKind::Monastery => [200, 180, 120, 255],
            BuildingKind::Castle => [140, 120, 100, 255],
            BuildingKind::Dock => [100, 80, 50, 255],
        }
    }
}

#[derive(Component)]
pub struct Building {
    pub kind: BuildingKind,
    pub rally_point: Option<Vec2>,
}

#[derive(Component)]
pub struct TrainingQueue {
    pub queue: Vec<TrainingSlot>,
}

pub struct TrainingSlot {
    pub kind: UnitKind,
    pub remaining: Timer,
}

#[derive(Component)]
pub struct UnderConstruction {
    pub progress: f32,
    pub build_time: f32,
}

impl UnderConstruction {
    pub fn new(kind: BuildingKind) -> Self {
        Self {
            progress: 0.0,
            build_time: kind.build_time(),
        }
    }
}

#[derive(Component)]
pub struct GatePassable {
    pub owner_team: u8,
}

#[derive(Component)]
pub struct GarrisonSlots {
    pub units: Vec<Entity>,
    pub capacity: u32,
}

impl GarrisonSlots {
    pub fn new(capacity: u32) -> Self {
        Self { units: Vec::new(), capacity }
    }

    pub fn has_space(&self) -> bool {
        (self.units.len() as u32) < self.capacity
    }
}

#[derive(Component)]
pub struct TowerAttack {
    pub range: f32,
    pub base_pierce_damage: f32,
    pub pierce_damage: f32,
    pub cooldown: Timer,
}

impl TowerAttack {
    pub fn watch_tower() -> Self {
        Self {
            range: 8.0,
            base_pierce_damage: 5.0,
            pierce_damage: 5.0,
            cooldown: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }

    pub fn town_center() -> Self {
        Self {
            range: 6.0,
            base_pierce_damage: 5.0,
            pierce_damage: 5.0,
            cooldown: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct RelicStorage {
    pub relics: Vec<Entity>,
}

impl RelicStorage {
    pub fn new() -> Self {
        Self { relics: Vec::new() }
    }
}

impl UnitKind {
    pub fn population_cost(self) -> u32 {
        match self {
            UnitKind::BatteringRam | UnitKind::Mangonel => 3,
            _ => 1,
        }
    }

    pub fn train_time(self) -> f32 {
        match self {
            UnitKind::Villager => 25.0,
            UnitKind::Militia => 21.0,
            UnitKind::ManAtArms => 21.0,
            UnitKind::Spearman => 22.0,
            UnitKind::Archer => 27.0,
            UnitKind::Skirmisher => 22.0,
            UnitKind::ScoutCavalry => 30.0,
            UnitKind::Knight => 30.0,
            UnitKind::BatteringRam => 36.0,
            UnitKind::Mangonel => 46.0,
            UnitKind::TradeCart => 30.0,
            UnitKind::Monk => 51.0,
            UnitKind::Longbowman => 19.0,
            UnitKind::ThrowingAxeman => 17.0,
            UnitKind::TeutonicKnight => 19.0,
            UnitKind::Mangudai => 26.0,
            UnitKind::FishingShip => 40.0,
            UnitKind::Galley => 60.0,
        }
    }

    pub fn train_cost(self) -> (u32, u32, u32, u32) {
        match self {
            UnitKind::Villager => (50, 0, 0, 0),
            UnitKind::Militia => (60, 0, 20, 0),
            UnitKind::ManAtArms => (60, 0, 20, 0),
            UnitKind::Spearman => (35, 0, 0, 0),
            UnitKind::Archer => (0, 25, 45, 0),
            UnitKind::Skirmisher => (25, 0, 0, 0),
            UnitKind::ScoutCavalry => (80, 0, 0, 0),
            UnitKind::Knight => (60, 0, 75, 0),
            UnitKind::BatteringRam => (0, 160, 0, 0),
            UnitKind::Mangonel => (0, 160, 0, 160),
            UnitKind::TradeCart => (0, 0, 100, 0),
            UnitKind::Monk => (0, 0, 100, 0),
            UnitKind::Longbowman => (0, 35, 40, 0),
            UnitKind::ThrowingAxeman => (55, 0, 25, 0),
            UnitKind::TeutonicKnight => (85, 0, 40, 0),
            UnitKind::Mangudai => (0, 55, 65, 0),
            UnitKind::FishingShip => (0, 75, 0, 0),
            UnitKind::Galley => (0, 90, 30, 0),
        }
    }

    pub fn required_age(self) -> Age {
        match self {
            UnitKind::Villager | UnitKind::Militia | UnitKind::ScoutCavalry => Age::Dark,
            UnitKind::ManAtArms | UnitKind::Spearman | UnitKind::Archer
            | UnitKind::Skirmisher | UnitKind::TradeCart => Age::Feudal,
            UnitKind::Knight | UnitKind::BatteringRam | UnitKind::Mangonel
            | UnitKind::Monk | UnitKind::Longbowman | UnitKind::ThrowingAxeman
            | UnitKind::TeutonicKnight | UnitKind::Mangudai => Age::Castle,
            UnitKind::FishingShip | UnitKind::Galley => Age::Feudal,
        }
    }
}
