use bevy::prelude::*;

/// The states the game can be in.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    /// Main menu state.
    #[default]
    Menu,
    /// Active play session.
    Play,
    /// One-frame transition used to restart play through normal setup.
    RestartPlay,
}

/// Despawn all entities marked with a screen or overlay component.
pub(crate) fn despawn_screen<T: Component>(
    to_despawn: Query<Entity, With<T>>,
    mut commands: Commands,
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
