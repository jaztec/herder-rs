use bevy::prelude::*;
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::world::tile::{GridPosition, MapConfig, TileMap, WorldBounds, WorldTile};

const BACKGROUND_SECTION_SIZE: u32 = 150;

pub fn create_world(mut tiles: ResMut<TileMap>) {
    let mut rng = rand::rng();

    println!("World size: h{}-w{}", tiles.height(), tiles.width());

    let finish_y = (0..tiles.height()).choose(&mut rng).unwrap();
    let finish_x = (0..tiles.width()).choose(&mut rng).unwrap();

    println!("Finish position set at y{}-x{}", finish_y, finish_x);

    for y in 0..tiles.height() {
        for x in 0..tiles.width() {
            let tile_options = [0_u32, 0, 0, 0, 0, 0, 1, 1, 2, 2, 2];
            let tile_index = tile_options.choose(&mut rng).unwrap();
            let mut index = *tile_index;

            if y == finish_y && x == finish_x {
                index = 3;
            }

            tiles.set(x, y, index);
        }
    }
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
