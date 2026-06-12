//! Play-session configuration shared by menus and gameplay setup.

use bevy::prelude::*;

use crate::world::{DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH, DEFAULT_TILE_SIZE, MapConfig};

/// Configuration for the next play session.
#[derive(Debug, Clone, PartialEq, Resource)]
pub struct RunConfig {
    /// Selected play mode.
    pub mode: PlayMode,
    /// Generated map dimensions and tile size.
    pub map: MapConfig,
    /// Number of sheep spawned for the run.
    pub sheep_count: usize,
    /// Terrain generation density settings.
    pub terrain: TerrainConfig,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            mode: PlayMode::Random,
            map: MapConfig {
                width: DEFAULT_MAP_WIDTH,
                height: DEFAULT_MAP_HEIGHT,
                tile_size: DEFAULT_TILE_SIZE,
            },
            sheep_count: 30,
            terrain: TerrainConfig::default(),
        }
    }
}

/// High-level mode for the active or next run.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayMode {
    /// Procedurally generated one-off map.
    Random,
    /// Ordered campaign level.
    Campaign { level_index: usize },
}

/// Density settings for generated non-grass terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrainConfig {
    /// Amount of blocked water terrain.
    pub water: TerrainAmount,
    /// Amount of slow flower terrain.
    pub flowers: TerrainAmount,
    /// Amount of fast path terrain.
    pub paths: TerrainAmount,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            water: TerrainAmount::Normal,
            flowers: TerrainAmount::Normal,
            paths: TerrainAmount::Normal,
        }
    }
}

/// Discrete terrain amount used by menus and campaign definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainAmount {
    /// Disable this terrain kind.
    None,
    /// Generate less than the default amount.
    Low,
    /// Current default terrain amount.
    Normal,
    /// Generate more than the default amount.
    High,
}

impl TerrainAmount {
    /// Convert the setting to a generation multiplier.
    pub fn multiplier(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Low => 0.55,
            Self::Normal => 1.0,
            Self::High => 1.55,
        }
    }

    /// Return the next setting for simple menu cycling.
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Low,
            Self::Low => Self::Normal,
            Self::Normal => Self::High,
            Self::High => Self::None,
        }
    }

    /// Human-readable menu label.
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
        }
    }
}
