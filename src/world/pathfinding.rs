//! Grid pathfinding over the generated terrain map.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bevy::prelude::*;

use crate::world::tile::{GridPosition, MapConfig, Tile, TileMap};

const MIN_STEP_COST: u32 = 80;
const BASE_STEP_COST: f32 = 100.0;
const SUBDIVISIONS: usize = 3;

/// Find a walkable path between two world-space positions.
///
/// The returned points are 3x3 sub-tile center world positions. Straight runs
/// are reduced to corner points so callers do not need to follow every subcell.
pub fn find_path(
    tiles: &TileMap,
    config: &MapConfig,
    start: Vec2,
    goal: Vec2,
) -> Option<Vec<Vec2>> {
    let start = PathNode::from_world(config, start)?;
    let goal = PathNode::from_world(config, goal)?;

    if !node_tile(tiles, start).is_some_and(|tile| tile.is_walkable())
        || !node_tile(tiles, goal).is_some_and(|tile| tile.is_walkable())
    {
        return None;
    }

    if start == goal {
        return Some(vec![goal.to_world(config)]);
    }

    let path = find_node_path(tiles, start, goal)?;
    Some(
        simplify_node_path(&path)
            .into_iter()
            .map(|position| position.to_world(config))
            .collect(),
    )
}

fn find_node_path(tiles: &TileMap, start: PathNode, goal: PathNode) -> Option<Vec<PathNode>> {
    let node_count = subgrid_width(tiles) * subgrid_height(tiles);
    let mut open = BinaryHeap::new();
    let mut came_from = vec![None; node_count];
    let mut g_score = vec![u32::MAX; node_count];

    g_score[node_index(tiles, start)] = 0;
    open.push(SearchNode {
        position: start,
        known_cost: 0,
        estimated_total_cost: heuristic(start, goal),
    });

    while let Some(current) = open.pop() {
        if current.position == goal {
            return Some(reconstruct_path(tiles, came_from, start, goal));
        }

        if current.known_cost > g_score[node_index(tiles, current.position)] {
            continue;
        }

        for neighbor in neighbors(tiles, current.position) {
            let Some(tile) = node_tile(tiles, neighbor) else {
                continue;
            };
            if !tile.is_walkable() {
                continue;
            }

            let tentative_cost = current.known_cost + movement_cost(tile);
            let neighbor_index = node_index(tiles, neighbor);
            if tentative_cost >= g_score[neighbor_index] {
                continue;
            }

            came_from[neighbor_index] = Some(current.position);
            g_score[neighbor_index] = tentative_cost;
            open.push(SearchNode {
                position: neighbor,
                known_cost: tentative_cost,
                estimated_total_cost: tentative_cost + heuristic(neighbor, goal),
            });
        }
    }

    None
}

fn reconstruct_path(
    tiles: &TileMap,
    came_from: Vec<Option<PathNode>>,
    start: PathNode,
    goal: PathNode,
) -> Vec<PathNode> {
    let mut path = vec![goal];
    let mut current = goal;

    while current != start {
        let Some(previous) = came_from[node_index(tiles, current)] else {
            break;
        };
        current = previous;
        path.push(current);
    }

    path.reverse();
    path
}

fn simplify_node_path(path: &[PathNode]) -> Vec<PathNode> {
    if path.len() <= 2 {
        return path.iter().copied().skip(1).collect();
    }

    let mut simplified = Vec::new();

    for index in 1..path.len() - 1 {
        let previous = path[index - 1];
        let current = path[index];
        let next = path[index + 1];

        let previous_direction = (
            current.x as isize - previous.x as isize,
            current.y as isize - previous.y as isize,
        );
        let next_direction = (
            next.x as isize - current.x as isize,
            next.y as isize - current.y as isize,
        );

        if previous_direction != next_direction {
            simplified.push(current);
        }
    }

    simplified.push(*path.last().expect("path contains at least one tile"));
    simplified
}

fn neighbors(tiles: &TileMap, position: PathNode) -> impl Iterator<Item = PathNode> {
    let mut neighbors = Vec::with_capacity(4);
    let width = subgrid_width(tiles);
    let height = subgrid_height(tiles);

    if position.y > 0 {
        neighbors.push(PathNode {
            x: position.x,
            y: position.y - 1,
        });
    }
    if position.x + 1 < width {
        neighbors.push(PathNode {
            x: position.x + 1,
            y: position.y,
        });
    }
    if position.y + 1 < height {
        neighbors.push(PathNode {
            x: position.x,
            y: position.y + 1,
        });
    }
    if position.x > 0 {
        neighbors.push(PathNode {
            x: position.x - 1,
            y: position.y,
        });
    }

    neighbors.into_iter()
}

fn movement_cost(tile: Tile) -> u32 {
    (BASE_STEP_COST / tile.movement_speed_multiplier()).round() as u32
}

fn heuristic(start: PathNode, goal: PathNode) -> u32 {
    ((start.x.abs_diff(goal.x) + start.y.abs_diff(goal.y)) as u32) * MIN_STEP_COST
}

fn node_tile(tiles: &TileMap, position: PathNode) -> Option<Tile> {
    let tile = position.to_tile();
    tiles.get(tile.x, tile.y).copied()
}

fn node_index(tiles: &TileMap, position: PathNode) -> usize {
    position.y * subgrid_width(tiles) + position.x
}

fn subgrid_width(tiles: &TileMap) -> usize {
    tiles.width() * SUBDIVISIONS
}

fn subgrid_height(tiles: &TileMap) -> usize {
    tiles.height() * SUBDIVISIONS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SearchNode {
    position: PathNode,
    known_cost: u32,
    estimated_total_cost: u32,
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .estimated_total_cost
            .cmp(&self.estimated_total_cost)
            .then_with(|| other.known_cost.cmp(&self.known_cost))
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PathNode {
    x: usize,
    y: usize,
}

impl PathNode {
    fn from_world(config: &MapConfig, position: Vec2) -> Option<Self> {
        let world_size = config.world_size();
        let left = -world_size.x / 2.0;
        let top = world_size.y / 2.0;
        let cell_size = config.tile_size / SUBDIVISIONS as f32;

        let local_x = position.x - left;
        let local_y = top - position.y;

        if local_x < 0.0 || local_y < 0.0 || local_x >= world_size.x || local_y >= world_size.y {
            return None;
        }

        Some(Self {
            x: (local_x / cell_size).floor() as usize,
            y: (local_y / cell_size).floor() as usize,
        })
    }

    fn to_world(self, config: &MapConfig) -> Vec2 {
        let world_size = config.world_size();
        let left = -world_size.x / 2.0;
        let top = world_size.y / 2.0;
        let cell_size = config.tile_size / SUBDIVISIONS as f32;

        Vec2::new(
            left + cell_size * (self.x as f32 + 0.5),
            top - cell_size * (self.y as f32 + 0.5),
        )
    }

    fn to_tile(self) -> GridPosition {
        GridPosition {
            x: self.x / SUBDIVISIONS,
            y: self.y / SUBDIVISIONS,
        }
    }
}
