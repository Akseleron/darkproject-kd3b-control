# First implementation prompt for OpenCode

Work only on Phase 1's offline foundation. Do not access or write to the physical keyboard.

First read `AGENTS.md`, `docs/SAFETY.md`, `docs/ROADMAP.md`, `docs/protocol/direct-rgb.md`, and `docs/protocol/key-offsets.csv`.

Implement the following:

1. In `kd3b-protocol`, create a strongly typed representation of the 87 logical keys and the key-to-protocol-offset mapping from `key-offsets.csv`.
2. Add an `Rgb8` type.
3. Implement a pure direct-RGB encoder that accepts a complete 87-key frame and returns the two 256-byte packets documented in `direct-rgb.md`.
4. Validate packet lengths and mapping bounds without panics in public APIs.
5. Add comprehensive unit/golden tests including black frame and isolated red/green/blue values for Esc, F1, A, Space, and Right arrow.
6. In `kd3b-device`, define a transport trait plus a recording `MockTransport` that can capture outbound packets, but do not add a real HID dependency yet.
7. Add tests proving the mock receives Packet A before Packet B when a future direct-frame operation is invoked through a small device abstraction.
8. Keep `dpctl` non-hardware-facing for this task. It may expose only a version/help placeholder if needed.
9. Update documentation only if implementation reveals an inconsistency. Never change protocol docs merely to fit code.

Required verification:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Before editing, summarize the protocol facts you are implementing and explicitly confirm that no real hardware I/O will be introduced in this task.
