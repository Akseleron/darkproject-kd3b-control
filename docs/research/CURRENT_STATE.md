# Current Research State

## Confirmed

- Target device is USB `195D:2061` and is publicly identified as Dark Project KD3B V2.
- Linux exposes the product string `Turing Gaming Keyboard`.
- Direct per-key RGB packet structure and full 87-key offset mapping are publicly implemented and are encoded by this project.
- Passive vendor Report-4 input events are observable in Chromium WebHID.
- The current Dark Project Web Lab does not request/support `195D:2061`.
- `dpctl devices` discovers all three target HID interfaces with the pinned `hidapi 2.6.6` static-libusb backend, including interface `2` at path `1-11:1.2` on the tested host.
- On 2026-08-16, an explicitly confirmed `dpctl probe` selected exactly interface `2`, opened its retained raw HIDAPI path once, immediately dropped the handle, and exited `0` without any application-level HID report read/write/feature operation.
- On 2026-08-16, the first explicitly approved Direct RGB hardware write used interface `2` at path `1-11:1.2`, requested packet A followed by packet B through HIDAPI, completed with exit code `0`, and produced the expected visible result: only physical `F1` was illuminated red while the other mapped keys were off.
- The successful first RGB write confirms that the pinned Linux HIDAPI/libusb path accepts this project's documented two-buffer Direct RGB sequence on the tested KD3B rev.2 unit.
- The public OpenRGB issue #2292 RED/GREEN/BLUE/OFF, named-effect, and Brightness0-100 captures were downloaded and parsed by the project's own pcapng/USBPcap tooling.
- Every non-descriptor endpoint-3 OUT payload in that capture set is exactly 256 bytes and uses one of the two already-known Direct RGB headers (`08 07 00 00 00` or `08 07 00 01 00`).
- WAVE, ConicBand, Spiral, Cycle, LinearWave, Ripple, Breathing, Rain, Fire, and Trigger in that capture set are host-streamed Direct RGB frame sequences rather than evidence of distinct effect-mode packets.
- The Brightness0-100 recording changes uniform Direct RGB channel values from black toward full white; no separate brightness command appears in the extracted endpoint-3 OUT stream.

## Implemented here

- typed 87-key logical layout and packet offsets;
- two-buffer 256-byte Direct RGB encoder;
- `PacketTransport` abstraction and hardware-free mock transport;
- read-only target HID discovery for `195D:2061`;
- deterministic interface-2 selection with typed no-target/no-interface/ambiguous states;
- opaque prepared interface probe retaining the original HIDAPI path privately;
- interactive, fail-closed `dpctl probe` with exactly one `HidApi::open_path` attempt and immediate handle drop;
- retained HIDAPI packet transport with full-write validation and no automatic retry/fallback;
- `dpctl info` read-only diagnostics;
- interactive volatile Direct RGB CLI for one-key frames, solid frames, and all-off frames;
- hardware-independent tests covering packet encoding, discovery, selection, open/drop injection, CLI parsing/confirmation, failure handling, short-write handling, and no-fallback behavior;
- standalone pcapng + USBPcap capture parser/diff tool for public protocol research;
- reproducible GitHub Actions capture-analysis workflow.

## Capture-analysis conclusion

The public issue #2292 captures do **not** establish an onboard effect protocol. They show the OEM application streaming the named effects as ordinary Direct RGB frames. This is sufficient evidence to implement equivalent categories of effects in a host-side software renderer without inventing persistent keyboard commands.

See `docs/research/openrgb-2292-capture-analysis.md` for hashes, frame counts, brightness observations, and the exact evidence boundary.

## Unknown / must be resolved

- exact USB-transfer framing used internally by HIDAPI/libusb when an application submits each 256-byte Direct RGB buffer to the USB-level 64-byte OUT endpoint;
- whether the keyboard exposes any separate persistent/onboard lighting-mode command path not present in the public captures;
- current-state readback, if any;
- base-layer remapping protocol;
- Fn layer programming protocol;
- macro storage format;
- onboard profiles and persistence format;
- whether configuration can be safely backed up/read before persistent writes.

## Evidence boundary for live validation

The successful open/drop probe proves that the exact selected interface-2 raw path can be opened and released through the configured HIDAPI/libusb backend.

The successful first RGB write additionally proves that, on the tested KD3B rev.2 unit and Linux stack, two complete 256-byte application writes generated from the documented Direct RGB codec were accepted and produced the expected `F1 = red`, all-other-mapped-keys-off visual state.

This does not establish how HIDAPI/libusb fragments or frames the writes at USB-transfer level, and it does not prove anything about persistence, remapping, macros, firmware, or configuration readback.

The capture analysis proves only what is present in those files. Absence of a distinct onboard-effect command in them is not proof that no such command exists elsewhere.

The backend may internally rescan, path-match, claim/release interfaces, detach/reattach a kernel driver where applicable, or establish interrupt-IN activity where applicable. The live results do not establish that each possible internal effect occurred.

## Wrong-path findings to avoid repeating

- `0416:C345` belongs to a different Dark Project/Witmod hardware family.
- The downloaded KD87 OEM package does not contain the target `195D:2061` transport backend.
- Passive Feature Report 7 reads returning zeros do not reveal the device configuration by themselves.
- The issue #2292 named-effect captures should not be described as proof of onboard-effect command packets; their endpoint-3 OUT traffic is Direct RGB frame streaming.
