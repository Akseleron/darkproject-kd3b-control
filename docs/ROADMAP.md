# Roadmap

## Phase 0 - Repository baseline

Status: prepared.

Acceptance criteria:

- OpenCode instructions present.
- Rust workspace skeleton builds.
- Current hardware/research facts documented.
- No real hardware writes in baseline.

## Phase 1 - Direct RGB CLI

Goal: establish a safe, tested path from typed RGB data to the documented KD3B direct-RGB packet format.

Tasks:

1. Implement keyboard key enum/layout and offset table.
2. Implement two 256-byte direct RGB packet encoders.
3. Add complete golden tests.
4. Define transport trait and recording mock transport.
5. Implement `dpctl devices` read-only discovery.
6. Resolve the correct Linux transport for the configuration interface.
7. Add `dpctl info` read-only diagnostics.
8. Add a hardware-gated `dpctl rgb key F1 ff0000` path.
9. After explicit user approval, perform the first one-key write.
10. Add solid/per-key RGB commands only after the first test is confirmed.

Exit criterion: reliable direct RGB control without GUI.

## Phase 2 - OEM onboard lighting protocol

Use the public KD3B rev.2 packet captures to recover:

- off;
- solid colors;
- wave;
- conic band;
- spiral;
- cycle;
- linear wave;
- ripple;
- breathing;
- rain;
- fire;
- trigger/reactive;
- brightness 0-100;
- speed/direction/color parameters if present.

Create a capture parser/diff tool before manually reading packets one by one.

Exit criterion: documented typed onboard-lighting API with tests.

## Phase 3 - Desktop shell

Scaffold Tauri 2 only after Phases 1-2 are stable.

Initial UI:

- device status;
- visual 87-key layout;
- per-key and group selection;
- solid/direct RGB;
- onboard effects;
- brightness/speed/direction controls;
- profile save/load.

## Phase 4 - Software effect engine

Implement frame-based host rendering:

```text
Effect + FrameContext -> [RGB; 87]
```

Context should eventually expose time, delta, layout coordinates, pressed-key events, groups, and parameters.

Initial effects:

- gradient;
- wave;
- radial ripple;
- reactive key pulse;
- trail;
- rain;
- heatmap.

Add frame-rate limiting and write coalescing so USB traffic is controlled.

## Phase 5 - Profiles and automation

- named profiles;
- import/export JSON;
- profile inheritance;
- manual override;
- application-based auto switching;
- optional daemon/tray runtime.

## Phase 6 - Onboard keymap/Fn/macros

Do not implement by guessing.

Recover from an exact KD3B OEM application or new controlled USB captures:

- base keymap;
- Fn layer;
- multimedia actions;
- macro storage;
- onboard profile format.

Host-side Linux remapping is secondary and must not be presented as equivalent to onboard programming.

## Phase 7 - Custom effect authoring

Two levels:

1. Visual composition: layers, masks, gradients, blend modes, timing, reactive nodes.
2. Scriptable/sandboxed effect API after the native engine interface stabilizes.

Select the scripting runtime only after benchmarking and defining the API boundary.

## Phase 8 - Packaging and releases

- Arch/CachyOS package path;
- portable Linux package as appropriate;
- optional Windows build;
- CI;
- signed/checksummed releases;
- user documentation and localization.
