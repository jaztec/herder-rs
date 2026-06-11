use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use bevy::prelude::*;

use crate::states::{GameState, game_state::despawn_screen, play::score::HerdScore};

use super::plugin::PlayState;

const HIGHSCORE_FILE: &str = "herder_highscores.txt";
const MAX_HIGHSCORES: usize = 5;
const FINISH_OVERLAY_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.62);
const FINISH_PANEL_COLOR: Color = Color::srgba(0.05, 0.07, 0.06, 0.94);
const FINISH_TEXT_COLOR: Color = Color::srgb(0.96, 0.94, 0.86);
const FINISH_MUTED_TEXT_COLOR: Color = Color::srgb(0.82, 0.84, 0.78);

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub(in crate::states::play) struct RunTimer {
    elapsed_seconds: f32,
}

impl Default for RunTimer {
    fn default() -> Self {
        Self {
            elapsed_seconds: 0.0,
        }
    }
}

impl RunTimer {
    pub(in crate::states::play) fn elapsed_seconds(self) -> f32 {
        self.elapsed_seconds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(in crate::states::play) struct FinishOverlay;

#[derive(Debug, Clone, Copy)]
struct Highscore {
    score: u32,
    seconds: f32,
}

pub(in crate::states::play) fn finish_state_plugin(app: &mut App) {
    app.init_resource::<RunTimer>()
        .add_systems(OnEnter(PlayState::Finished), setup_finish_overlay)
        .add_systems(OnExit(PlayState::Finished), despawn_screen::<FinishOverlay>)
        .add_systems(
            Update,
            handle_finished_input.run_if(in_state(PlayState::Finished)),
        );
}

pub(in crate::states::play) fn reset_run_timer(mut timer: ResMut<RunTimer>) {
    timer.elapsed_seconds = 0.0;
}

pub(in crate::states::play) fn tick_run_timer(time: Res<Time>, mut timer: ResMut<RunTimer>) {
    timer.elapsed_seconds += time.delta_secs();
}

pub(in crate::states::play) fn finish_when_herd_complete(
    score: Res<HerdScore>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    if score.is_complete() {
        next_play_state.set(PlayState::Finished);
    }
}

fn setup_finish_overlay(mut commands: Commands, score: Res<HerdScore>, timer: Res<RunTimer>) {
    let final_time = timer.elapsed_seconds();
    let highscores = save_highscore(score.score(), final_time).unwrap_or_else(|err| {
        eprintln!("Failed to save highscore: {err}");
        read_highscores().unwrap_or_default()
    });

    commands.spawn((
        FinishOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(FINISH_OVERLAY_COLOR),
        children![(
            Node {
                width: Val::Px(460.0),
                padding: UiRect::all(Val::Px(30.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(FINISH_PANEL_COLOR),
            children![
                (
                    Text::new("Finished"),
                    TextFont {
                        font_size: 46.0,
                        ..default()
                    },
                    TextColor(FINISH_TEXT_COLOR),
                ),
                (
                    Text::new(format!(
                        "Score: {}\nTime: {}",
                        score.score(),
                        format_time(final_time)
                    )),
                    TextFont {
                        font_size: 25.0,
                        ..default()
                    },
                    TextColor(FINISH_TEXT_COLOR),
                ),
                (
                    Text::new(highscore_text(&highscores)),
                    TextFont {
                        font_size: 19.0,
                        ..default()
                    },
                    TextColor(FINISH_MUTED_TEXT_COLOR),
                ),
                (
                    Text::new("Enter: new game    Esc: main menu"),
                    TextFont {
                        font_size: 17.0,
                        ..default()
                    },
                    TextColor(FINISH_MUTED_TEXT_COLOR),
                ),
            ],
        )],
    ));
}

fn handle_finished_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    if input.just_pressed(KeyCode::Enter) {
        next_play_state.set(PlayState::Disabled);
        next_game_state.set(GameState::RestartPlay);
    } else if input.just_pressed(KeyCode::Escape) {
        next_play_state.set(PlayState::Disabled);
        next_game_state.set(GameState::Menu);
    }
}

fn save_highscore(score: u32, seconds: f32) -> io::Result<Vec<Highscore>> {
    let mut highscores = read_highscores()?;
    highscores.push(Highscore { score, seconds });
    highscores.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.seconds.total_cmp(&right.seconds))
    });
    highscores.truncate(MAX_HIGHSCORES);

    let mut file = fs::File::create(HIGHSCORE_FILE)?;
    for highscore in &highscores {
        writeln!(file, "{},{}", highscore.score, highscore.seconds)?;
    }

    Ok(highscores)
}

fn read_highscores() -> io::Result<Vec<Highscore>> {
    if !Path::new(HIGHSCORE_FILE).exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(HIGHSCORE_FILE)?;
    let highscores = contents
        .lines()
        .filter_map(|line| {
            let (score, seconds) = line.split_once(',')?;
            Some(Highscore {
                score: score.parse().ok()?,
                seconds: seconds.parse().ok()?,
            })
        })
        .collect();

    Ok(highscores)
}

fn highscore_text(highscores: &[Highscore]) -> String {
    if highscores.is_empty() {
        return "Highscores\nNo scores yet".to_string();
    }

    let rows = highscores
        .iter()
        .enumerate()
        .map(|(index, highscore)| {
            format!(
                "{}. {} - {}",
                index + 1,
                highscore.score,
                format_time(highscore.seconds)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("Highscores\n{rows}")
}

fn format_time(seconds: f32) -> String {
    let total_seconds = seconds.round() as u32;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}
