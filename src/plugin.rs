//! Root Bevy plugin for the Herder game.
//!
//! This plugin wires together the camera, menu, play, and game-over state
//! plugins. Feature-specific systems stay in their own modules.

use bevy::prelude::*;

use crate::{
    states::{GameState, game_over_state_plugin, menu_state_plugin, play_state_plugin},
    world::setup_camera,
};

/// Root plugin added by `main`.
pub struct HerderGame;

impl Plugin for HerderGame {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Startup, setup_camera)
            .add_plugins((menu_state_plugin, play_state_plugin, game_over_state_plugin));
    }
}
