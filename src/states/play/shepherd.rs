use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{
    states::play::common::{Facing, FacingDirection, Velocity, load_texture},
    world::WorldBounds,
};

pub const SHEPHERD_NAME: &str = "Shepherd";
pub const SHEPHERD_SPEED: f32 = 260.0;
pub const SHEPHERD_WIDTH: u32 = 50;
pub const SHEPHERD_HEIGHT: u32 = 75;

#[derive(Component)]
pub struct Shepherd;

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

    commands.spawn((
        Name::new(SHEPHERD_NAME),
        Shepherd,
        Sprite {
            custom_size: Some(Vec2::new(SHEPHERD_WIDTH as f32, SHEPHERD_HEIGHT as f32)),
            ..Sprite::from_atlas_image(handle, TextureAtlas::from(atlas))
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
        Velocity {
            speed: SHEPHERD_SPEED,
        },
        Facing {
            direction: FacingDirection::Right,
        },
    ));
}

pub fn move_shepherd(
    mut shepherd: Single<(&mut Transform, &Velocity, &mut Facing), With<Shepherd>>,
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

pub fn move_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Shepherd>)>,
    shepherd: Single<&Transform, (With<Shepherd>, Without<Camera2d>)>,
    bounds: Res<WorldBounds>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let half_window = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    let half_world = bounds.size / 2.0;

    let x = if bounds.size.x <= window.width() {
        0.0
    } else {
        shepherd
            .translation
            .x
            .clamp(-half_world.x + half_window.x, half_world.x - half_window.x)
    };
    let y = if bounds.size.y <= window.height() {
        0.0
    } else {
        shepherd
            .translation
            .y
            .clamp(-half_world.y + half_window.y, half_world.y - half_window.y)
    };

    camera.translation.x = x.round();
    camera.translation.y = y.round();
}
