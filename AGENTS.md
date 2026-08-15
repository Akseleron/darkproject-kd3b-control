# OpenCode Project Instructions

## Mission

Build a Linux-first, safe, modern configurator for the Dark Project KD3B rev.2 keyboard (`VID 0x195D`, `PID 0x2061`). Reproduce the useful OEM configuration capabilities and then extend them with custom software-driven effects, better profiles, reactive lighting, and a clean desktop UI.

## Mandatory context

Before changing code, read the files loaded through `opencode.json`:

- `docs/PROJECT_SPEC.md`
- `docs/SAFETY.md`
- `docs/DEVELOPMENT.md`

Read protocol/research files lazily when relevant. Do not load every research artifact into context without need.

## Evidence discipline

For protocol work, always distinguish:

- **Confirmed**: directly observed on the user's device, present in packet captures, or supported by a primary/public implementation source.
- **Reconstructed**: strongly inferred from multiple observations.
- **Hypothesis**: plausible but unverified.

Never upgrade a hypothesis to a confirmed fact without evidence. Never invent packet bytes, report IDs, interface numbers, checksums, EEPROM layouts, firmware commands, or device state.

Every newly discovered protocol fact must be documented under `docs/protocol/` with its evidence source.

## Hardware safety

Read `docs/SAFETY.md` before touching real hardware.

Hard rules:

- NEVER flash firmware or enter a bootloader.
- NEVER send undocumented HID/USB commands to physical hardware.
- NEVER brute-force command bytes, report IDs, feature reports, control transfers, or packet lengths.
- NEVER run a hardware-write test without explicit user approval in the current session.
- BEFORE any hardware write, show the exact target device, interface/transport, packet bytes or documented packet generator, expected effect, and rollback/recovery behavior.
- Prefer `MockTransport` for all normal development and tests.
- Treat direct RGB writes as volatile until proven otherwise; do not claim persistence.

## Scope and architecture

Keep low-level protocol logic independent of HID transport and UI.

Planned layers:

- `kd3b-protocol`: pure protocol types, packet encoders/decoders, keyboard layout, no OS I/O.
- `kd3b-device`: device discovery and transport abstraction; mock backend first, real Linux HID backend later.
- `dpctl`: CLI for diagnostics and controlled feature access.
- software effect engine: later, after reliable direct RGB.
- desktop UI: later, after protocol/CLI milestones; Tauri 2 is the current architectural choice.

Do not start desktop GUI work while the current roadmap milestone is protocol/CLI unless the user explicitly changes priority.

## Code quality

- Rust identifiers, public API, file names, and code comments: English.
- Keep protocol constants named, typed, and documented. No unexplained magic byte arrays in application/UI code.
- Use small modules and explicit domain types.
- Avoid `unwrap()`/`expect()` in hardware-facing runtime paths unless the invariant is local and documented.
- Errors must contain actionable context without leaking irrelevant internals.
- Parsing and packet encoding must have unit tests.
- Add golden byte-level tests for every documented packet format.
- Keep hardware tests separate and ignored by default.

Before considering a code task complete, run when applicable:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Reverse engineering and licensing

OpenRGB contains support for this exact VID/PID and is GPL-2.0-or-later. Use it as a source of protocol facts and attribution, not as a source to copy implementation code from, unless a deliberate licensing decision is made.

OEM Windows binaries and extracted resources are research inputs only. Do not commit proprietary binaries, firmware updaters, DLLs, or application assets into the repository.

Public packet captures should be fetched into ignored local research directories unless redistribution rights are clear.

## Git and external services

- Local inspection commands such as `git status`, `git diff`, and `git log` are safe.
- Do not push, create remote repositories, create issues/PRs/releases, upload artifacts, or otherwise mutate GitHub/external services without explicit user approval immediately before the action.
- Do not rewrite public history.
- Do not commit generated build outputs or proprietary research binaries.

## User environment

Primary target is CachyOS/Arch Linux with KDE Plasma on Wayland. User-facing shell instructions must be valid for `fish`, not silently written as Bash syntax. If Bash is genuinely required, invoke it explicitly with `bash -lc '...'`.

## Change discipline

For each task:

1. State what evidence/specification is being implemented.
2. Inspect existing code and relevant docs.
3. Make the smallest coherent change.
4. Add/update tests.
5. Run focused verification.
6. Update protocol/research docs if understanding changed.
7. Report what is confirmed versus still unknown.
