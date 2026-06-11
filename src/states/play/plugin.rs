use bevy::prelude::*;

use crate::{
    states::{
        GameState,
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
}

pub fn play_state_plugin(app: &mut App) {
    let map_config = MapConfig::default();

    app.init_state::<PlayState>()
        .insert_resource(map_config)
        .insert_resource(TileMap::from(&map_config))
        .insert_resource(WorldBounds::from(&map_config))
        .init_resource::<DogRoute>()
        .init_resource::<HerdScore>()
        .add_systems(OnEnter(GameState::Play), setup_play_state)
        .add_systems(
            OnEnter(PlayState::Playing),
            (
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
