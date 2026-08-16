# Roadmap

## Phase 0 - Repository baseline

Status: complete.

Acceptance criteria:

- OpenCode instructions present.
- Rust workspace skeleton builds.
- Current hardware/research facts documented.
- No real hardware writes in baseline.

## Phase 1 - Direct RGB CLI

Status: complete on the tested KD3B rev.2 unit.

Goal: establish a safe, tested path from typed RGB data to the documented KD3B direct-RGB packet format.

Completed:

1. Keyboard key enum/layout and offset table.
2. Two 256-byte Direct RGB packet encoder.
3. Golden tests.
4. Transport trait and recording mock transport.
5. `dpctl devices` read-only discovery.
6. Exact interface-2 Linux HIDAPI/libusb transport.
7. `dpctl info` read-only diagnostics.
8. Hardware-gated open/drop probe.
9. Explicitly approved first one-key write: physical F1 became red, all other mapped keys turned off, exit code 0.
10. General volatile `rgb key`, `rgb solid`, and `rgb off` CLI paths.

Exit criterion: **met**. Reliable Direct RGB control exists without GUI.

## Phase 2 - Public capture classification

Status: complete for the OpenRGB issue #2292 capture set.

A dedicated pcapng/USBPcap parser and diff tool was implemented before interpreting captures manually.

Result:

- RED/GREEN/BLUE/OFF are ordinary 256-byte Direct RGB frame streams.
- WAVE, ConicBand, Spiral, Cycle, LinearWave, Ripple, Breathing, Rain, Fire, and Trigger are also ordinary Direct RGB frame streams.
- `Brightness0-100` changes the streamed RGB channel values; no separate brightness command appears in the captured endpoint-3 OUT stream.
- No distinct onboard-effect command packet was found in this capture set.

See `docs/research/openrgb-2292-capture-analysis.md`.

Consequence: do not invent an onboard-lighting protocol from these files. The captured OEM effects are suitable references for a host-rendered software effect engine. Persistent/onboard lighting remains unknown unless different evidence is recovered.

Exit criterion: **met**. The capture set is classified and its evidence boundary is documented.

## Phase 3 - Software effect engine

Status: complete for the initial host-rendered effect set.

Implemented frame model:

```text
Effect + FrameContext -> [RGB; 87]
```

Implemented effects and transforms:

- gradient;
- wave;
- conic band;
- spiral;
- cycle;
- linear wave;
- radial ripple;
- breathing;
- rain;
- fire;
- host-side brightness;
- speed and direction controls.

The engine is hardware-independent and covered by deterministic tests. The hardware worker coalesces identical consecutive frames and is cancellable. Reactive key pulse/trigger remains tied to future passive key-event integration rather than blocking the desktop application.

Exit criterion: **met**. `kd3b-effects` produces reusable 87-key Direct RGB frames independently of the UI and hardware transport.

## Phase 4 - Desktop application

Status: implementation complete up to the next physical hardware-streaming validation gate.

Implemented:

- Tauri 2 Linux-first desktop application;
- automatic NVIDIA/Wayland WebKitGTK DMABUF workaround before WebKit initialization;
- read-only device status and unique interface-2 selection;
- accurate visual 87-key TKL geometry;
- continuous 60 FPS target live preview without scroll-induced suspension;
- software effect selection and brightness/speed/direction/color controls;
- Direct RGB mode with per-key selection and reusable key groups;
- complete 87-key host frame editing, including an all-key solid-color workflow;
- session-only explicit hardware-output arming gate;
- one-shot full-frame Direct RGB path behind the gate;
- cancellable continuous effect worker behind the gate;
- named host profile save/load/delete containing effect state and the complete Direct RGB frame;
- visible separation between host profiles and factory/onboard state;
- read-only KD3-family documentation for the three onboard profile shortcuts and Fn shortcut groups;
- JavaScript syntax validation plus Rust fmt/check/clippy/test in CI.

Still required before Phase 4 can be merged as validated:

1. Local GUI QA of the current branch after the full editor/profile changes.
2. One explicitly approved physical continuous-effect test on the KD3B.
3. Confirm clean stop, keyboard responsiveness, device enumeration and no stranded writer.
4. Record the hardware result and then decide whether the conservative hardware stream interval should be raised.

The desktop runtime uses the same protocol/device/effect crates as the CLI rather than duplicating USB logic.

Exit criterion: **pending only the physical continuous-stream validation and resulting documentation**.

See `docs/PHASE4_STATUS.md` and `docs/research/FACTORY_LAYER.md`.

## Phase 5 - Profiles and automation

- JSON import/export for host profiles;
- profile inheritance;
- manual override policy;
- application-based auto switching;
- optional daemon/tray runtime;
- filesystem-backed profile library when the persistence UX is finalized.

## Phase 6 - Persistent/onboard protocol, keymap, Fn and macros

Do not implement by guessing.

Recover from an exact KD3B OEM application or new controlled USB captures:

- any actual onboard lighting-mode command path;
- base keymap;
- Fn layer;
- multimedia actions;
- macro storage;
- three-slot onboard profile format;
- persistence/readback semantics.

KD3-family evidence for `Fn + F1..F4` sound, `Fn + F5..F9` lighting and `Fn + F10..F12` three onboard profiles is documented in `docs/research/FACTORY_LAYER.md`. This is a user-facing capability fact, not a recovered write protocol.

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
