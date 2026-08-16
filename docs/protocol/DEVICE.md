# Device Identity and Interfaces

Status: **CONFIRMED**, except where marked otherwise.

## Identity

Observed on the target Linux host:

```text
VID:PID       195d:2061
USB vendor DB Itron Technology iONE
Manufacturer  Turing Gaming Keyboard
Product       Turing Gaming Keyboard
bcdDevice     31.31
```

Independent OpenRGB support identifies `0x195D:0x2061` as **Dark Project KD3B V2**.

## Linux USB interface observations

Observed with `lsusb -v`:

### Interface 0

- HID
- Boot Interface Subclass
- Keyboard protocol
- endpoint `0x81 IN`
- max packet 64 bytes

### Interface 1

- HID
- Boot Interface Subclass
- Mouse protocol
- endpoint `0x82 IN`
- max packet 64 bytes

Its exposed HID collections include standard mouse/consumer/keyboard collections plus vendor-defined reports. See `input-report-4.md`.

### Interface 2

Observed:

- HID class
- endpoint `0x03 OUT`
- max packet 64 bytes in the USB descriptor
- no kernel driver was bound in the captured Linux state

OpenRGB's exact-device detector additionally targets interface `2`, usage page `0xFFC2`, usage `4` for this VID/PID.

The USB descriptor exposes a 64-byte OUT max packet size, while the application-level direct-RGB codec submits 256-byte buffers through HIDAPI. The successful live write described below proves the application-level path works on the tested system, but it does not by itself reveal how HIDAPI/libusb frames or fragments those buffers at USB-transfer level.

### Live Phase-1 open/drop validation

On 2026-08-16, the target CachyOS host ran:

```text
cargo run -p dpctl -- probe
```

The probe enumerated the target as:

```text
Interface: 2
VID:PID: 195d:2061
Product: Turing Gaming Keyboard
Manufacturer: Turing Gaming Keyboard
Serial: <unavailable>
Release: 0x3131
Bus: usb
Path: 1-11:1.2
```

After the user manually entered the exact confirmation phrase `OPEN INTERFACE 2`, the process completed successfully with exit code `0` and reported:

```text
No HID report read/write/feature operation was requested.
```

This confirms that the selected interface-2 raw HIDAPI path could be opened successfully with the pinned Linux static-libusb HIDAPI backend and that the returned handle could then be dropped. The application did not request any HID report read, write, feature-report, report-descriptor, RGB, or configuration operation during this probe.

### Live Phase-1 first direct-RGB validation

Later on 2026-08-16, after explicit approval for the first hardware RGB write, the target host ran:

```text
cargo run -p dpctl -- rgb key F1 ff0000
```

The command selected the same target metadata:

```text
Interface: 2
VID:PID: 195d:2061
Path: 1-11:1.2
Release: 0x3131
Bus: usb
```

After the user manually entered the exact confirmation phrase required by that validation build, `dpctl` requested exactly two 256-byte HIDAPI writes in documented packet-A then packet-B order. HIDAPI accepted the complete pair and the process exited `0`.

The physical keyboard produced the expected visible result: **only F1 illuminated red; every other mapped key was off**.

This live result confirms the tested Linux transport path, write order, direct-RGB codec, and the documented F1 logical mapping are mutually compatible on this exact KD3B rev.2 unit. It does not prove persistence or any onboard configuration protocol, and it does not reveal the backend's lower-level USB transfer framing.

HIDAPI/libusb may internally rescan, path-match, open, claim/release an interface, detach/reattach a kernel driver where applicable, or establish backend-managed interrupt-IN activity where applicable. The live results are not evidence that every possible backend effect occurred.

## udev state used during research

A local rule was added during WebHID research:

```udev
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="195d", ATTRS{idProduct}=="2061", TAG+="uaccess"
SUBSYSTEM=="usb", ATTR{idVendor}=="195d", ATTR{idProduct}=="2061", TAG+="uaccess"
```

Do not silently install or modify udev rules from the application. Packaging/setup should make permissions explicit.
