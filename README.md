# Herder

A Bevy remake of an old SDL/C++ herding game from 2008.

Guide the shepherd and dog, draw routes for the dog, and push the sheep into the finish tile. The remake keeps the original spirit but uses Bevy-style systems, resources, states, and sprite atlases instead of a direct line-by-line port.

## Run

```sh
cargo run --release
```

Or use the Justfile:

```sh
just run
```

## Controls

- `WASD`: move the shepherd
- Mouse drag: draw a route for the dog
- Mouse wheel: zoom the camera
- `Esc`: pause or resume during play
- Finish screen:
  - type a name to save the highscore
  - `Enter`: save, then start a new game
  - `Esc`: save and return to menu

## Current Features

- Random map mode with configurable map size, sheep count, and terrain amounts
- Campaign mode with 20 deterministic generated levels
- Procedural tile map with finish placement constraints
- Generated autotile terrain for grass, flowers, water, and paths
- Terrain walkability and movement-speed rules
- Island-free walkable map generation
- Animated shepherd, dog, and sheep sprites from atlas sheets
- Dog A* routes that avoid water and prefer faster terrain
- Dog bark sounds
- Sheep flocking, fear propagation, bravery, wandering, and finish scoring
- Off-screen indicators for finish, dog, and sheep clusters
- Pause and finish overlays
- Local highscores with player names, including campaign level tables and a campaign grand table

## Project Layout

- `src/world`: map generation, tile data, camera setup
- `src/states`: game/menu/play state plugins
- `src/states/play`: gameplay systems and UI
- `src/states/play/sheep`: sheep AI, movement, animation, audio, spawning, finish logic
- `assets/textures`: converted sprite and tile assets
- `assets/sounds`: converted game audio

## Docs

The project docs are an mdBook in `docs/src`.

```sh
just docs
```

The generated Rust code overview is built with rustdoc:

```sh
just docs-code
```

Open it locally with:

```sh
just docs-code-open
```

If `mdbook` is not installed, run `cargo install mdbook`.

Random-map highscores are stored locally in `herder_highscores.txt`. Campaign levels use their own `herder_highscores_campaign_*.txt` files, plus `herder_highscores_campaign_grand.txt` for completed campaign attempts.
