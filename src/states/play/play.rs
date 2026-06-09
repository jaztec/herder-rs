use bevy::prelude::*;

use crate::{
    states::{
        GameState,
        play::shepherd::{move_camera, move_shepherd, setup_shepherd},
    },
    world::{TileMap, create_world, draw_world},
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, States)]
enum PlayState {
    #[default]
    Disabled,
    Playing,
}

pub fn play_state_plugin(app: &mut App) {
    app.init_state::<PlayState>()
        .insert_resource(TileMap::new(8, 4))
        .add_systems(OnEnter(GameState::Play), setup_play_state)
        .add_systems(
            OnEnter(PlayState::Playing),
            (create_world, draw_world, setup_shepherd).chain(),
        )
        .add_systems(FixedUpdate, (move_shepherd, move_camera));
}

fn setup_play_state(mut game_state: ResMut<NextState<PlayState>>) {
    game_state.set(PlayState::Playing);
}
