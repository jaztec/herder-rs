//! Camera setup for the 2D play field.

use bevy::{core_pipeline::bloom::Bloom, prelude::*};

/// Spawn the main 2D camera.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            ..default()
        },
        Bloom::NATURAL,
    ));
}
