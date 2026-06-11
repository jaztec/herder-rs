//! Game-state modules.
//!
//! The project is split into state plugins: menu, play, and game over. The play
//! state owns most gameplay systems.

mod game_over;
mod game_state;
mod menu;
mod play;

pub use game_over::game_over_state_plugin;
pub use game_state::GameState;
pub use menu::menu_state_plugin;
pub use play::play_state_plugin;
