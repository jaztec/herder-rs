use bevy::prelude::*;

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
