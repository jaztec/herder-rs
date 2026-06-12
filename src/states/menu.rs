//! Main menu state and UI systems.

use bevy::{color::palettes::css::CRIMSON, prelude::*};

use crate::{
    run_config::{PlayMode, RunConfig},
    states::{GameState, game_state::despawn_screen},
};

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const MAP_SIZE_OPTIONS: [(usize, usize, &str); 3] =
    [(18, 14, "Small"), (24, 18, "Medium"), (32, 24, "Large")];
const SHEEP_COUNT_OPTIONS: [usize; 4] = [15, 30, 45, 60];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
enum MenuState {
    #[default]
    Main,
    RandomSetup,
    Disabled,
}

#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnRandomSetupScreen;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    OpenRandomSetup,
    StartRandom,
    StartCampaign,
    CycleMapSize,
    CycleSheepCount,
    CycleTerrain(TerrainOption),
    BackToMain,
    Quit,
}

#[derive(Component)]
struct SelectedOption;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TerrainOption {
    Water,
    Flowers,
    Paths,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
enum RandomOptionText {
    MapSize,
    SheepCount,
    Water,
    Flowers,
    Paths,
}

type ButtonStyleQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        Option<&'static SelectedOption>,
    ),
    (Changed<Interaction>, With<Button>),
>;

type MenuActionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static MenuButtonAction),
    (Changed<Interaction>, With<Button>),
>;

fn button_system(mut interaction_query: ButtonStyleQuery) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

/// Register menu state systems.
pub fn menu_state_plugin(app: &mut App) {
    app.init_state::<MenuState>()
        .add_systems(OnEnter(GameState::Menu), main_menu_setup)
        .add_systems(OnEnter(MenuState::Main), main_setup)
        .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
        .add_systems(OnEnter(MenuState::RandomSetup), random_setup)
        .add_systems(
            OnExit(MenuState::RandomSetup),
            despawn_screen::<OnRandomSetupScreen>,
        )
        .add_systems(
            Update,
            (menu_action, button_system, update_random_option_labels)
                .run_if(in_state(GameState::Menu)),
        );
}

fn main_menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn main_setup(mut commands: Commands) {
    // Common style for all buttons on the screen
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnMainMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
            children![
                // Display the game name
                (
                    Text::new("Herder game"),
                    TextFont {
                        font_size: 67.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::OpenRandomSetup,
                    children![(
                        Text::new("Random Map"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::StartCampaign,
                    children![(
                        Text::new("Campaign"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                    children![(Text::new("Quit"), button_text_font, TextColor(TEXT_COLOR),),]
                ),
            ]
        )],
    ));
}

fn random_setup(mut commands: Commands, run_config: Res<RunConfig>) {
    let button_node = Node {
        width: Val::Px(340.0),
        height: Val::Px(52.0),
        margin: UiRect::axes(Val::Px(12.0), Val::Px(7.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = TextFont {
        font_size: 23.0,
        ..default()
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnRandomSetupScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(28.0)),
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
            children![
                (
                    Text::new("Random map"),
                    TextFont {
                        font_size: 54.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::bottom(Val::Px(22.0)),
                        ..default()
                    },
                ),
                random_option_button(
                    button_node.clone(),
                    button_text_font.clone(),
                    RandomOptionText::MapSize,
                    MenuButtonAction::CycleMapSize,
                    map_size_label(&run_config),
                ),
                random_option_button(
                    button_node.clone(),
                    button_text_font.clone(),
                    RandomOptionText::SheepCount,
                    MenuButtonAction::CycleSheepCount,
                    sheep_count_label(&run_config),
                ),
                random_option_button(
                    button_node.clone(),
                    button_text_font.clone(),
                    RandomOptionText::Water,
                    MenuButtonAction::CycleTerrain(TerrainOption::Water),
                    terrain_label("Water", run_config.terrain.water),
                ),
                random_option_button(
                    button_node.clone(),
                    button_text_font.clone(),
                    RandomOptionText::Flowers,
                    MenuButtonAction::CycleTerrain(TerrainOption::Flowers),
                    terrain_label("Flowers", run_config.terrain.flowers),
                ),
                random_option_button(
                    button_node.clone(),
                    button_text_font.clone(),
                    RandomOptionText::Paths,
                    MenuButtonAction::CycleTerrain(TerrainOption::Paths),
                    terrain_label("Paths", run_config.terrain.paths),
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::StartRandom,
                    children![(
                        Text::new("Start"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    )]
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::BackToMain,
                    children![(Text::new("Back"), button_text_font, TextColor(TEXT_COLOR),)]
                ),
            ]
        )],
    ));
}

fn random_option_button(
    node: Node,
    font: TextFont,
    text_kind: RandomOptionText,
    action: MenuButtonAction,
    label: String,
) -> impl Bundle {
    (
        Button,
        node,
        BackgroundColor(NORMAL_BUTTON),
        action,
        children![(Text::new(label), text_kind, font, TextColor(TEXT_COLOR),)],
    )
}

fn menu_action(
    interaction_query: MenuActionQuery,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut run_config: ResMut<RunConfig>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                MenuButtonAction::OpenRandomSetup => {
                    run_config.mode = PlayMode::Random;
                    menu_state.set(MenuState::RandomSetup);
                }
                MenuButtonAction::StartRandom => {
                    run_config.mode = PlayMode::Random;
                    run_config.seed = None;
                    game_state.set(GameState::Play);
                    menu_state.set(MenuState::Disabled)
                }
                MenuButtonAction::StartCampaign => {
                    *run_config = RunConfig::from_campaign_level(0);
                    game_state.set(GameState::Play);
                    menu_state.set(MenuState::Disabled)
                }
                MenuButtonAction::CycleMapSize => cycle_map_size(&mut run_config),
                MenuButtonAction::CycleSheepCount => cycle_sheep_count(&mut run_config),
                MenuButtonAction::CycleTerrain(option) => cycle_terrain(&mut run_config, *option),
                MenuButtonAction::BackToMain => menu_state.set(MenuState::Main),
            }
        }
    }
}

fn cycle_map_size(run_config: &mut RunConfig) {
    let current_index = MAP_SIZE_OPTIONS
        .iter()
        .position(|(width, height, _)| {
            run_config.map.width == *width && run_config.map.height == *height
        })
        .unwrap_or(1);
    let (width, height, _) = MAP_SIZE_OPTIONS[(current_index + 1) % MAP_SIZE_OPTIONS.len()];

    run_config.map.width = width;
    run_config.map.height = height;
}

fn cycle_sheep_count(run_config: &mut RunConfig) {
    let current_index = SHEEP_COUNT_OPTIONS
        .iter()
        .position(|count| run_config.sheep_count == *count)
        .unwrap_or(1);
    run_config.sheep_count = SHEEP_COUNT_OPTIONS[(current_index + 1) % SHEEP_COUNT_OPTIONS.len()];
}

fn cycle_terrain(run_config: &mut RunConfig, option: TerrainOption) {
    match option {
        TerrainOption::Water => run_config.terrain.water = run_config.terrain.water.next(),
        TerrainOption::Flowers => run_config.terrain.flowers = run_config.terrain.flowers.next(),
        TerrainOption::Paths => run_config.terrain.paths = run_config.terrain.paths.next(),
    }
}

fn update_random_option_labels(
    run_config: Res<RunConfig>,
    mut labels: Query<(&RandomOptionText, &mut Text)>,
) {
    if !run_config.is_changed() {
        return;
    }

    for (kind, mut text) in &mut labels {
        text.0 = match kind {
            RandomOptionText::MapSize => map_size_label(&run_config),
            RandomOptionText::SheepCount => sheep_count_label(&run_config),
            RandomOptionText::Water => terrain_label("Water", run_config.terrain.water),
            RandomOptionText::Flowers => terrain_label("Flowers", run_config.terrain.flowers),
            RandomOptionText::Paths => terrain_label("Paths", run_config.terrain.paths),
        };
    }
}

fn map_size_label(run_config: &RunConfig) -> String {
    let preset_name = MAP_SIZE_OPTIONS
        .iter()
        .find(|(width, height, _)| {
            run_config.map.width == *width && run_config.map.height == *height
        })
        .map_or("Custom", |(_, _, label)| *label);

    format!(
        "Map: {preset_name} ({} x {})",
        run_config.map.width, run_config.map.height
    )
}

fn sheep_count_label(run_config: &RunConfig) -> String {
    format!("Sheep: {}", run_config.sheep_count)
}

fn terrain_label(name: &str, amount: crate::run_config::TerrainAmount) -> String {
    format!("{name}: {}", amount.label())
}
