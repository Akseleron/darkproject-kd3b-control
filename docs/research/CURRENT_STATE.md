# Current Research State

## Confirmed

- Target device is USB `195D:2061` and is publicly identified as Dark Project KD3B V2.
- Linux exposes the product string `Turing Gaming Keyboard`.
- Direct per-key RGB packet structure and full 87-key offset mapping are publicly implemented.
- OEM packet captures exist for multiple onboard effects and brightness.
- Passive vendor Report-4 input events are observable in Chromium WebHID.
- The current Dark Project Web Lab does not request/support `195D:2061`.

## Known but not yet implemented here

- two 256-byte direct RGB buffers;
- full logical-key-to-packet-offset table;
- physical passive F1-F4/Fn Report-4 patterns.

## Unknown / must be resolved

- exact Linux transport/backend needed to reproduce the public 256-byte HID writes to interface 2;
- onboard effect packet field meanings;
- brightness field mapping from capture;
- speed/direction parameter encoding;
- current-state readback, if any;
- base-layer remapping protocol;
- Fn layer programming protocol;
- macro storage format;
- onboard profiles and persistence format;
- whether configuration can be safely backed up/read before persistent writes.

## Wrong-path findings to avoid repeating

- `0416:C345` belongs to a different Dark Project/Witmod hardware family.
- The downloaded KD87 OEM package does not contain the target `195D:2061` transport backend.
- Passive Feature Report 7 reads returning zeros do not reveal the device configuration by themselves.
