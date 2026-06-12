//! Sheep components and tuning constants.

use bevy::prelude::*;

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

pub(super) const SHEEP_SCARED_BASE_SPEED: f32 = 190.0;
pub(super) const SHEEP_BRAVERY_SPEED: f32 = 10.0;
pub(super) const SHEEP_WANDER_SPEED: f32 = 38.0;
pub(super) const SHEEP_SEPARATION_RADIUS: f32 = 46.0;

/// Marker component for sheep entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Component)]
pub struct Sheep;

/// Stable identifier for a sheep within the current run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(super) struct SheepId(pub(super) usize);

/// Per-sheep bravery value used by panic and leader-following behavior.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(in crate::states::play) struct Bravery {
    /// Higher values make sheep faster and more likely to act as leaders.
    pub(super) value: f32,
}

/// Current AI-selected sheep velocity.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(in crate::states::play) struct SheepMotion {
    /// Velocity in world units per second.
    pub(super) velocity: Vec2,
}

/// Temporary panic state caused by direct threats or nearby panicked leaders.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub(in crate::states::play) struct SheepPanic {
    /// Direction the sheep tries to flee.
    pub(super) direction: Vec2,
    /// Panic strength before duration falloff is applied.
    pub(super) intensity: f32,
    /// Remaining panic duration in seconds.
    pub(super) remaining: f32,
    /// Original panic duration in seconds.
    pub(super) duration: f32,
    /// True when panic came from another sheep instead of a direct threat.
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
    /// Return true while the panic state can affect movement.
    pub(super) fn active(self) -> bool {
        self.remaining > 0.0 && self.intensity > 0.0 && self.direction.length_squared() > 0.0
    }

    /// Current panic strength after remaining-time falloff.
    pub(super) fn strength(self) -> f32 {
        if !self.active() || self.duration <= 0.0 {
            return 0.0;
        }

        self.intensity * (self.remaining / self.duration).clamp(0.0, 1.0)
    }

    /// Replace the current panic state.
    pub(super) fn set(&mut self, direction: Vec2, intensity: f32, duration: f32, propagated: bool) {
        self.direction = direction.normalize_or_zero();
        self.intensity = intensity;
        self.remaining = duration;
        self.duration = duration;
        self.propagated = propagated;
    }

    /// Clear panic back to idle state.
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Idle wandering timer and direction.
#[derive(Debug, Component)]
pub(in crate::states::play) struct Wander {
    /// Timer controlling when the wander direction changes.
    pub(super) timer: Timer,
    /// Current idle movement direction.
    pub(super) direction: Vec2,
}

impl Wander {
    /// Create a one-shot wander timer and initial direction.
    pub(super) fn new(duration: f32, direction: Vec2) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            direction,
        }
    }
}
