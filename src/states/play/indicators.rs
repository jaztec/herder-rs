use bevy::window::PrimaryWindow;
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::states::play::{dog::Dog, score::FinishArea, sheep::Sheep};
use crate::world::MapConfig;

const MAX_SHEEP_INDICATORS: usize = 3;
const SHEEP_CLUSTER_DISTANCE: f32 = 520.0;
const INDICATOR_WIDTH: f32 = 92.0;
const INDICATOR_HEIGHT: f32 = 38.0;
const INDICATOR_EDGE_MARGIN: f32 = 28.0;
const INDICATOR_ICON_SIZE: f32 = 26.0;
const ACTOR_ICON_WIDTH: u32 = 75;
const ACTOR_ICON_HEIGHT: u32 = 60;
const TILE_ICON_SIZE: u32 = 150;
const FRONT_FRAME_INDEX: usize = 0;
const FINISH_TILE_INDEX: usize = 3;
const FINISH_COLOR: Color = Color::srgba(0.1, 0.38, 0.16, 0.86);
const DOG_COLOR: Color = Color::srgba(0.18, 0.19, 0.24, 0.88);
const SHEEP_COLOR: Color = Color::srgba(0.42, 0.38, 0.18, 0.88);
const TEXT_COLOR: Color = Color::srgb(0.96, 0.96, 0.9);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(in crate::states::play) enum IndicatorKind {
    Finish,
    Dog,
    SheepCluster(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(in crate::states::play) struct IndicatorRoot {
    kind: IndicatorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub(in crate::states::play) struct IndicatorLabel {
    kind: IndicatorKind,
}

#[derive(Debug, Clone, Copy)]
struct IndicatorState {
    kind: IndicatorKind,
    visible: bool,
    position: Vec2,
    count: usize,
    distance_tiles: u32,
}

#[derive(Debug, Clone, Copy)]
struct CameraView {
    center: Vec2,
    half_world: Vec2,
    window_size: Vec2,
    projection_scale: f32,
}

#[derive(Debug, Clone, Copy)]
struct SheepCluster {
    position: Vec2,
    count: usize,
}

type CameraIndicatorQuery<'w> =
    Single<'w, (&'static GlobalTransform, &'static Projection), With<Camera2d>>;

#[derive(SystemParam)]
pub(in crate::states::play) struct IndicatorParams<'w, 's> {
    finish: Res<'w, FinishArea>,
    config: Res<'w, MapConfig>,
    window: Single<'w, &'static Window, With<PrimaryWindow>>,
    camera: CameraIndicatorQuery<'w>,
    dog: Query<'w, 's, &'static Transform, With<Dog>>,
    sheep: Query<'w, 's, &'static Transform, With<Sheep>>,
    roots: Query<
        'w,
        's,
        (
            &'static IndicatorRoot,
            &'static mut Node,
            &'static mut Visibility,
        ),
    >,
    labels: Query<'w, 's, (&'static IndicatorLabel, &'static mut Text)>,
}

pub fn setup_indicators(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let actor_atlas = texture_atlases.add(TextureAtlasLayout::from_grid(
        UVec2::new(ACTOR_ICON_WIDTH, ACTOR_ICON_HEIGHT),
        4,
        5,
        None,
        None,
    ));
    let tile_atlas = texture_atlases.add(TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_ICON_SIZE),
        3,
        3,
        None,
        None,
    ));

    let finish_icon = IndicatorIcon {
        image: asset_server.load("textures/backgrounds.png"),
        atlas: tile_atlas,
        index: FINISH_TILE_INDEX,
    };
    let dog_icon = IndicatorIcon {
        image: asset_server.load("textures/dog.png"),
        atlas: actor_atlas.clone(),
        index: FRONT_FRAME_INDEX,
    };
    let sheep_icon = IndicatorIcon {
        image: asset_server.load("textures/sheep.png"),
        atlas: actor_atlas,
        index: FRONT_FRAME_INDEX,
    };

    spawn_indicator(
        &mut commands,
        IndicatorKind::Finish,
        FINISH_COLOR,
        &finish_icon,
    );
    spawn_indicator(&mut commands, IndicatorKind::Dog, DOG_COLOR, &dog_icon);

    for index in 0..MAX_SHEEP_INDICATORS {
        spawn_indicator(
            &mut commands,
            IndicatorKind::SheepCluster(index),
            SHEEP_COLOR,
            &sheep_icon,
        );
    }
}

pub fn update_indicators(params: IndicatorParams) {
    let IndicatorParams {
        finish,
        config,
        window,
        camera,
        dog,
        sheep,
        mut roots,
        mut labels,
    } = params;

    let view = camera_view(&window, &camera);
    let mut states = vec![
        target_state(
            IndicatorKind::Finish,
            finish.center,
            &view,
            config.tile_size,
            1,
        ),
        dog.iter()
            .next()
            .map(|dog| {
                target_state(
                    IndicatorKind::Dog,
                    dog.translation.truncate(),
                    &view,
                    config.tile_size,
                    1,
                )
            })
            .unwrap_or_else(|| hidden_state(IndicatorKind::Dog)),
    ];

    let sheep_clusters = cluster_offscreen_sheep(&sheep, &view);
    for index in 0..MAX_SHEEP_INDICATORS {
        let kind = IndicatorKind::SheepCluster(index);
        let state = sheep_clusters
            .get(index)
            .map(|cluster| {
                target_state(
                    kind,
                    cluster.position,
                    &view,
                    config.tile_size,
                    cluster.count,
                )
            })
            .unwrap_or_else(|| hidden_state(kind));

        states.push(state);
    }

    for (indicator, mut node, mut visibility) in &mut roots {
        let Some(state) = states.iter().find(|state| state.kind == indicator.kind) else {
            continue;
        };

        *visibility = if state.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        node.left = Val::Px(state.position.x - INDICATOR_WIDTH / 2.0);
        node.top = Val::Px(state.position.y - INDICATOR_HEIGHT / 2.0);
    }

    for (indicator, mut text) in &mut labels {
        let Some(state) = states.iter().find(|state| state.kind == indicator.kind) else {
            continue;
        };

        text.0 = indicator_text(state);
    }
}

#[derive(Clone)]
struct IndicatorIcon {
    image: Handle<Image>,
    atlas: Handle<TextureAtlasLayout>,
    index: usize,
}

fn spawn_indicator(
    commands: &mut Commands,
    kind: IndicatorKind,
    color: Color,
    icon: &IndicatorIcon,
) {
    let mut atlas = TextureAtlas::from(icon.atlas.clone());
    atlas.index = icon.index;

    commands.spawn((
        IndicatorRoot { kind },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(INDICATOR_WIDTH),
            height: Val::Px(INDICATOR_HEIGHT),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(color),
        Visibility::Hidden,
        children![
            (
                ImageNode::from_atlas_image(icon.image.clone(), atlas),
                Node {
                    width: Val::Px(INDICATOR_ICON_SIZE),
                    height: Val::Px(INDICATOR_ICON_SIZE),
                    margin: UiRect::right(Val::Px(6.0)),
                    ..default()
                },
            ),
            (
                IndicatorLabel { kind },
                Text::new(""),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            )
        ],
    ));
}

fn target_state(
    kind: IndicatorKind,
    target: Vec2,
    view: &CameraView,
    tile_size: f32,
    count: usize,
) -> IndicatorState {
    let distance_tiles = (target.distance(view.center) / tile_size).round() as u32;

    IndicatorState {
        kind,
        visible: !is_world_position_visible(target, view),
        position: edge_position(target, view),
        count,
        distance_tiles,
    }
}

fn hidden_state(kind: IndicatorKind) -> IndicatorState {
    IndicatorState {
        kind,
        visible: false,
        position: Vec2::ZERO,
        count: 0,
        distance_tiles: 0,
    }
}

fn camera_view(window: &Window, camera: &(&GlobalTransform, &Projection)) -> CameraView {
    let projection_scale = match camera.1 {
        Projection::Orthographic(projection) => projection.scale,
        _ => 1.0,
    };

    let window_size = Vec2::new(window.width(), window.height());
    CameraView {
        center: camera.0.translation().truncate(),
        half_world: window_size * projection_scale / 2.0,
        window_size,
        projection_scale,
    }
}

fn is_world_position_visible(position: Vec2, view: &CameraView) -> bool {
    let offset = position - view.center;
    offset.x.abs() <= view.half_world.x && offset.y.abs() <= view.half_world.y
}

fn edge_position(target: Vec2, view: &CameraView) -> Vec2 {
    let world_offset = target - view.center;
    let mut screen_direction =
        Vec2::new(world_offset.x, -world_offset.y) / view.projection_scale.max(0.01);

    if screen_direction.length_squared() <= f32::EPSILON {
        screen_direction = Vec2::X;
    }

    let half_screen = view.window_size / 2.0;
    let bounds = Vec2::new(
        half_screen.x - INDICATOR_WIDTH / 2.0 - INDICATOR_EDGE_MARGIN,
        half_screen.y - INDICATOR_HEIGHT / 2.0 - INDICATOR_EDGE_MARGIN,
    );
    let direction = screen_direction.normalize();
    let x_scale = if direction.x.abs() > f32::EPSILON {
        bounds.x / direction.x.abs()
    } else {
        f32::INFINITY
    };
    let y_scale = if direction.y.abs() > f32::EPSILON {
        bounds.y / direction.y.abs()
    } else {
        f32::INFINITY
    };

    half_screen + direction * x_scale.min(y_scale)
}

fn cluster_offscreen_sheep(
    sheep: &Query<&Transform, With<Sheep>>,
    view: &CameraView,
) -> Vec<SheepCluster> {
    let mut clusters = Vec::<SheepCluster>::new();

    for transform in sheep {
        let position = transform.translation.truncate();
        if is_world_position_visible(position, view) {
            continue;
        }

        if let Some(cluster) = clusters
            .iter_mut()
            .find(|cluster| cluster.position.distance(position) <= SHEEP_CLUSTER_DISTANCE)
        {
            let count = cluster.count as f32;
            cluster.position = (cluster.position * count + position) / (count + 1.0);
            cluster.count += 1;
        } else {
            clusters.push(SheepCluster { position, count: 1 });
        }
    }

    clusters.sort_by(|left, right| right.count.cmp(&left.count));
    clusters
}

fn indicator_text(state: &IndicatorState) -> String {
    if state.count > 1 {
        format!("x{} {}t", state.count, state.distance_tiles)
    } else {
        format!("{}t", state.distance_tiles)
    }
}
