# Testing strategy

Development follows red, green, refactor. Every behaviour change starts with a failing test, then the smallest implementation, then cleanup.

## Required layers

- Rust unit tests beside domain code.
- `proptest` for sanitisation, protocol and state-machine invariants.
- Integration tests for SQLite and a test-only HTTP fixture server.
- Vitest and React Testing Library for UI logic and keyboard flows.
- axe assertions for the main window, dialogs and first-run flow.
- Tauri/WebdriverIO smoke tests on Windows x64 and Linux x64.
- Criterion benchmarks for protocol parsing, sanitisation and progress aggregation.

Unit tests must be deterministic. Inject clocks, random sources and filesystem boundaries. Never use a shipped HTTP server, polling loop or remote service in tests or production.
