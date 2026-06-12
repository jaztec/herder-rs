# Development Notes

## Style

The remake favors Bevy-native patterns:

- plugins for feature areas
- resources for shared game state
- components for entity data
- systems for behavior
- state gates for pause, finish, and menu transitions

The code intentionally avoids a direct C++ class-by-class port from the old project.

## Module Layout

The project uses the newer Rust module style:

- `src/states/play/sheep.rs` declares the sheep module
- implementation files live in `src/states/play/sheep/`

This avoids the older `mod.rs` layout.

## Cleanup

Play entities are cleaned up when leaving `GameState::Play`. This includes map tiles, actors, sheep, dog waypoints, indicators, score UI, and pause UI.

## Local Runtime Files

`herder_highscores.txt` is a local runtime file. It should not be treated as source data.

## Generated Code Docs

Use rustdoc for the generated code overview:

```sh
just docs-code
```

The docs include private items so internal Bevy systems, components, resources, and helper functions are visible.

## Terrain Semantics

Tile rendering and tile gameplay rules are deliberately separate. The renderer
chooses sprite-atlas frames from neighboring tiles, while `Tile` and `TileMap`
provide walkability and movement-speed information. This keeps movement and
pathfinding focused on map costs instead of sprite details.

The dog route system now uses the shared world pathfinder. It asks for a path
between route targets and receives world-space corner waypoints, so the dog
code does not need to know the internals of A*. The pathfinder navigates on a
3x3 subgrid per tile to avoid route markers snapping too aggressively to tile
centers.

The shared speed model is deliberately simple: actor systems start with a base
movement speed and multiply it by `Tile::movement_speed_multiplier`. The same
tile multiplier is inverted into a pathfinding cost, keeping movement tuning
and route selection aligned.
