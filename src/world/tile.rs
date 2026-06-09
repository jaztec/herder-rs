use bevy::prelude::*;

pub const DEFAULT_MAP_WIDTH: usize = 24;
pub const DEFAULT_MAP_HEIGHT: usize = 18;
pub const DEFAULT_TILE_SIZE: f32 = 150.0;

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct MapConfig {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_MAP_WIDTH,
            height: DEFAULT_MAP_HEIGHT,
            tile_size: DEFAULT_TILE_SIZE,
        }
    }
}

impl MapConfig {
    pub fn world_size(&self) -> Vec2 {
        Vec2::new(
            self.width as f32 * self.tile_size,
            self.height as f32 * self.tile_size,
        )
    }

    pub fn tile_world_position(&self, x: usize, y: usize) -> Vec3 {
        let world_size = self.world_size();
        let left = -world_size.x / 2.0;
        let top = world_size.y / 2.0;

        Vec3::new(
            left + self.tile_size * (x as f32 + 0.5),
            top - self.tile_size * (y as f32 + 0.5),
            0.0,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct WorldBounds {
    pub size: Vec2,
}

impl From<&MapConfig> for WorldBounds {
    fn from(config: &MapConfig) -> Self {
        Self {
            size: config.world_size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Resource)]
pub struct TileMap {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

impl TileMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            tiles: vec![vec![Tile::Grass; width]; height],
            width,
            height,
        }
    }

    /// Get the tile at the given position.
    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y)?.get(x)
    }

    /// Set the tile at the given position.
    pub fn set<T>(&mut self, x: usize, y: usize, tile: T)
    where
        T: Into<Tile>,
    {
        if let Some(row) = self.tiles.get_mut(y) {
            if let Some(tile_cell) = row.get_mut(x) {
                *tile_cell = tile.into();
            }
        }
    }

    /// Get the width of the tile map.
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the tile map.
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }
}

impl From<&MapConfig> for TileMap {
    fn from(config: &MapConfig) -> Self {
        Self::new(config.width, config.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct WorldTile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
#[repr(u8)]
pub enum Tile {
    #[default]
    Grass,
    Water,
    Flowers,
    Finish,
}

impl Tile {
    /// Check if the tile is walkable.
    #[inline]
    pub fn is_walkable(&self) -> bool {
        match self {
            Tile::Grass | Tile::Water | Tile::Flowers | Tile::Finish => true,
        }
    }

    /// Get the index of the tile in the tileset.
    #[inline]
    pub fn index(&self) -> usize {
        let index: u32 = self.into();
        index as usize
    }
}

impl From<u32> for Tile {
    fn from(value: u32) -> Self {
        match value {
            0 => Tile::Grass,
            1 => Tile::Water,
            2 => Tile::Flowers,
            3 => Tile::Finish,
            _ => Tile::Grass,
        }
    }
}

impl From<Tile> for u32 {
    fn from(value: Tile) -> Self {
        impl_from_tile_for_u32(&value)
    }
}

impl From<&Tile> for u32 {
    fn from(value: &Tile) -> Self {
        impl_from_tile_for_u32(value)
    }
}

#[inline]
fn impl_from_tile_for_u32(tile: &Tile) -> u32 {
    match tile {
        Tile::Grass => 0,
        Tile::Water => 1,
        Tile::Flowers => 2,
        Tile::Finish => 3,
    }
}

impl From<u8> for Tile {
    fn from(value: u8) -> Self {
        match value {
            0 => Tile::Grass,
            1 => Tile::Water,
            2 => Tile::Flowers,
            3 => Tile::Finish,
            _ => Tile::Grass,
        }
    }
}

impl From<Tile> for u8 {
    fn from(value: Tile) -> Self {
        match value {
            Tile::Grass => 0,
            Tile::Water => 1,
            Tile::Flowers => 2,
            Tile::Finish => 3,
        }
    }
}
