//! Score, finish-area, and HUD systems.

use bevy::prelude::*;

use crate::{
    run_config::RunConfig,
    world::{FinishTilePosition, MapConfig},
};

const FINISH_SCORE: u32 = 40;
const COMBO_WINDOW_SECONDS: f32 = 1.0;
const SCORE_TEXT_COLOR: Color = Color::srgb(0.94, 0.94, 0.9);
const SCORE_BACKGROUND_COLOR: Color = Color::srgba(0.04, 0.05, 0.04, 0.72);

/// World-space finish area used for sheep overlap checks and indicators.
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub(super) struct FinishArea {
    /// Center point of the finish tile in world coordinates.
    pub center: Vec2,
    /// Size of the finish area in world units.
    pub size: Vec2,
}

/// Score and completion state for the current herd.
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

    /// Record a sheep entering the finish area and apply combo scoring.
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
            "Score: {} points   Sheep: {}/{}",
            self.score, self.finished, self.total
        )
    }

    /// Multi-line label used by the pause overlay.
    pub(in crate::states::play) fn pause_label(&self) -> String {
        format!(
            "Score: {}\nSheep herded: {}/{}",
            self.score, self.finished, self.total
        )
    }

    /// Current score value.
    pub(in crate::states::play) fn score(&self) -> u32 {
        self.score
    }

    /// Return true once all sheep have been scored.
    pub(in crate::states::play) fn is_complete(&self) -> bool {
        self.total > 0 && self.finished >= self.total
    }
}

impl Default for HerdScore {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Marker component for the score HUD root node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(in crate::states::play) struct ScoreHud;

/// Marker component for the mutable score text node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(in crate::states::play) struct ScoreText;

/// Create the finish-area resource from the generated finish tile.
pub fn setup_finish_area(
    mut commands: Commands,
    finish: Res<FinishTilePosition>,
    config: Res<MapConfig>,
) {
    commands.insert_resource(FinishArea {
        center: config.tile_world_position(finish.x, finish.y).truncate(),
        size: Vec2::splat(config.tile_size),
    });
}

/// Reset the score resource for a new run.
pub fn setup_herd_score(mut score: ResMut<HerdScore>, run_config: Res<RunConfig>) {
    *score = HerdScore::new(run_config.sheep_count);
}

/// Spawn the in-game score HUD.
pub fn setup_score_hud(mut commands: Commands, score: Res<HerdScore>) {
    commands.spawn((
        ScoreHud,
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

/// Refresh the HUD text when the score changes.
pub fn update_score_hud(score: Res<HerdScore>, mut score_text: Query<&mut Text, With<ScoreText>>) {
    if !score.is_changed() {
        return;
    }

    for mut text in &mut score_text {
        text.0 = score.label();
    }
}
