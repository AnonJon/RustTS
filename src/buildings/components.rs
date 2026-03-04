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
    Barracks,
    ArcheryRange,
    Stable,
    Farm,
    LumberCamp,
    MiningCamp,
    Mill,
}

impl BuildingKind {
    pub fn build_cost(self) -> (u32, u32, u32, u32) {
        match self {
            BuildingKind::TownCenter => (0, 275, 0, 100),
            BuildingKind::Barracks => (0, 175, 0, 0),
            BuildingKind::ArcheryRange => (0, 175, 0, 0),
            BuildingKind::Stable => (0, 175, 0, 0),
            BuildingKind::Farm => (60, 0, 0, 0),
            BuildingKind::LumberCamp => (0, 100, 0, 0),
            BuildingKind::MiningCamp => (0, 100, 0, 0),
            BuildingKind::Mill => (0, 100, 0, 0),
        }
    }

    pub fn max_hp(self) -> f32 {
        match self {
            BuildingKind::TownCenter => 2400.0,
            BuildingKind::Barracks => 1200.0,
            BuildingKind::ArcheryRange => 1200.0,
            BuildingKind::Stable => 1200.0,
            BuildingKind::Farm => 480.0,
            BuildingKind::LumberCamp => 600.0,
            BuildingKind::MiningCamp => 600.0,
            BuildingKind::Mill => 600.0,
        }
    }

    pub fn tile_size(self) -> (u32, u32) {
        match self {
            BuildingKind::TownCenter => (4, 4),
            BuildingKind::Farm => (3, 3),
            BuildingKind::LumberCamp | BuildingKind::MiningCamp | BuildingKind::Mill => (1, 1),
            _ => (3, 3),
        }
    }

    pub fn can_train(self) -> &'static [UnitKind] {
        match self {
            BuildingKind::TownCenter => &[UnitKind::Villager],
            BuildingKind::Barracks => &[UnitKind::Militia],
            BuildingKind::ArcheryRange => &[UnitKind::Archer],
            BuildingKind::Stable => &[UnitKind::Knight],
            _ => &[],
        }
    }

    pub fn required_age(self) -> Age {
        match self {
            BuildingKind::TownCenter
            | BuildingKind::Barracks
            | BuildingKind::Farm
            | BuildingKind::LumberCamp
            | BuildingKind::MiningCamp
            | BuildingKind::Mill => Age::Dark,
            BuildingKind::ArcheryRange | BuildingKind::Stable => Age::Feudal,
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

    pub fn color(self) -> [u8; 4] {
        match self {
            BuildingKind::TownCenter => [180, 140, 80, 255],
            BuildingKind::Barracks => [140, 60, 60, 255],
            BuildingKind::ArcheryRange => [60, 120, 60, 255],
            BuildingKind::Stable => [100, 80, 140, 255],
            BuildingKind::Farm => [160, 180, 60, 255],
            BuildingKind::LumberCamp => [100, 70, 40, 255],
            BuildingKind::MiningCamp => [130, 130, 130, 255],
            BuildingKind::Mill => [170, 150, 100, 255],
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

impl UnitKind {
    pub fn train_time(self) -> f32 {
        match self {
            UnitKind::Villager => 25.0,
            UnitKind::Militia => 21.0,
            UnitKind::Archer => 27.0,
            UnitKind::Knight => 30.0,
        }
    }

    pub fn train_cost(self) -> (u32, u32, u32, u32) {
        match self {
            UnitKind::Villager => (50, 0, 0, 0),
            UnitKind::Militia => (60, 0, 20, 0),
            UnitKind::Archer => (0, 25, 45, 0),
            UnitKind::Knight => (60, 0, 75, 0),
        }
    }

    pub fn required_age(self) -> Age {
        match self {
            UnitKind::Villager | UnitKind::Militia => Age::Dark,
            UnitKind::Archer => Age::Feudal,
            UnitKind::Knight => Age::Castle,
        }
    }
}
