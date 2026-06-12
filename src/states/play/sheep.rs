//! Sheep feature module.
//!
//! Sheep behavior is split into small systems for spawning, threat sensing,
//! panic propagation, steering, movement, animation, audio, and finish handling.

mod ai;
mod animation;
mod audio;
mod components;
mod finish;
mod movement;
mod spawn;

pub(super) use ai::{decay_sheep_panic, propagate_sheep_panic, sense_sheep_threats, steer_sheep};
pub(super) use animation::{animate_sheep, update_sheep_animation_range};
pub(super) use audio::{play_idle_sheep_sounds, setup_sheep_audio};
pub(in crate::states::play) use components::Sheep;
pub(super) use finish::finish_sheep;
pub(super) use movement::move_sheep;
pub(super) use spawn::setup_herd;
