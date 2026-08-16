use std::fmt::{self, Write as _};

use kd3b_device::DiscoveredHidInterface;

const TARGET_HEADER: &str = "Target: Dark Project KD3B rev.2 (VID:PID 195d:2061)\n";
const CANDIDATE: &str = "unvalidated configuration-interface candidate; no application-level HID handle/report request by dpctl; HIDAPI/libusb may open visible HID devices for metadata; transport/writability not validated";

pub(crate) fn render_interfaces(
    interfaces: &[DiscoveredHidInterface],
) -> Result<String, fmt::Error> {
    let mut output = format!("{TARGET_HEADER}Matches: {}\n", interfaces.len());
    if interfaces.is_empty() {
        output.push_str("No matching HID interfaces found.\n");
        return Ok(output);
    }

    for interface in interfaces {
        let interface_suffix = match interface.interface_number {
            -1 => " (not reported)",
            _ => "",
        };
        let candidate = if interface.is_unvalidated_configuration_interface_candidate() {
            CANDIDATE
        } else {
            "none"
        };
        writeln!(
            output,
            "Interface: {}{interface_suffix}\nVID:PID: {:04x}:{:04x}\nProduct: {}\nManufacturer: {}\nSerial: {}\nRelease: 0x{:04x}\nBus: {}\nPath: {}\nCandidate: {candidate}",
            interface.interface_number,
            interface.vendor_id,
            interface.product_id,
            interface
                .product_string
                .as_deref()
                .unwrap_or("<unavailable>"),
            interface
                .manufacturer_string
                .as_deref()
                .unwrap_or("<unavailable>"),
            interface
                .serial_number
                .as_deref()
                .unwrap_or("<unavailable>"),
            interface.release_number,
            interface.bus_type,
            interface.path,
        )?;
    }

    Ok(output)
}
