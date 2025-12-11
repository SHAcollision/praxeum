# Praxeum

Praxeum is a Rust-based learning engine for Austrian economics exercises. The core crate is UI-agnostic so it can power a CLI today and Dioxus-based mobile/desktop frontends later.

## Layout
- `praxeum-core/`: Library crate with engine, loader, validation helpers, and session metrics.
- `praxeum-core/examples/`: Starter TOML/JSON content and the `SCHEMA.md` format reference.
- `praxeum-cli/`: Binary crate that consumes `praxeum-core`.
- `roadmap.md`: High-level milestones and priorities.

## Getting started
1. Install the Rust toolchain (stable) with `rustup`. The repo includes `rust-toolchain.toml` to pin components.
2. From the repository root, build the workspace:
   ```bash
   cargo build
   ```
3. Run the interactive CLI against the bundled example set for a streak-based score chase (arrow keys/space/enter navigation and a clean single-screen flow between positions):
   ```bash
   cargo run -p praxeum-cli
   ```
   The CLI shows combo streaks, speed bonuses, arrow-key selection menus, and a session summary so you can sprint through the positions.
4. Provide your own exercise file (TOML or JSON) with `--file`:
   ```bash
   cargo run -p praxeum-cli -- --file path/to/exercises.toml
   ```
5. Preflight content programmatically without running the CLI using `ExerciseLoader::validate_path` or `ExerciseLoader::validate_str`.

## Linting and tests
- Check formatting: `cargo fmt --all -- --check`
- Run clippy with warnings as errors: `cargo clippy --all-targets -- -D warnings`
- Execute tests (including validation and serialization coverage): `cargo test`

## Content format
Exercises are stored as arrays under the `exercise` key in TOML (or a top-level JSON array / `{ "exercises": [...] }` wrapper). See `praxeum-core/examples/SCHEMA.md` for field-level rules and `praxeum-core/examples/exercises_basic.{toml,json}` for canonical examples covering classification, multiple-choice, and scenario exercises.

Validation helpers return structured issues that include the exercise ID, field, and a clear message (duplicate IDs, empty prompts, out-of-range indices, missing correct choices, etc.).

## Contributing
- Avoid global statics; keep engine state instance-scoped for multi-platform builds.
- Prefer deterministic, testable functions with clear error types.
- Open an issue before large changes to validate design direction against the roadmap.
