use bevy::prelude::*;

use crate::{
    states::{
        GameState,
        game_state::despawn_screen,
        play::{
            dog::{
                DogRoute, animate_dog, handle_dog_route_input, move_dog, setup_dog,
                setup_dog_audio, update_dog_animation_range,
            },
            indicators::{setup_indicators, update_indicators},
            score::{
                HerdScore, setup_finish_area, setup_herd_score, setup_score_hud, update_score_hud,
            },
            sheep::{
                animate_sheep, decay_sheep_panic, finish_sheep, move_sheep, play_idle_sheep_sounds,
                propagate_sheep_panic, sense_sheep_threats, setup_herd, setup_sheep_audio,
                steer_sheep, update_sheep_animation_range,
            },
            shepherd::{
                animate_shepherd, move_camera, move_shepherd, setup_shepherd,
                update_shepherd_animation_range, zoom_camera,
            },
        },
    },
    world::{MapConfig, TileMap, WorldBounds, create_world, draw_world},
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, States)]
enum PlayState {
    #[default]
    Disabled,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct PauseOverlay;

pub fn play_state_plugin(app: &mut App) {
    let map_config = MapConfig::default();

    app.init_state::<PlayState>()
        .insert_resource(map_config)
        .insert_resource(TileMap::from(&map_config))
        .insert_resource(WorldBounds::from(&map_config))
        .init_resource::<DogRoute>()
        .init_resource::<HerdScore>()
        .add_systems(
            OnEnter(GameState::Play),
            (
                setup_play_state,
                create_world,
                setup_finish_area,
                setup_herd_score,
                draw_world,
                setup_score_hud,
                setup_shepherd,
                setup_dog,
                setup_dog_audio,
                setup_sheep_audio,
                setup_herd,
                setup_indicators,
            )
                .chain(),
        )
        .add_systems(OnEnter(PlayState::Paused), setup_pause_overlay)
        .add_systems(OnExit(PlayState::Paused), despawn_screen::<PauseOverlay>)
        .add_systems(Update, toggle_pause.run_if(in_state(GameState::Play)))
        .add_systems(
            Update,
            (handle_dog_route_input, zoom_camera).run_if(in_state(PlayState::Playing)),
        )
        .add_systems(
            FixedUpdate,
            (
                move_shepherd,
                move_dog,
                sense_sheep_threats,
                propagate_sheep_panic,
                steer_sheep,
                decay_sheep_panic,
                move_sheep,
                finish_sheep,
                update_shepherd_animation_range,
                update_dog_animation_range,
                update_sheep_animation_range,
                animate_shepherd,
                animate_dog,
                animate_sheep,
                play_idle_sheep_sounds,
                update_score_hud,
                move_camera,
                update_indicators,
            )
                .chain()
                .run_if(in_state(PlayState::Playing)),
        );
}

fn setup_play_state(mut game_state: ResMut<NextState<PlayState>>) {
    game_state.set(PlayState::Playing);
}

fn toggle_pause(
    input: Res<ButtonInput<KeyCode>>,
    play_state: Res<State<PlayState>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    if !input.just_pressed(KeyCode::Escape) {
        return;
    }

    match play_state.get() {
        PlayState::Playing => next_play_state.set(PlayState::Paused),
        PlayState::Paused => next_play_state.set(PlayState::Playing),
        PlayState::Disabled => {}
    }
}

fn setup_pause_overlay(mut commands: Commands, score: Res<HerdScore>) {
    commands.spawn((
        PauseOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.58)),
        children![(
            Node {
                width: Val::Px(360.0),
                padding: UiRect::all(Val::Px(28.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.07, 0.07, 0.92)),
            children![
                (
                    Text::new("Paused"),
                    TextFont {
                        font_size: 42.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.96, 0.94, 0.86)),
                ),
                (
                    Text::new(score.pause_label()),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.86)),
                ),
            ],
        )],
    ));
}
