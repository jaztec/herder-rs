use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum MovementDirection {
    Right = 5,
    Left = 10,
    Down = 15,
    Up = 20,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum FacingDirection {
    Right,
    Left,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum MovementFrame {
    First,
    Second,
    Third,
    Fourth,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum Standing {
    Down,
    Up,
    Right,
    Left,
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
