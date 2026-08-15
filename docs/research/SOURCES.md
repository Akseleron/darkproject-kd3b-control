# Research Sources

## Exact-device public implementation

OpenRGB, Dark Project controller:

- Detector: https://github.com/CalcProgrammer1/OpenRGB/blob/master/Controllers/DarkProject/DarkProjectControllerDetect.cpp
- Controller header: https://github.com/CalcProgrammer1/OpenRGB/blob/master/Controllers/DarkProject/DarkProjectKeyboardController.h
- Controller implementation: https://github.com/CalcProgrammer1/OpenRGB/blob/master/Controllers/DarkProject/DarkProjectKeyboardController.cpp
- RGB controller/layout: https://github.com/CalcProgrammer1/OpenRGB/blob/master/Controllers/DarkProject/RGBController_DarkProjectKeyboard.cpp
- Initial KD3B support commit: https://github.com/CalcProgrammer1/OpenRGB/commit/e9eca70e72911d82df145534043f67a735b8c5a4

OpenRGB labels `0x195D:0x2061` as Dark Project KD3B V2 and includes a direct per-key RGB implementation.

License note: OpenRGB source is GPL-2.0-or-later. Use it as evidence for protocol facts. Do not copy implementation code into this repository unless the repository licensing strategy explicitly becomes GPL-compatible.

## Original support issue and packet captures

- https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/2292

The issue contains OEM screenshots and packet captures for:

- report descriptor;
- RED/GREEN/BLUE/OFF;
- WAVE;
- ConicBand;
- Spiral;
- Cycle;
- LinearWave;
- Ripple;
- Breathing;
- Rain;
- Fire;
- Trigger;
- Brightness0-100.

Use `scripts/fetch-reference-captures.fish` to fetch local copies into the ignored `captures/raw/` directory.

## Dark Project Web Lab research

Current Web Lab:

- https://software.darkproject.eu/

The current site uses WebHID but does not include `195D:2061` in its supported-device request filters. The target keyboard is therefore not supported by the current Web Lab despite WebHID functioning on Linux/Chromium.

## OEM package inspected during research

Downloaded legacy package hash:

```text
SHA256 8164b11d7c03e2dd3246a7f30740e9fa1a474dcfda7e8e8136548e8a2dd0a23c
```

Extracted archive hash:

```text
SHA256 c62f0097ddc8a14b35dc066a9ccaec777b2dbf656265d3581eeaf1ff5bf3819f
```

Main extracted application:

```text
Dark Project.exe
SHA256 06ce2f2d64c512a16d557cfe34ce53a275d61ebe7603aaa4e10ce7916943de06
```

This package belongs to a different/newer hardware family and is not the correct transport implementation for `195D:2061`. It remains useful only as general OEM UI/data-model research. Proprietary binaries should not be committed.
