# Week 7-8 Updated Planning Notes

## Summary

This note now serves as the short planning overview for Week 7-8. The canonical implementation-ready plan for the `ext-render` + `ext-report` + Tauri desktop continuation lives here:

- `docs/superpowers/plans/2026-04-06-tauri-render-report-integration.md`

That plan supersedes the older drift-first summary below whenever the two disagree.

## Locked Baseline

- canonical check numbering remains `1-8`, with torsional irregularity as Check 5
- displacement wind remains its own output section, but not its own canonical check number
- `ext-render` is the single chart-definition layer
- `ext-report` consumes SVG strings from `ext-render`, not file-writing `ext-render::svg::*` helpers
- `ext-api` remains the only orchestration layer; `ext-tauri` is a desktop adapter, not a second workflow layer
- persisted `calc_output.json` is the canonical contract shared by CLI and Tauri; desktop memory cache is optional only
- the report target remains tabloid landscape (`17 x 11 in`)
- the desktop app must implement explicit Tauri commands for chart retrieval and report generation
- release builds must wire SSR features and local ECharts assets intentionally; they are not implied by the crate docs alone

## Updated Build Sequence

1. stabilize `CalcOutput` snapshots and remaining calc checks
2. expand `ext-render` to the final in-memory HTML/SVG contract
3. expand `ext-report` to consume SVG maps and write tabloid report artifacts
4. wire `ext-tauri` commands, app state, and output directories
5. replace frontend report mocks with real desktop flows
6. add CLI/API wrappers only after the desktop contract is stable

## Scope Reminder

The detailed plan intentionally focuses on three coupled subsystems that now need a shared contract:

- `crates/ext-render`
- `crates/ext-report`
- `crates/ext-tauri` plus `apps/desktop`

`ext-calc` remains the upstream contract owner and should only change here when required to support stable render/report DTOs.
