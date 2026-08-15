# Dark Project KD3B Control

Native Linux-first configurator for **Dark Project KD3B rev.2** keyboards using USB VID:PID `195D:2061`.

The project has two goals:

1. Reproduce the useful configuration features of the original Dark Project Windows software on Linux.
2. Go beyond the OEM application with a programmable software effect engine, better profiles, reactive lighting, safer protocol tooling, and a modern desktop UI.

## Current hardware target

- Device: Dark Project KD3B rev.2
- USB VID: `0x195D`
- USB PID: `0x2061`
- USB product string observed on Linux: `Turing Gaming Keyboard`
- Known direct-RGB protocol: documented in `docs/protocol/direct-rgb.md`
- Known passive input reports: documented in `docs/protocol/input-report-4.md`

Only this exact device is in scope for the first implementation. Support for other Dark Project models must be added explicitly later.

## Development strategy

The first milestone is deliberately CLI-only:

1. Pure packet encoder with golden tests.
2. Mock transport.
3. Linux device enumeration.
4. Read-only device inspection.
5. Explicitly approved hardware write test for one key.
6. Solid/per-key RGB CLI.
7. Reverse-engineer and implement OEM onboard effects.
8. Only then scaffold the Tauri desktop application.

See `docs/ROADMAP.md`.

## OpenCode

This repository is prepared for OpenCode:

- `AGENTS.md` contains project rules.
- `opencode.json` loads the core instructions and applies conservative command permissions.
- `.opencode/agents/` contains project-specific subagents.
- `.opencode/commands/` contains repeatable project workflows.

Open the repository root in OpenCode. Do **not** run `/init` blindly over the existing `AGENTS.md`; the repository already contains curated instructions.

Recommended first command inside OpenCode:

```text
/context
```

Then use:

```text
/plan-sprint
```

The first implementation prompt is also stored at `prompts/FIRST_IMPLEMENTATION_PROMPT.md`.

## Local setup on CachyOS / Arch

The initial Rust workspace has no native HID dependency yet. First verify the base toolchain:

```fish
./scripts/check-env.fish
```

The project should stay buildable without a connected keyboard. Real hardware access is introduced only after the protocol and mock transport are tested.

## Source policy

Public reverse-engineering sources are used as evidence for protocol facts. Do not copy GPL implementation code into this repository unless the project deliberately adopts a compatible license. See `docs/research/SOURCES.md`.

## License

No project license has been selected yet. Choose one before the first public release.
