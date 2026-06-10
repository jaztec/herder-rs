use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

use crate::states::play::{
    dog::Dog,
    sheep::{
        audio::{SheepAudio, SheepSoundKind, SheepSoundState, play_scare_sound},
        components::{
            Bravery, DIRECT_PANIC_DURATION, DOG_THREAT_RADIUS, LEADER_BRAVERY_THRESHOLD,
            LEADER_FOLLOW_RADIUS, LEADER_FOLLOW_SPEED, LEADER_STOP_RADIUS,
            PROPAGATED_PANIC_DURATION, PROPAGATED_PANIC_RADIUS, SHEEP_BRAVERY_SPEED,
            SHEEP_SCARED_BASE_SPEED, SHEEP_SEPARATION_RADIUS, SHEEP_WANDER_SPEED,
            SHEPHERD_THREAT_RADIUS, Sheep, SheepMotion, SheepPanic, Wander,
        },
    },
    shepherd::Shepherd,
};

#[derive(Debug, Clone, Copy)]
struct SheepSnapshot {
    entity: Entity,
    position: Vec2,
    bravery: f32,
    panic: SheepPanic,
}

type DogTransformQuery<'w, 's> =
    Query<'w, 's, &'static Transform, (With<Dog>, Without<Sheep>, Without<Shepherd>)>;

type SheepSnapshotQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static Bravery,
        &'static SheepPanic,
    ),
    With<Sheep>,
>;

type SheepPanicQuery<'w, 's> = Query<'w, 's, &'static mut SheepPanic, With<Sheep>>;

type SheepSteeringQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static Bravery,
        &'static SheepPanic,
        &'static mut SheepMotion,
        &'static mut Wander,
    ),
    With<Sheep>,
>;

pub(in crate::states::play) fn sense_sheep_threats(
    mut commands: Commands,
    time: Res<Time>,
    audio: Res<SheepAudio>,
    mut sound_state: ResMut<SheepSoundState>,
    shepherd: Query<&Transform, (With<Shepherd>, Without<Sheep>)>,
    dog: DogTransformQuery,
    mut sheep: Query<(&Transform, &mut SheepPanic), With<Sheep>>,
) {
    sound_state.scare_cooldown = (sound_state.scare_cooldown - time.delta_secs()).max(0.0);

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
        let mut strongest: Option<(Vec2, f32, SheepSoundKind)> = None;

        if let Some(threat_position) = shepherd_position {
            update_strongest_threat(
                &mut strongest,
                position,
                threat_position,
                SHEPHERD_THREAT_RADIUS,
                0.85,
                SheepSoundKind::Bleat1,
            );
        }

        if let Some(threat_position) = dog_position {
            update_strongest_threat(
                &mut strongest,
                position,
                threat_position,
                DOG_THREAT_RADIUS,
                1.15,
                SheepSoundKind::Bleat2,
            );
        }

        if let Some((direction, intensity, sound)) = strongest
            && (!fear.active() || !fear.propagated || intensity >= fear.intensity)
        {
            fear.set(direction, intensity, DIRECT_PANIC_DURATION, false);
            play_scare_sound(&mut commands, &audio, &mut sound_state, sound);
        }
    }
}

pub(in crate::states::play) fn propagate_sheep_panic(
    mut sheep: ParamSet<(SheepSnapshotQuery, SheepPanicQuery)>,
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

        if let Ok(mut fear) = sheep.p1().get_mut(candidate.entity)
            && (!fear.active() || fear.propagated || intensity > fear.strength())
        {
            fear.set(
                leader.panic.direction,
                intensity,
                PROPAGATED_PANIC_DURATION,
                true,
            );
        }
    }
}

pub(in crate::states::play) fn steer_sheep(
    time: Res<Time>,
    mut sheep: ParamSet<(SheepSnapshotQuery, SheepSteeringQuery)>,
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

pub(in crate::states::play) fn decay_sheep_panic(
    time: Res<Time>,
    mut sheep: Query<&mut SheepPanic, With<Sheep>>,
) {
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

fn update_strongest_threat(
    strongest: &mut Option<(Vec2, f32, SheepSoundKind)>,
    sheep_position: Vec2,
    threat_position: Vec2,
    radius: f32,
    weight: f32,
    sound: SheepSoundKind,
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

    if strongest.is_none_or(|(_, current, _)| intensity > current) {
        *strongest = Some((direction, intensity, sound));
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

fn find_calm_leader(
    entity: Entity,
    position: Vec2,
    bravery: f32,
    snapshots: &[SheepSnapshot],
) -> Option<&SheepSnapshot> {
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

pub(super) fn random_wander_direction(rng: &mut impl Rng) -> Vec2 {
    if rng.random_bool(0.35) {
        return Vec2::ZERO;
    }

    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    Vec2::from_angle(angle)
}
