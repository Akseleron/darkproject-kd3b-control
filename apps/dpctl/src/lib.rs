use std::fmt::{self, Write as _};

use kd3b_device::{DiscoveredHidInterface, enumerate_target_hid_interfaces};

const USAGE: &str = "Usage: dpctl devices\n";
const HELP: &str = "Usage: dpctl devices\n\nEnumerate read-only HID metadata for Dark Project KD3B rev.2 (195d:2061).\n";
const TARGET_HEADER: &str = "Target: Dark Project KD3B rev.2 (VID:PID 195d:2061)\n";
const CANDIDATE: &str = "unvalidated configuration-interface candidate; not opened by dpctl; transport/writability not validated";

/// Complete process output for one `dpctl` invocation.
#[derive(Debug, PartialEq, Eq)]
pub struct CommandOutput {
    pub exit_code: u8,
    pub stdout: String,
    pub stderr: String,
}

enum Command {
    Help,
    Devices,
    UsageError(String),
}

/// Dispatches arguments through the real read-only device metadata enumerator.
pub fn run<I, S>(arguments: I) -> CommandOutput
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    run_with_discovery(arguments, enumerate_target_hid_interfaces)
}

/// Dispatches arguments with an injected metadata enumerator for offline callers and tests.
pub fn run_with_discovery<I, S, F, E>(arguments: I, discover: F) -> CommandOutput
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    F: FnOnce() -> Result<Vec<DiscoveredHidInterface>, E>,
    E: fmt::Display,
{
    match parse_arguments(arguments) {
        Command::Help => CommandOutput {
            exit_code: 0,
            stdout: HELP.to_owned(),
            stderr: String::new(),
        },
        Command::Devices => match discover() {
            Ok(interfaces) => match render_interfaces(&interfaces) {
                Ok(stdout) => CommandOutput {
                    exit_code: 0,
                    stdout,
                    stderr: String::new(),
                },
                Err(error) => CommandOutput {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: format!("error: could not render KD3B HID interfaces: {error}\n"),
                },
            },
            Err(error) => CommandOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: format!(
                    "error: could not enumerate KD3B HID interfaces: {error}\nNo HID device handle was opened by dpctl and no read/write/report operation was requested.\n"
                ),
            },
        },
        Command::UsageError(error) => CommandOutput {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("error: {error}\n{USAGE}"),
        },
    }
}

fn parse_arguments<I, S>(arguments: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut arguments = arguments.into_iter();
    let Some(command) = arguments.next() else {
        return Command::UsageError("missing command".to_owned());
    };
    let command = command.as_ref();

    match (command, arguments.next()) {
        ("--help" | "-h" | "help", None) => Command::Help,
        ("devices", None) => Command::Devices,
        ("devices", Some(extra)) => Command::UsageError(format!(
            "unexpected argument '{}' after 'devices'",
            extra.as_ref()
        )),
        (unknown, _) => Command::UsageError(format!("unknown command '{unknown}'")),
    }
}

fn render_interfaces(interfaces: &[DiscoveredHidInterface]) -> Result<String, fmt::Error> {
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

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use kd3b_device::{BusType, DiscoveredHidInterface};

    use super::{HELP, run_with_discovery};

    #[test]
    fn help_succeeds_without_discovery_when_help_form_is_used() {
        // Given
        for arguments in [["--help"], ["-h"], ["help"]] {
            let discovery_called = Cell::new(false);

            // When
            let output = run_with_discovery(arguments, || {
                discovery_called.set(true);
                Ok::<_, &str>(Vec::new())
            });

            // Then
            assert_eq!(output.exit_code, 0);
            assert_eq!(output.stdout, HELP);
            assert_eq!(output.stderr, "");
            assert!(!discovery_called.get());
        }
    }

    #[test]
    fn usage_fails_without_discovery_when_arguments_are_invalid() {
        // Given
        let cases: &[(&[&str], &str)] = &[
            (&[], "error: missing command\nUsage: dpctl devices\n"),
            (
                &["unknown"],
                "error: unknown command 'unknown'\nUsage: dpctl devices\n",
            ),
            (
                &["unknown", "extra"],
                "error: unknown command 'unknown'\nUsage: dpctl devices\n",
            ),
            (
                &["devices", "extra"],
                "error: unexpected argument 'extra' after 'devices'\nUsage: dpctl devices\n",
            ),
        ];

        for (arguments, expected_stderr) in cases {
            let discovery_called = Cell::new(false);

            // When
            let output = run_with_discovery(arguments.iter().copied(), || {
                discovery_called.set(true);
                Ok::<_, &str>(Vec::new())
            });

            // Then
            assert_eq!(output.exit_code, 2);
            assert_eq!(output.stdout, "");
            assert_eq!(output.stderr, *expected_stderr);
            assert!(!discovery_called.get());
        }
    }

    #[test]
    fn devices_reports_zero_matches_when_discovery_is_empty() {
        // Given
        let expected = "Target: Dark Project KD3B rev.2 (VID:PID 195d:2061)\nMatches: 0\nNo matching HID interfaces found.\n";

        // When
        let output = run_with_discovery(["devices"], || Ok::<_, &str>(Vec::new()));

        // Then
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, expected);
        assert_eq!(output.stderr, "");
    }

    #[test]
    fn devices_renders_every_record_in_source_order_when_matches_exist() {
        // Given
        let interfaces = vec![
            DiscoveredHidInterface {
                interface_number: 2,
                vendor_id: 0x195d,
                product_id: 0x2061,
                product_string: Some("Turing Gaming Keyboard".to_owned()),
                manufacturer_string: None,
                serial_number: Some("serial-a".to_owned()),
                release_number: 0x010a,
                bus_type: BusType::Usb,
                path: "/dev/hidraw2".to_owned(),
            },
            DiscoveredHidInterface {
                interface_number: -1,
                vendor_id: 0x195d,
                product_id: 0x2061,
                product_string: None,
                manufacturer_string: Some("Dark Project".to_owned()),
                serial_number: None,
                release_number: 0x0000,
                bus_type: BusType::Bluetooth,
                path: "/safe\\path\\x1b".to_owned(),
            },
        ];
        let expected = concat!(
            "Target: Dark Project KD3B rev.2 (VID:PID 195d:2061)\nMatches: 2\n",
            "Interface: 2\nVID:PID: 195d:2061\nProduct: Turing Gaming Keyboard\n",
            "Manufacturer: <unavailable>\nSerial: serial-a\nRelease: 0x010a\nBus: usb\n",
            "Path: /dev/hidraw2\nCandidate: unvalidated configuration-interface candidate; not opened by dpctl; transport/writability not validated\n",
            "Interface: -1 (not reported)\nVID:PID: 195d:2061\nProduct: <unavailable>\n",
            "Manufacturer: Dark Project\nSerial: <unavailable>\nRelease: 0x0000\nBus: bluetooth\n",
            "Path: /safe\\path\\x1b\nCandidate: none\n",
        );

        // When
        let output = run_with_discovery(["devices"], || Ok::<_, &str>(interfaces));

        // Then
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, expected);
        assert_eq!(output.stderr, "");
    }

    #[test]
    fn devices_reports_safety_context_when_discovery_fails() {
        // Given
        let expected = concat!(
            "error: could not enumerate KD3B HID interfaces: test enumeration failure\n",
            "No HID device handle was opened by dpctl and no read/write/report operation was requested.\n",
        );

        // When
        let output = run_with_discovery(["devices"], || {
            Err::<Vec<DiscoveredHidInterface>, _>("test enumeration failure")
        });

        // Then
        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, expected);
        assert_eq!(
            output.stderr.lines().nth(1),
            Some(
                "No HID device handle was opened by dpctl and no read/write/report operation was requested."
            )
        );
    }
}
