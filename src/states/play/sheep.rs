use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    states::play::{
        common::{AnimationFrames, AnimationTimer, Facing, FacingDirection, Moving, load_texture},
        dog::Dog,
        herd::Herd,
        shepherd::Shepherd,
    },
    world::WorldBounds,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub struct Sheep;

const SHEEP_COUNT: usize = 30;
const SHEEP_NAME: &str = "Sheep";
const SHEEP_WIDTH: u32 = 75;
const SHEEP_HEIGHT: u32 = 60;
const SHEEP_Z: f32 = 10.5;

const SHEEP_ANIMATION_FRAME_TIME: f32 = 0.1;
const SHEEP_STAND_DOWN: usize = 0;
const SHEEP_STAND_UP: usize = 1;
const SHEEP_STAND_LEFT: usize = 2;
const SHEEP_STAND_RIGHT: usize = 3;
const SHEEP_WALK_RIGHT: AnimationFrames = AnimationFrames::range(4, 7);
const SHEEP_WALK_LEFT: AnimationFrames = AnimationFrames::range(8, 11);
const SHEEP_WALK_DOWN: AnimationFrames = AnimationFrames::range(12, 15);
const SHEEP_WALK_UP: AnimationFrames = AnimationFrames::range(16, 19);

const HERD_START_SIZE: f32 = 220.0;
const MIN_SPAWN_EDGE_MARGIN: f32 = 160.0;

const SHEPHERD_THREAT_RADIUS: f32 = 180.0;
const DOG_THREAT_RADIUS: f32 = 280.0;
const DIRECT_PANIC_DURATION: f32 = 0.85;
const PROPAGATED_PANIC_RADIUS: f32 = 300.0;
const PROPAGATED_PANIC_DURATION: f32 = 0.65;

const LEADER_BRAVERY_THRESHOLD: f32 = 7.0;
const LEADER_FOLLOW_RADIUS: f32 = 320.0;
const LEADER_STOP_RADIUS: f32 = 80.0;
const LEADER_FOLLOW_SPEED: f32 = 55.0;

const SHEEP_SCARED_BASE_SPEED: f32 = 120.0;
const SHEEP_BRAVERY_SPEED: f32 = 16.0;
const SHEEP_WANDER_SPEED: f32 = 38.0;
const SHEEP_SEPARATION_RADIUS: f32 = 46.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct SheepId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(super) struct Bravery {
    value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(super) struct SheepMotion {
    velocity: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(super) struct SheepPanic {
    direction: Vec2,
    intensity: f32,
    remaining: f32,
    duration: f32,
    propagated: bool,
}

impl Default for SheepPanic {
    fn default() -> Self {
        Self {
            direction: Vec2::ZERO,
            intensity: 0.0,
            remaining: 0.0,
            duration: 0.0,
            propagated: false,
        }
    }
}

impl SheepPanic {
    fn active(self) -> bool {
        self.remaining > 0.0 && self.intensity > 0.0 && self.direction.length_squared() > 0.0
    }

    fn strength(self) -> f32 {
        if !self.active() || self.duration <= 0.0 {
            return 0.0;
        }

        self.intensity * (self.remaining / self.duration).clamp(0.0, 1.0)
    }

    fn set(&mut self, direction: Vec2, intensity: f32, duration: f32, propagated: bool) {
        self.direction = direction.normalize_or_zero();
        self.intensity = intensity;
        self.remaining = duration;
        self.duration = duration;
        self.propagated = propagated;
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Component)]
pub(super) struct Wander {
    timer: Timer,
    direction: Vec2,
}

impl Wander {
    fn new(duration: f32, direction: Vec2) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            direction,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SheepSnapshot {
    entity: Entity,
    position: Vec2,
    bravery: f32,
    panic: SheepPanic,
}

pub fn setup_herd(
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
                random_wander_direction(&mut rng),
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

pub fn sense_sheep_threats(
    shepherd: Query<&Transform, (With<Shepherd>, Without<Sheep>)>,
    dog: Query<&Transform, (With<Dog>, Without<Sheep>, Without<Shepherd>)>,
    mut sheep: Query<(&Transform, &mut SheepPanic), With<Sheep>>,
) {
    let shepherd_position = shepherd
        .iter()
        .next()
        .map(|transform| transform.translation.truncate());
    let dog_position = dog
        .iter()
        .next()
        .map(|transform| transform.translation.truncate());

    for (transform, mut fear) in &mut sheep {
        let position = transform.translation.truncate();
        let mut strongest: Option<(Vec2, f32)> = None;

        if let Some(threat_position) = shepherd_position {
            update_strongest_threat(
                &mut strongest,
                position,
                threat_position,
                SHEPHERD_THREAT_RADIUS,
                0.85,
            );
        }

        if let Some(threat_position) = dog_position {
            update_strongest_threat(
                &mut strongest,
                position,
                threat_position,
                DOG_THREAT_RADIUS,
                1.15,
            );
        }

        if let Some((direction, intensity)) = strongest {
            if !fear.active() || !fear.propagated || intensity >= fear.intensity {
                fear.set(direction, intensity, DIRECT_PANIC_DURATION, false);
            }
        }
    }
}

pub fn propagate_sheep_panic(
    mut sheep: ParamSet<(
        Query<(Entity, &Transform, &Bravery, &SheepPanic), With<Sheep>>,
        Query<&mut SheepPanic, With<Sheep>>,
    )>,
) {
    let snapshots = sheep
        .p0()
        .iter()
        .map(|(entity, transform, bravery, fear)| SheepSnapshot {
            entity,
            position: transform.translation.truncate(),
            bravery: bravery.value,
            panic: *fear,
        })
        .collect::<Vec<_>>();

    for candidate in &snapshots {
        if candidate.panic.active() && !candidate.panic.propagated {
            continue;
        }

        let Some((leader, intensity)) = find_panicked_leader(candidate, &snapshots) else {
            continue;
        };

        if let Ok(mut fear) = sheep.p1().get_mut(candidate.entity) {
            if !fear.active() || fear.propagated || intensity > fear.strength() {
                fear.set(
                    leader.panic.direction,
                    intensity,
                    PROPAGATED_PANIC_DURATION,
                    true,
                );
            }
        }
    }
}

pub fn steer_sheep(
    time: Res<Time>,
    mut sheep: ParamSet<(
        Query<(Entity, &Transform, &Bravery, &SheepPanic), With<Sheep>>,
        Query<
            (
                Entity,
                &Transform,
                &Bravery,
                &SheepPanic,
                &mut SheepMotion,
                &mut Wander,
            ),
            With<Sheep>,
        >,
    )>,
) {
    let snapshots = sheep
        .p0()
        .iter()
        .map(|(entity, transform, bravery, fear)| SheepSnapshot {
            entity,
            position: transform.translation.truncate(),
            bravery: bravery.value,
            panic: *fear,
        })
        .collect::<Vec<_>>();

    let mut rng = rand::rng();

    for (entity, transform, bravery, fear, mut motion, mut wander) in &mut sheep.p1() {
        let position = transform.translation.truncate();
        let separation = separation_direction(entity, position, &snapshots);

        let mut velocity = if fear.active() {
            let speed = SHEEP_SCARED_BASE_SPEED + bravery.value * SHEEP_BRAVERY_SPEED;
            let direction = (fear.direction + separation * 0.35).normalize_or_zero();
            direction * speed * fear.strength().clamp(0.45, 1.2)
        } else if let Some(leader) = find_calm_leader(entity, position, bravery.value, &snapshots) {
            let to_leader = leader.position - position;

            if to_leader.length() <= LEADER_STOP_RADIUS {
                Vec2::ZERO
            } else {
                let courage_gap = (leader.bravery - bravery.value).clamp(1.0, 10.0);
                let speed = LEADER_FOLLOW_SPEED * (0.45 + courage_gap / 14.0);
                (to_leader.normalize_or_zero() + separation * 0.8).normalize_or_zero() * speed
            }
        } else {
            update_wander(&mut wander, &time, &mut rng);
            (wander.direction + separation * 0.9).normalize_or_zero() * SHEEP_WANDER_SPEED
        };

        if velocity.length_squared() < 1.0 {
            velocity = Vec2::ZERO;
        }

        motion.velocity = velocity;
    }
}

pub fn decay_sheep_panic(time: Res<Time>, mut sheep: Query<&mut SheepPanic, With<Sheep>>) {
    for mut fear in &mut sheep {
        if !fear.active() {
            fear.clear();
            continue;
        }

        fear.remaining -= time.delta_secs();
        if fear.remaining <= 0.0 {
            fear.clear();
        }
    }
}

pub fn move_sheep(
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

pub fn update_sheep_animation_range(
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

pub fn animate_sheep(
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

fn update_strongest_threat(
    strongest: &mut Option<(Vec2, f32)>,
    sheep_position: Vec2,
    threat_position: Vec2,
    radius: f32,
    weight: f32,
) {
    let to_sheep = sheep_position - threat_position;
    let distance = to_sheep.length();
    if distance > radius {
        return;
    }

    let intensity = (0.45 + (1.0 - distance / radius) * 0.65) * weight;
    let direction = if distance > 0.0 {
        to_sheep / distance
    } else {
        Vec2::X
    };

    if strongest.is_none_or(|(_, current)| intensity > current) {
        *strongest = Some((direction, intensity));
    }
}

fn find_panicked_leader<'a>(
    candidate: &SheepSnapshot,
    snapshots: &'a [SheepSnapshot],
) -> Option<(&'a SheepSnapshot, f32)> {
    snapshots
        .iter()
        .filter(|other| {
            other.entity != candidate.entity
                && other.bravery > candidate.bravery
                && other.panic.active()
                && candidate.position.distance(other.position) <= PROPAGATED_PANIC_RADIUS
        })
        .filter_map(|other| {
            let distance = candidate.position.distance(other.position);
            let distance_factor = 1.0 - distance / PROPAGATED_PANIC_RADIUS;
            let bravery_factor = ((other.bravery - candidate.bravery) / 10.0).clamp(0.15, 1.0);
            let intensity = other.panic.strength() * distance_factor * (0.55 + bravery_factor);

            (intensity > 0.2).then_some((other, intensity))
        })
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
}

fn find_calm_leader<'a>(
    entity: Entity,
    position: Vec2,
    bravery: f32,
    snapshots: &'a [SheepSnapshot],
) -> Option<&'a SheepSnapshot> {
    if bravery >= LEADER_BRAVERY_THRESHOLD {
        return None;
    }

    snapshots
        .iter()
        .filter(|other| {
            other.entity != entity
                && other.bravery > bravery
                && !other.panic.active()
                && position.distance(other.position) <= LEADER_FOLLOW_RADIUS
        })
        .min_by(|left, right| {
            let left_distance = position.distance(left.position);
            let right_distance = position.distance(right.position);
            left_distance.total_cmp(&right_distance)
        })
}

fn separation_direction(entity: Entity, position: Vec2, snapshots: &[SheepSnapshot]) -> Vec2 {
    let mut separation = Vec2::ZERO;

    for other in snapshots {
        if other.entity == entity {
            continue;
        }

        let away = position - other.position;
        let distance = away.length();
        if distance > 0.0 && distance < SHEEP_SEPARATION_RADIUS {
            separation += away.normalize_or_zero() * (1.0 - distance / SHEEP_SEPARATION_RADIUS);
        }
    }

    separation.normalize_or_zero()
}

fn update_wander(wander: &mut Wander, time: &Time, rng: &mut impl Rng) {
    wander.timer.tick(time.delta());
    if !wander.timer.just_finished() {
        return;
    }

    wander.direction = random_wander_direction(rng);
    wander
        .timer
        .set_duration(Duration::from_secs_f32(rng.random_range(0.5..2.2)));
    wander.timer.reset();
}

fn random_wander_direction(rng: &mut impl Rng) -> Vec2 {
    if rng.random_bool(0.35) {
        return Vec2::ZERO;
    }

    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    Vec2::from_angle(angle)
}

fn random_facing(rng: &mut impl Rng) -> FacingDirection {
    match rng.random_range(0..4) {
        0 => FacingDirection::Right,
        1 => FacingDirection::Left,
        2 => FacingDirection::Down,
        _ => FacingDirection::Up,
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

fn standing_frame(direction: FacingDirection) -> usize {
    match direction {
        FacingDirection::Right => SHEEP_STAND_RIGHT,
        FacingDirection::Left => SHEEP_STAND_LEFT,
        FacingDirection::Down => SHEEP_STAND_DOWN,
        FacingDirection::Up => SHEEP_STAND_UP,
    }
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

fn clamp_position_to_world(position: &mut Vec2, bounds: &WorldBounds) {
    let half_size = Vec2::new(SHEEP_WIDTH as f32 / 2.0, SHEEP_HEIGHT as f32 / 2.0);
    let half_world = bounds.size / 2.0;
    position.x = position
        .x
        .clamp(-half_world.x + half_size.x, half_world.x - half_size.x);
    position.y = position
        .y
        .clamp(-half_world.y + half_size.y, half_world.y - half_size.y);
}
