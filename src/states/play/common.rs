use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum FacingDirection {
    Right,
    Left,
    Down,
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

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    pub speed: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Facing {
    pub direction: FacingDirection,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Moving {
    pub is_moving: bool,
}

#[derive(Component, Debug)]
pub struct AnimationTimer {
    pub timer: Timer,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationFrames {
    pub first: usize,
    pub last: usize,
}

impl AnimationFrames {
    pub const fn single(index: usize) -> Self {
        Self {
            first: index,
            last: index,
        }
    }

    pub const fn range(first: usize, last: usize) -> Self {
        Self { first, last }
    }
}
