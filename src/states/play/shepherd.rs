use bevy::window::PrimaryWindow;
use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::{
    states::play::common::{
        AnimationFrames, AnimationTimer, Facing, FacingDirection, Moving, Velocity, load_texture,
    },
    world::WorldBounds,
};

pub const SHEPHERD_NAME: &str = "Shepherd";
pub const SHEPHERD_SPEED: f32 = 260.0;
pub const SHEPHERD_WIDTH: u32 = 50;
pub const SHEPHERD_HEIGHT: u32 = 75;

const SHEPHERD_ANIMATION_FRAME_TIME: f32 = 0.08;
const SHEPHERD_STAND_DOWN: usize = 0;
const SHEPHERD_STAND_UP: usize = 1;
const SHEPHERD_STAND_RIGHT: usize = 2;
const SHEPHERD_STAND_LEFT: usize = 3;
const SHEPHERD_WALK_RIGHT: AnimationFrames = AnimationFrames::range(4, 7);
const SHEPHERD_WALK_LEFT: AnimationFrames = AnimationFrames::range(8, 11);
const SHEPHERD_WALK_DOWN: AnimationFrames = AnimationFrames::range(12, 15);
const SHEPHERD_WALK_UP: AnimationFrames = AnimationFrames::range(16, 19);
const MIN_CAMERA_SCALE: f32 = 0.6;
const MAX_CAMERA_SCALE: f32 = 2.0;
const CAMERA_ZOOM_STEP: f32 = 0.12;
const SHEPHERD_SPAWN_POSITION: Vec2 = Vec2::ZERO;

#[derive(Component)]
pub struct Shepherd;

type CameraFollowQuery<'w> =
    Single<'w, (&'static mut Transform, &'static Projection), (With<Camera2d>, Without<Shepherd>)>;

pub fn setup_shepherd(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (handle, atlas) = load_texture(
        asset_server,
        texture_atlases,
        "textures/character.png",
        UVec2 {
            x: SHEPHERD_WIDTH,
            y: SHEPHERD_HEIGHT,
        },
    );

    let mut texture_atlas = TextureAtlas::from(atlas);
    texture_atlas.index = SHEPHERD_STAND_RIGHT;

    commands.spawn((
        Name::new(SHEPHERD_NAME),
        Shepherd,
        Sprite {
            custom_size: Some(Vec2::new(SHEPHERD_WIDTH as f32, SHEPHERD_HEIGHT as f32)),
            ..Sprite::from_atlas_image(handle, texture_atlas)
        },
        Transform::from_xyz(SHEPHERD_SPAWN_POSITION.x, SHEPHERD_SPAWN_POSITION.y, 10.0),
        Velocity {
            speed: SHEPHERD_SPEED,
        },
        Facing {
            direction: FacingDirection::Right,
        },
        Moving { is_moving: false },
        AnimationFrames::single(SHEPHERD_STAND_RIGHT),
        AnimationTimer {
            timer: Timer::from_seconds(SHEPHERD_ANIMATION_FRAME_TIME, TimerMode::Repeating),
        },
    ));
}

pub(in crate::states::play) fn shepherd_spawn_position() -> Vec2 {
    SHEPHERD_SPAWN_POSITION
}

pub fn move_shepherd(
    mut shepherd: Single<(&mut Transform, &Velocity, &mut Facing, &mut Moving), With<Shepherd>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    bounds: Res<WorldBounds>,
) {
    let mut velocity = Vec2::ZERO;

    if input.pressed(KeyCode::KeyD) {
        velocity.x += 1.;
        shepherd.2.direction = FacingDirection::Right;
    }
    if input.pressed(KeyCode::KeyA) {
        velocity.x -= 1.;
        shepherd.2.direction = FacingDirection::Left;
    }
    if input.pressed(KeyCode::KeyW) {
        velocity.y += 1.;
        shepherd.2.direction = FacingDirection::Up;
    }
    if input.pressed(KeyCode::KeyS) {
        velocity.y -= 1.;
        shepherd.2.direction = FacingDirection::Down;
    }

    shepherd.3.is_moving = velocity.length_squared() > 0.0;

    let move_delta = velocity.normalize_or_zero() * shepherd.1.speed * time.delta_secs();
    shepherd.0.translation += move_delta.extend(0.);

    let half_size = Vec2::new(SHEPHERD_WIDTH as f32 / 2.0, SHEPHERD_HEIGHT as f32 / 2.0);
    let half_world = bounds.size / 2.0;
    shepherd.0.translation.x = shepherd
        .0
        .translation
        .x
        .clamp(-half_world.x + half_size.x, half_world.x - half_size.x);
    shepherd.0.translation.y = shepherd
        .0
        .translation
        .y
        .clamp(-half_world.y + half_size.y, half_world.y - half_size.y);
}

pub fn update_shepherd_animation_range(
    mut shepherd: Single<(&Facing, &Moving, &mut AnimationFrames, &mut Sprite), With<Shepherd>>,
) {
    let frames = if shepherd.1.is_moving {
        match shepherd.0.direction {
            FacingDirection::Right => SHEPHERD_WALK_RIGHT,
            FacingDirection::Left => SHEPHERD_WALK_LEFT,
            FacingDirection::Down => SHEPHERD_WALK_DOWN,
            FacingDirection::Up => SHEPHERD_WALK_UP,
        }
    } else {
        AnimationFrames::single(match shepherd.0.direction {
            FacingDirection::Right => SHEPHERD_STAND_RIGHT,
            FacingDirection::Left => SHEPHERD_STAND_LEFT,
            FacingDirection::Down => SHEPHERD_STAND_DOWN,
            FacingDirection::Up => SHEPHERD_STAND_UP,
        })
    };

    if *shepherd.2 != frames {
        *shepherd.2 = frames;
        if let Some(atlas) = &mut shepherd.3.texture_atlas {
            atlas.index = frames.first;
        }
    }
}

pub fn animate_shepherd(
    time: Res<Time>,
    mut shepherd: Single<(&mut Sprite, &AnimationFrames, &mut AnimationTimer), With<Shepherd>>,
) {
    if shepherd.1.first == shepherd.1.last {
        return;
    }

    shepherd.2.timer.tick(time.delta());

    if !shepherd.2.timer.just_finished() {
        return;
    }

    let first = shepherd.1.first;
    let last = shepherd.1.last;
    if let Some(atlas) = &mut shepherd.0.texture_atlas {
        atlas.index = if atlas.index < first || atlas.index >= last {
            first
        } else {
            atlas.index + 1
        };
    }
}

pub fn zoom_camera(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera: Single<&mut Projection, With<Camera2d>>,
) {
    let scroll: f32 = mouse_wheel.read().map(|event| event.y).sum();
    if scroll == 0.0 {
        return;
    }

    let Projection::Orthographic(projection) = &mut **camera else {
        return;
    };

    let zoom_factor = (1.0 - scroll * CAMERA_ZOOM_STEP).clamp(0.2, 5.0);
    projection.scale = (projection.scale * zoom_factor).clamp(MIN_CAMERA_SCALE, MAX_CAMERA_SCALE);
}

pub fn move_camera(
    mut camera: CameraFollowQuery,
    shepherd: Single<&Transform, (With<Shepherd>, Without<Camera2d>)>,
    bounds: Res<WorldBounds>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let projection_scale = match camera.1 {
        Projection::Orthographic(projection) => projection.scale,
        _ => 1.0,
    };
    let half_window = Vec2::new(window.width(), window.height()) * projection_scale / 2.0;
    let visible_size = half_window * 2.0;
    let half_world = bounds.size / 2.0;

    let x = if bounds.size.x <= visible_size.x {
        0.0
    } else {
        shepherd
            .translation
            .x
            .clamp(-half_world.x + half_window.x, half_world.x - half_window.x)
    };
    let y = if bounds.size.y <= visible_size.y {
        0.0
    } else {
        shepherd
            .translation
            .y
            .clamp(-half_world.y + half_window.y, half_world.y - half_window.y)
    };

    camera.0.translation.x = x.round();
    camera.0.translation.y = y.round();
}
