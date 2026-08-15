# Direct Per-Key RGB Protocol

Status: **CONFIRMED BY PUBLIC IMPLEMENTATION**, not yet hardware-validated by this repository.

Evidence: OpenRGB's Dark Project KD3B V2 controller for VID:PID `195D:2061`.

## Packet model

Direct RGB uses two outbound buffers, each 256 bytes.

Packet A header:

```text
offset 0..4: 08 07 00 00 00
```

Packet B header:

```text
offset 0..4: 08 07 00 01 00
```

For each logical key, resolve its protocol `key_offset` from `key-offsets.csv`.

Packet A:

```text
red   -> byte 5   + key_offset
green -> byte 107 + key_offset
```

Packet B:

```text
blue  -> byte 5 + key_offset
```

Unused bytes remain zero in the known direct-control implementation.

The known implementation writes Packet A, then Packet B.

## Important transport caveat

The public implementation uses HIDAPI `hid_write(..., 256)` against the exact device interface it opens.

Do not assume that a raw Linux USB endpoint write should also be 256 bytes. The repository must first reproduce the exact HIDAPI transport/device selection safely and confirm how the Linux backend frames the transfer.

## Tests required before hardware

The protocol crate must have:

- exact packet length assertions;
- exact header assertions;
- key offset range checks;
- golden test for all-black frame;
- golden tests for red/green/blue on at least Esc, F1, A, Space, and Right arrow;
- test that one key does not alter bytes belonging to another channel/key;
- validation for exactly 87 logical keys.
