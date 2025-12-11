# Praxeum Roadmap

An implementation plan for a Rust-based learning platform focused on Austrian economics. The roadmap favors UI-agnostic core APIs to support CLI and future Dioxus frontends (Android/iOS/Desktop/WebAssembly).

## Goals and Principles
- **Content-first**: Define exercises and progress data in a portable, versioned file format (TOML primary; JSON interop) to allow authoring without code changes.
- **Engine-centric**: Keep a minimal, testable core crate that performs loading, validation, sequencing, and scoring without hard-coding any UI concerns.
- **Future-ready**: APIs friendly to async/front-end use (no global state, minimal blocking I/O, deterministic evaluation) to work in native and WASM targets.
- **Telemetry-aware**: Capture timing, streaks, and accuracy metrics suitable for leaderboards and spaced-repetition-like features without mandating a specific backend.

## Milestones

### M1: Repository Bootstrap
- Initialize workspace with `praxeum_core` crate and CLI binary target.
- Configure Rust toolchain metadata (edition 2021), linting defaults, and CI stubs (fmt/clippy/test).
- Document contribution basics in `README.md` (how to build, run CLI, and add content).

### M2: Data Model & Formats
- Define core domain types: `Exercise`, `Answer`, `Evaluation`, `SessionMetrics` (timing/accuracy), and `ProgressSnapshot`.
- Establish canonical file schema in **TOML** (primary authoring format) with JSON compatibility for interoperability.
- Add schema validation helpers (e.g., semantic checks for indices/categories, duplicate IDs) with human-friendly errors.
- Provide sample content files under `examples/` and a schema reference doc.

### M3: Loading & Validation Layer
- Implement `ExerciseLoader` to ingest TOML/JSON from filesystem or in-memory strings, returning typed exercises.
- Add format inference by extension and deterministic error reporting (per-file, per-exercise context).
- Expose validation API so UIs can preflight content before starting a session.

### M4: Engine Core
- Build `ExerciseEngine` that sequences exercises, evaluates answers, and produces `Evaluation` results.
- Support exercise kinds: classification, multiple-choice (single/multi), and scenario choices with feedback.
- Ensure pure, deterministic evaluation (no hidden state, no global statics). Cursor/state should be instance-bound for multithreaded safety and WASM use.
- Provide iterator-style traversal plus random-access lookup by ID for future adaptive routing.

### M5: Scoring & Metrics
- Introduce scoring model that tracks:
  - Per-exercise correctness and partial credit.
  - Session-level metrics: accuracy, completion time, question pacing, streaks.
  - Derived points system (base points + speed/accuracy multipliers) with tunable parameters.
- Emit structured telemetry events that frontends can consume (e.g., `SessionEvent::Started`, `Answered`, `Completed`).
- Add serialization for metrics to allow persisting progress snapshots.

### M6: CLI Experience
- Provide `praxeum_cli` for loading exercises from a path and playing through them interactively.
- Add flags for format selection, shuffling, limiting by ID prefix, and verbosity of feedback.
- Display session summary with points, accuracy, and duration.
- Include example commands in README and integration tests for CLI parsing where feasible.

### M7: Testing & Quality
- Unit tests for loader validation, engine evaluation logic, and scoring math.
- Golden-file tests using example TOML/JSON fixtures.
- Add `cargo fmt` and `cargo clippy` hooks (CI or local scripts) and ensure WASM-compatibility where practical (avoid platform-specific syscalls in core).

### M8: Frontend Readiness Hooks
- Keep core APIs async-friendly: allow passing timers/clock providers so UIs can inject monotonic clocks (use traits, no global time sources).
- Provide feature flags for `serde` serialization of progress/metrics to support storage layers (local files, browser storage, mobile storage).
- Document how Dioxus frontends can embed the engine: state lifecycle, threading expectations, and data flow.

### Stretch: Adaptive & Content Ops
- Add tagging and difficulty metadata to exercises for filtering and adaptive selection.
- Introduce spaced-repetition-friendly scheduling hooks (next-review suggestions) without enforcing a specific algorithm.
- Provide content lints (e.g., duplicate IDs, unreachable questions, empty prompts) and author tooling scripts.

## Deliverables per Milestone
- **Code**: crate modules, binary, and tests implementing the milestone scope.
- **Docs**: README updates, inline rustdoc, and example content expansions.
- **Assets**: Updated example exercise files and schema reference notes.

## Risks and Mitigations
- **Format drift**: Mitigate with versioned schema keys and validation tests.
- **Global state hazards**: Avoid statics; keep engine state instance-scoped to prevent duplication issues across crates.
- **Platform differences**: Abstract clocks and I/O for WASM/mobile; keep core free of blocking I/O beyond loader entry points.

## Next Steps
1. Confirm roadmap scope and priorities with stakeholders.
2. Scaffold the Rust workspace and initial crate structure (Milestone M1).
3. Implement data models and sample content (M2), then iterate through loader and engine milestones.
