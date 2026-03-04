use bevy::prelude::*;
use crate::GameSettings;
use crate::GameState;
use crate::map::generation::MapType;
use crate::civilization::{Civilization, PlayerCivilization};

#[derive(Component)]
pub struct LobbyRoot;

#[derive(Component)]
pub struct MapTypeButton(pub MapType);

#[derive(Component)]
pub struct PlayerCountLabel;

#[derive(Component)]
pub struct PlayerCountButton(pub i32); // -1 or +1

#[derive(Component)]
pub struct StartButton;

#[derive(Component)]
pub struct MapDescription;

#[derive(Component)]
pub struct CivButton(pub Civilization);

#[derive(Component)]
pub struct CivDescription;

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 4;

pub fn setup_lobby(mut commands: Commands) {
    commands
        .spawn((
            LobbyRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.06, 0.10)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    row_gap: Val::Px(24.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.10, 0.10, 0.16, 0.95)),
            ))
            .with_children(|panel| {
                // Title
                panel.spawn((
                    Text::new("Age of Rust"),
                    TextFont { font_size: 42.0, ..default() },
                    TextColor(Color::srgb(0.95, 0.80, 0.30)),
                ));

                // Subtitle
                panel.spawn((
                    Text::new("Select Map & Players"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));

                // Map type buttons
                panel.spawn((
                    Text::new("Map Type"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                ));

                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                }).with_children(|row| {
                    for map_type in MapType::ALL {
                        let is_selected = map_type == MapType::Arabia;
                        row.spawn((
                            MapTypeButton(map_type),
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(if is_selected {
                                Color::srgb(0.25, 0.45, 0.70)
                            } else {
                                Color::srgb(0.18, 0.18, 0.24)
                            }),
                        )).with_child((
                            Text::new(map_type.label()),
                            TextFont { font_size: 15.0, ..default() },
                            TextColor(Color::srgb(0.95, 0.95, 0.95)),
                        ));
                    }
                });

                // Map description
                panel.spawn((
                    MapDescription,
                    Text::new(MapType::Arabia.description()),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(0.55, 0.55, 0.55)),
                ));

                // Player count
                panel.spawn((
                    Text::new("Players"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                ));

                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(16.0),
                    ..default()
                }).with_children(|row| {
                    row.spawn((
                        PlayerCountButton(-1),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.18, 0.24)),
                    )).with_child((
                        Text::new("<"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));

                    row.spawn((
                        PlayerCountLabel,
                        Text::new("2"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                    ));

                    row.spawn((
                        PlayerCountButton(1),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.18, 0.24)),
                    )).with_child((
                        Text::new(">"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

                // Civilization selection
                panel.spawn((
                    Text::new("Civilization"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                ));

                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                }).with_children(|row| {
                    for civ in Civilization::ALL {
                        let is_selected = civ == Civilization::Britons;
                        row.spawn((
                            CivButton(civ),
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(if is_selected {
                                Color::srgb(0.25, 0.45, 0.70)
                            } else {
                                Color::srgb(0.18, 0.18, 0.24)
                            }),
                        )).with_child((
                            Text::new(civ.label()),
                            TextFont { font_size: 15.0, ..default() },
                            TextColor(Color::srgb(0.95, 0.95, 0.95)),
                        ));
                    }
                });

                panel.spawn((
                    CivDescription,
                    Text::new(Civilization::Britons.description()),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(0.55, 0.55, 0.55)),
                ));

                // Start button
                panel.spawn((
                    StartButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(40.0), Val::Px(14.0)),
                        margin: UiRect::top(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.55, 0.22)),
                )).with_child((
                    Text::new("Start Game"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                ));
            });
        });
}

pub fn teardown_lobby(mut commands: Commands, query: Query<Entity, With<LobbyRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn lobby_map_type_buttons(
    interaction_q: Query<
        (&Interaction, &MapTypeButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut all_map_btns: Query<(&MapTypeButton, &mut BackgroundColor)>,
    mut settings: ResMut<GameSettings>,
    mut desc_q: Query<&mut Text, With<MapDescription>>,
) {
    let mut pressed_type = None;
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            pressed_type = Some(btn.0);
        }
    }

    let Some(map_type) = pressed_type else { return };
    settings.map_type = map_type;

    for (map_btn, mut bg) in &mut all_map_btns {
        *bg = if map_btn.0 == settings.map_type {
            BackgroundColor(Color::srgb(0.25, 0.45, 0.70))
        } else {
            BackgroundColor(Color::srgb(0.18, 0.18, 0.24))
        };
    }

    for mut text in &mut desc_q {
        **text = settings.map_type.description().to_string();
    }
}

pub fn lobby_player_count_buttons(
    interaction_q: Query<
        (&Interaction, &PlayerCountButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings: ResMut<GameSettings>,
    mut label_q: Query<&mut Text, With<PlayerCountLabel>>,
) {
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            let new_count = (settings.num_players as i32 + btn.0)
                .clamp(MIN_PLAYERS as i32, MAX_PLAYERS as i32) as usize;
            settings.num_players = new_count;

            for mut text in &mut label_q {
                **text = new_count.to_string();
            }
        }
    }
}

pub fn lobby_start_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::InGame);
        }
    }
}

pub fn lobby_civ_buttons(
    interaction_q: Query<
        (&Interaction, &CivButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut all_civ_btns: Query<(&CivButton, &mut BackgroundColor)>,
    mut player_civ: ResMut<PlayerCivilization>,
    mut desc_q: Query<&mut Text, With<CivDescription>>,
) {
    let mut pressed_civ = None;
    for (interaction, btn) in &interaction_q {
        if *interaction == Interaction::Pressed {
            pressed_civ = Some(btn.0);
        }
    }

    let Some(civ) = pressed_civ else { return };
    player_civ.0 = civ;

    for (civ_btn, mut bg) in &mut all_civ_btns {
        *bg = if civ_btn.0 == player_civ.0 {
            BackgroundColor(Color::srgb(0.25, 0.45, 0.70))
        } else {
            BackgroundColor(Color::srgb(0.18, 0.18, 0.24))
        };
    }

    for mut text in &mut desc_q {
        **text = civ.description().to_string();
    }
}

pub fn lobby_button_hover(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            Without<MapTypeButton>,
            Without<PlayerCountButton>,
            Without<CivButton>,
        ),
    >,
) {
    for (interaction, mut bg) in &mut interaction_q {
        match *interaction {
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.22, 0.62, 0.28));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgb(0.18, 0.55, 0.22));
            }
            _ => {}
        }
    }
}
