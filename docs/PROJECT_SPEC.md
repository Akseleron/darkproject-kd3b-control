# Project Specification

## Product vision

Create a native Linux-first configuration application for Dark Project KD3B rev.2 that matches useful OEM functionality and then exceeds it.

The application should eventually provide:

- exact-device discovery and diagnostics;
- per-key RGB control;
- onboard lighting modes recovered from OEM captures;
- brightness, speed, direction, colors, and mode-specific parameters where supported by firmware;
- software-driven custom effects that are not limited by firmware modes;
- reactive effects driven by keyboard HID events;
- named profiles, profile inheritance, import/export, and automatic profile switching;
- key groups and masks;
- visual custom-effect composition;
- a stable scriptable effect API after the native effect engine is mature;
- onboard remapping, Fn layer editing, multimedia actions, macros, and profile storage if the protocol can be safely recovered;
- backup/restore of readable device configuration if safe read/write semantics are discovered;
- a developer mode with decoded packets and evidence annotations, without exposing unsafe arbitrary writes by default;
- optional Windows support from the same codebase after Linux is stable.

## Hardware scope

Initial and only guaranteed device:

- Dark Project KD3B rev.2
- VID `0x195D`
- PID `0x2061`
- Linux USB product string observed: `Turing Gaming Keyboard`

Do not generalize to other Dark Project devices merely because they share branding or keyboard layouts.

## Functional model

Lighting must be divided into two explicit categories:

### Onboard effects

Commands configure a firmware effect. The keyboard continues the effect without a running application when the device actually supports that behavior.

### Software effects

The application/daemon calculates frames and streams direct RGB packets. These effects can be arbitrary but require the runtime to stay active.

The UI must clearly distinguish these two modes so the user is never misled about persistence.

## Architecture constraints

Protocol knowledge must not be embedded in UI components.

Core boundaries:

1. Pure protocol representation and packet codecs.
2. Transport abstraction.
3. Device orchestration/state.
4. Effects and profiles.
5. CLI.
6. Desktop UI.

The pure protocol crate must be testable on any machine without USB hardware.

## UX direction

The final desktop application should be substantially cleaner than the old OEM software:

- accurate visual TKL layout;
- single-key and multi-key selection;
- drag/rectangle selection;
- reusable key groups such as WASD, numbers, function row, arrows;
- color palette and gradients;
- effect preview before hardware apply;
- clear distinction between hardware and software effects;
- profile hierarchy and searchable settings;
- RU/EN localization architecture from the beginning, even if the first UI prototype is English-only;
- no hidden destructive device actions.

## Non-goals for early milestones

- firmware flashing;
- bootloader tooling;
- support for every Dark Project keyboard;
- host-side Linux key remapping as a substitute for onboard remapping;
- GUI before the direct-RGB protocol path is proven through CLI and tests.
