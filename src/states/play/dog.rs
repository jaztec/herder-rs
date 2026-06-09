use std::collections::VecDeque;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{
    states::play::{
        common::{
            AnimationFrames, AnimationTimer, Facing, FacingDirection, Moving, Velocity,
            load_texture,
        },
        shepherd::Shepherd,
    },
    world::WorldBounds,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub struct Dog;

pub const DOG_NAME: &str = "Dog";
pub const DOG_SPEED: f32 = 360.0;
pub const DOG_WIDTH: u32 = 75;
pub const DOG_HEIGHT: u32 = 60;

const DOG_ANIMATION_FRAME_TIME: f32 = 0.08;
const DOG_STAND_DOWN: usize = 0;
const DOG_STAND_UP: usize = 1;
const DOG_STAND_LEFT: usize = 2;
const DOG_STAND_RIGHT: usize = 3;
const DOG_WALK_RIGHT: AnimationFrames = AnimationFrames::range(4, 7);
const DOG_WALK_LEFT: AnimationFrames = AnimationFrames::range(8, 11);
const DOG_WALK_DOWN: AnimationFrames = AnimationFrames::range(12, 15);
const DOG_WALK_UP: AnimationFrames = AnimationFrames::range(16, 19);

const DOG_FOLLOW_SHEPHERD_RADIUS: f32 = 120.0;
const DOG_WAYPOINT_RADIUS: f32 = 40.0;
const WAYPOINT_DISTANCE: f32 = 50.0;
const WAYPOINT_EDGE_MARGIN: f32 = 30.0;
const MAX_WAYPOINTS: usize = 500;
const WAYPOINT_Z: f32 = 20.0;
const DOG_Z: f32 = 11.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum DogMode {
    FollowingShepherd,
    FollowingRoute,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
struct RoutePoint {
    position: Vec2,
    marker: Entity,
}

#[derive(Debug, Default, Resource)]
pub struct DogRoute {
    points: VecDeque<RoutePoint>,
    is_drawing: bool,
    reached_points: u32,
}

impl DogRoute {
    fn has_points(&self) -> bool {
        !self.points.is_empty()
    }
}

#[derive(Debug, Resource)]
pub struct DogAudio {
    route_bark: Handle<AudioSource>,
    command_bark: Handle<AudioSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct RouteWaypoint;

pub fn setup_dog(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (handle, atlas) = load_texture(
        asset_server,
        texture_atlases,
        "textures/dog.png",
        UVec2 {
            x: DOG_WIDTH,
            y: DOG_HEIGHT,
        },
    );

    let mut texture_atlas = TextureAtlas::from(atlas);
    texture_atlas.index = DOG_STAND_DOWN;

    let spawn_position = Vec3::new(90.0, -90.0, DOG_Z);

    commands.spawn((
        Name::new(DOG_NAME),
        Dog,
        Sprite {
            custom_size: Some(Vec2::new(DOG_WIDTH as f32, DOG_HEIGHT as f32)),
            ..Sprite::from_atlas_image(handle, texture_atlas)
        },
        Transform::from_translation(spawn_position),
        Velocity { speed: DOG_SPEED },
        Facing {
            direction: FacingDirection::Down,
        },
        Moving { is_moving: false },
        DogMode::FollowingShepherd,
        AnimationFrames::single(DOG_STAND_DOWN),
        AnimationTimer {
            timer: Timer::from_seconds(DOG_ANIMATION_FRAME_TIME, TimerMode::Repeating),
        },
    ));
}

pub fn setup_dog_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DogAudio {
        route_bark: asset_server.load("sounds/bark_2.ogg"),
        command_bark: asset_server.load("sounds/bark_1.ogg"),
    });
}

pub fn handle_dog_route_input(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    asset_server: Res<AssetServer>,
    bounds: Res<WorldBounds>,
    mut route: ResMut<DogRoute>,
    mut dog_mode: Single<&mut DogMode, With<Dog>>,
    dog_audio: Res<DogAudio>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        clear_route(&mut commands, &mut route);
        route.is_drawing = true;

        if let Some(position) = cursor_world_position(&window, &camera) {
            add_waypoint(
                &mut commands,
                &asset_server,
                &bounds,
                &mut route,
                position,
                true,
            );
        }
    }

    if buttons.pressed(MouseButton::Left) && route.is_drawing {
        if let Some(position) = cursor_world_position(&window, &camera) {
            add_waypoint(
                &mut commands,
                &asset_server,
                &bounds,
                &mut route,
                position,
                false,
            );
        }
    }

    if buttons.just_released(MouseButton::Left) && route.is_drawing {
        if let Some(position) = cursor_world_position(&window, &camera) {
            add_waypoint(
                &mut commands,
                &asset_server,
                &bounds,
                &mut route,
                position,
                true,
            );
        }

        route.is_drawing = false;
        if route.has_points() {
            **dog_mode = DogMode::FollowingRoute;
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        if route.is_drawing || route.has_points() {
            clear_route(&mut commands, &mut route);
            if **dog_mode == DogMode::FollowingRoute {
                **dog_mode = DogMode::FollowingShepherd;
            }
            return;
        }

        **dog_mode = match **dog_mode {
            DogMode::Stopped => DogMode::FollowingShepherd,
            DogMode::FollowingShepherd | DogMode::FollowingRoute => DogMode::Stopped,
        };

        play_bark(&mut commands, dog_audio.command_bark.clone());
    }
}

pub fn move_dog(
    mut commands: Commands,
    time: Res<Time>,
    bounds: Res<WorldBounds>,
    dog_audio: Res<DogAudio>,
    mut route: ResMut<DogRoute>,
    mut dog: Single<
        (
            &mut Transform,
            &Velocity,
            &mut Facing,
            &mut Moving,
            &mut DogMode,
        ),
        With<Dog>,
    >,
    shepherd: Single<&Transform, (With<Shepherd>, Without<Dog>)>,
) {
    let dog_position = dog.0.translation.truncate();

    let (target, arrival_radius) = match *dog.4 {
        DogMode::Stopped => {
            dog.3.is_moving = false;
            return;
        }
        DogMode::FollowingShepherd => (shepherd.translation.truncate(), DOG_FOLLOW_SHEPHERD_RADIUS),
        DogMode::FollowingRoute => {
            resolve_reached_route_points(
                &mut commands,
                &mut route,
                dog_position,
                &dog_audio.route_bark,
            );

            let Some(point) = route.points.front() else {
                *dog.4 = DogMode::Stopped;
                dog.3.is_moving = false;
                return;
            };

            (point.position, DOG_WAYPOINT_RADIUS)
        }
    };

    let to_target = target - dog_position;
    let distance = to_target.length();

    if distance <= arrival_radius {
        if *dog.4 == DogMode::FollowingRoute {
            resolve_reached_route_points(
                &mut commands,
                &mut route,
                dog_position,
                &dog_audio.route_bark,
            );
        }
        dog.3.is_moving = *dog.4 == DogMode::FollowingRoute && route.has_points();
        return;
    }

    let direction = to_target.normalize_or_zero();
    dog.2.direction = direction_to_facing(direction);
    dog.3.is_moving = true;

    let step = dog.1.speed * time.delta_secs();
    let movement = if *dog.4 == DogMode::FollowingRoute {
        direction * step.min(distance)
    } else {
        direction * step.min(distance - arrival_radius)
    };
    dog.0.translation += movement.extend(0.0);
    clamp_to_world(&mut dog.0.translation, &bounds);

    if *dog.4 == DogMode::FollowingRoute {
        let dog_position = dog.0.translation.truncate();
        resolve_reached_route_points(
            &mut commands,
            &mut route,
            dog_position,
            &dog_audio.route_bark,
        );

        if route.points.is_empty() {
            *dog.4 = DogMode::Stopped;
            dog.3.is_moving = false;
        }
    }
}

fn resolve_reached_route_points(
    commands: &mut Commands,
    route: &mut DogRoute,
    dog_position: Vec2,
    route_bark: &Handle<AudioSource>,
) {
    while route
        .points
        .front()
        .is_some_and(|point| dog_position.distance(point.position) <= DOG_WAYPOINT_RADIUS)
    {
        let Some(point) = route.points.pop_front() else {
            break;
        };

        commands.entity(point.marker).despawn();
        route.reached_points += 1;

        if route.reached_points % 3 == 1 {
            play_bark(commands, route_bark.clone());
        }
    }
}

pub fn update_dog_animation_range(
    mut dog: Single<(&Facing, &Moving, &mut AnimationFrames, &mut Sprite), With<Dog>>,
) {
    let frames = if dog.1.is_moving {
        match dog.0.direction {
            FacingDirection::Right => DOG_WALK_RIGHT,
            FacingDirection::Left => DOG_WALK_LEFT,
            FacingDirection::Down => DOG_WALK_DOWN,
            FacingDirection::Up => DOG_WALK_UP,
        }
    } else {
        AnimationFrames::single(match dog.0.direction {
            FacingDirection::Right => DOG_STAND_RIGHT,
            FacingDirection::Left => DOG_STAND_LEFT,
            FacingDirection::Down => DOG_STAND_DOWN,
            FacingDirection::Up => DOG_STAND_UP,
        })
    };

    if *dog.2 != frames {
        *dog.2 = frames;
        if let Some(atlas) = &mut dog.3.texture_atlas {
            atlas.index = frames.first;
        }
    }
}

pub fn animate_dog(
    time: Res<Time>,
    mut dog: Single<(&mut Sprite, &AnimationFrames, &mut AnimationTimer), With<Dog>>,
) {
    if dog.1.first == dog.1.last {
        return;
    }

    dog.2.timer.tick(time.delta());

    if !dog.2.timer.just_finished() {
        return;
    }

    let first = dog.1.first;
    let last = dog.1.last;
    if let Some(atlas) = &mut dog.0.texture_atlas {
        atlas.index = if atlas.index < first || atlas.index >= last {
            first
        } else {
            atlas.index + 1
        };
    }
}

fn cursor_world_position(window: &Window, camera: &(&Camera, &GlobalTransform)) -> Option<Vec2> {
    let cursor_position = window.cursor_position()?;
    camera
        .0
        .viewport_to_world_2d(camera.1, cursor_position)
        .ok()
}

fn add_waypoint(
    commands: &mut Commands,
    asset_server: &AssetServer,
    bounds: &WorldBounds,
    route: &mut DogRoute,
    position: Vec2,
    force: bool,
) {
    if route.points.len() >= MAX_WAYPOINTS || !is_inside_world(position, bounds) {
        return;
    }

    if let Some(last) = route.points.back() {
        if !force && last.position.distance(position) < WAYPOINT_DISTANCE {
            return;
        }

        if force && last.position.distance(position) < DOG_WAYPOINT_RADIUS {
            return;
        }
    }

    let marker = commands
        .spawn((
            Name::new("Dog route waypoint"),
            RouteWaypoint,
            Sprite::from_image(asset_server.load("textures/waypoint.png")),
            Transform::from_xyz(position.x, position.y, WAYPOINT_Z),
        ))
        .id();

    route.points.push_back(RoutePoint { position, marker });
}

fn clear_route(commands: &mut Commands, route: &mut DogRoute) {
    for point in route.points.drain(..) {
        commands.entity(point.marker).despawn();
    }
    route.is_drawing = false;
    route.reached_points = 0;
}

fn is_inside_world(position: Vec2, bounds: &WorldBounds) -> bool {
    let half_world = bounds.size / 2.0;
    position.x > -half_world.x + WAYPOINT_EDGE_MARGIN
        && position.x < half_world.x - WAYPOINT_EDGE_MARGIN
        && position.y > -half_world.y + WAYPOINT_EDGE_MARGIN
        && position.y < half_world.y - WAYPOINT_EDGE_MARGIN
}

fn direction_to_facing(direction: Vec2) -> FacingDirection {
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

fn clamp_to_world(translation: &mut Vec3, bounds: &WorldBounds) {
    let half_size = Vec2::new(DOG_WIDTH as f32 / 2.0, DOG_HEIGHT as f32 / 2.0);
    let half_world = bounds.size / 2.0;
    translation.x = translation
        .x
        .clamp(-half_world.x + half_size.x, half_world.x - half_size.x);
    translation.y = translation
        .y
        .clamp(-half_world.y + half_size.y, half_world.y - half_size.y);
}

fn play_bark(commands: &mut Commands, bark: Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(bark), PlaybackSettings::DESPAWN));
}
