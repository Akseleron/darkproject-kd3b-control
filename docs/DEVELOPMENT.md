# Development Guide

## Primary platform

CachyOS / Arch Linux, KDE Plasma, Wayland.

User-facing terminal instructions must use `fish` syntax. If a command requires Bash semantics, invoke Bash explicitly.

## Initial stack

- Rust workspace for protocol/device/CLI.
- Tauri 2 desktop application later, after protocol milestones.
- TypeScript frontend later.

Do not introduce frontend dependencies during the CLI protocol phase.

## Workspace policy

Every commit should leave the offline workspace buildable when practical.

Expected verification:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Hardware tests are never part of the default test suite.

## Protocol coding rules

- Pure encoder/decoder functions accept typed inputs and return typed packets/errors.
- Use fixed-size arrays when the packet size is known.
- Give every byte region a named constant or documented structure.
- Store key layout/mapping data in one authoritative module, not duplicated across CLI and UI.
- Golden tests should compare the complete packet bytes.
- Do not silently clamp values unless the documented protocol specifies it; return validation errors.

## Transport rules

Define a narrow transport trait before adding the real HID backend.

The mock backend must be capable of recording outbound operations for tests.

Real transport selection must validate VID/PID and, where available, interface/usage metadata before opening a writable endpoint.

## Documentation rules

Protocol documents use these status markers:

- `CONFIRMED`
- `RECONSTRUCTED`
- `HYPOTHESIS`
- `UNKNOWN`

Every packet description must name its evidence.

## Dependency policy

Prefer mature, narrowly scoped crates. Do not add large frameworks to solve small protocol problems.

Before adding a new dependency, record why the standard library/current dependencies are insufficient.

## UI policy for later phases

The frontend may call typed backend commands only. It must not construct raw USB/HID packets.
