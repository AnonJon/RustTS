use bevy::prelude::*;
use crate::units::components::*;
use crate::buildings::components::*;
use crate::GameState;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioCooldowns>()
            .init_resource::<GameSounds>()
            .add_systems(Update, (
                detect_selection_sound,
                detect_attack_sound,
                detect_gather_sound,
            ).run_if(in_state(GameState::InGame)));
    }
}

/// Holds optional handles to audio assets. When audio files are added to
/// `assets/audio/`, load them in a startup system and populate these fields.
#[derive(Resource, Default)]
pub struct GameSounds {
    pub select: Option<Handle<AudioSource>>,
    pub command: Option<Handle<AudioSource>>,
    pub attack: Option<Handle<AudioSource>>,
    pub build: Option<Handle<AudioSource>>,
    pub complete: Option<Handle<AudioSource>>,
    pub gather: Option<Handle<AudioSource>>,
    pub death: Option<Handle<AudioSource>>,
    pub age_up: Option<Handle<AudioSource>>,
}

#[derive(Resource, Default)]
struct AudioCooldowns {
    selection: f32,
    attack: f32,
    gather: f32,
}

fn play_sound(commands: &mut Commands, handle: &Option<Handle<AudioSource>>) {
    if let Some(source) = handle {
        commands.spawn(AudioPlayer::new(source.clone()));
    }
}

fn detect_selection_sound(
    mut commands: Commands,
    mut cooldowns: ResMut<AudioCooldowns>,
    time: Res<Time>,
    newly_selected: Query<Entity, Added<Selected>>,
    sounds: Res<GameSounds>,
) {
    cooldowns.selection -= time.delta_secs();
    cooldowns.attack -= time.delta_secs();
    cooldowns.gather -= time.delta_secs();

    if !newly_selected.is_empty() && cooldowns.selection <= 0.0 {
        play_sound(&mut commands, &sounds.select);
        cooldowns.selection = 0.15;
    }
}

fn detect_attack_sound(
    mut commands: Commands,
    units: Query<&UnitState, (With<Unit>, Changed<UnitState>)>,
    mut cooldowns: ResMut<AudioCooldowns>,
    sounds: Res<GameSounds>,
) {
    if cooldowns.attack > 0.0 { return; }
    for state in &units {
        if *state == UnitState::Attacking {
            play_sound(&mut commands, &sounds.attack);
            cooldowns.attack = 0.3;
            break;
        }
    }
}

fn detect_gather_sound(
    mut commands: Commands,
    gatherers: Query<&UnitState, (With<Unit>, Changed<UnitState>)>,
    mut cooldowns: ResMut<AudioCooldowns>,
    sounds: Res<GameSounds>,
) {
    if cooldowns.gather > 0.0 { return; }
    for state in &gatherers {
        if matches!(state, UnitState::Gathering { .. }) {
            play_sound(&mut commands, &sounds.gather);
            cooldowns.gather = 1.0;
            break;
        }
    }
}
