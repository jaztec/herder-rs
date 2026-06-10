use bevy::prelude::*;
use rand::Rng;

use crate::states::play::{
    common::{AnimationFrames, AnimationTimer, Facing, FacingDirection, Moving},
    sheep::components::Sheep,
};

pub(super) const SHEEP_ANIMATION_FRAME_TIME: f32 = 0.1;
const SHEEP_STAND_DOWN: usize = 0;
const SHEEP_STAND_UP: usize = 1;
const SHEEP_STAND_LEFT: usize = 2;
const SHEEP_STAND_RIGHT: usize = 3;
const SHEEP_WALK_RIGHT: AnimationFrames = AnimationFrames::range(4, 7);
const SHEEP_WALK_LEFT: AnimationFrames = AnimationFrames::range(8, 11);
const SHEEP_WALK_DOWN: AnimationFrames = AnimationFrames::range(12, 15);
const SHEEP_WALK_UP: AnimationFrames = AnimationFrames::range(16, 19);

pub(in crate::states::play) fn update_sheep_animation_range(
    mut sheep: Query<(&Facing, &Moving, &mut AnimationFrames, &mut Sprite), With<Sheep>>,
) {
    for (facing, moving, mut frames, mut sprite) in &mut sheep {
        let next_frames = if moving.is_moving {
            match facing.direction {
                FacingDirection::Right => SHEEP_WALK_RIGHT,
                FacingDirection::Left => SHEEP_WALK_LEFT,
                FacingDirection::Down => SHEEP_WALK_DOWN,
                FacingDirection::Up => SHEEP_WALK_UP,
            }
        } else {
            AnimationFrames::single(standing_frame(facing.direction))
        };

        if *frames != next_frames {
            *frames = next_frames;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = frames.first;
            }
        }
    }
}

pub(in crate::states::play) fn animate_sheep(
    time: Res<Time>,
    mut sheep: Query<(&mut Sprite, &AnimationFrames, &mut AnimationTimer), With<Sheep>>,
) {
    for (mut sprite, frames, mut timer) in &mut sheep {
        if frames.first == frames.last {
            continue;
        }

        timer.timer.tick(time.delta());
        if !timer.timer.just_finished() {
            continue;
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = if atlas.index < frames.first || atlas.index >= frames.last {
                frames.first
            } else {
                atlas.index + 1
            };
        }
    }
}

pub(super) fn standing_frame(direction: FacingDirection) -> usize {
    match direction {
        FacingDirection::Right => SHEEP_STAND_RIGHT,
        FacingDirection::Left => SHEEP_STAND_LEFT,
        FacingDirection::Down => SHEEP_STAND_DOWN,
        FacingDirection::Up => SHEEP_STAND_UP,
    }
}

pub(super) fn random_facing(rng: &mut impl Rng) -> FacingDirection {
    match rng.random_range(0..4) {
        0 => FacingDirection::Right,
        1 => FacingDirection::Left,
        2 => FacingDirection::Down,
        _ => FacingDirection::Up,
    }
}
