# Assets

The project uses converted assets from the older game.

## Textures

Texture assets live in `assets/textures`.

Important files:

- `character.png`: shepherd sprite sheet
- `dog.png`: dog sprite sheet
- `sheep.png`: sheep sprite sheet
- `backgrounds.png`: source tile atlas; the game still uses its grass and finish tiles
- `water_autotile.png`: generated 16-mask water atlas
- `flowers_autotile.png`: generated 16-mask flower atlas
- `waypoint.png`: route marker
- `main_header.png`: menu/header art
- `finish_background.png`: finish-related art

The character, dog, and sheep textures are used as sprite atlases. Animation systems select frames based on movement direction.

The Bevy app uses:

```rust
ImagePlugin::default_nearest()
```

This keeps pixel-art style assets crisp and avoids blurry filtering.

## Generated Terrain Atlases

Water and flower autotile atlases can be regenerated from `backgrounds.png`:

```sh
just tiles
```

The generated atlases use a 4-bit cardinal-neighbor mask. Atlas index equals
the mask value:

| Bit | Direction |
| --- | --- |
| `1` | North |
| `2` | East |
| `4` | South |
| `8` | West |

For example, a water tile connected north and west uses atlas index `9`.

At runtime, grass and finish tiles use `backgrounds.png`. Water and flower
tiles use the generated autotile atlases, so the original flower tile remains
source material only.

## Sounds

Sound assets live in `assets/sounds`.

Important files:

- `bark_1.ogg`
- `bark_2.ogg`
- `beh_1.ogg`
- `beh_2.ogg`
- `finish.ogg`

Audio files should stay in formats Bevy can decode reliably. The current converted `.ogg` files are used to avoid unsupported legacy formats.
