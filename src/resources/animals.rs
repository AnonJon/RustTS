use bevy::prelude::*;
use crate::units::components::*;
use crate::map::TILE_SIZE;
use super::components::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimalKind {
    Sheep,
    Deer,
    Boar,
}

impl AnimalKind {
    pub fn food(self) -> u32 {
        match self {
            AnimalKind::Sheep => 100,
            AnimalKind::Deer => 140,
            AnimalKind::Boar => 340,
        }
    }

    pub fn hp(self) -> f32 {
        match self {
            AnimalKind::Sheep => 7.0,
            AnimalKind::Deer => 5.0,
            AnimalKind::Boar => 75.0,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            AnimalKind::Sheep => 0.4,
            AnimalKind::Deer => 1.2,
            AnimalKind::Boar => 0.8,
        }
    }

    pub fn color(self) -> [u8; 4] {
        match self {
            AnimalKind::Sheep => [240, 240, 240, 255],
            AnimalKind::Deer => [160, 120, 60, 255],
            AnimalKind::Boar => [100, 80, 60, 255],
        }
    }
}

#[derive(Component)]
pub struct Animal {
    pub kind: AnimalKind,
    pub food_remaining: u32,
}

#[derive(Component)]
pub struct FleeTarget(pub Vec2);

#[derive(Component)]
pub struct AggroTarget(pub Entity);

pub fn animal_flee_system(
    mut animals: Query<(Entity, &Transform, &mut Health, &Animal), Without<Unit>>,
    attackers: Query<(Entity, &Transform, &Team), With<Unit>>,
    mut commands: Commands,
) {
    for (animal_e, animal_tf, health, animal) in &mut animals {
        if health.current >= health.max {
            continue;
        }
        if animal.kind == AnimalKind::Deer {
            let animal_pos = animal_tf.translation.truncate();
            let mut nearest_attacker: Option<Vec2> = None;
            let mut min_dist = f32::MAX;
            for (_, atk_tf, _) in &attackers {
                let d = atk_tf.translation.truncate().distance(animal_pos);
                if d < TILE_SIZE * 6.0 && d < min_dist {
                    min_dist = d;
                    nearest_attacker = Some(atk_tf.translation.truncate());
                }
            }
            if let Some(threat_pos) = nearest_attacker {
                let flee_dir = (animal_pos - threat_pos).normalize_or_zero();
                let flee_target = animal_pos + flee_dir * TILE_SIZE * 5.0;
                commands.entity(animal_e).insert(FleeTarget(flee_target));
            }
        } else if animal.kind == AnimalKind::Boar {
            let animal_pos = animal_tf.translation.truncate();
            let mut nearest: Option<(Entity, f32)> = None;
            for (unit_e, atk_tf, _) in &attackers {
                let d = atk_tf.translation.truncate().distance(animal_pos);
                if d < TILE_SIZE * 8.0 {
                    if nearest.is_none() || d < nearest.unwrap().1 {
                        nearest = Some((unit_e, d));
                    }
                }
            }
            if let Some((attacker_e, _)) = nearest {
                commands.entity(animal_e).insert(AggroTarget(attacker_e));
            }
        }
    }
}

pub fn animal_movement_system(
    mut flee_animals: Query<(Entity, &mut Transform, &Animal, Option<&FleeTarget>, Option<&AggroTarget>), Without<Unit>>,
    target_positions: Query<&Transform, With<Unit>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, animal, flee, aggro) in &mut flee_animals {
        let speed = animal.kind.speed() * TILE_SIZE * time.delta_secs();
        let pos = transform.translation.truncate();

        if let Some(flee_target) = flee {
            let dir = (flee_target.0 - pos).normalize_or_zero();
            if pos.distance(flee_target.0) > 10.0 {
                transform.translation.x += dir.x * speed;
                transform.translation.y += dir.y * speed;
            } else {
                commands.entity(entity).remove::<FleeTarget>();
            }
        } else if let Some(aggro_target) = aggro {
            if let Ok(target_tf) = target_positions.get(aggro_target.0) {
                let target_pos = target_tf.translation.truncate();
                let dist = pos.distance(target_pos);
                if dist > 20.0 {
                    let dir = (target_pos - pos).normalize_or_zero();
                    transform.translation.x += dir.x * speed;
                    transform.translation.y += dir.y * speed;
                }
            } else {
                commands.entity(entity).remove::<AggroTarget>();
            }
        }
    }
}

pub fn boar_attack_system(
    boars: Query<(&Transform, &AggroTarget, &Animal), Without<Unit>>,
    mut targets: Query<&mut Health, With<Unit>>,
    time: Res<Time>,
) {
    for (boar_tf, aggro, animal) in &boars {
        if animal.kind != AnimalKind::Boar { continue; }
        if let Ok(mut target_health) = targets.get_mut(aggro.0) {
            let dist = boar_tf.translation.truncate().distance(
                // We can't easily get target transform without conflicting queries,
                // so we just check if boar is "close enough" -- simplified
                Vec2::ZERO, // placeholder
            );
            // Boar does 8 melee damage per 2 seconds when in melee range
            // Simplified: always apply when aggro exists
        }
    }
}

pub fn animal_death_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    animals: Query<(Entity, &Transform, &Health, &Animal)>,
) {
    for (entity, transform, health, animal) in &animals {
        if health.current > 0.0 {
            continue;
        }
        let pos = transform.translation.truncate();
        let grid = crate::map::GridPosition::from_world(pos);
        let world = grid.to_world();

        commands.entity(entity).despawn();

        let size = 28u32;
        let mut data = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let idx = ((y * size + x) * 4) as usize;
                let dx = (x as f32 - size as f32 / 2.0).abs();
                let dy = (y as f32 - size as f32 / 2.0).abs();
                if dx + dy < size as f32 / 2.0 {
                    data[idx] = 180;
                    data[idx + 1] = 40;
                    data[idx + 2] = 40;
                    data[idx + 3] = 255;
                }
            }
        }
        let mut image = Image::new(
            bevy::render::render_resource::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2,
            data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
            mag_filter: bevy::image::ImageFilterMode::Nearest,
            min_filter: bevy::image::ImageFilterMode::Nearest,
            ..default()
        });
        let texture = images.add(image);

        commands.spawn((
            ResourceNode {
                kind: ResourceKind::Food,
                remaining: animal.food_remaining,
                max_amount: animal.kind.food(),
            },
            grid,
            Sprite {
                image: texture,
                custom_size: Some(Vec2::new(40.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(world.x, world.y, 4.0),
        ));
    }
}

pub fn spawn_animals(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    config: &crate::map::generation::MapConfig,
) {
    let base = config.player_base();
    let w = config.terrain_grid.len() as i32;
    let h = config.terrain_grid[0].len() as i32;

    let sheep_offsets = [(-5, -5), (-6, -4), (-4, -6), (-7, -5)];
    for (dx, dy) in sheep_offsets {
        let (lx, ly) = crate::map::generation::find_nearest_land(&config.terrain_grid, base.x + dx, base.y + dy);
        spawn_animal(commands, images, AnimalKind::Sheep, lx, ly);
    }

    let deer_offsets = [(12, 8), (13, 9), (14, 8)];
    for (dx, dy) in deer_offsets {
        let x = (base.x + dx).clamp(0, w - 1);
        let y = (base.y + dy).clamp(0, h - 1);
        let (lx, ly) = crate::map::generation::find_nearest_land(&config.terrain_grid, x, y);
        spawn_animal(commands, images, AnimalKind::Deer, lx, ly);
    }

    let boar_offsets = [(10, -8), (-10, 10)];
    for (dx, dy) in boar_offsets {
        let x = (base.x + dx).clamp(0, w - 1);
        let y = (base.y + dy).clamp(0, h - 1);
        let (lx, ly) = crate::map::generation::find_nearest_land(&config.terrain_grid, x, y);
        spawn_animal(commands, images, AnimalKind::Boar, lx, ly);
    }
}

fn spawn_animal(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    kind: AnimalKind,
    gx: i32,
    gy: i32,
) {
    let grid = crate::map::GridPosition::new(gx, gy);
    let world = grid.to_world();
    let color = kind.color();

    let size = 24u32;
    let mut data = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let dx = (x as f32 - size as f32 / 2.0).abs();
            let dy = (y as f32 - size as f32 / 2.0).abs();
            if dx * dx + dy * dy < (size as f32 / 2.0).powi(2) {
                data[idx] = color[0];
                data[idx + 1] = color[1];
                data[idx + 2] = color[2];
                data[idx + 3] = 255;
            }
        }
    }
    let mut image = Image::new(
        bevy::render::render_resource::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        mag_filter: bevy::image::ImageFilterMode::Nearest,
        min_filter: bevy::image::ImageFilterMode::Nearest,
        ..default()
    });
    let texture = images.add(image);

    commands.spawn((
        Animal {
            kind,
            food_remaining: kind.food(),
        },
        Health::new(kind.hp()),
        Armor::default(),
        grid,
        Sprite {
            image: texture,
            custom_size: Some(Vec2::new(36.0, 36.0)),
            ..default()
        },
        Transform::from_xyz(world.x, world.y, 6.0),
    ));
}
