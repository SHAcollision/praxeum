# Praxeum

Praxeum is a Rust-based learning engine for Austrian economics exercises. The core crate is UI-agnostic so it can power a CLI today and Dioxus-based mobile/desktop frontends later.

## Layout
- `praxeum-core/`: Library crate with engine, loader, validation helpers, scoring, and session metrics.
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
4. Provide your own exercise file (TOML or JSON) with `--file` and optional flags:
   ```bash
   cargo run -p praxeum-cli -- \
     --file path/to/exercises.toml \
     --format toml|json|auto \
     --id-prefix intro_ --shuffle --limit 10
   ```
   Use `--validate-only` to run schema checks without entering the interactive prompt. CLI preflight mirrors the library-level `ExerciseLoader::preflight_path`/`preflight_str` helpers, which return contextual issues (exercise id, field, message) without panicking on bad content.
5. Preflight content programmatically without running the CLI using `ExerciseLoader::preflight_path` or `ExerciseLoader::preflight_str`, then load with `ExerciseLoader::load_from_path` or `ExerciseLoader::load_from_str` once validation passes.

## Linting and tests
- Check formatting: `cargo fmt --all -- --check`
- Run clippy with warnings as errors: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Execute tests (including validation, engine, scoring, and CLI coverage): `cargo test --workspace --all-features`

## Content format
Exercises are stored as arrays under the `exercise` key in TOML (or a top-level JSON array / `{ "exercises": [...] }` wrapper). See `praxeum-core/examples/SCHEMA.md` for field-level rules and `praxeum-core/examples/exercises_basic.{toml,json}` for canonical examples covering classification, multiple-choice, and scenario exercises.

Validation helpers return structured issues that include the exercise ID, field, and a clear message (duplicate IDs, empty prompts, out-of-range indices, missing correct choices, etc.).

## Contributing
- Avoid global statics; keep engine state instance-scoped for multi-platform builds.
- Prefer deterministic, testable functions with clear error types.
- Open an issue before large changes to validate design direction against the roadmap.
