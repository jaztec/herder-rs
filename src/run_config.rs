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
    /// Optional deterministic generation seed.
    pub seed: Option<u64>,
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
            seed: None,
        }
    }
}

impl RunConfig {
    /// Build a run config for a campaign level.
    pub fn from_campaign_level(level_index: usize) -> Self {
        let level = campaign_level(level_index);

        Self {
            mode: PlayMode::Campaign {
                level_index: level.index,
            },
            map: level.map,
            sheep_count: level.sheep_count,
            terrain: level.terrain,
            seed: Some(level.seed),
        }
    }

    /// Stable score-table id for this run.
    pub fn score_table_id(&self) -> String {
        match &self.mode {
            PlayMode::Random => "random".to_string(),
            PlayMode::Campaign { level_index } => {
                format!("campaign_{}", campaign_level(*level_index).id)
            }
        }
    }

    /// Human-readable score-table name.
    pub fn score_table_name(&self) -> String {
        match &self.mode {
            PlayMode::Random => "Random Map".to_string(),
            PlayMode::Campaign { level_index } => {
                let level = campaign_level(*level_index);
                format!("Campaign {}: {}", level.index + 1, level.name)
            }
        }
    }
}

/// Cumulative progress for the current campaign attempt.
#[derive(Debug, Clone, PartialEq, Resource)]
pub struct CampaignProgress {
    /// Number of completed campaign levels in the current attempt.
    pub completed_levels: usize,
    /// Sum of level scores in the current attempt.
    pub total_score: u32,
    /// Sum of level times in the current attempt.
    pub total_seconds: f32,
}

impl Default for CampaignProgress {
    fn default() -> Self {
        Self {
            completed_levels: 0,
            total_score: 0,
            total_seconds: 0.0,
        }
    }
}

impl CampaignProgress {
    /// Reset the campaign attempt before starting level one.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Add a completed level to the campaign attempt.
    pub fn record_level(&mut self, score: u32, seconds: f32) {
        self.completed_levels += 1;
        self.total_score += score;
        self.total_seconds += seconds;
    }

    /// Return true when the full campaign has been completed.
    pub fn is_complete(&self) -> bool {
        self.completed_levels >= CAMPAIGN_LEVELS.len()
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

/// Fixed campaign level definition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CampaignLevel {
    /// Zero-based campaign index.
    pub index: usize,
    /// Stable identifier for save data and highscores.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Deterministic map-generation seed.
    pub seed: u64,
    /// Level map dimensions.
    pub map: MapConfig,
    /// Number of sheep in this level.
    pub sheep_count: usize,
    /// Terrain generation density.
    pub terrain: TerrainConfig,
}

/// Campaign maps in order.
pub const CAMPAIGN_LEVELS: [CampaignLevel; 20] = [
    campaign_level_definition(
        0,
        "meadow-1",
        "Meadow Start",
        10_001,
        18,
        14,
        18,
        low_terrain(),
    ),
    campaign_level_definition(
        1,
        "meadow-2",
        "Wide Meadow",
        10_113,
        20,
        15,
        20,
        low_terrain(),
    ),
    campaign_level_definition(
        2,
        "pond-1",
        "First Pond",
        10_229,
        22,
        16,
        22,
        normal_terrain(),
    ),
    campaign_level_definition(3, "path-1", "Old Track", 10_337, 22, 16, 24, path_terrain()),
    campaign_level_definition(
        4,
        "flowers-1",
        "Flower Field",
        10_441,
        24,
        18,
        26,
        flower_terrain(),
    ),
    campaign_level_definition(
        5,
        "pond-2",
        "Twin Ponds",
        10_559,
        24,
        18,
        28,
        normal_terrain(),
    ),
    campaign_level_definition(6, "path-2", "Long Path", 10_667, 26, 18, 30, path_terrain()),
    campaign_level_definition(
        7,
        "water-1",
        "Wet Grass",
        10_771,
        26,
        20,
        32,
        water_terrain(),
    ),
    campaign_level_definition(
        8,
        "mixed-1",
        "Broken Ground",
        10_889,
        28,
        20,
        34,
        normal_terrain(),
    ),
    campaign_level_definition(
        9,
        "flowers-2",
        "Heavy Flowers",
        10_991,
        28,
        20,
        36,
        flower_terrain(),
    ),
    campaign_level_definition(
        10,
        "path-3",
        "Cross Roads",
        11_107,
        30,
        21,
        38,
        path_terrain(),
    ),
    campaign_level_definition(
        11,
        "water-2",
        "Lake Edge",
        11_213,
        30,
        21,
        40,
        water_terrain(),
    ),
    campaign_level_definition(
        12,
        "mixed-2",
        "Far Pasture",
        11_327,
        30,
        22,
        42,
        normal_terrain(),
    ),
    campaign_level_definition(
        13,
        "pond-3",
        "Deep Ponds",
        11_431,
        32,
        22,
        44,
        water_terrain(),
    ),
    campaign_level_definition(
        14,
        "flowers-3",
        "Slow Bloom",
        11_543,
        32,
        22,
        46,
        flower_terrain(),
    ),
    campaign_level_definition(
        15,
        "path-4",
        "Fast Lanes",
        11_659,
        34,
        23,
        48,
        path_terrain(),
    ),
    campaign_level_definition(
        16,
        "mixed-3",
        "Open Range",
        11_761,
        34,
        24,
        50,
        normal_terrain(),
    ),
    campaign_level_definition(
        17,
        "water-3",
        "Marshlands",
        11_879,
        36,
        24,
        52,
        water_terrain(),
    ),
    campaign_level_definition(
        18,
        "mixed-4",
        "Final Drive",
        11_983,
        36,
        25,
        55,
        normal_terrain(),
    ),
    campaign_level_definition(
        19,
        "final",
        "The Big Herd",
        12_101,
        38,
        26,
        60,
        hard_terrain(),
    ),
];

/// Return a campaign level, clamped to the available campaign range.
pub fn campaign_level(level_index: usize) -> CampaignLevel {
    CAMPAIGN_LEVELS[level_index.min(CAMPAIGN_LEVELS.len() - 1)]
}

#[expect(
    clippy::too_many_arguments,
    reason = "campaign level data is clearer as a compact table"
)]
const fn campaign_level_definition(
    index: usize,
    id: &'static str,
    name: &'static str,
    seed: u64,
    width: usize,
    height: usize,
    sheep_count: usize,
    terrain: TerrainConfig,
) -> CampaignLevel {
    CampaignLevel {
        index,
        id,
        name,
        seed,
        map: MapConfig {
            width,
            height,
            tile_size: DEFAULT_TILE_SIZE,
        },
        sheep_count,
        terrain,
    }
}

const fn low_terrain() -> TerrainConfig {
    TerrainConfig {
        water: TerrainAmount::Low,
        flowers: TerrainAmount::Low,
        paths: TerrainAmount::Normal,
    }
}

const fn normal_terrain() -> TerrainConfig {
    TerrainConfig {
        water: TerrainAmount::Normal,
        flowers: TerrainAmount::Normal,
        paths: TerrainAmount::Normal,
    }
}

const fn path_terrain() -> TerrainConfig {
    TerrainConfig {
        water: TerrainAmount::Low,
        flowers: TerrainAmount::Normal,
        paths: TerrainAmount::High,
    }
}

const fn flower_terrain() -> TerrainConfig {
    TerrainConfig {
        water: TerrainAmount::Low,
        flowers: TerrainAmount::High,
        paths: TerrainAmount::Normal,
    }
}

const fn water_terrain() -> TerrainConfig {
    TerrainConfig {
        water: TerrainAmount::High,
        flowers: TerrainAmount::Normal,
        paths: TerrainAmount::Normal,
    }
}

const fn hard_terrain() -> TerrainConfig {
    TerrainConfig {
        water: TerrainAmount::High,
        flowers: TerrainAmount::High,
        paths: TerrainAmount::Low,
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
