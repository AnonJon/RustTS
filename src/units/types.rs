use bevy::prelude::*;
use super::components::*;
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
                speed: 2.5,
                color: [200, 160, 60, 255],
            },
            UnitKind::Militia => UnitStats {
                hp: 40.0,
                damage: 4.0,
                range: 1.0,
                speed: 3.0,
                color: [40, 80, 220, 255],
            },
            UnitKind::Archer => UnitStats {
                hp: 30.0,
                damage: 5.0,
                range: 4.0,
                speed: 3.0,
                color: [30, 160, 30, 255],
            },
            UnitKind::Knight => UnitStats {
                hp: 100.0,
                damage: 10.0,
                range: 1.0,
                speed: 4.5,
                color: [180, 40, 180, 255],
            },
        }
    }
}

pub fn spawn_unit(
    commands: &mut Commands,
    texture: &Handle<Image>,
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
        Sprite {
            image: texture.clone(),
            custom_size: Some(Vec2::splat(48.0)),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
    ));

    if kind == UnitKind::Villager {
        ec.insert(crate::resources::components::Carrying::default());
    }

    ec.id()
}
