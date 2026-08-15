# Hardware Safety Policy

This repository interacts with physical USB hardware. Safety rules are part of the specification, not optional style guidance.

## Safety levels

### Level 0: offline/read-only

Examples:

- build/test/lint;
- inspect files and packet captures;
- enumerate USB/HID devices;
- read descriptors and strings;
- decode passive input reports;
- use mock transport.

These are the default development activities.

### Level 1: documented volatile writes

Example: a direct RGB packet format that has independent public evidence and golden tests.

Requirements before executing on hardware:

1. Exact VID/PID must match the supported device.
2. Exact interface/transport must be selected deliberately.
3. Packet generator must have byte-level tests.
4. User must explicitly approve the hardware write in the current session.
5. The agent must state the expected visible result.
6. No persistent/firmware claim may be made without evidence.

### Level 2: partially understood or persistent configuration writes

Examples:

- unknown feature reports;
- EEPROM/profile writes;
- undocumented remap/macro storage;
- arbitrary control transfers.

Forbidden until the complete packet semantics are documented from OEM traces or equivalent evidence and separately reviewed.

### Level 3: firmware/bootloader

Forbidden for this project unless the user explicitly creates a separate firmware-recovery objective in the future.

Never:

- enter bootloader;
- erase or write firmware;
- send guessed firmware updater commands;
- run vendor firmware executables against the physical device during protocol research.

## Implementation safeguards

The real transport must not accept arbitrary byte arrays from UI code.

Prefer typed operations such as:

```text
set_direct_rgb(frame)
set_onboard_effect(effect)
set_brightness(value)
```

Each typed operation must map to a documented codec. A developer packet inspector may display raw bytes but must not become a generic "send arbitrary packet" button in normal builds.

Hardware integration tests must be ignored by default and named clearly.
