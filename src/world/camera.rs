use bevy::{core_pipeline::bloom::Bloom, prelude::*};

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
