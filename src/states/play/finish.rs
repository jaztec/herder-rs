//! Finish state, run timer, highscore persistence, and finish overlay UI.

use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use bevy::{
    ecs::system::SystemParam,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use crate::{
    run_config::RunConfig,
    states::{GameState, game_state::despawn_screen, play::score::HerdScore},
};

use super::plugin::PlayState;

const HIGHSCORE_FILE: &str = "herder_highscores.txt";
const MAX_HIGHSCORES: usize = 5;
const FINISH_OVERLAY_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.62);
const FINISH_PANEL_COLOR: Color = Color::srgba(0.05, 0.07, 0.06, 0.94);
const FINISH_TEXT_COLOR: Color = Color::srgb(0.96, 0.94, 0.86);
const FINISH_MUTED_TEXT_COLOR: Color = Color::srgb(0.82, 0.84, 0.78);
const FINISH_HIGHLIGHT_TEXT_COLOR: Color = Color::srgb(1.0, 0.86, 0.32);
const DEFAULT_PLAYER_NAME: &str = "Player";
const MAX_PLAYER_NAME_CHARS: usize = 14;

/// Timer for elapsed active play time.
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
    /// Return elapsed active play time in seconds.
    pub(in crate::states::play) fn elapsed_seconds(self) -> f32 {
        self.elapsed_seconds
    }
}

/// Marker component for the finish overlay root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(in crate::states::play) struct FinishOverlay;

#[derive(Debug, Clone, PartialEq, Resource)]
struct PendingHighscore {
    name: String,
    score: u32,
    seconds: f32,
    saved: bool,
    saved_highscores: Option<Vec<HighscoreDisplayRow>>,
}

#[derive(Debug, Clone, PartialEq)]
struct StoredHighscore {
    name: String,
    score: u32,
    seconds: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct HighscoreDisplayRow {
    name: String,
    score: u32,
    seconds: f32,
    highlighted: bool,
}

impl HighscoreDisplayRow {
    fn from_stored(highscore: &StoredHighscore) -> Self {
        Self {
            name: highscore.name.clone(),
            score: highscore.score,
            seconds: highscore.seconds,
            highlighted: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct FinishNameText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct FinishInstructionText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
struct FinishHighscoreRow(usize);

type FinishHighscoreRowsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static FinishHighscoreRow,
        &'static mut Text,
        &'static mut TextColor,
        &'static mut Node,
    ),
>;
type FinishUiFields<'w, 's> = ParamSet<
    'w,
    's,
    (
        Single<'w, &'static mut Text, With<FinishNameText>>,
        Single<'w, &'static mut Text, With<FinishInstructionText>>,
        FinishHighscoreRowsQuery<'w, 's>,
    ),
>;

#[derive(SystemParam)]
struct FinishUi<'w, 's> {
    fields: FinishUiFields<'w, 's>,
}

/// Register finish-state systems.
pub(in crate::states::play) fn finish_state_plugin(app: &mut App) {
    app.init_resource::<RunTimer>()
        .add_systems(OnEnter(PlayState::Finished), setup_finish_overlay)
        .add_systems(OnExit(PlayState::Finished), despawn_screen::<FinishOverlay>)
        .add_systems(
            Update,
            handle_finished_input.run_if(in_state(PlayState::Finished)),
        );
}

/// Reset the run timer for a new play session.
pub(in crate::states::play) fn reset_run_timer(mut timer: ResMut<RunTimer>) {
    timer.elapsed_seconds = 0.0;
}

/// Tick active play time.
pub(in crate::states::play) fn tick_run_timer(time: Res<Time>, mut timer: ResMut<RunTimer>) {
    timer.elapsed_seconds += time.delta_secs();
}

/// Enter the finished state when every sheep has been scored.
pub(in crate::states::play) fn finish_when_herd_complete(
    score: Res<HerdScore>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    if score.is_complete() {
        next_play_state.set(PlayState::Finished);
    }
}

fn setup_finish_overlay(
    mut commands: Commands,
    score: Res<HerdScore>,
    timer: Res<RunTimer>,
    run_config: Res<RunConfig>,
) {
    let final_time = timer.elapsed_seconds();
    let highscores = read_highscores(&run_config).unwrap_or_default();

    commands.insert_resource(PendingHighscore {
        name: String::new(),
        score: score.score(),
        seconds: final_time,
        saved: false,
        saved_highscores: None,
    });

    let highscore_rows = display_rows_from_stored(&highscores);
    let highscore_title = run_config.score_table_name();

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
                highscore_list_node(&highscore_rows, &highscore_title),
                (
                    Text::new("Name: _"),
                    FinishNameText,
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(FINISH_TEXT_COLOR),
                ),
                (
                    Text::new("Type name   Enter: save score   Esc: save & menu"),
                    FinishInstructionText,
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
    mut keyboard_input: EventReader<KeyboardInput>,
    mut pending: ResMut<PendingHighscore>,
    mut finish_ui: FinishUi,
    run_config: Res<RunConfig>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    if !pending.saved {
        for event in keyboard_input.read() {
            if !event.state.is_pressed() {
                continue;
            }

            match (&event.logical_key, &event.text) {
                (Key::Backspace, _) => {
                    pending.name.pop();
                }
                (_, Some(text)) => {
                    for character in text
                        .chars()
                        .filter(|character| is_name_character(*character))
                    {
                        if pending.name.chars().count() >= MAX_PLAYER_NAME_CHARS {
                            break;
                        }
                        pending.name.push(character);
                    }
                }
                _ => {}
            }
        }
    }

    finish_ui.fields.p0().0 = if pending.saved {
        format!("Name: {}", normalized_player_name(&pending.name))
    } else {
        format!("Name: {}", display_name_input(&pending.name))
    };

    if input.just_pressed(KeyCode::Enter) {
        if pending.saved {
            next_play_state.set(PlayState::Disabled);
            next_game_state.set(GameState::RestartPlay);
        } else {
            save_pending_highscore(&mut pending, &run_config);
            finish_ui.fields.p0().0 = format!("Name: {}", normalized_player_name(&pending.name));
            finish_ui.fields.p1().0 = "Enter: new game   Esc: menu".to_string();

            if let Some(rows) = pending.saved_highscores.clone() {
                apply_highscore_rows(&rows, &mut finish_ui.fields.p2());
            }
        }
    } else if input.just_pressed(KeyCode::Escape) {
        save_pending_highscore(&mut pending, &run_config);
        next_play_state.set(PlayState::Disabled);
        next_game_state.set(GameState::Menu);
    }
}

fn save_pending_highscore(pending: &mut PendingHighscore, run_config: &RunConfig) {
    if pending.saved {
        return;
    }

    let name = normalized_player_name(&pending.name);
    match save_highscore(&name, pending.score, pending.seconds, run_config) {
        Ok(highscores) => {
            pending.saved_highscores = Some(highscores);
        }
        Err(err) => {
            eprintln!("Failed to save highscore: {err}");
        }
    }

    pending.saved = true;
}

fn save_highscore(
    name: &str,
    score: u32,
    seconds: f32,
    run_config: &RunConfig,
) -> io::Result<Vec<HighscoreDisplayRow>> {
    let current = StoredHighscore {
        name: name.to_string(),
        score,
        seconds,
    };
    let mut highscores = read_highscores(run_config)?
        .into_iter()
        .map(|highscore| HighscoreDisplayRow {
            name: highscore.name,
            score: highscore.score,
            seconds: highscore.seconds,
            highlighted: false,
        })
        .collect::<Vec<_>>();

    highscores.push(HighscoreDisplayRow {
        name: current.name,
        score: current.score,
        seconds: current.seconds,
        highlighted: true,
    });
    highscores.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.seconds.total_cmp(&right.seconds))
    });
    highscores.truncate(MAX_HIGHSCORES);

    let mut file = fs::File::create(highscore_file_name(run_config))?;
    for highscore in &highscores {
        writeln!(
            file,
            "{},{},{}",
            highscore.name, highscore.score, highscore.seconds
        )?;
    }

    Ok(highscores)
}

/// Highscore list text used by overlays outside the finish screen.
pub(in crate::states::play) fn highscore_text_for_overlay(run_config: &RunConfig) -> String {
    highscore_text(
        &run_config.score_table_name(),
        &display_rows_from_stored(&read_highscores(run_config).unwrap_or_default()),
    )
}

fn read_highscores(run_config: &RunConfig) -> io::Result<Vec<StoredHighscore>> {
    let file_name = highscore_file_name(run_config);
    if !Path::new(&file_name).exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(file_name)?;
    let highscores = contents
        .lines()
        .filter_map(|line| {
            let fields = line.split(',').collect::<Vec<_>>();
            match fields.as_slice() {
                [name, score, seconds] => Some(StoredHighscore {
                    name: sanitize_stored_name(name),
                    score: score.parse().ok()?,
                    seconds: seconds.parse().ok()?,
                }),
                [score, seconds] => Some(StoredHighscore {
                    name: DEFAULT_PLAYER_NAME.to_string(),
                    score: score.parse().ok()?,
                    seconds: seconds.parse().ok()?,
                }),
                _ => None,
            }
        })
        .collect();

    Ok(highscores)
}

fn display_rows_from_stored(highscores: &[StoredHighscore]) -> Vec<HighscoreDisplayRow> {
    highscores
        .iter()
        .map(HighscoreDisplayRow::from_stored)
        .collect()
}

fn highscore_text(title: &str, highscores: &[HighscoreDisplayRow]) -> String {
    if highscores.is_empty() {
        return format!("Highscores - {title}\nNo scores yet");
    }

    let rows = highscores
        .iter()
        .enumerate()
        .map(|(index, highscore)| {
            format!(
                "{}. {}  {} - {}",
                index + 1,
                highscore.name,
                highscore.score,
                format_time(highscore.seconds)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("Highscores - {title}\n{rows}")
}

fn highscore_list_node(rows: &[HighscoreDisplayRow], title: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(5.0),
            ..default()
        },
        children![
            (
                Text::new(format!("Highscores - {title}")),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(FINISH_MUTED_TEXT_COLOR),
            ),
            highscore_row_node(rows, 0),
            highscore_row_node(rows, 1),
            highscore_row_node(rows, 2),
            highscore_row_node(rows, 3),
            highscore_row_node(rows, 4),
        ],
    )
}

fn highscore_file_name(run_config: &RunConfig) -> String {
    match &run_config.mode {
        crate::run_config::PlayMode::Random => HIGHSCORE_FILE.to_string(),
        crate::run_config::PlayMode::Campaign { .. } => {
            format!("herder_highscores_{}.txt", run_config.score_table_id())
        }
    }
}

fn highscore_row_node(rows: &[HighscoreDisplayRow], index: usize) -> impl Bundle {
    let mut node = Node::default();
    let (label, color) = highscore_row_label(rows, index);
    if label.is_empty() {
        node.display = Display::None;
    }

    (
        FinishHighscoreRow(index),
        node,
        Text::new(label),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(color),
    )
}

fn apply_highscore_rows(rows: &[HighscoreDisplayRow], query: &mut FinishHighscoreRowsQuery) {
    for (row, mut text, mut color, mut node) in query {
        let (label, text_color) = highscore_row_label(rows, row.0);
        text.0 = label;
        color.0 = text_color;
        node.display = if text.0.is_empty() {
            Display::None
        } else {
            Display::Flex
        };
    }
}

fn highscore_row_label(rows: &[HighscoreDisplayRow], index: usize) -> (String, Color) {
    if rows.is_empty() && index == 0 {
        return ("No scores yet".to_string(), FINISH_MUTED_TEXT_COLOR);
    }

    let Some(row) = rows.get(index) else {
        return (String::new(), FINISH_MUTED_TEXT_COLOR);
    };

    let color = if row.highlighted {
        FINISH_HIGHLIGHT_TEXT_COLOR
    } else {
        FINISH_MUTED_TEXT_COLOR
    };

    (
        format!(
            "{}. {}  {} - {}",
            index + 1,
            row.name,
            row.score,
            format_time(row.seconds)
        ),
        color,
    )
}

fn display_name_input(name: &str) -> String {
    if name.is_empty() {
        "_".to_string()
    } else {
        format!("{name}_")
    }
}

fn normalized_player_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .filter(|character| is_name_character(*character))
        .collect::<String>();
    let trimmed = sanitized.trim();

    if trimmed.is_empty() {
        DEFAULT_PLAYER_NAME.to_string()
    } else {
        trimmed.to_string()
    }
}

fn sanitize_stored_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .filter(|character| is_name_character(*character))
        .take(MAX_PLAYER_NAME_CHARS)
        .collect::<String>();

    if sanitized.trim().is_empty() {
        DEFAULT_PLAYER_NAME.to_string()
    } else {
        sanitized
    }
}

fn is_name_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_' | '.')
}

fn format_time(seconds: f32) -> String {
    let total_seconds = seconds.round() as u32;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}
