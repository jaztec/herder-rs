# Assets

The project uses converted assets from the older game.

## Textures

Texture assets live in `assets/textures`.

Important files:

- `character.png`: shepherd sprite sheet
- `dog.png`: dog sprite sheet
- `sheep.png`: sheep sprite sheet
- `backgrounds.png`: tile atlas
- `waypoint.png`: route marker
- `main_header.png`: menu/header art
- `finish_background.png`: finish-related art

The character, dog, and sheep textures are used as sprite atlases. Animation systems select frames based on movement direction.

The Bevy app uses:

```rust
ImagePlugin::default_nearest()
```

This keeps pixel-art style assets crisp and avoids blurry filtering.

## Sounds

Sound assets live in `assets/sounds`.

Important files:

- `bark_1.ogg`
- `bark_2.ogg`
- `beh_1.ogg`
- `beh_2.ogg`
- `finish.ogg`

Audio files should stay in formats Bevy can decode reliably. The current converted `.ogg` files are used to avoid unsupported legacy formats.
