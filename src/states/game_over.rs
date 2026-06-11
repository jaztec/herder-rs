//! Placeholder game-over plugin.
//!
//! The current game flow uses the finish play sub-state instead of a separate
//! game-over screen, but the plugin remains as a future extension point.

use bevy::prelude::*;

/// Register game-over systems.
pub fn game_over_state_plugin(_app: &mut App) {}
