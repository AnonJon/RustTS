use bevy::prelude::*;
use super::components::*;
use super::animation::AnimationConfig;
use crate::map::GridPosition;
use crate::map::fog::LineOfSight;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Villager,
    // Infantry line
    Militia,
    ManAtArms,
    LongSwordsman,
    TwoHandedSwordsman,
    Champion,
    // Spearman line
    Spearman,
    Pikeman,
    Halberdier,
    // Archer line
    Archer,
    Crossbowman,
    Arbalester,
    // Skirmisher line
    Skirmisher,
    EliteSkirmisher,
    // Scout line
    ScoutCavalry,
    LightCavalry,
    Hussar,
    // Knight line
    Knight,
    Cavalier,
    Paladin,
    // Siege
    BatteringRam,
    Mangonel,
    Scorpion,
    // Unique units
    Longbowman,
    EliteLongbowman,
    ThrowingAxeman,
    EliteThrowingAxeman,
    TeutonicKnight,
    EliteTeutonicKnight,
    Mangudai,
    EliteMangudai,
    // Other
    TradeCart,
    Monk,
    FishingShip,
    Galley,
}

pub struct UnitStats {
    pub hp: f32,
    pub melee_damage: f32,
    pub pierce_damage: f32,
    pub bonuses: Vec<BonusDamage>,
    pub melee_armor: f32,
    pub pierce_armor: f32,
    pub range: f32,
    pub speed: f32,
    pub unit_class: UnitClass,
    pub color: [u8; 4],
}

impl UnitKind {
    pub fn stats(self) -> UnitStats {
        match self {
            UnitKind::Villager => UnitStats {
                hp: 25.0,
                melee_damage: 3.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 1.0,
                speed: 0.8,
                unit_class: UnitClass::Villager,
                color: [200, 160, 60, 255],
            },
            UnitKind::Militia => UnitStats {
                hp: 40.0,
                melee_damage: 4.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 1.0,
                range: 1.0,
                speed: 0.9,
                unit_class: UnitClass::Infantry,
                color: [40, 80, 220, 255],
            },
            UnitKind::ManAtArms => UnitStats {
                hp: 45.0,
                melee_damage: 6.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 1.0,
                range: 1.0,
                speed: 0.9,
                unit_class: UnitClass::Infantry,
                color: [30, 60, 200, 255],
            },
            UnitKind::LongSwordsman => UnitStats {
                hp: 55.0,
                melee_damage: 9.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 1.0,
                pierce_armor: 1.0,
                range: 1.0,
                speed: 0.9,
                unit_class: UnitClass::Infantry,
                color: [25, 50, 190, 255],
            },
            UnitKind::TwoHandedSwordsman => UnitStats {
                hp: 60.0,
                melee_damage: 11.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 1.0,
                pierce_armor: 1.0,
                range: 1.0,
                speed: 0.9,
                unit_class: UnitClass::Infantry,
                color: [20, 40, 180, 255],
            },
            UnitKind::Champion => UnitStats {
                hp: 70.0,
                melee_damage: 13.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 1.0,
                pierce_armor: 1.0,
                range: 1.0,
                speed: 0.9,
                unit_class: UnitClass::Infantry,
                color: [15, 30, 170, 255],
            },
            UnitKind::Spearman => UnitStats {
                hp: 45.0,
                melee_damage: 3.0,
                pierce_damage: 0.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Cavalry, amount: 15.0 },
                ],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 1.0,
                speed: 1.0,
                unit_class: UnitClass::Infantry,
                color: [80, 50, 20, 255],
            },
            UnitKind::Pikeman => UnitStats {
                hp: 55.0,
                melee_damage: 4.0,
                pierce_damage: 0.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Cavalry, amount: 22.0 },
                ],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 1.0,
                speed: 1.0,
                unit_class: UnitClass::Infantry,
                color: [70, 45, 15, 255],
            },
            UnitKind::Halberdier => UnitStats {
                hp: 60.0,
                melee_damage: 6.0,
                pierce_damage: 0.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Cavalry, amount: 32.0 },
                ],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 1.0,
                speed: 1.0,
                unit_class: UnitClass::Infantry,
                color: [60, 40, 10, 255],
            },
            UnitKind::Archer => UnitStats {
                hp: 30.0,
                melee_damage: 0.0,
                pierce_damage: 4.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 4.0,
                speed: 0.96,
                unit_class: UnitClass::Archer,
                color: [30, 160, 30, 255],
            },
            UnitKind::Crossbowman => UnitStats {
                hp: 35.0,
                melee_damage: 0.0,
                pierce_damage: 5.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 5.0,
                speed: 0.96,
                unit_class: UnitClass::Archer,
                color: [25, 140, 25, 255],
            },
            UnitKind::Arbalester => UnitStats {
                hp: 40.0,
                melee_damage: 0.0,
                pierce_damage: 6.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 5.0,
                speed: 0.96,
                unit_class: UnitClass::Archer,
                color: [20, 120, 20, 255],
            },
            UnitKind::Skirmisher => UnitStats {
                hp: 30.0,
                melee_damage: 2.0,
                pierce_damage: 1.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Archer, amount: 3.0 },
                ],
                melee_armor: 0.0,
                pierce_armor: 3.0,
                range: 4.0,
                speed: 0.96,
                unit_class: UnitClass::Archer,
                color: [20, 130, 80, 255],
            },
            UnitKind::EliteSkirmisher => UnitStats {
                hp: 35.0,
                melee_damage: 3.0,
                pierce_damage: 1.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Archer, amount: 5.0 },
                ],
                melee_armor: 0.0,
                pierce_armor: 4.0,
                range: 5.0,
                speed: 0.96,
                unit_class: UnitClass::Archer,
                color: [15, 115, 70, 255],
            },
            UnitKind::Knight => UnitStats {
                hp: 100.0,
                melee_damage: 10.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 2.0,
                pierce_armor: 2.0,
                range: 1.0,
                speed: 1.35,
                unit_class: UnitClass::Cavalry,
                color: [180, 40, 180, 255],
            },
            UnitKind::Cavalier => UnitStats {
                hp: 120.0,
                melee_damage: 12.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 2.0,
                pierce_armor: 2.0,
                range: 1.0,
                speed: 1.35,
                unit_class: UnitClass::Cavalry,
                color: [160, 30, 160, 255],
            },
            UnitKind::Paladin => UnitStats {
                hp: 160.0,
                melee_damage: 14.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 2.0,
                pierce_armor: 3.0,
                range: 1.0,
                speed: 1.35,
                unit_class: UnitClass::Cavalry,
                color: [140, 20, 140, 255],
            },
            UnitKind::ScoutCavalry => UnitStats {
                hp: 60.0,
                melee_damage: 3.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 2.0,
                range: 1.0,
                speed: 1.55,
                unit_class: UnitClass::Cavalry,
                color: [160, 120, 40, 255],
            },
            UnitKind::LightCavalry => UnitStats {
                hp: 60.0,
                melee_damage: 7.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 2.0,
                range: 1.0,
                speed: 1.5,
                unit_class: UnitClass::Cavalry,
                color: [150, 110, 35, 255],
            },
            UnitKind::Hussar => UnitStats {
                hp: 75.0,
                melee_damage: 7.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 2.0,
                range: 1.0,
                speed: 1.5,
                unit_class: UnitClass::Cavalry,
                color: [140, 100, 30, 255],
            },
            UnitKind::BatteringRam => UnitStats {
                hp: 175.0,
                melee_damage: 2.0,
                pierce_damage: 0.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Building, amount: 125.0 },
                ],
                melee_armor: -3.0,
                pierce_armor: 180.0,
                range: 1.0,
                speed: 0.5,
                unit_class: UnitClass::Siege,
                color: [80, 60, 30, 255],
            },
            UnitKind::Mangonel => UnitStats {
                hp: 50.0,
                melee_damage: 0.0,
                pierce_damage: 40.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 6.0,
                range: 7.0,
                speed: 0.6,
                unit_class: UnitClass::Siege,
                color: [90, 70, 40, 255],
            },
            UnitKind::TradeCart => UnitStats {
                hp: 70.0,
                melee_damage: 0.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 0.0,
                speed: 1.0,
                unit_class: UnitClass::Villager,
                color: [180, 150, 50, 255],
            },
            UnitKind::Monk => UnitStats {
                hp: 25.0,
                melee_damage: 0.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 9.0,
                speed: 0.7,
                unit_class: UnitClass::Villager,
                color: [220, 200, 80, 255],
            },
            UnitKind::Scorpion => UnitStats {
                hp: 40.0,
                melee_damage: 0.0,
                pierce_damage: 12.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Siege, amount: 1.0 },
                ],
                melee_armor: 0.0,
                pierce_armor: 6.0,
                range: 7.0,
                speed: 0.65,
                unit_class: UnitClass::Siege,
                color: [85, 65, 35, 255],
            },
            UnitKind::Longbowman => UnitStats {
                hp: 35.0,
                melee_damage: 0.0,
                pierce_damage: 6.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 6.0,
                speed: 0.96,
                unit_class: UnitClass::Archer,
                color: [20, 100, 20, 255],
            },
            UnitKind::EliteLongbowman => UnitStats {
                hp: 40.0,
                melee_damage: 0.0,
                pierce_damage: 7.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 1.0,
                range: 7.0,
                speed: 0.96,
                unit_class: UnitClass::Archer,
                color: [15, 85, 15, 255],
            },
            UnitKind::ThrowingAxeman => UnitStats {
                hp: 60.0,
                melee_damage: 7.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 1.0,
                range: 3.0,
                speed: 1.0,
                unit_class: UnitClass::Infantry,
                color: [50, 50, 180, 255],
            },
            UnitKind::EliteThrowingAxeman => UnitStats {
                hp: 70.0,
                melee_damage: 8.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 1.0,
                pierce_armor: 1.0,
                range: 4.0,
                speed: 1.0,
                unit_class: UnitClass::Infantry,
                color: [40, 40, 165, 255],
            },
            UnitKind::TeutonicKnight => UnitStats {
                hp: 80.0,
                melee_damage: 12.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 5.0,
                pierce_armor: 2.0,
                range: 1.0,
                speed: 0.65,
                unit_class: UnitClass::Infantry,
                color: [100, 100, 120, 255],
            },
            UnitKind::EliteTeutonicKnight => UnitStats {
                hp: 100.0,
                melee_damage: 17.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 10.0,
                pierce_armor: 2.0,
                range: 1.0,
                speed: 0.65,
                unit_class: UnitClass::Infantry,
                color: [85, 85, 105, 255],
            },
            UnitKind::Mangudai => UnitStats {
                hp: 60.0,
                melee_damage: 0.0,
                pierce_damage: 6.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Siege, amount: 3.0 },
                ],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 4.0,
                speed: 1.45,
                unit_class: UnitClass::Cavalry,
                color: [160, 100, 30, 255],
            },
            UnitKind::EliteMangudai => UnitStats {
                hp: 60.0,
                melee_damage: 0.0,
                pierce_damage: 8.0,
                bonuses: vec![
                    BonusDamage { vs_class: UnitClass::Siege, amount: 5.0 },
                ],
                melee_armor: 0.0,
                pierce_armor: 0.0,
                range: 4.0,
                speed: 1.45,
                unit_class: UnitClass::Cavalry,
                color: [145, 90, 25, 255],
            },
            UnitKind::FishingShip => UnitStats {
                hp: 60.0,
                melee_damage: 0.0,
                pierce_damage: 0.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 4.0,
                range: 0.0,
                speed: 1.26,
                unit_class: UnitClass::Villager,
                color: [100, 140, 180, 255],
            },
            UnitKind::Galley => UnitStats {
                hp: 120.0,
                melee_damage: 0.0,
                pierce_damage: 6.0,
                bonuses: vec![],
                melee_armor: 0.0,
                pierce_armor: 6.0,
                range: 5.0,
                speed: 1.43,
                unit_class: UnitClass::Archer,
                color: [60, 80, 120, 255],
            },
        }
    }

    pub fn sprite_path(self) -> Option<&'static str> {
        match self {
            UnitKind::Villager => Some("sprites/units/villager.png"),
            _ => Some("sprites/units/militia.png"),
        }
    }

    pub fn line_of_sight(self) -> u32 {
        match self {
            UnitKind::ScoutCavalry | UnitKind::LightCavalry | UnitKind::Hussar => 10,
            UnitKind::Archer | UnitKind::Crossbowman | UnitKind::Arbalester
            | UnitKind::Longbowman | UnitKind::EliteLongbowman
            | UnitKind::Mangudai | UnitKind::EliteMangudai => 6,
            UnitKind::Skirmisher | UnitKind::EliteSkirmisher => 6,
            _ => 4,
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
            _ => &self.militia,
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
        kind,
        team,
        grid,
        Speed(stats.speed),
        Health::new(stats.hp),
        Armor::new(stats.melee_armor, stats.pierce_armor),
        stats.unit_class,
        LineOfSight(kind.line_of_sight()),
        AttackStats {
            melee_damage: stats.melee_damage,
            pierce_damage: stats.pierce_damage,
            bonuses: stats.bonuses,
            range: stats.range,
            cooldown: Timer::from_seconds(1.0, TimerMode::Repeating),
            unit_class: stats.unit_class,
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

    if kind == UnitKind::Villager || kind == UnitKind::FishingShip {
        ec.insert(crate::resources::components::Carrying::default());
    }

    if matches!(kind, UnitKind::Mangonel | UnitKind::Scorpion) {
        ec.insert(AreaDamage { radius: 1.5 });
    }

    if kind == UnitKind::Monk {
        ec.insert(MonkUnit);
    }

    if matches!(kind, UnitKind::FishingShip | UnitKind::Galley) {
        ec.insert(NavalUnit);
    }

    ec.id()
}
