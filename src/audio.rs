use bevy::prelude::*;
use crate::units::components::*;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioCooldowns>()
            .add_systems(Update, (
                play_selection_sound,
                play_attack_sound,
            ));
    }
}

#[derive(Resource, Default)]
struct AudioCooldowns {
    selection_cooldown: f32,
    attack_cooldown: f32,
}

fn play_selection_sound(
    mut cooldowns: ResMut<AudioCooldowns>,
    time: Res<Time>,
    _selected: Query<Entity, Added<Selected>>,
) {
    cooldowns.selection_cooldown -= time.delta_secs();
    cooldowns.attack_cooldown -= time.delta_secs();
}

fn play_attack_sound(
    _attackers: Query<&UnitState, (With<Unit>, Changed<UnitState>)>,
) {
    // Placeholder: when audio assets are added, play sounds here
    // For now just detect state changes for future SFX hookup
}
