# Vendor Input Report 4

Status: **PARTIALLY RECONSTRUCTED** from passive WebHID captures on the user's physical keyboard.

No writes were used to obtain these observations.

## HID collection

Observed on the selectable WebHID device:

```text
Usage page: 0xFF00
Usage:      0xFF00
Input:      Report ID 4
Output:     Report ID 4
Payload:    8 bytes returned by WebHID event data
```

Additional vendor collections exist for Report 6 input and Report 7 feature data.

## Physical key events

Repeated events have the shape:

```text
f1 f2 AA BB PP CC FF 00
```

Current interpretation:

- `f1 f2`: physical matrix-event prefix. **CONFIRMED pattern**.
- `AA BB`: physical key matrix coordinate. **RECONSTRUCTED**.
- `PP`: `01` press, `00` release. **RECONSTRUCTED with strong evidence**.
- `CC`: current count/state related to simultaneously held keys. **RECONSTRUCTED**.
- `FF`: Fn-state related flag in tested sequences. **RECONSTRUCTED**.
- final byte observed `00` in these tests.

Known physical positions from controlled F-row test:

```text
F1 -> 02 00
F2 -> 03 00
F3 -> 04 00
F4 -> 05 00
Fn -> 09 05
```

## Fn semantic events

With Fn held, additional Report-4 packets appeared with `e1` prefix:

```text
Fn press        e1 04 01 00 00 00 00 00
Fn release      e1 05 02 00 00 00 00 00
Fn + F1         e1 01 01 00 00 00 00 00
Fn + F2         e1 01 02 00 00 00 00 00
Fn + F3         e1 01 03 00 00 00 00 00
Fn + F4         e1 01 04 00 00 00 00 00
```

Treat semantic meaning beyond these exact tested combinations as **UNKNOWN**.

## Feature Report 7

Passive reads returned 64 bytes through WebHID:

```text
07 00 00 00 ... 00
```

and did not change during tested Fn/lighting actions.

Do not treat Feature Report 7 as a current-settings snapshot. Its command semantics remain **UNKNOWN** and no writes to it are permitted without stronger evidence.

## Future use

Report 4 is useful for software-reactive lighting because it can expose physical keyboard events independently of desktop key-remapping layers. It is not required for the first direct-RGB milestone.
