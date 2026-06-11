//! Herd and sheep spawning.
//!
//! The herd spawns as a compact cluster away from the finish tile and away from
//! the shepherd/dog start tiles when the map allows it.

use bevy::prelude::*;
use rand::{Rng, prelude::IndexedRandom};

use crate::{
    states::play::{
        common::{AnimationFrames, AnimationTimer, Facing, Moving, load_texture},
        dog::dog_spawn_position,
        herd::Herd,
        sheep::{
            animation::{SHEEP_ANIMATION_FRAME_TIME, random_facing, standing_frame},
            components::{
                Bravery, HERD_START_SIZE, MIN_SPAWN_EDGE_MARGIN, SHEEP_COUNT, SHEEP_HEIGHT,
                SHEEP_NAME, SHEEP_WIDTH, SHEEP_Z, Sheep, SheepId, SheepMotion, SheepPanic, Wander,
            },
            movement::clamp_position_to_world,
        },
        shepherd::shepherd_spawn_position,
    },
    world::{FinishTilePosition, GridPosition, MapConfig, WorldBounds},
};

const HERD_MIN_FINISH_DISTANCE_TILES: f32 = 7.0;
const SPAWN_POSITION_ATTEMPTS: usize = 20;

/// Spawn the herd marker and all sheep for a new run.
pub(in crate::states::play) fn setup_herd(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    bounds: Res<WorldBounds>,
    config: Res<MapConfig>,
    finish: Res<FinishTilePosition>,
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
    let forbidden_tiles = actor_spawn_tiles(&config);
    let cluster_center =
        random_cluster_center(&bounds, &config, &finish, &forbidden_tiles, &mut rng);

    for index in 0..SHEEP_COUNT {
        let spawn_position = random_sheep_spawn_position(
            cluster_center,
            &bounds,
            &config,
            &forbidden_tiles,
            &mut rng,
        );

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

fn random_cluster_center(
    bounds: &WorldBounds,
    config: &MapConfig,
    finish: &FinishTilePosition,
    forbidden_tiles: &[GridPosition],
    rng: &mut impl Rng,
) -> Vec2 {
    let candidates = cluster_center_candidates(bounds, config, forbidden_tiles);
    if candidates.is_empty() {
        return Vec2::ZERO;
    }

    let max_finish_distance = candidates
        .iter()
        .map(|candidate| candidate.distance_to_finish_tiles(finish))
        .fold(0.0, f32::max);
    let minimum_finish_distance = HERD_MIN_FINISH_DISTANCE_TILES.min(max_finish_distance);

    candidates
        .iter()
        .filter(|candidate| {
            candidate.distance_to_finish_tiles(finish) >= minimum_finish_distance - f32::EPSILON
        })
        .collect::<Vec<_>>()
        .choose(rng)
        .map(|candidate| candidate.position)
        .unwrap_or(Vec2::ZERO)
}

fn cluster_center_candidates(
    bounds: &WorldBounds,
    config: &MapConfig,
    forbidden_tiles: &[GridPosition],
) -> Vec<ClusterCenterCandidate> {
    let half_world = bounds.size / 2.0;
    let margin = cluster_spawn_margin(bounds, config);

    let mut candidates = Vec::with_capacity(config.width * config.height);

    for y in 0..config.height {
        for x in 0..config.width {
            if forbidden_tiles
                .iter()
                .any(|tile| tile.x == x && tile.y == y)
            {
                continue;
            }

            let position = config.tile_world_position(x, y).truncate();
            if position.x < -half_world.x + margin.x
                || position.x > half_world.x - margin.x
                || position.y < -half_world.y + margin.y
                || position.y > half_world.y - margin.y
            {
                continue;
            }

            candidates.push(ClusterCenterCandidate { x, y, position });
        }
    }

    candidates
}

fn random_sheep_spawn_position(
    cluster_center: Vec2,
    bounds: &WorldBounds,
    config: &MapConfig,
    forbidden_tiles: &[GridPosition],
    rng: &mut impl Rng,
) -> Vec2 {
    let mut fallback = cluster_center;
    clamp_position_to_world(&mut fallback, bounds);

    for _ in 0..SPAWN_POSITION_ATTEMPTS {
        let spawn_offset = Vec2::new(
            rng.random_range(-HERD_START_SIZE / 2.0..=HERD_START_SIZE / 2.0),
            rng.random_range(-HERD_START_SIZE / 2.0..=HERD_START_SIZE / 2.0),
        );
        let mut spawn_position = cluster_center + spawn_offset;
        clamp_position_to_world(&mut spawn_position, bounds);

        if !is_forbidden_tile(spawn_position, config, forbidden_tiles) {
            return spawn_position;
        }
    }

    fallback
}

fn actor_spawn_tiles(config: &MapConfig) -> Vec<GridPosition> {
    [shepherd_spawn_position(), dog_spawn_position()]
        .into_iter()
        .filter_map(|position| config.world_tile_position(position))
        .collect()
}

fn is_forbidden_tile(position: Vec2, config: &MapConfig, forbidden_tiles: &[GridPosition]) -> bool {
    config
        .world_tile_position(position)
        .is_some_and(|tile| forbidden_tiles.contains(&tile))
}

fn cluster_spawn_margin(bounds: &WorldBounds, config: &MapConfig) -> Vec2 {
    let half_world = bounds.size / 2.0;
    let desired_margin = MIN_SPAWN_EDGE_MARGIN + HERD_START_SIZE / 2.0;
    let half_tile = config.tile_size / 2.0;

    Vec2::new(
        desired_margin.min((half_world.x - half_tile).max(0.0)),
        desired_margin.min((half_world.y - half_tile).max(0.0)),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ClusterCenterCandidate {
    x: usize,
    y: usize,
    position: Vec2,
}

impl ClusterCenterCandidate {
    fn distance_to_finish_tiles(self, finish: &FinishTilePosition) -> f32 {
        Vec2::new(
            self.x as f32 - finish.x as f32,
            self.y as f32 - finish.y as f32,
        )
        .length()
    }
}
