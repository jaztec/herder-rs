use bevy::prelude::*;

use crate::{
    states::play::{
        common::{Facing, FacingDirection, Moving},
        sheep::components::{SHEEP_HEIGHT, SHEEP_WIDTH, Sheep, SheepMotion},
    },
    world::WorldBounds,
};

pub(in crate::states::play) fn move_sheep(
    time: Res<Time>,
    bounds: Res<WorldBounds>,
    mut sheep: Query<(&mut Transform, &SheepMotion, &mut Facing, &mut Moving), With<Sheep>>,
) {
    for (mut transform, motion, mut facing, mut moving) in &mut sheep {
        let velocity = motion.velocity;
        moving.is_moving = velocity.length_squared() > 1.0;

        if moving.is_moving {
            facing.direction = direction_to_facing(velocity);
        }

        transform.translation += (velocity * time.delta_secs()).extend(0.0);
        let mut position = transform.translation.truncate();
        clamp_position_to_world(&mut position, &bounds);
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

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
