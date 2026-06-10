use bevy::prelude::*;
use rand::Rng;

use crate::{
    states::play::{
        common::{AnimationFrames, AnimationTimer, Facing, Moving, load_texture},
        herd::Herd,
        sheep::{
            animation::{SHEEP_ANIMATION_FRAME_TIME, random_facing, standing_frame},
            components::{
                Bravery, HERD_START_SIZE, MIN_SPAWN_EDGE_MARGIN, SHEEP_COUNT, SHEEP_HEIGHT,
                SHEEP_NAME, SHEEP_WIDTH, SHEEP_Z, Sheep, SheepId, SheepMotion, SheepPanic, Wander,
            },
            movement::clamp_position_to_world,
        },
    },
    world::WorldBounds,
};

pub(in crate::states::play) fn setup_herd(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    bounds: Res<WorldBounds>,
) {
    commands.spawn((Name::new("Herd"), Herd));

    let (handle, atlas) = load_texture(
        asset_server,
        texture_atlases,
        "textures/sheep.png",
        UVec2 {
            x: SHEEP_WIDTH,
            y: SHEEP_HEIGHT,
        },
    );

    let mut rng = rand::rng();
    let cluster_center = random_cluster_center(&bounds, &mut rng);

    for index in 0..SHEEP_COUNT {
        let spawn_offset = Vec2::new(
            rng.random_range(-HERD_START_SIZE / 2.0..=HERD_START_SIZE / 2.0),
            rng.random_range(-HERD_START_SIZE / 2.0..=HERD_START_SIZE / 2.0),
        );
        let mut spawn_position = cluster_center + spawn_offset;
        clamp_position_to_world(&mut spawn_position, &bounds);

        let facing = random_facing(&mut rng);
        let mut texture_atlas = TextureAtlas::from(atlas.clone());
        texture_atlas.index = standing_frame(facing);

        commands.spawn((
            Name::new(format!("{SHEEP_NAME} {}", index + 1)),
            Sheep,
            SheepId(index + 1),
            Bravery {
                value: rng.random_range(1..=10) as f32,
            },
            SheepMotion {
                velocity: Vec2::ZERO,
            },
            SheepPanic::default(),
            Wander::new(
                rng.random_range(0.3..1.8),
                super::ai::random_wander_direction(&mut rng),
            ),
            Sprite {
                custom_size: Some(Vec2::new(SHEEP_WIDTH as f32, SHEEP_HEIGHT as f32)),
                ..Sprite::from_atlas_image(handle.clone(), texture_atlas)
            },
            Transform::from_xyz(spawn_position.x, spawn_position.y, SHEEP_Z),
            Facing { direction: facing },
            Moving { is_moving: false },
            AnimationFrames::single(standing_frame(facing)),
            AnimationTimer {
                timer: Timer::from_seconds(SHEEP_ANIMATION_FRAME_TIME, TimerMode::Repeating),
            },
        ));
    }
}

fn random_cluster_center(bounds: &WorldBounds, rng: &mut impl Rng) -> Vec2 {
    let half_world = bounds.size / 2.0;
    let margin = Vec2::new(
        (MIN_SPAWN_EDGE_MARGIN + HERD_START_SIZE / 2.0).min(half_world.x),
        (MIN_SPAWN_EDGE_MARGIN + HERD_START_SIZE / 2.0).min(half_world.y),
    );

    Vec2::new(
        rng.random_range(-half_world.x + margin.x..=half_world.x - margin.x),
        rng.random_range(-half_world.y + margin.y..=half_world.y - margin.y),
    )
}
