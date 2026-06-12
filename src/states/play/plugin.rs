//! Play-state plugin and scheduling.
//!
//! This module wires setup, cleanup, pause handling, and the fixed-update
//! gameplay pipeline together behind Bevy states.

use bevy::prelude::*;

use crate::{
    run_config::RunConfig,
    states::{
        GameState,
        game_state::despawn_screen,
        play::{
            dog::{
                Dog, DogRoute, RouteWaypoint, animate_dog, handle_dog_route_input, move_dog,
                reset_dog_route, setup_dog, setup_dog_audio, update_dog_animation_range,
            },
            finish::{
                finish_state_plugin, finish_when_herd_complete, highscore_text_for_overlay,
                reset_run_timer, tick_run_timer,
            },
            herd::Herd,
            indicators::{IndicatorRoot, setup_indicators, update_indicators},
            score::{
                HerdScore, ScoreHud, setup_finish_area, setup_herd_score, setup_score_hud,
                update_score_hud,
            },
            sheep::{
                Sheep, animate_sheep, decay_sheep_panic, finish_sheep, move_sheep,
                play_idle_sheep_sounds, propagate_sheep_panic, sense_sheep_threats, setup_herd,
                setup_sheep_audio, steer_sheep, update_sheep_animation_range,
            },
            shepherd::{
                Shepherd, animate_shepherd, move_camera, move_shepherd, setup_shepherd,
                update_shepherd_animation_range, zoom_camera,
            },
        },
    },
    world::{TileMap, WorldBounds, WorldTile, create_world, draw_world},
};

/// Nested state for the active play session.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, States)]
pub(in crate::states::play) enum PlayState {
    /// No play session is currently active.
    #[default]
    Disabled,
    /// Gameplay systems are running.
    Playing,
    /// Gameplay is paused and the pause overlay is visible.
    Paused,
    /// All sheep have been herded and the finish overlay is visible.
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct PauseOverlay;

type PlayCleanupFilter = Or<(
    With<WorldTile>,
    With<Shepherd>,
    With<Dog>,
    With<Sheep>,
    With<Herd>,
    With<RouteWaypoint>,
    With<IndicatorRoot>,
    With<ScoreHud>,
    With<PauseOverlay>,
)>;

type PlayCleanupQuery<'w, 's> = Query<'w, 's, Entity, PlayCleanupFilter>;

/// Register all play-state resources, systems, and nested plugins.
pub fn play_state_plugin(app: &mut App) {
    app.add_plugins(finish_state_plugin)
        .init_state::<PlayState>()
        .init_resource::<DogRoute>()
        .init_resource::<HerdScore>()
        .add_systems(OnEnter(GameState::RestartPlay), restart_play)
        .add_systems(
            OnEnter(GameState::Play),
            (
                setup_play_state,
                setup_run_resources,
                reset_dog_route,
                create_world,
                setup_finish_area,
                setup_herd_score,
                reset_run_timer,
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
        .add_systems(OnExit(GameState::Play), cleanup_play_entities)
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
                tick_run_timer,
                sense_sheep_threats,
                propagate_sheep_panic,
                steer_sheep,
                decay_sheep_panic,
                move_sheep,
                finish_sheep,
                finish_when_herd_complete,
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

fn setup_run_resources(mut commands: Commands, run_config: Res<RunConfig>) {
    commands.insert_resource(run_config.map);
    commands.insert_resource(TileMap::from(&run_config.map));
    commands.insert_resource(WorldBounds::from(&run_config.map));
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
        PlayState::Disabled | PlayState::Finished => {}
    }
}

fn setup_pause_overlay(mut commands: Commands, score: Res<HerdScore>, run_config: Res<RunConfig>) {
    let highscore_text = highscore_text_for_overlay(&run_config);

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
                width: Val::Px(430.0),
                padding: UiRect::all(Val::Px(28.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
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
                (
                    Text::new(highscore_text),
                    TextFont {
                        font_size: 19.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.84, 0.78)),
                ),
            ],
        )],
    ));
}

fn restart_play(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::Play);
}

fn cleanup_play_entities(mut commands: Commands, entities: PlayCleanupQuery) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
