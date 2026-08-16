use std::fmt::{self, Write as _};

use kd3b_device::{DiscoveredHidInterface, select_configuration_interface};

pub(crate) fn render_info(
    interfaces: &[DiscoveredHidInterface],
) -> Result<String, fmt::Error> {
    let mut output = String::from(
        "Target: Dark Project KD3B rev.2 (VID:PID 195d:2061)\nOperation: metadata enumeration only; dpctl info does not open the selected interface or request HID reports.\n",
    );
    writeln!(output, "Matching HID interfaces: {}", interfaces.len())?;

    match select_configuration_interface(interfaces) {
        Ok(selected_index) => {
            let selected = &interfaces[selected_index.get()];
            writeln!(
                output,
                "Configuration interface selection: unique\nInterface: {}\nPath: {}\nRelease: 0x{:04x}\nBus: {}",
                selected.interface_number,
                selected.path,
                selected.release_number,
                selected.bus_type,
            )?;
        }
        Err(error) => {
            writeln!(
                output,
                "Configuration interface selection: unavailable ({error})"
            )?;
        }
    }

    output.push_str(
        "Direct RGB codec: implemented and hardware-independent.\nReal packet transport: implemented but not exercised by this command.\nHardware RGB write validation: not performed by dpctl info.\n",
    );

    Ok(output)
}

#[cfg(test)]
mod tests {
    use kd3b_device::{BusType, DiscoveredHidInterface};

    use super::render_info;

    fn interface(number: i32, path: &str) -> DiscoveredHidInterface {
        DiscoveredHidInterface {
            interface_number: number,
            vendor_id: 0x195d,
            product_id: 0x2061,
            product_string: Some("Turing Gaming Keyboard".to_owned()),
            manufacturer_string: Some("Turing Gaming Keyboard".to_owned()),
            serial_number: None,
            release_number: 0x3131,
            bus_type: BusType::Usb,
            path: path.to_owned(),
        }
    }

    #[test]
    fn info_reports_unique_interface_two_without_opening_semantics() {
        let output = render_info(&[
            interface(0, "1-11:1.0"),
            interface(1, "1-11:1.1"),
            interface(2, "1-11:1.2"),
        ])
        .expect("render succeeds");

        assert!(output.contains("Matching HID interfaces: 3\n"));
        assert!(output.contains("Configuration interface selection: unique\n"));
        assert!(output.contains("Interface: 2\nPath: 1-11:1.2\n"));
        assert!(output.contains("does not open the selected interface or request HID reports"));
    }

    #[test]
    fn info_reports_ambiguous_selection_without_guessing() {
        let output = render_info(&[interface(2, "a"), interface(2, "b")])
            .expect("render succeeds");

        assert!(output.contains(
            "Configuration interface selection: unavailable (target HID discovery returned 2 interface-2 records; refusing to choose)\n"
        ));
    }
}
