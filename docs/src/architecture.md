# Architecture

The code follows Bevy's plugin and state model.

## Entry Point

`src/main.rs` creates the Bevy app, configures nearest-neighbor image sampling, registers `HerderGame`, and initializes the top-level game state.

## Root Plugin

`src/plugin.rs` wires together:

- camera setup
- menu plugin
- play plugin
- game-over plugin

## States

The top-level state lives in `src/states/game_state.rs`.

Important states:

- `Menu`: main menu
- `Play`: active game session
- `RestartPlay`: transition helper for clean restarts
- `GameOver`: currently present as a separate state plugin

The play state has its own nested state in `src/states/play/plugin.rs`:

- `Disabled`
- `Playing`
- `Paused`
- `Finished`

Gameplay systems run only while `PlayState::Playing` is active. Pause and finish overlays are separate state-driven UI screens.

## World

`src/world` owns map-related code:

- `tile.rs`: map config, tile map, grid positions, world bounds
- `builder.rs`: procedural map generation and tile entity spawning
- `camera.rs`: 2D camera setup

The map is generated from `MapConfig`. The finish tile is stored as a `FinishTilePosition` resource so scoring, indicators, and spawning can use the same source of truth.

## Play Systems

`src/states/play` owns gameplay:

- `shepherd.rs`: shepherd movement, animation, camera follow, zoom
- `dog.rs`: dog spawning, waypoint routing, movement, barking, animation
- `sheep/`: sheep spawning, AI, movement, animation, sounds, finish consumption
- `score.rs`: score resource, finish area, HUD
- `finish.rs`: run timer, finish overlay, highscores
- `indicators.rs`: off-screen indicators
- `plugin.rs`: play-state schedule wiring and cleanup
