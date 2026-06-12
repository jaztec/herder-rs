//! Dog actor, route input, waypoint following, and barking.

use std::collections::VecDeque;

use bevy::window::PrimaryWindow;
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    states::play::{
        common::{
            AnimationFrames, AnimationTimer, Facing, FacingDirection, Moving, Velocity,
            load_texture,
        },
        shepherd::Shepherd,
    },
    world::{MapConfig, TileMap, WorldBounds, find_path},
};

/// Marker component for the dog entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub struct Dog;

/// Display/debug name for the spawned dog entity.
pub const DOG_NAME: &str = "Dog";
/// Dog movement speed in world units per second.
pub const DOG_SPEED: f32 = 430.0;
/// Dog sprite frame width.
pub const DOG_WIDTH: u32 = 75;
/// Dog sprite frame height.
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
const DOG_SPAWN_POSITION: Vec2 = Vec2::new(90.0, -90.0);

/// Current high-level behavior mode for the dog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum DogMode {
    /// Stay near the shepherd when no route is active.
    FollowingShepherd,
    /// Follow the drawn waypoint route.
    FollowingRoute,
    /// Stay in place until commanded again.
    Stopped,
}

#[derive(Debug, Clone, Copy)]
struct RoutePoint {
    position: Vec2,
    marker: Entity,
}

/// Current route state built from mouse-drawn waypoints.
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

pub(in crate::states::play) fn reset_dog_route(mut route: ResMut<DogRoute>) {
    *route = DogRoute::default();
}

/// Audio handles used by dog systems.
#[derive(Debug, Resource)]
pub struct DogAudio {
    route_bark: Handle<AudioSource>,
    command_bark: Handle<AudioSource>,
}

/// Marker for waypoint entities drawn in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct RouteWaypoint;

/// Spawn the dog at its initial position.
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

    commands.spawn((
        Name::new(DOG_NAME),
        Dog,
        Sprite {
            custom_size: Some(Vec2::new(DOG_WIDTH as f32, DOG_HEIGHT as f32)),
            ..Sprite::from_atlas_image(handle, texture_atlas)
        },
        Transform::from_xyz(DOG_SPAWN_POSITION.x, DOG_SPAWN_POSITION.y, DOG_Z),
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

/// Initial dog spawn position in world coordinates.
pub(in crate::states::play) fn dog_spawn_position() -> Vec2 {
    DOG_SPAWN_POSITION
}

/// Load dog bark audio handles.
pub fn setup_dog_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(DogAudio {
        route_bark: asset_server.load("sounds/bark_2.ogg"),
        command_bark: asset_server.load("sounds/bark_1.ogg"),
    });
}

/// Grouped system parameters for dog route input.
#[derive(SystemParam)]
pub struct DogRouteInputParams<'w> {
    buttons: Res<'w, ButtonInput<MouseButton>>,
    window: Single<'w, &'static Window, With<PrimaryWindow>>,
    camera: Single<'w, (&'static Camera, &'static GlobalTransform), With<Camera2d>>,
    asset_server: Res<'w, AssetServer>,
    bounds: Res<'w, WorldBounds>,
    config: Res<'w, MapConfig>,
    tiles: Res<'w, TileMap>,
    route: ResMut<'w, DogRoute>,
    dog: Single<'w, (&'static mut DogMode, &'static Transform), With<Dog>>,
    dog_audio: Res<'w, DogAudio>,
}

#[derive(SystemParam)]
pub struct DogMoveParams<'w> {
    time: Res<'w, Time>,
    bounds: Res<'w, WorldBounds>,
    config: Res<'w, MapConfig>,
    tiles: Res<'w, TileMap>,
    dog_audio: Res<'w, DogAudio>,
    route: ResMut<'w, DogRoute>,
    dog: Single<
        'w,
        (
            &'static mut Transform,
            &'static Velocity,
            &'static mut Facing,
            &'static mut Moving,
            &'static mut DogMode,
        ),
        With<Dog>,
    >,
    shepherd: Single<'w, &'static Transform, (With<Shepherd>, Without<Dog>)>,
}

struct WaypointTerrain<'a> {
    bounds: &'a WorldBounds,
    config: &'a MapConfig,
    tiles: &'a TileMap,
}

/// Handle mouse input for route drawing and dog commands.
pub fn handle_dog_route_input(mut commands: Commands, input: DogRouteInputParams) {
    let DogRouteInputParams {
        buttons,
        window,
        camera,
        asset_server,
        bounds,
        config,
        tiles,
        mut route,
        mut dog,
        dog_audio,
    } = input;
    let dog_position = dog.1.translation.truncate();

    if buttons.just_pressed(MouseButton::Left) {
        clear_route(&mut commands, &mut route);
        route.is_drawing = true;

        if let Some(position) = cursor_world_position(&window, &camera) {
            add_waypoint(
                &mut commands,
                &asset_server,
                WaypointTerrain {
                    bounds: &bounds,
                    config: &config,
                    tiles: &tiles,
                },
                &mut route,
                dog_position,
                position,
                true,
            );
        }
    }

    if buttons.pressed(MouseButton::Left)
        && route.is_drawing
        && let Some(position) = cursor_world_position(&window, &camera)
    {
        add_waypoint(
            &mut commands,
            &asset_server,
            WaypointTerrain {
                bounds: &bounds,
                config: &config,
                tiles: &tiles,
            },
            &mut route,
            dog_position,
            position,
            false,
        );
    }

    if buttons.just_released(MouseButton::Left) && route.is_drawing {
        if let Some(position) = cursor_world_position(&window, &camera) {
            add_waypoint(
                &mut commands,
                &asset_server,
                WaypointTerrain {
                    bounds: &bounds,
                    config: &config,
                    tiles: &tiles,
                },
                &mut route,
                dog_position,
                position,
                true,
            );
        }

        route.is_drawing = false;
        if route.has_points() {
            *dog.0 = DogMode::FollowingRoute;
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        if route.is_drawing || route.has_points() {
            clear_route(&mut commands, &mut route);
            if *dog.0 == DogMode::FollowingRoute {
                *dog.0 = DogMode::FollowingShepherd;
            }
            return;
        }

        *dog.0 = match *dog.0 {
            DogMode::Stopped => DogMode::FollowingShepherd,
            DogMode::FollowingShepherd | DogMode::FollowingRoute => DogMode::Stopped,
        };

        play_bark(&mut commands, dog_audio.command_bark.clone());
    }
}

/// Move the dog according to its current behavior mode.
pub fn move_dog(mut commands: Commands, params: DogMoveParams) {
    let DogMoveParams {
        time,
        bounds,
        config,
        tiles,
        dog_audio,
        mut route,
        mut dog,
        shepherd,
    } = params;
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

    let speed_multiplier = tiles
        .movement_speed_at_world_position(&config, dog_position)
        .max(0.1);
    let step = dog.1.speed * speed_multiplier * time.delta_secs();
    let movement = if *dog.4 == DogMode::FollowingRoute {
        direction * step.min(distance)
    } else {
        direction * step.min(distance - arrival_radius)
    };

    let half_size = Vec2::new(DOG_WIDTH as f32 / 2.0, DOG_HEIGHT as f32 / 2.0);
    let next_position =
        move_with_terrain(dog_position, movement, half_size, &bounds, &config, &tiles);
    dog.3.is_moving = next_position.distance_squared(dog_position) > 1.0;
    dog.0.translation.x = next_position.x;
    dog.0.translation.y = next_position.y;

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

/// Select the dog's standing or walking atlas range from movement state.
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

/// Advance the dog's sprite atlas frame when its animation timer ticks.
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
    terrain: WaypointTerrain,
    route: &mut DogRoute,
    dog_position: Vec2,
    position: Vec2,
    force: bool,
) {
    if route.points.len() >= MAX_WAYPOINTS
        || !is_inside_world(position, terrain.bounds)
        || !terrain
            .tiles
            .is_world_position_walkable(terrain.config, position)
    {
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

    let start = route
        .points
        .back()
        .map_or(dog_position, |point| point.position);
    let Some(path) = find_path(terrain.tiles, terrain.config, start, position) else {
        return;
    };

    for position in path {
        push_route_point(commands, asset_server, route, position);
    }
}

fn push_route_point(
    commands: &mut Commands,
    asset_server: &AssetServer,
    route: &mut DogRoute,
    position: Vec2,
) {
    if route
        .points
        .back()
        .is_some_and(|point| point.position.distance(position) < DOG_WAYPOINT_RADIUS)
    {
        return;
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

fn move_with_terrain(
    current: Vec2,
    movement: Vec2,
    half_size: Vec2,
    bounds: &WorldBounds,
    config: &MapConfig,
    tiles: &TileMap,
) -> Vec2 {
    let mut position = current;
    let mut x_translation = Vec3::new(position.x + movement.x, position.y, 0.0);
    clamp_to_world(&mut x_translation, bounds);
    let x_position = x_translation.truncate();
    if tiles.is_world_rect_walkable(config, x_position, half_size) {
        position.x = x_position.x;
    }

    let mut y_translation = Vec3::new(position.x, position.y + movement.y, 0.0);
    clamp_to_world(&mut y_translation, bounds);
    let y_position = y_translation.truncate();
    if tiles.is_world_rect_walkable(config, y_position, half_size) {
        position.y = y_position.y;
    }

    position
}

fn play_bark(commands: &mut Commands, bark: Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(bark), PlaybackSettings::DESPAWN));
}
