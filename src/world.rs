//! World generation, tile data, and camera setup.
//!
//! The world module owns the tile map resources and the systems that turn those
//! resources into renderable tile entities.

mod builder;
mod camera;
mod pathfinding;
mod tile;

pub use builder::{create_world, draw_world};
pub use camera::setup_camera;
pub use pathfinding::find_path;
pub use tile::{FinishTilePosition, GridPosition, MapConfig, TileMap, WorldBounds, WorldTile};
