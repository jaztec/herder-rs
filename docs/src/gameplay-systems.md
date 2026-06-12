# Gameplay Systems

## Play Modes

The game currently has two play modes:

- Random map: configurable map size, sheep count, water amount, flower amount, and path amount.
- Campaign: 20 deterministic generated levels played in order.

Random maps use the selected settings with a fresh random seed each run.
Campaign levels use fixed seeds so every level can have its own repeatable
highscore table.

## Map Generation

The world is a tile grid generated at the start of each run.

Current constraints:

- grass is the default terrain
- flower and water tiles are generated as blobs so adjacent tiles can merge visually
- path tiles are generated as wandering cardinal trails and avoid overwriting water
- disconnected walkable islands are removed after generation
- the finish tile is placed at least two tiles from any edge when the map size allows it
- on very small maps, the edge margin is reduced automatically
- the finish tile position is stored as a resource

Water, flowers, and paths are rendered with generated 4-bit autotile atlases.
Neighboring tiles of the same type choose matching atlas frames, which lets
flower patches cross tile borders, water groups read as ponds or lakes, and
paths connect as straight segments, turns, T-junctions, or crossings.

Terrain also has gameplay semantics:

| Tile | Walkable | Movement multiplier |
| --- | --- | --- |
| Path | Yes | `1.25` |
| Grass | Yes | `1.0` |
| Flowers | Yes | `0.72` |
| Finish | Yes | `1.0` |
| Water | No | `0.0` |

The movement systems and pathfinder use the same tile metadata: water blocks
actor rectangles, while walkable tiles expose movement speed
multipliers.

Terrain multipliers apply to shepherd, dog, and sheep movement. The dog
pathfinder also converts those multipliers into movement costs, so routes prefer
paths, treat grass and finish tiles as normal, avoid flowers when possible, and
reject water.

## Shepherd

The shepherd is controlled with `WASD`. The camera follows the shepherd and supports mouse-wheel zoom. Movement is clamped to the world bounds.

The current base speed is `190` world units per second before terrain
modifiers. This intentionally makes the shepherd slower than panicked sheep and
much slower than the dog.

## Dog

The dog starts near the shepherd. Dragging with the left mouse button creates a route made of waypoint entities. The dog follows those waypoints and can play bark sounds when commanded.

Dog routes use A* pathfinding over a 3x3 subgrid inside every terrain tile. The
cursor points are treated as requested targets; the actual route is expanded
into walkable sub-tile centers, simplified to corner waypoints, and will avoid
water. The pathfinder still uses the parent terrain tile for movement cost, so
paths are favored over grass and flowers are less attractive.

The current base dog speed is `430` world units per second before terrain
modifiers, making the dog the fastest actor by a clear margin.

The herd cannot spawn on the same tile as the shepherd or the dog.

## Sheep

Sheep spawn as a herd cluster. The cluster center is selected away from the finish:

- target minimum distance: seven tiles from the finish
- fallback: the furthest available valid distance when the map is too small

Each sheep has a random bravery value. Bravery affects panic speed and flocking behavior.

Sheep movement speeds are behavior dependent before terrain modifiers:

| Behavior | Base speed |
| --- | --- |
| Wander | `38` |
| Follow braver sheep | up to roughly `55`, scaled by courage gap |
| Panic | `190 + bravery * 10`, scaled by fear strength |

Important behavior:

- sheep flee from nearby shepherd or dog threats
- panic can propagate through the herd
- weaker sheep tend to follow braver sheep while idle
- sheep separate from each other to avoid collapsing into one point
- sheep wander when idle

## Finish and Score

When a sheep overlaps the finish area:

- the sheep entity is despawned
- a finish sound is played
- the score is updated
- the HUD refreshes

When all sheep are herded, the game enters the finish state. The finish screen shows score, time, name input, and highscores.

## Highscores

Random-map highscores are stored locally in `herder_highscores.txt`.
Campaign levels are stored in separate `herder_highscores_campaign_*.txt`
files. Completing the final campaign level also writes the aggregate campaign
score and time to `herder_highscores_campaign_grand.txt`.

The format is simple CSV-like text:

```text
name,score,seconds
```

Older two-field entries are still read and displayed with the default name `Player`.
