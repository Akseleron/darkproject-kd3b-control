# Current Research State

## Confirmed

- Target device is USB `195D:2061` and is publicly identified as Dark Project KD3B V2.
- Linux exposes the product string `Turing Gaming Keyboard`.
- Direct per-key RGB packet structure and full 87-key offset mapping are publicly implemented and are encoded by this project.
- OEM packet captures exist for multiple onboard effects and brightness.
- Passive vendor Report-4 input events are observable in Chromium WebHID.
- The current Dark Project Web Lab does not request/support `195D:2061`.
- `dpctl devices` discovers all three target HID interfaces with the pinned `hidapi 2.6.6` static-libusb backend, including interface `2` at path `1-11:1.2` on the tested host.
- On 2026-08-16, an explicitly confirmed `dpctl probe` selected exactly interface `2`, opened its retained raw HIDAPI path once, immediately dropped the handle, and exited `0` without any application-level HID report read/write/feature operation.

## Implemented here

- typed 87-key logical layout and packet offsets;
- two-buffer 256-byte direct-RGB encoder;
- `PacketTransport` abstraction and hardware-free mock transport;
- read-only target HID discovery for `195D:2061`;
- deterministic interface-2 selection with typed no-target/no-interface/ambiguous states;
- opaque prepared interface probe retaining the original HIDAPI path privately;
- interactive, fail-closed `dpctl probe` with exactly one `HidApi::open_path` attempt and immediate handle drop;
- hardware-independent tests covering packet encoding, discovery, selection, open/drop injection, CLI confirmation, failure handling, and no-fallback behavior.

## Unknown / must be resolved

- whether the pinned Linux HIDAPI/libusb backend accepts the documented 256-byte direct-RGB application writes on interface `2`;
- how those 256-byte writes are framed against the USB-level 64-byte OUT endpoint;
- whether the documented packet-A then packet-B sequence produces the expected direct RGB result on this exact Linux path;
- onboard effect packet field meanings;
- brightness field mapping from capture;
- speed/direction parameter encoding;
- current-state readback, if any;
- base-layer remapping protocol;
- Fn layer programming protocol;
- macro storage format;
- onboard profiles and persistence format;
- whether configuration can be safely backed up/read before persistent writes.

## Evidence boundary for the successful probe

The successful live probe proves only that the exact selected interface-2 raw path could be opened and released through the configured HIDAPI/libusb backend. It does not prove writability, 256-byte framing, RGB behavior, persistence, readback, or any configuration protocol.

The backend may internally rescan, path-match, claim/release interfaces, detach/reattach a kernel driver where applicable, or establish interrupt-IN activity where applicable. The live result does not establish that each possible internal effect occurred.

## Wrong-path findings to avoid repeating

- `0416:C345` belongs to a different Dark Project/Witmod hardware family.
- The downloaded KD87 OEM package does not contain the target `195D:2061` transport backend.
- Passive Feature Report 7 reads returning zeros do not reveal the device configuration by themselves.
