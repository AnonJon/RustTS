use bevy::prelude::*;
use super::components::*;

#[derive(Component)]
pub struct AnimationConfig {
    pub idle_frames: u32,
    pub walk_frames: u32,
    pub attack_frames: u32,
    pub frame_timer: Timer,
    pub current_frame: u32,
}

impl AnimationConfig {
    pub fn new(idle: u32, walk: u32, attack: u32, fps: f32) -> Self {
        Self {
            idle_frames: idle,
            walk_frames: walk,
            attack_frames: attack,
            frame_timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
            current_frame: 0,
        }
    }

    fn frame_count(&self, state: &UnitState) -> u32 {
        match state {
            UnitState::Idle => 1,
            UnitState::Moving
            | UnitState::Gathering { .. }
            | UnitState::Returning { .. }
            | UnitState::FarmingAt { .. }
            | UnitState::Constructing { .. } => self.walk_frames,
            UnitState::Attacking => self.attack_frames,
            _ => 1,
        }
    }
}

pub fn animation_system(
    mut query: Query<(&mut AnimationConfig, &mut Sprite, &UnitState), With<Unit>>,
    time: Res<Time>,
) {
    for (mut anim, mut sprite, state) in &mut query {
        anim.frame_timer.tick(time.delta());
        if anim.frame_timer.just_finished() {
            let frame_count = anim.frame_count(state);
            if frame_count > 0 {
                anim.current_frame = (anim.current_frame + 1) % frame_count;
            }
        }

        let frame_count = anim.frame_count(state);
        if frame_count > 0 && anim.current_frame >= frame_count {
            anim.current_frame = 0;
        }

        let base_offset = match state {
            UnitState::Idle => 0,
            UnitState::Moving
            | UnitState::Gathering { .. }
            | UnitState::Returning { .. }
            | UnitState::FarmingAt { .. }
            | UnitState::Constructing { .. } => anim.idle_frames,
            UnitState::Attacking => anim.idle_frames + anim.walk_frames,
            _ => 0,
        };

        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = (base_offset + anim.current_frame) as usize;
        }
    }
}

pub fn facing_system(
    mut query: Query<(&mut Sprite, &Transform, Option<&MoveTarget>, Option<&AttackTarget>), With<Unit>>,
    target_transforms: Query<&Transform, Without<Speed>>,
) {
    for (mut sprite, transform, move_target, attack_target) in &mut query {
        let target_pos = if let Some(attack) = attack_target {
            target_transforms.get(attack.0).ok().map(|t| t.translation.truncate())
        } else {
            move_target.map(|m| m.0)
        };

        if let Some(target) = target_pos {
            let diff = target.x - transform.translation.x;
            sprite.flip_x = diff < 0.0;
        }
    }
}
