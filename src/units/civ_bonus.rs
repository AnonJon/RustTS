use bevy::prelude::*;
use super::components::*;
use crate::civilization::PlayerCivilization;

pub fn apply_civ_bonuses(
    mut commands: Commands,
    mut units: Query<
        (Entity, &UnitClass, &mut Health, &mut Speed, &mut Armor, &mut AttackStats),
        With<NeedsCivBonus>,
    >,
    civ: Res<PlayerCivilization>,
) {
    let c = civ.0;
    for (entity, &unit_class, mut health, mut speed, mut armor, mut attack) in &mut units {
        match unit_class {
            UnitClass::Cavalry => {
                let mult = c.cavalry_hp_multiplier();
                if mult != 1.0 {
                    health.max *= mult;
                    health.current *= mult;
                }
            }
            UnitClass::Infantry => {
                let bonus = c.infantry_melee_armor_bonus();
                if bonus > 0.0 {
                    armor.melee += bonus;
                }
            }
            UnitClass::Archer => {
                let bonus = c.archer_range_bonus();
                if bonus > 0.0 {
                    attack.range += bonus;
                }
            }
            UnitClass::Siege => {
                let mult = c.siege_speed_multiplier();
                if mult != 1.0 {
                    speed.0 *= mult;
                }
            }
            _ => {}
        }
        commands.entity(entity).remove::<NeedsCivBonus>();
    }
}
