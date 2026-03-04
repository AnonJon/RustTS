use bevy::prelude::*;
use super::components::*;
use crate::units::components::*;
use crate::buildings::components::*;
use crate::map::TILE_SIZE;

#[derive(Resource)]
pub struct MarketPrices {
    pub food_price: u32,
    pub wood_price: u32,
    pub stone_price: u32,
}

impl Default for MarketPrices {
    fn default() -> Self {
        Self {
            food_price: 100,
            wood_price: 100,
            stone_price: 100,
        }
    }
}

impl MarketPrices {
    pub fn buy(&mut self, kind: ResourceKind, resources: &mut PlayerResources) -> bool {
        let price = self.price_of(kind);
        if resources.gold < price {
            return false;
        }
        resources.gold -= price;
        resources.add(kind, 100);
        *self.price_mut(kind) = (self.price_of(kind) + 3).min(9999);
        true
    }

    pub fn sell(&mut self, kind: ResourceKind, resources: &mut PlayerResources) -> bool {
        let amount = 100u32;
        let has_enough = match kind {
            ResourceKind::Food => resources.food >= amount,
            ResourceKind::Wood => resources.wood >= amount,
            ResourceKind::Stone => resources.stone >= amount,
            ResourceKind::Gold => false,
        };
        if !has_enough {
            return false;
        }
        match kind {
            ResourceKind::Food => resources.food -= amount,
            ResourceKind::Wood => resources.wood -= amount,
            ResourceKind::Stone => resources.stone -= amount,
            ResourceKind::Gold => {}
        }
        let price = self.price_of(kind);
        resources.gold += price;
        *self.price_mut(kind) = self.price_of(kind).saturating_sub(3).max(20);
        true
    }

    fn price_of(&self, kind: ResourceKind) -> u32 {
        match kind {
            ResourceKind::Food => self.food_price,
            ResourceKind::Wood => self.wood_price,
            ResourceKind::Stone => self.stone_price,
            ResourceKind::Gold => 100,
        }
    }

    fn price_mut(&mut self, kind: ResourceKind) -> &mut u32 {
        match kind {
            ResourceKind::Food => &mut self.food_price,
            ResourceKind::Wood => &mut self.wood_price,
            ResourceKind::Stone => &mut self.stone_price,
            ResourceKind::Gold => &mut self.food_price, // unused
        }
    }
}

#[derive(Component)]
pub struct TradeRoute {
    pub home_market: Entity,
    pub target_market: Option<Entity>,
    pub going_to_target: bool,
    pub gold_earned: u32,
}

pub fn trade_cart_system(
    mut commands: Commands,
    mut carts: Query<(Entity, &Transform, &mut TradeRoute, &Team), With<Unit>>,
    markets: Query<(Entity, &Transform, &Building, &Team), Without<Unit>>,
    mut resources: ResMut<PlayerResources>,
) {
    for (cart_entity, cart_tf, mut route, cart_team) in &mut carts {
        if cart_team.0 != 0 { continue; }

        if route.target_market.is_none() {
            let home_pos = markets.get(route.home_market)
                .map(|(_, tf, _, _)| tf.translation.truncate());
            let Ok(home_pos) = home_pos else { continue };

            let mut best: Option<(Entity, f32)> = None;
            for (m_entity, m_tf, m_bld, m_team) in &markets {
                if m_bld.kind != BuildingKind::Market { continue; }
                if m_entity == route.home_market { continue; }
                if m_team.0 != cart_team.0 { continue; }
                let dist = home_pos.distance(m_tf.translation.truncate());
                if best.is_none() || dist > best.unwrap().1 {
                    best = Some((m_entity, dist));
                }
            }

            if let Some((target, _)) = best {
                route.target_market = Some(target);
                route.going_to_target = true;
                if let Ok((_, target_tf, _, _)) = markets.get(target) {
                    commands.entity(cart_entity)
                        .insert(MoveTarget(target_tf.translation.truncate()));
                }
            }
            continue;
        }

        let target_entity = if route.going_to_target {
            route.target_market.unwrap()
        } else {
            route.home_market
        };

        let Ok((_, target_tf, _, _)) = markets.get(target_entity) else {
            route.target_market = None;
            continue;
        };

        let dist = cart_tf.translation.truncate().distance(target_tf.translation.truncate());

        if dist < TILE_SIZE * 2.0 {
            if route.going_to_target {
                let home_pos = markets.get(route.home_market)
                    .map(|(_, tf, _, _)| tf.translation.truncate())
                    .unwrap_or_default();
                let trade_dist = home_pos.distance(target_tf.translation.truncate());
                let gold = ((trade_dist / TILE_SIZE) * 0.5).max(1.0) as u32;
                resources.gold += gold;
                route.gold_earned += gold;
            }

            route.going_to_target = !route.going_to_target;
            let next = if route.going_to_target {
                route.target_market.unwrap()
            } else {
                route.home_market
            };
            if let Ok((_, next_tf, _, _)) = markets.get(next) {
                commands.entity(cart_entity)
                    .insert(MoveTarget(next_tf.translation.truncate()));
            }
        }
    }
}
