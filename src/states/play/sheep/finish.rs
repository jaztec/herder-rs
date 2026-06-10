use bevy::prelude::*;

use crate::states::play::{
    score::{FinishArea, HerdScore},
    sheep::{
        audio::{SheepAudio, SheepSoundKind, play_sheep_sound},
        components::{SHEEP_HEIGHT, SHEEP_WIDTH, Sheep},
    },
};

pub(in crate::states::play) fn finish_sheep(
    mut commands: Commands,
    time: Res<Time>,
    finish_area: Res<FinishArea>,
    audio: Res<SheepAudio>,
    mut score: ResMut<HerdScore>,
    sheep: Query<(Entity, &Transform), With<Sheep>>,
) {
    for (entity, transform) in &sheep {
        if !overlaps_finish(transform.translation.truncate(), &finish_area) {
            continue;
        }

        commands.entity(entity).despawn();
        score.record_finish(time.elapsed_secs());
        play_sheep_sound(&mut commands, &audio, SheepSoundKind::Finish);
    }
}

fn overlaps_finish(sheep_position: Vec2, finish_area: &FinishArea) -> bool {
    let sheep_half_size = Vec2::new(SHEEP_WIDTH as f32 / 2.0, SHEEP_HEIGHT as f32 / 2.0);
    let finish_half_size = finish_area.size / 2.0;
    let offset = sheep_position - finish_area.center;

    offset.x.abs() <= sheep_half_size.x + finish_half_size.x
        && offset.y.abs() <= sheep_half_size.y + finish_half_size.y
}
