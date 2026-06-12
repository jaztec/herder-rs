//! Tile-map data structures and coordinate conversion helpers.

use bevy::prelude::*;

/// Default generated map width in tiles.
pub const DEFAULT_MAP_WIDTH: usize = 24;
/// Default generated map height in tiles.
pub const DEFAULT_MAP_HEIGHT: usize = 18;
/// Default square tile size in world units.
pub const DEFAULT_TILE_SIZE: f32 = 150.0;

/// Configuration for the generated tile map.
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct MapConfig {
    /// Number of tiles along the x axis.
    pub width: usize,
    /// Number of tiles along the y axis.
    pub height: usize,
    /// Size of one square tile in world units.
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
    /// Return the full world size in world units.
    pub fn world_size(&self) -> Vec2 {
        Vec2::new(
            self.width as f32 * self.tile_size,
            self.height as f32 * self.tile_size,
        )
    }

    /// Return the world-space center of a tile.
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

    /// Convert a world-space position to the tile that contains it.
    pub fn world_tile_position(&self, position: Vec2) -> Option<GridPosition> {
        let world_size = self.world_size();
        let left = -world_size.x / 2.0;
        let top = world_size.y / 2.0;

        let local_x = position.x - left;
        let local_y = top - position.y;

        if local_x < 0.0 || local_y < 0.0 || local_x >= world_size.x || local_y >= world_size.y {
            return None;
        }

        Some(GridPosition {
            x: (local_x / self.tile_size).floor() as usize,
            y: (local_y / self.tile_size).floor() as usize,
        })
    }
}

/// Current world bounds in world units.
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct WorldBounds {
    /// Width and height of the playable world.
    pub size: Vec2,
}

impl From<&MapConfig> for WorldBounds {
    fn from(config: &MapConfig) -> Self {
        Self {
            size: config.world_size(),
        }
    }
}

/// Dense tile grid used as the source data for map rendering and gameplay.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Resource)]
pub struct TileMap {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

impl TileMap {
    /// Create a grass-filled tile map.
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

    /// Return the tile containing a world-space position.
    pub fn tile_at_world_position(&self, config: &MapConfig, position: Vec2) -> Option<Tile> {
        let position = config.world_tile_position(position)?;
        self.get(position.x, position.y).copied()
    }

    /// Return true when the world-space position sits on a walkable tile.
    pub fn is_world_position_walkable(&self, config: &MapConfig, position: Vec2) -> bool {
        self.tile_at_world_position(config, position)
            .is_some_and(|tile| tile.is_walkable())
    }

    /// Return true when an actor-sized rectangle fits on walkable terrain.
    pub fn is_world_rect_walkable(
        &self,
        config: &MapConfig,
        center: Vec2,
        half_size: Vec2,
    ) -> bool {
        [
            center,
            center + Vec2::new(-half_size.x, -half_size.y),
            center + Vec2::new(half_size.x, -half_size.y),
            center + Vec2::new(-half_size.x, half_size.y),
            center + Vec2::new(half_size.x, half_size.y),
        ]
        .into_iter()
        .all(|position| self.is_world_position_walkable(config, position))
    }

    /// Movement multiplier for the tile containing a world-space position.
    pub fn movement_speed_at_world_position(&self, config: &MapConfig, position: Vec2) -> f32 {
        self.tile_at_world_position(config, position)
            .map_or(0.0, |tile| tile.movement_speed_multiplier())
    }

    /// Set the tile at the given position.
    pub fn set<T>(&mut self, x: usize, y: usize, tile: T)
    where
        T: Into<Tile>,
    {
        if let Some(row) = self.tiles.get_mut(y)
            && let Some(tile_cell) = row.get_mut(x)
        {
            *tile_cell = tile.into();
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

/// Tile position in grid coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct GridPosition {
    /// Horizontal tile index.
    pub x: usize,
    /// Vertical tile index.
    pub y: usize,
}

/// Grid position of the finish tile for the current run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Resource)]
pub struct FinishTilePosition {
    /// Horizontal tile index.
    pub x: usize,
    /// Vertical tile index.
    pub y: usize,
}

/// Marker component for spawned tile entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct WorldTile;

/// Tile kinds available in the terrain atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
#[repr(u8)]
pub enum Tile {
    /// Basic grass tile.
    #[default]
    Grass,
    /// Water tile that blocks actor movement.
    Water,
    /// Decorative flower tile.
    Flowers,
    /// Goal tile that consumes sheep.
    Finish,
    /// Dirt path tile that connects cardinally to neighboring paths.
    Path,
}

impl Tile {
    /// Check if the tile is walkable.
    #[inline]
    pub fn is_walkable(&self) -> bool {
        match self {
            Tile::Grass | Tile::Flowers | Tile::Finish | Tile::Path => true,
            Tile::Water => false,
        }
    }

    /// Relative movement speed for actors crossing this terrain.
    #[inline]
    pub fn movement_speed_multiplier(&self) -> f32 {
        match self {
            Tile::Path => 1.25,
            Tile::Grass | Tile::Finish => 1.0,
            Tile::Flowers => 0.72,
            Tile::Water => 0.0,
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
            4 => Tile::Path,
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
        Tile::Path => 4,
    }
}

impl From<u8> for Tile {
    fn from(value: u8) -> Self {
        match value {
            0 => Tile::Grass,
            1 => Tile::Water,
            2 => Tile::Flowers,
            3 => Tile::Finish,
            4 => Tile::Path,
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
            Tile::Path => 4,
        }
    }
}
