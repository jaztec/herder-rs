# Gameplay Systems

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

| Tile | Walkable | Movement |
| --- | --- | --- |
| Path | Yes | Fast |
| Grass | Yes | Normal |
| Flowers | Yes | Slow |
| Finish | Yes | Normal |
| Water | No | Blocked |

The movement systems use the same tile metadata that future pathfinding should
use: water blocks actor rectangles, while walkable tiles expose movement speed
multipliers.

## Shepherd

The shepherd is controlled with `WASD`. The camera follows the shepherd and supports mouse-wheel zoom. Movement is clamped to the world bounds.

## Dog

The dog starts near the shepherd. Dragging with the left mouse button creates a route made of waypoint entities. The dog follows those waypoints and can play bark sounds when commanded.

The herd cannot spawn on the same tile as the shepherd or the dog.

## Sheep

Sheep spawn as a herd cluster. The cluster center is selected away from the finish:

- target minimum distance: seven tiles from the finish
- fallback: the furthest available valid distance when the map is too small

Each sheep has a random bravery value. Bravery affects panic speed and flocking behavior.

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

Highscores are stored locally in `herder_highscores.txt`.

The format is simple CSV-like text:

```text
name,score,seconds
```

Older two-field entries are still read and displayed with the default name `Player`.
