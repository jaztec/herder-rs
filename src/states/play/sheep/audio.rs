//! Sheep audio resource and playback systems.

use bevy::prelude::*;
use rand::Rng;

use crate::states::play::sheep::components::Sheep;

/// Cooldown between threat-triggered bleats.
pub(super) const SHEEP_SCARE_SOUND_COOLDOWN: f32 = 0.35;
const SHEEP_IDLE_SOUND_MIN_SECONDS: f32 = 4.0;
const SHEEP_IDLE_SOUND_MAX_SECONDS: f32 = 8.0;

/// Loaded audio handles for sheep sounds.
#[derive(Debug, Resource)]
pub(in crate::states::play) struct SheepAudio {
    bleat_1: Handle<AudioSource>,
    bleat_2: Handle<AudioSource>,
    finish: Handle<AudioSource>,
}

/// Runtime sheep audio timers.
#[derive(Debug, Resource)]
pub(in crate::states::play) struct SheepSoundState {
    /// Shared cooldown for scare sounds.
    pub(super) scare_cooldown: f32,
    idle_timer: f32,
}

/// Sheep sound variants used by gameplay systems.
#[derive(Debug, Clone, Copy)]
pub(super) enum SheepSoundKind {
    /// First bleat variant.
    Bleat1,
    /// Second bleat variant.
    Bleat2,
    /// Sound played when a sheep enters the finish.
    Finish,
}

/// Load sheep audio handles and initialize playback timers.
pub(in crate::states::play) fn setup_sheep_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(SheepAudio {
        bleat_1: asset_server.load("sounds/beh_1.ogg"),
        bleat_2: asset_server.load("sounds/beh_2.ogg"),
        finish: asset_server.load("sounds/finish.ogg"),
    });

    commands.insert_resource(SheepSoundState {
        scare_cooldown: 0.0,
        idle_timer: 2.0,
    });
}

/// Occasionally play an idle bleat while sheep remain in the world.
pub(in crate::states::play) fn play_idle_sheep_sounds(
    mut commands: Commands,
    time: Res<Time>,
    audio: Res<SheepAudio>,
    mut sound_state: ResMut<SheepSoundState>,
    sheep: Query<(), With<Sheep>>,
) {
    if sheep.is_empty() {
        return;
    }

    sound_state.idle_timer -= time.delta_secs();
    if sound_state.idle_timer > 0.0 {
        return;
    }

    let mut rng = rand::rng();
    let sound = if rng.random_bool(0.5) {
        SheepSoundKind::Bleat1
    } else {
        SheepSoundKind::Bleat2
    };

    play_sheep_sound(&mut commands, &audio, sound);
    sound_state.idle_timer =
        rng.random_range(SHEEP_IDLE_SOUND_MIN_SECONDS..=SHEEP_IDLE_SOUND_MAX_SECONDS);
}

/// Play a scare sound if the scare cooldown has elapsed.
pub(super) fn play_scare_sound(
    commands: &mut Commands,
    audio: &SheepAudio,
    sound_state: &mut SheepSoundState,
    sound: SheepSoundKind,
) {
    if sound_state.scare_cooldown > 0.0 {
        return;
    }

    play_sheep_sound(commands, audio, sound);
    sound_state.scare_cooldown = SHEEP_SCARE_SOUND_COOLDOWN;
}

/// Spawn a one-shot sheep audio player.
pub(super) fn play_sheep_sound(commands: &mut Commands, audio: &SheepAudio, sound: SheepSoundKind) {
    let handle = match sound {
        SheepSoundKind::Bleat1 => audio.bleat_1.clone(),
        SheepSoundKind::Bleat2 => audio.bleat_2.clone(),
        SheepSoundKind::Finish => audio.finish.clone(),
    };

    commands.spawn((AudioPlayer::new(handle), PlaybackSettings::DESPAWN));
}
