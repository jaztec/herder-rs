use bevy::prelude::*;

use crate::{
    states::play::sheep::SHEEP_COUNT,
    world::{MapConfig, Tile, TileMap},
};

const FINISH_SCORE: u32 = 40;
const COMBO_WINDOW_SECONDS: f32 = 1.0;
const SCORE_TEXT_COLOR: Color = Color::srgb(0.94, 0.94, 0.9);
const SCORE_BACKGROUND_COLOR: Color = Color::srgba(0.04, 0.05, 0.04, 0.72);

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub(super) struct FinishArea {
    pub center: Vec2,
    pub size: Vec2,
}

#[derive(Debug, Resource)]
pub(super) struct HerdScore {
    score: u32,
    finished: usize,
    total: usize,
    last_finish_seconds: Option<f32>,
    combo_index: u32,
}

impl HerdScore {
    fn new(total: usize) -> Self {
        Self {
            score: 0,
            finished: 0,
            total,
            last_finish_seconds: None,
            combo_index: 0,
        }
    }

    pub(super) fn record_finish(&mut self, elapsed_seconds: f32) {
        let add_score = if self
            .last_finish_seconds
            .is_some_and(|last| elapsed_seconds - last < COMBO_WINDOW_SECONDS)
        {
            self.combo_index += 1;
            (FINISH_SCORE * 2) * self.combo_index
        } else {
            self.combo_index = 0;
            FINISH_SCORE
        };

        self.score += add_score;
        self.finished += 1;
        self.last_finish_seconds = Some(elapsed_seconds);
    }

    fn label(&self) -> String {
        format!(
            "Score: {} punten   Sheep: {}/{}",
            self.score, self.finished, self.total
        )
    }

    pub(in crate::states::play) fn pause_label(&self) -> String {
        format!(
            "Score: {}\nSheep herded: {}/{}",
            self.score, self.finished, self.total
        )
    }
}

impl Default for HerdScore {
    fn default() -> Self {
        Self::new(SHEEP_COUNT)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(super) struct ScoreText;

pub fn setup_finish_area(mut commands: Commands, tiles: Res<TileMap>, config: Res<MapConfig>) {
    for y in 0..tiles.height() {
        for x in 0..tiles.width() {
            if !tiles.get(x, y).is_some_and(|tile| *tile == Tile::Finish) {
                continue;
            }

            commands.insert_resource(FinishArea {
                center: config.tile_world_position(x, y).truncate(),
                size: Vec2::splat(config.tile_size),
            });
            return;
        }
    }
}

pub fn setup_herd_score(mut score: ResMut<HerdScore>) {
    *score = HerdScore::new(SHEEP_COUNT);
}

pub fn setup_score_hud(mut commands: Commands, score: Res<HerdScore>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(SCORE_BACKGROUND_COLOR),
        children![(
            ScoreText,
            Text::new(score.label()),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(SCORE_TEXT_COLOR),
        )],
    ));
}

pub fn update_score_hud(score: Res<HerdScore>, mut score_text: Query<&mut Text, With<ScoreText>>) {
    if !score.is_changed() {
        return;
    }

    for mut text in &mut score_text {
        text.0 = score.label();
    }
}
