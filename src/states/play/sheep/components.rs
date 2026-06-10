use bevy::prelude::*;

pub(in crate::states::play) const SHEEP_COUNT: usize = 30;
pub(super) const SHEEP_NAME: &str = "Sheep";
pub(super) const SHEEP_WIDTH: u32 = 75;
pub(super) const SHEEP_HEIGHT: u32 = 60;
pub(super) const SHEEP_Z: f32 = 10.5;

pub(super) const HERD_START_SIZE: f32 = 220.0;
pub(super) const MIN_SPAWN_EDGE_MARGIN: f32 = 160.0;

pub(super) const SHEPHERD_THREAT_RADIUS: f32 = 180.0;
pub(super) const DOG_THREAT_RADIUS: f32 = 280.0;
pub(super) const DIRECT_PANIC_DURATION: f32 = 0.85;
pub(super) const PROPAGATED_PANIC_RADIUS: f32 = 300.0;
pub(super) const PROPAGATED_PANIC_DURATION: f32 = 0.65;

pub(super) const LEADER_BRAVERY_THRESHOLD: f32 = 7.0;
pub(super) const LEADER_FOLLOW_RADIUS: f32 = 320.0;
pub(super) const LEADER_STOP_RADIUS: f32 = 80.0;
pub(super) const LEADER_FOLLOW_SPEED: f32 = 55.0;

pub(super) const SHEEP_SCARED_BASE_SPEED: f32 = 120.0;
pub(super) const SHEEP_BRAVERY_SPEED: f32 = 16.0;
pub(super) const SHEEP_WANDER_SPEED: f32 = 38.0;
pub(super) const SHEEP_SEPARATION_RADIUS: f32 = 46.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub struct Sheep;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(super) struct SheepId(pub(super) usize);

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(in crate::states::play) struct Bravery {
    pub(super) value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(in crate::states::play) struct SheepMotion {
    pub(super) velocity: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(in crate::states::play) struct SheepPanic {
    pub(super) direction: Vec2,
    pub(super) intensity: f32,
    pub(super) remaining: f32,
    pub(super) duration: f32,
    pub(super) propagated: bool,
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
    pub(super) fn active(self) -> bool {
        self.remaining > 0.0 && self.intensity > 0.0 && self.direction.length_squared() > 0.0
    }

    pub(super) fn strength(self) -> f32 {
        if !self.active() || self.duration <= 0.0 {
            return 0.0;
        }

        self.intensity * (self.remaining / self.duration).clamp(0.0, 1.0)
    }

    pub(super) fn set(&mut self, direction: Vec2, intensity: f32, duration: f32, propagated: bool) {
        self.direction = direction.normalize_or_zero();
        self.intensity = intensity;
        self.remaining = duration;
        self.duration = duration;
        self.propagated = propagated;
    }

    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Component)]
pub(in crate::states::play) struct Wander {
    pub(super) timer: Timer,
    pub(super) direction: Vec2,
}

impl Wander {
    pub(super) fn new(duration: f32, direction: Vec2) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            direction,
        }
    }
}
