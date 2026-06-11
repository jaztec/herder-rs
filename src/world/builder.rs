use bevy::prelude::*;
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::world::tile::{
    FinishTilePosition, GridPosition, MapConfig, TileMap, WorldBounds, WorldTile,
};

const BACKGROUND_SECTION_SIZE: u32 = 150;
const FINISH_EDGE_MARGIN_TILES: usize = 2;

pub fn create_world(mut commands: Commands, mut tiles: ResMut<TileMap>) {
    let mut rng = rand::rng();

    println!("World size: h{}-w{}", tiles.height(), tiles.width());

    let finish_position = random_finish_position(&tiles, &mut rng);
    commands.insert_resource(finish_position);

    println!(
        "Finish position set at y{}-x{}",
        finish_position.y, finish_position.x
    );

    for y in 0..tiles.height() {
        for x in 0..tiles.width() {
            let tile_options = [0_u32, 0, 0, 0, 0, 0, 1, 1, 2, 2, 2];
            let tile_index = tile_options.choose(&mut rng).unwrap();
            let mut index = *tile_index;

            if y == finish_position.y && x == finish_position.x {
                index = 3;
            }

            tiles.set(x, y, index);
        }
    }
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

pub fn draw_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    tiles: Res<TileMap>,
    config: Res<MapConfig>,
    mut bounds: ResMut<WorldBounds>,
) {
    let texture_handle = asset_server.load("textures/backgrounds.png");
    let texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::splat(BACKGROUND_SECTION_SIZE), 3, 3, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    bounds.size = config.world_size();

    for y in 0..tiles.height() {
        for x in 0..tiles.width() {
            let Some(tile) = tiles.get(x, y) else {
                continue;
            };

            let mut atlas = TextureAtlas::from(texture_atlas_handle.clone());
            atlas.index = tile.index();

            commands.spawn((
                WorldTile,
                *tile,
                GridPosition { x, y },
                Sprite {
                    custom_size: Some(Vec2::splat(config.tile_size)),
                    ..Sprite::from_atlas_image(texture_handle.clone(), atlas)
                },
                Transform::from_translation(config.tile_world_position(x, y)),
            ));
        }
    }
}
