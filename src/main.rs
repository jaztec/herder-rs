use bevy::prelude::*;

use crate::plugin::HerderGame;

mod plugin;
mod states;
mod world;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(HerderGame)
        .init_state::<states::GameState>()
        .run();
}
