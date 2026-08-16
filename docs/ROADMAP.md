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

Implement frame-based host rendering:

```text
Effect + FrameContext -> [RGB; 87]
```

The layout coordinate system should derive from the documented KD3B protocol offset grid, while keeping logical-key identity separate from geometry.

Initial effects:

- solid and gradient;
- wave;
- conic band;
- spiral;
- cycle;
- linear wave;
- radial ripple;
- breathing;
- rain;
- fire;
- reactive key pulse/trigger after passive key-event integration.

Engine requirements:

- deterministic hardware-free frame generation tests;
- brightness as a host-side transform;
- speed and direction parameters where meaningful;
- frame-rate limiting;
- write coalescing so identical frames are not resent;
- cancellation without leaving a background writer alive;
- no persistent keyboard configuration writes.

Exit criterion: reusable effect engine capable of producing 87-key Direct RGB frames independently of the UI and hardware transport.

## Phase 4 - Desktop application

Scaffold the native Linux-first desktop application after the effect-engine API is stable enough to expose without protocol churn.

Initial UI:

- device status;
- visual 87-key layout;
- per-key and group selection;
- solid/direct RGB;
- software effects;
- brightness/speed/direction controls;
- live preview;
- explicit distinction between volatile host-rendered lighting and any future persistent/onboard settings;
- profile save/load.

The desktop runtime must use the same protocol/device/effect crates as the CLI rather than duplicating USB logic.

## Phase 5 - Profiles and automation

- named profiles;
- import/export JSON;
- profile inheritance;
- manual override;
- application-based auto switching;
- optional daemon/tray runtime.

## Phase 6 - Persistent/onboard protocol, keymap, Fn and macros

Do not implement by guessing.

Recover from an exact KD3B OEM application or new controlled USB captures:

- any actual onboard lighting-mode command path;
- base keymap;
- Fn layer;
- multimedia actions;
- macro storage;
- onboard profile format;
- persistence/readback semantics.

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
