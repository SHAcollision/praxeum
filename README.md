# Praxeum

Praxeum is a Rust-based learning engine for Austrian economics exercises. The core crate is UI-agnostic so it can power a CLI today and Dioxus-based mobile/desktop frontends later.

## Layout
- `praxeum-core/`: Library crate with engine and loader.
- `praxeum-core/examples/`: Starter TOML content used by the CLI demo.
- `praxeum-cli/`: Binary crate that consumes `praxeum-core`.
- `roadmap.md`: High-level milestones and priorities.

## Getting started
1. Install the Rust toolchain (stable) with `rustup`. The repo includes `rust-toolchain.toml` to pin components.
2. From the repository root, build the workspace:
   ```bash
   cargo build
   ```
3. Run the interactive CLI against the bundled example set for a streak-based score chase:
   ```bash
   cargo run -p praxeum-cli
   ```
   The CLI shows combo streaks, speed bonuses, and a session summary so you can sprint through the positions.
4. Provide your own exercise file (TOML or JSON) with `--file`:
   ```bash
   cargo run -p praxeum-cli -- --file path/to/exercises.toml
   ```

## Linting and tests
- Check formatting: `cargo fmt -- --check`
- Run clippy with warnings as errors: `cargo clippy --all-targets -- -D warnings`
- Execute tests: `cargo test`

## Content format
Exercises are stored as arrays under the `exercise` key in TOML (or a top-level JSON array). See `praxeum-core/examples/exercises_basic.toml` for a complete reference of the three exercise kinds (classification, multiple-choice, scenario) along with fast-paced Austrian econ drills (action vs. event, means–ends matching, opportunity cost snaps, exchange sequences, socialist calculation, intervention cascades).

## Contributing
- Avoid global statics; keep engine state instance-scoped for multi-platform builds.
- Prefer deterministic, testable functions with clear error types.
- Open an issue before large changes to validate design direction against the roadmap.
