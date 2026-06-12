run:
    cargo run --release

docs:
    mdbook serve

docs-build:
    mdbook build

docs-code:
    cargo doc --no-deps --document-private-items

docs-code-open:
    cargo doc --no-deps --document-private-items --open

docs-all:
    mdbook build
    cargo doc --no-deps --document-private-items

tiles:
    python3 tools/generate_tile_atlases.py
