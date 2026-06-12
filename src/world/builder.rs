//! Procedural world creation and tile rendering systems.

use std::collections::VecDeque;

use bevy::prelude::*;
use rand::{Rng, seq::IteratorRandom};

use crate::{
    run_config::{RunConfig, TerrainAmount},
    world::tile::{
        FinishTilePosition, GridPosition, MapConfig, Tile, TileMap, WorldBounds, WorldTile,
    },
};

const BACKGROUND_SECTION_SIZE: u32 = 150;
const AUTOTILE_SECTION_SIZE: u32 = 150;
const FINISH_EDGE_MARGIN_TILES: usize = 2;
const WATER_BLOB_AREA_DIVISOR: usize = 140;
const FLOWER_BLOB_AREA_DIVISOR: usize = 100;
const PATH_AREA_DIVISOR: usize = 150;
const AUTOTILE_NORTH: usize = 1;
const AUTOTILE_EAST: usize = 2;
const AUTOTILE_SOUTH: usize = 4;
const AUTOTILE_WEST: usize = 8;

struct TerrainAtlasHandles {
    base_texture: Handle<Image>,
    water_texture: Handle<Image>,
    flower_texture: Handle<Image>,
    path_texture: Handle<Image>,
    base_layout: Handle<TextureAtlasLayout>,
    autotile_layout: Handle<TextureAtlasLayout>,
}

/// Fill the tile map for a new run and choose a finish tile.
///
/// The finish tile is constrained away from the edge when the map is large
/// enough. Its grid position is stored as a resource so scoring, indicators,
/// and spawn placement all share the same source of truth.
pub fn create_world(
    mut commands: Commands,
    run_config: Res<RunConfig>,
    mut tiles: ResMut<TileMap>,
) {
    let mut rng = rand::rng();

    println!("World size: h{}-w{}", tiles.height(), tiles.width());

    let finish_position = random_finish_position(&tiles, &mut rng);
    commands.insert_resource(finish_position);

    println!(
        "Finish position set at y{}-x{}",
        finish_position.y, finish_position.x
    );

    let flower_blob_count = flower_blob_count(&tiles, run_config.terrain.flowers);
    let water_blob_count = water_blob_count(&tiles, run_config.terrain.water);
    let path_count = path_count(&tiles, run_config.terrain.paths);

    fill_tiles(&mut tiles, Tile::Grass);
    paint_tile_blobs(
        &mut tiles,
        Tile::Flowers,
        flower_blob_count,
        1.4..=3.0,
        &mut rng,
    );
    paint_tile_blobs(
        &mut tiles,
        Tile::Water,
        water_blob_count,
        2.2..=4.3,
        &mut rng,
    );
    paint_paths(&mut tiles, path_count, &mut rng);
    carve_start_to_finish_corridor(&mut tiles, finish_position);
    clear_start_area(&mut tiles);
    remove_disconnected_walkable_regions(&mut tiles);
    tiles.set(finish_position.x, finish_position.y, Tile::Finish);
}

fn random_finish_position(tiles: &TileMap, rng: &mut impl rand::Rng) -> FinishTilePosition {
    let x_margin = edge_margin_for(tiles.width(), FINISH_EDGE_MARGIN_TILES);
    let y_margin = edge_margin_for(tiles.height(), FINISH_EDGE_MARGIN_TILES);

    FinishTilePosition {
        x: (x_margin..tiles.width() - x_margin).choose(rng).unwrap(),
        y: (y_margin..tiles.height() - y_margin).choose(rng).unwrap(),
    }
}

fn edge_margin_for(size: usize, preferred_margin: usize) -> usize {
    preferred_margin.min(size.saturating_sub(1) / 2)
}

fn fill_tiles(tiles: &mut TileMap, tile: Tile) {
    for y in 0..tiles.height() {
        for x in 0..tiles.width() {
            tiles.set(x, y, tile);
        }
    }
}

fn water_blob_count(tiles: &TileMap, amount: TerrainAmount) -> usize {
    scaled_count(
        (tiles.width() * tiles.height() / WATER_BLOB_AREA_DIVISOR).clamp(1, 4),
        amount,
    )
}

fn flower_blob_count(tiles: &TileMap, amount: TerrainAmount) -> usize {
    scaled_count(
        (tiles.width() * tiles.height() / FLOWER_BLOB_AREA_DIVISOR).clamp(2, 7),
        amount,
    )
}

fn path_count(tiles: &TileMap, amount: TerrainAmount) -> usize {
    scaled_count(
        (tiles.width() * tiles.height() / PATH_AREA_DIVISOR).clamp(2, 5),
        amount,
    )
}

fn scaled_count(default_count: usize, amount: TerrainAmount) -> usize {
    (default_count as f32 * amount.multiplier()).round() as usize
}

fn paint_tile_blobs(
    tiles: &mut TileMap,
    tile: Tile,
    count: usize,
    radius_range: std::ops::RangeInclusive<f32>,
    rng: &mut impl Rng,
) {
    for _ in 0..count {
        let center_x = rng.random_range(0..tiles.width()) as f32;
        let center_y = rng.random_range(0..tiles.height()) as f32;
        let radius_x = rng.random_range(radius_range.clone());
        let radius_y = rng.random_range(radius_range.clone());

        for y in 0..tiles.height() {
            for x in 0..tiles.width() {
                let dx = (x as f32 - center_x) / radius_x;
                let dy = (y as f32 - center_y) / radius_y;
                let wobble = 0.16 * ((x as f32 * 1.7 + y as f32 * 0.9).sin());

                if dx * dx + dy * dy <= 1.0 + wobble {
                    tiles.set(x, y, tile);
                }
            }
        }
    }
}

fn paint_paths(tiles: &mut TileMap, count: usize, rng: &mut impl Rng) {
    for _ in 0..count {
        let mut x = rng.random_range(0..tiles.width());
        let mut y = rng.random_range(0..tiles.height());
        let mut direction = random_cardinal_direction(rng);
        let steps =
            rng.random_range(tiles.width().min(tiles.height())..=tiles.width() + tiles.height());

        for _ in 0..steps {
            if tiles.get(x, y).is_some_and(|tile| *tile != Tile::Water) {
                tiles.set(x, y, Tile::Path);
            }

            if rng.random_bool(0.24) {
                direction = random_cardinal_direction(rng);
            }

            match direction {
                CardinalDirection::North => y = y.saturating_sub(1),
                CardinalDirection::East => x = (x + 1).min(tiles.width() - 1),
                CardinalDirection::South => y = (y + 1).min(tiles.height() - 1),
                CardinalDirection::West => x = x.saturating_sub(1),
            }
        }
    }
}

fn clear_start_area(tiles: &mut TileMap) {
    let center_x = tiles.width() / 2;
    let center_y = tiles.height() / 2;

    clear_walkable_radius(tiles, center_x, center_y, 1, Tile::Grass);
}

fn carve_start_to_finish_corridor(tiles: &mut TileMap, finish: FinishTilePosition) {
    let mut x = tiles.width() / 2;
    let mut y = tiles.height() / 2;

    while x != finish.x {
        clear_walkable_radius(tiles, x, y, 1, Tile::Path);
        if x < finish.x {
            x += 1;
        } else {
            x -= 1;
        }
    }

    while y != finish.y {
        clear_walkable_radius(tiles, x, y, 1, Tile::Path);
        if y < finish.y {
            y += 1;
        } else {
            y -= 1;
        }
    }

    clear_walkable_radius(tiles, x, y, 1, Tile::Path);
}

fn clear_walkable_radius(
    tiles: &mut TileMap,
    center_x: usize,
    center_y: usize,
    radius: usize,
    tile: Tile,
) {
    for y in center_y.saturating_sub(radius)..=(center_y + radius).min(tiles.height() - 1) {
        for x in center_x.saturating_sub(radius)..=(center_x + radius).min(tiles.width() - 1) {
            if x.abs_diff(center_x) <= radius && y.abs_diff(center_y) <= radius {
                tiles.set(x, y, tile);
            }
        }
    }
}

fn remove_disconnected_walkable_regions(tiles: &mut TileMap) {
    let start_x = tiles.width() / 2;
    let start_y = tiles.height() / 2;
    let mut reachable = vec![vec![false; tiles.width()]; tiles.height()];
    let mut queue = VecDeque::from([(start_x, start_y)]);

    while let Some((x, y)) = queue.pop_front() {
        if reachable[y][x] || !tiles.get(x, y).is_some_and(|tile| tile.is_walkable()) {
            continue;
        }

        reachable[y][x] = true;

        if x > 0 {
            queue.push_back((x - 1, y));
        }
        if x + 1 < tiles.width() {
            queue.push_back((x + 1, y));
        }
        if y > 0 {
            queue.push_back((x, y - 1));
        }
        if y + 1 < tiles.height() {
            queue.push_back((x, y + 1));
        }
    }

    for (y, row) in reachable.iter().enumerate() {
        for (x, is_reachable) in row.iter().enumerate() {
            if !is_reachable && tiles.get(x, y).is_some_and(|tile| tile.is_walkable()) {
                tiles.set(x, y, Tile::Water);
            }
        }
    }
}

fn random_cardinal_direction(rng: &mut impl Rng) -> CardinalDirection {
    match rng.random_range(0..4) {
        0 => CardinalDirection::North,
        1 => CardinalDirection::East,
        2 => CardinalDirection::South,
        _ => CardinalDirection::West,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardinalDirection {
    North,
    East,
    South,
    West,
}

/// Spawn one sprite entity for each tile in the current tile map.
pub fn draw_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    tiles: Res<TileMap>,
    config: Res<MapConfig>,
    mut bounds: ResMut<WorldBounds>,
) {
    let base_texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::splat(BACKGROUND_SECTION_SIZE), 3, 3, None, None);
    let autotile_texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::splat(AUTOTILE_SECTION_SIZE), 4, 4, None, None);
    let terrain_atlases = TerrainAtlasHandles {
        base_texture: asset_server.load("textures/backgrounds.png"),
        water_texture: asset_server.load("textures/water_autotile.png"),
        flower_texture: asset_server.load("textures/flowers_autotile.png"),
        path_texture: asset_server.load("textures/path_autotile.png"),
        base_layout: texture_atlases.add(base_texture_atlas),
        autotile_layout: texture_atlases.add(autotile_texture_atlas),
    };

    bounds.size = config.world_size();

    for y in 0..tiles.height() {
        for x in 0..tiles.width() {
            let Some(tile) = tiles.get(x, y) else {
                continue;
            };

            let (texture, layout, index) = tile_visual(&tiles, x, y, *tile, &terrain_atlases);

            let mut atlas = TextureAtlas::from(layout);
            atlas.index = index;

            commands.spawn((
                WorldTile,
                *tile,
                GridPosition { x, y },
                Sprite {
                    custom_size: Some(Vec2::splat(config.tile_size)),
                    ..Sprite::from_atlas_image(texture, atlas)
                },
                Transform::from_translation(config.tile_world_position(x, y)),
            ));
        }
    }
}

fn tile_visual(
    tiles: &TileMap,
    x: usize,
    y: usize,
    tile: Tile,
    atlases: &TerrainAtlasHandles,
) -> (Handle<Image>, Handle<TextureAtlasLayout>, usize) {
    match tile {
        Tile::Grass => (atlases.base_texture.clone(), atlases.base_layout.clone(), 0),
        Tile::Finish => (atlases.base_texture.clone(), atlases.base_layout.clone(), 3),
        Tile::Water => (
            atlases.water_texture.clone(),
            atlases.autotile_layout.clone(),
            autotile_mask(tiles, x, y, tile),
        ),
        Tile::Flowers => (
            atlases.flower_texture.clone(),
            atlases.autotile_layout.clone(),
            autotile_mask(tiles, x, y, tile),
        ),
        Tile::Path => (
            atlases.path_texture.clone(),
            atlases.autotile_layout.clone(),
            autotile_mask(tiles, x, y, tile),
        ),
    }
}

fn autotile_mask(tiles: &TileMap, x: usize, y: usize, tile: Tile) -> usize {
    let mut mask = 0;

    if y > 0
        && tiles
            .get(x, y - 1)
            .is_some_and(|neighbor| *neighbor == tile)
    {
        mask |= AUTOTILE_NORTH;
    }
    if tiles
        .get(x + 1, y)
        .is_some_and(|neighbor| *neighbor == tile)
    {
        mask |= AUTOTILE_EAST;
    }
    if tiles
        .get(x, y + 1)
        .is_some_and(|neighbor| *neighbor == tile)
    {
        mask |= AUTOTILE_SOUTH;
    }
    if x > 0
        && tiles
            .get(x - 1, y)
            .is_some_and(|neighbor| *neighbor == tile)
    {
        mask |= AUTOTILE_WEST;
    }

    mask
}
