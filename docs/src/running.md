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

Or:

```sh
just docs
```

If mdBook is missing:

```sh
cargo install mdbook
```

## Rust Code Documentation

Generate a local docs.rs-style code overview with:

```sh
just docs-code
```

Open it in a browser with:

```sh
just docs-code-open
```

The underlying command is:

```sh
cargo doc --no-deps --document-private-items
```

`--document-private-items` is intentional because Herder is currently a binary crate and most Bevy systems are internal to the crate.
