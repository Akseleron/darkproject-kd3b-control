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

The relationship between the USB-level 64-byte OUT endpoint and the 256-byte HIDAPI writes used by OpenRGB must be verified on Linux before the first hardware write. Do not assume the transport backend or report framing.

Status of that relationship: **UNKNOWN / first Phase-1 transport task**.

## udev state used during research

A local rule was added during WebHID research:

```udev
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="195d", ATTRS{idProduct}=="2061", TAG+="uaccess"
SUBSYSTEM=="usb", ATTR{idVendor}=="195d", ATTR{idProduct}=="2061", TAG+="uaccess"
```

Do not silently install or modify udev rules from the application. Packaging/setup should make permissions explicit.
