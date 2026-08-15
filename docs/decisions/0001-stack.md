# ADR 0001: Rust core, CLI first, Tauri later

Status: Accepted for initial development.

## Decision

Use Rust for protocol/device/effect/profile logic and CLI. Add Tauri 2 with a TypeScript frontend after the protocol and CLI paths are stable.

## Reasons

- Binary USB protocols benefit from strong types and fixed-size data structures.
- The same core can support Linux and optional Windows builds.
- Protocol logic remains independent from UI.
- A CLI gives a narrow surface for first hardware validation.
- Tauri allows a modern web-style UI without making the protocol backend depend on frontend code.

## Consequences

- No desktop frontend dependencies during initial protocol work.
- All UI operations later call typed Rust commands.
- Raw packet construction stays in protocol crates.
