use bevy::prelude::*;
use super::components::*;
use super::animation::AnimationConfig;
use crate::map::GridPosition;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    Villager,
    Militia,
    Archer,
    Knight,
}

pub struct UnitStats {
    pub hp: f32,
    pub damage: f32,
    pub range: f32,
    pub speed: f32,
    pub color: [u8; 4],
}

impl UnitKind {
    pub fn stats(self) -> UnitStats {
        match self {
            UnitKind::Villager => UnitStats {
                hp: 25.0,
                damage: 3.0,
                range: 1.0,
                speed: 0.8,
                color: [200, 160, 60, 255],
            },
            UnitKind::Militia => UnitStats {
                hp: 40.0,
                damage: 4.0,
                range: 1.0,
                speed: 0.9,
                color: [40, 80, 220, 255],
            },
            UnitKind::Archer => UnitStats {
                hp: 30.0,
                damage: 5.0,
                range: 4.0,
                speed: 0.96,
                color: [30, 160, 30, 255],
            },
            UnitKind::Knight => UnitStats {
                hp: 100.0,
                damage: 10.0,
                range: 1.0,
                speed: 1.35,
                color: [180, 40, 180, 255],
            },
        }
    }

    pub fn sprite_path(self) -> Option<&'static str> {
        match self {
            UnitKind::Villager => Some("sprites/units/villager.png"),
            UnitKind::Militia => Some("sprites/units/militia.png"),
            _ => None,
        }
    }

    pub fn animation_config(self) -> AnimationConfig {
        AnimationConfig::new(2, 4, 3, 8.0)
    }

    pub fn frame_count(self) -> usize {
        9 // 2 idle + 4 walk + 3 attack
    }
}

pub struct UnitSpriteSheet {
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
pub struct UnitSprites {
    pub villager: UnitSpriteSheet,
    pub militia: UnitSpriteSheet,
}

impl UnitSprites {
    pub fn get(&self, kind: UnitKind) -> &UnitSpriteSheet {
        match kind {
            UnitKind::Villager => &self.villager,
            UnitKind::Militia => &self.militia,
            UnitKind::Archer => &self.militia,
            UnitKind::Knight => &self.militia,
        }
    }
}

pub fn spawn_unit(
    commands: &mut Commands,
    sheet: &UnitSpriteSheet,
    kind: UnitKind,
    team: Team,
    grid: GridPosition,
    world_pos: Vec2,
) -> Entity {
    let stats = kind.stats();

    let mut ec = commands.spawn((
        Unit,
        team,
        grid,
        Speed(stats.speed),
        Health::new(stats.hp),
        AttackStats {
            damage: stats.damage,
            range: stats.range,
            cooldown: Timer::from_seconds(1.0, TimerMode::Repeating),
        },
        UnitState::default(),
        kind.animation_config(),
        Sprite {
            image: sheet.texture.clone(),
            custom_size: Some(Vec2::splat(48.0)),
            texture_atlas: Some(TextureAtlas {
                layout: sheet.atlas_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
    ));

    if kind == UnitKind::Villager {
        ec.insert(crate::resources::components::Carrying::default());
    }

    ec.id()
}
