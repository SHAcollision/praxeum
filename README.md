# Praxeum

Praxeum is a Rust-based learning engine for Austrian economics exercises. The core crate is UI-agnostic so it can power a CLI today and Dioxus-based mobile/desktop frontends later.

## Layout
- `praxeum_core/`: Library crate with engine, loader, and CLI binary `praxeum_cli`.
- `praxeum_core/examples/`: Starter TOML content used by the CLI demo.
- `roadmap.md`: High-level milestones and priorities.

## Getting started
1. Install the Rust toolchain (stable) with `rustup`. The repo includes `rust-toolchain.toml` to pin components.
2. From the repository root, build the workspace:
   ```bash
   cargo build
   ```
3. Run the interactive CLI against the bundled example set:
   ```bash
   cargo run -p praxeum_core --bin praxeum_cli
   ```
4. Provide your own exercise file (TOML or JSON) with `--file`:
   ```bash
   cargo run -p praxeum_core --bin praxeum_cli -- --file path/to/exercises.toml
   ```

## Linting and tests
- Check formatting: `cargo fmt -- --check`
- Run clippy with warnings as errors: `cargo clippy --all-targets -- -D warnings`
- Execute tests: `cargo test`

## Content format
Exercises are stored as arrays under the `exercise` key in TOML (or a top-level JSON array). See `praxeum_core/examples/exercises_basic.toml` for a complete reference of the three exercise kinds (classification, multiple-choice, scenario).

## Contributing
- Avoid global statics; keep engine state instance-scoped for multi-platform builds.
- Prefer deterministic, testable functions with clear error types.
- Open an issue before large changes to validate design direction against the roadmap.
