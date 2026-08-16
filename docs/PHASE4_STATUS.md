# Phase 4 desktop status

Status: **implementation complete enough for desktop QA; continuous hardware streaming remains unvalidated**.

## Implemented

- Tauri 2 Linux-first desktop shell using the existing `kd3b-device`, `kd3b-effects` and `kd3b-protocol` crates.
- Automatic NVIDIA + Wayland WebKitGTK DMABUF workaround before WebKit initialization.
- Read-only device metadata page with unique interface-2 selection status.
- Accurate visual TKL geometry for all 87 logical keys, including wide modifiers and 6.25u Space.
- 60 FPS host preview target driven by `requestAnimationFrame`; scrolling does not suspend animation.
- Software-effect catalog backed by the Rust effect engine.
- Effect brightness, speed, direction and primary/secondary colors.
- Direct RGB editor with per-key toggle selection and reusable groups: all, WASD, arrows, F1-F12 and navigation cluster.
- Direct RGB editing always keeps a complete 87-key host frame; the UI does not pretend a one-key edit is an incremental hardware write.
- Host profile save/load/delete using WebView local storage. Profiles include effect state and the complete Direct RGB frame.
- Separate read-only Factory / Onboard section so application profiles are not confused with the keyboard's Default/Fn layers or onboard memory.
- Explicit session-only hardware arming gate before any volatile RGB writes.
- One-shot full-frame write command and a cancellable software-effect worker are implemented behind that gate.
- JavaScript syntax validation is part of CI in addition to Rust fmt/check/clippy/test.

## Deliberately not claimed yet

- Continuous software-effect streaming has not yet been exercised on the physical KD3B.
- The current hardware worker interval is conservative and must not be presented as the keyboard's maximum supported frame rate.
- Factory/onboard profile switching, Fn assignments, macros, key remapping and persistence are not implemented because their exact KD3B rev.2 protocol has not been validated.
- Host profiles are not equivalent to onboard profiles.

## Next hardware boundary

Before merging Phase 4 to `main`, perform one explicitly approved continuous volatile RGB test on interface 2:

1. arm hardware output for the current application session;
2. start one visually obvious software effect at the conservative stream interval;
3. verify keyboard input remains responsive and the expected effect is visible;
4. stop the stream cleanly;
5. verify no background worker remains and the device stays enumerated;
6. record exact terminal/UI observations before changing stream rate.

Only after this test should the project tune/raise the hardware frame rate and mark the streaming path validated.
