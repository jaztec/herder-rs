use bevy::prelude::*;

use crate::{
    states::{
        GameState,
        play::shepherd::{
            animate_shepherd, move_camera, move_shepherd, setup_shepherd,
            update_shepherd_animation_range,
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
        .add_systems(OnEnter(GameState::Play), setup_play_state)
        .add_systems(
            OnEnter(PlayState::Playing),
            (create_world, draw_world, setup_shepherd).chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                move_shepherd,
                update_shepherd_animation_range,
                animate_shepherd,
                move_camera,
            )
                .chain(),
        );
}

fn setup_play_state(mut game_state: ResMut<NextState<PlayState>>) {
    game_state.set(PlayState::Playing);
}
