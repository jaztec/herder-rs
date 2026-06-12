# Roadmap

Recently completed terrain and route work:

- water, flowers, and paths have generated autotile atlases
- water blocks actors and dog routes
- paths, grass, and flowers apply movement-speed modifiers
- map generation removes disconnected walkable islands
- dog routes use A* over a 3x3 subgrid to avoid water and use terrain costs

Likely next areas:

- dedicated obstacle tiles or objects beyond water
- better menu presentation using the converted header art
- more robust dog route editing, preview, and cancellation
- stronger visual feedback when sheep are scared or scored
- configurable map size and sheep count
- highscore reset or profile management
- tests for deterministic map and spawn constraints

The main design direction is to keep the code modular enough for larger maps and more varied level rules.
