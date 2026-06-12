//! Entry point for the Herder Bevy application.
//!
//! The game is currently a binary crate. Local code documentation is generated
//! with `cargo doc --document-private-items` so internal Bevy systems and
//! components show up in the rustdoc output.

use bevy::prelude::*;

use crate::plugin::HerderGame;

mod plugin;
mod run_config;
mod states;
mod world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(HerderGame)
        .init_state::<states::GameState>()
        .run();
}
