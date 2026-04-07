# Repository Guidelines

## Project Structure & Module Organization
This repo is a mixed Rust and TypeScript workspace. Core Rust crates live in `crates/`: `ext-core` holds domain logic, `ext-api` exposes workflows, `ext` is the CLI, `ext-tauri` is the desktop shell, and supporting crates such as `ext-db`, `ext-calc`, `ext-report`, `ext-error`, `ext-render`, and `ext-agent*` provide storage, calculation, reporting, error handling, rendering, and agent features. The desktop frontend lives in `apps/desktop/src`, and shared TypeScript contracts live in `packages/shared/src`.

Tests are mostly crate-local: Rust unit tests sit beside source files, and integration tests live under `crates/*/tests`. Large test fixtures are stored in paths like `crates/ext-calc/tests/fixtures`.

## Build, Test, and Development Commands
- `pnpm dev`: run the Tauri desktop app in development mode.
- `pnpm build`: build the desktop app bundle through Tauri.
- `pnpm --filter desktop build`: build only the Vite/React frontend.
- `pnpm gen-types`: regenerate shared TS types from Rust test/type generation flows.
- `cargo test --workspace`: run the standard Rust test suite.
- `cargo test -p ext`: run CLI tests, including snapshot coverage.
- `cargo fmt --check --all`: verify Rust formatting.
- `cargo clippy --workspace --all-targets`: enforce workspace lint rules.

## Coding Style & Naming Conventions
Follow `rustfmt` defaults for Rust and keep `clippy` warning-free; workspace lints treat broad Clippy findings as errors. Use `snake_case` for Rust modules/functions and `PascalCase` for Rust types.

Frontend code is TypeScript + React with 4-space indentation, `PascalCase` component files such as `WorkspacePanel.tsx`, and `camelCase` hooks/store helpers such as `projectStore.ts`. Keep shared type definitions in `packages/shared/src/types` aligned with Rust sources.

## Testing Guidelines
Prefer focused crate tests while developing, then run `cargo test --workspace` before submitting. Name new integration tests by behavior, for example `vcs_cycle.rs` or `foundation_cli.rs`. Snapshot-based CLI tests live under `crates/ext/tests`.

Live ETABS coverage is opt-in only: `crates/ext-api/tests/etabs_live.rs` requires a real ETABS install plus `EXT_ETABS_*` environment variables from the workspace root `.env`.

## Commit & Pull Request Guidelines
Recent history uses short, lower-case subjects such as `update week 7 8` and `fix some comment of week 5 6`. Keep commits brief, imperative, and scoped to one change. For pull requests, include:
- a short summary of affected crates/apps
- linked issue or spec when applicable
- exact test commands run
- screenshots or screen recordings for `apps/desktop` UI changes
