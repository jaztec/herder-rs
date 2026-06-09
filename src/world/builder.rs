use bevy::prelude::*;
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::world::tile::TileMap;

const BACKGROUND_SECTION_SIZE: u32 = 150;
const BACKGROUND_SECTION_SIZE_F32: f32 = BACKGROUND_SECTION_SIZE as f32;

pub fn create_world(mut tiles: ResMut<TileMap>) {
    let mut rng = rand::rng();

    println!("World size: h{}-w{}", tiles.height(), tiles.width());

    let finish_y = (0..tiles.height() - 1).choose(&mut rng).unwrap();
    let finish_x = (0..tiles.width() - 1).choose(&mut rng).unwrap();

    println!("Finish position set at y{}-x{}", finish_y, finish_x);

    for y in 0..tiles.height() {
        for x in 0..tiles.width() {
            let tile_options: Vec<u32> = vec![0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 2];
            let tile_index = tile_options.choose(&mut rng).unwrap();
            let mut index = *tile_index;

            if y == finish_y && x == finish_x {
                index = 3;
            }

            tiles.set(y, x, index);
        }
    }
}

pub fn draw_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    tiles: Res<TileMap>,
) {
    let texture_handle = asset_server.load("textures/backgrounds.png");
    let texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::splat(BACKGROUND_SECTION_SIZE), 3, 3, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..Default::default()
        })
        .with_children(|builder| {
            for y in 0..tiles.height() {
                builder
                    .spawn(Node {
                        width: Val::Percent(100.),
                        height: Val::Px(BACKGROUND_SECTION_SIZE_F32),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    })
                    .with_children(|builder| {
                        for x in 0..tiles.width() {
                            let mut atlas = TextureAtlas::from(texture_atlas_handle.clone());
                            atlas.index = tiles.get(x, y).unwrap().index();

                            let image_node =
                                ImageNode::from_atlas_image(texture_handle.clone(), atlas);

                            builder.spawn((
                                image_node,
                                Node {
                                    width: Val::Px(BACKGROUND_SECTION_SIZE_F32),
                                    height: Val::Px(BACKGROUND_SECTION_SIZE_F32),
                                    ..Default::default()
                                },
                            ));
                        }
                    });
            }
        });
}
