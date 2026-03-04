use bevy::prelude::*;
use crate::units::components::*;
use crate::buildings::components::*;
use crate::GameState;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioCooldowns>()
            .init_resource::<GameSounds>()
            .add_systems(OnEnter(GameState::InGame), load_game_sounds)
            .add_systems(Update, (
                detect_selection_sound,
                detect_attack_sound,
                detect_gather_sound,
                detect_build_complete_sound,
                detect_age_up_sound,
                detect_death_sound,
            ).run_if(in_state(GameState::InGame)));
    }
}

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
    build_complete: f32,
    death: f32,
}

fn try_load(asset_server: &AssetServer, path: &str) -> Option<Handle<AudioSource>> {
    let full_path = format!("audio/{path}");
    if std::path::Path::new(&format!("assets/audio/{path}")).exists() {
        Some(asset_server.load(full_path))
    } else {
        None
    }
}

fn load_game_sounds(
    mut sounds: ResMut<GameSounds>,
    asset_server: Res<AssetServer>,
) {
    sounds.select = try_load(&asset_server, "select.ogg");
    sounds.command = try_load(&asset_server, "command.ogg");
    sounds.attack = try_load(&asset_server, "attack.ogg");
    sounds.build = try_load(&asset_server, "build.ogg");
    sounds.complete = try_load(&asset_server, "complete.ogg");
    sounds.gather = try_load(&asset_server, "gather.ogg");
    sounds.death = try_load(&asset_server, "death.ogg");
    sounds.age_up = try_load(&asset_server, "age_up.ogg");
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
    cooldowns.build_complete -= time.delta_secs();
    cooldowns.death -= time.delta_secs();

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

fn detect_build_complete_sound(
    mut commands: Commands,
    buildings: Query<(&Building, &Team), Added<Building>>,
    sounds: Res<GameSounds>,
    mut cooldowns: ResMut<AudioCooldowns>,
) {
    if cooldowns.build_complete > 0.0 { return; }
    for (_, team) in &buildings {
        if team.0 == 0 {
            play_sound(&mut commands, &sounds.complete);
            cooldowns.build_complete = 0.5;
            break;
        }
    }
}

fn detect_age_up_sound(
    mut commands: Commands,
    age: Res<CurrentAge>,
    sounds: Res<GameSounds>,
) {
    if age.is_changed() && !age.is_added() {
        play_sound(&mut commands, &sounds.age_up);
    }
}

fn detect_death_sound(
    mut commands: Commands,
    dead_units: Query<(&Health, &Team), (With<Unit>, Changed<Health>)>,
    sounds: Res<GameSounds>,
    mut cooldowns: ResMut<AudioCooldowns>,
) {
    if cooldowns.death > 0.0 { return; }
    for (health, _team) in &dead_units {
        if health.current <= 0.0 {
            play_sound(&mut commands, &sounds.death);
            cooldowns.death = 0.2;
            break;
        }
    }
}
