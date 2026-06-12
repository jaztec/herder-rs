//! Sheep movement and world-bound clamping.

use bevy::prelude::*;

use crate::{
    states::play::{
        common::{Facing, FacingDirection, Moving},
        sheep::components::{SHEEP_HEIGHT, SHEEP_WIDTH, Sheep, SheepMotion},
    },
    world::{MapConfig, TileMap, WorldBounds},
};

/// Apply AI velocity to every sheep and update movement-facing state.
pub(in crate::states::play) fn move_sheep(
    time: Res<Time>,
    bounds: Res<WorldBounds>,
    config: Res<MapConfig>,
    tiles: Res<TileMap>,
    mut sheep: Query<(&mut Transform, &SheepMotion, &mut Facing, &mut Moving), With<Sheep>>,
) {
    for (mut transform, motion, mut facing, mut moving) in &mut sheep {
        let current_position = transform.translation.truncate();
        let speed_multiplier = tiles
            .movement_speed_at_world_position(&config, current_position)
            .max(0.1);
        let velocity = motion.velocity * speed_multiplier;
        moving.is_moving = velocity.length_squared() > 1.0;

        if moving.is_moving {
            facing.direction = direction_to_facing(velocity);
        }

        let half_size = Vec2::new(SHEEP_WIDTH as f32 / 2.0, SHEEP_HEIGHT as f32 / 2.0);
        let position = move_with_terrain(
            current_position,
            velocity * time.delta_secs(),
            half_size,
            &bounds,
            &config,
            &tiles,
        );
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

/// Convert a velocity direction into the closest sprite-facing direction.
pub(super) fn direction_to_facing(direction: Vec2) -> FacingDirection {
    if direction.x.abs() > direction.y.abs() {
        if direction.x >= 0.0 {
            FacingDirection::Right
        } else {
            FacingDirection::Left
        }
    } else if direction.y >= 0.0 {
        FacingDirection::Up
    } else {
        FacingDirection::Down
    }
}

/// Clamp a sheep-sized position to the playable world.
pub(super) fn clamp_position_to_world(position: &mut Vec2, bounds: &WorldBounds) {
    let half_size = Vec2::new(SHEEP_WIDTH as f32 / 2.0, SHEEP_HEIGHT as f32 / 2.0);
    let half_world = bounds.size / 2.0;
    position.x = position
        .x
        .clamp(-half_world.x + half_size.x, half_world.x - half_size.x);
    position.y = position
        .y
        .clamp(-half_world.y + half_size.y, half_world.y - half_size.y);
}

fn move_with_terrain(
    current: Vec2,
    movement: Vec2,
    half_size: Vec2,
    bounds: &WorldBounds,
    config: &MapConfig,
    tiles: &TileMap,
) -> Vec2 {
    let mut position = current;

    let mut x_position = position + Vec2::new(movement.x, 0.0);
    clamp_position_to_world(&mut x_position, bounds);
    if tiles.is_world_rect_walkable(config, x_position, half_size) {
        position.x = x_position.x;
    }

    let mut y_position = position + Vec2::new(0.0, movement.y);
    clamp_position_to_world(&mut y_position, bounds);
    if tiles.is_world_rect_walkable(config, y_position, half_size) {
        position.y = y_position.y;
    }

    position
}
