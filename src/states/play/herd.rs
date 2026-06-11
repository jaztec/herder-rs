//! Herd marker component.

use bevy::prelude::*;

/// Marker for the herd entity that owns no behavior itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub struct Herd;
