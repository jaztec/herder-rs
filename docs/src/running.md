# Running the Game

## Requirements

- Rust stable with edition 2024 support
- Native dependencies required by Bevy on your platform

## Run

```sh
cargo run --release
```

The repository also includes a Justfile:

```sh
just run
```

## Checks

Before committing gameplay changes, run:

```sh
cargo fmt
cargo check
cargo clippy
```

## Documentation

The documentation is built with mdBook.

```sh
mdbook serve
```

If mdBook is missing:

```sh
cargo install mdbook
```
