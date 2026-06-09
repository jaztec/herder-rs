use bevy::prelude::*;

use crate::states::play::common::{Facing, FacingDirection, Velocity, load_texture};

pub const SHEPHERD_NAME: &str = "Shepherd";
pub const SHEPHERD_SPEED: f32 = 5.0;
pub const SHEPHERD_WIDTH: u32 = 50;
pub const SHEPHERD_HEIGHT: u32 = 75;

const CAMERA_DECAY_RATE: f32 = 0.1;

#[derive(Component)]
pub struct Shepherd;

pub fn setup_shepherd(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
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
        Transform::from_scale(Vec3::splat(0.3)),
        Velocity {
            speed: SHEPHERD_SPEED,
        },
        Facing {
            direction: FacingDirection::Right,
        },
        ImageNode::from_atlas_image(handle, TextureAtlas::from(atlas)),
        Node {
            width: Val::Px(50.),
            height: Val::Px(75.),
            ..default()
        },
    ));
}

pub fn move_shepherd(
    mut shepherd: Single<(&mut Transform, &Velocity, &mut Facing), With<Shepherd>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut velocity = Vec2::ZERO;

    if input.pressed(KeyCode::KeyD) {
        velocity.x -= 1.;
        shepherd.2.direction = FacingDirection::Right;
    }
    if input.pressed(KeyCode::KeyA) {
        velocity.x += 1.;
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
}

pub fn move_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Shepherd>)>,
    shepherd: Single<&Transform, (With<Shepherd>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = shepherd.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
