# Packet Capture Analysis Plan

Do not manually eyeball every pcap first.

Build a small analysis pipeline that:

1. Opens all reference `pcapng` files.
2. Extracts USB/HID host-to-device transfers for the target device.
3. Normalizes timestamps and removes unrelated enumeration noise.
4. Groups writes by transfer type, endpoint/report, and packet length.
5. Produces a hex diff across captures.
6. Identifies bytes that vary with mode/color/brightness.
7. Emits machine-readable JSON fixtures for protocol tests.
8. Records conclusions in `docs/protocol/` with status markers.

The parser must never need a connected keyboard.
