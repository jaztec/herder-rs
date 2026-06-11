//! Shared components and helpers used by animated actors.

use bevy::prelude::*;

/// Cardinal direction an actor is facing.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum FacingDirection {
    /// Facing right.
    Right,
    /// Facing left.
    Left,
    /// Facing down toward the camera.
    Down,
    /// Facing up away from the camera.
    Up,
}

/// Load a texture and its corresponding texture atlas layout.
pub fn load_texture<A>(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    filename: &str,
    tile_size: UVec2,
) -> (Handle<A>, Handle<TextureAtlasLayout>)
where
    A: Asset,
{
    let texture_handle = asset_server.load(filename);
    let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 4, 5, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    (texture_handle, texture_atlas_handle)
}

/// Scalar movement speed component.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    /// Movement speed in world units per second.
    pub speed: f32,
}

/// Current actor facing direction.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Facing {
    /// Direction used for animation-frame selection.
    pub direction: FacingDirection,
}

/// Whether an actor should use moving or standing animation frames.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Moving {
    /// True while movement input or AI steering produces velocity.
    pub is_moving: bool,
}

/// Repeating timer used by sprite animation systems.
#[derive(Component, Debug)]
pub struct AnimationTimer {
    /// Bevy timer controlling when the next frame is selected.
    pub timer: Timer,
}

/// Inclusive sprite-atlas frame range for the current animation.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationFrames {
    /// First atlas index in the animation range.
    pub first: usize,
    /// Last atlas index in the animation range.
    pub last: usize,
}

impl AnimationFrames {
    /// Create an animation range containing one still frame.
    pub const fn single(index: usize) -> Self {
        Self {
            first: index,
            last: index,
        }
    }

    /// Create an inclusive animation range.
    pub const fn range(first: usize, last: usize) -> Self {
        Self { first, last }
    }
}
