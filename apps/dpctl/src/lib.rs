use std::{
    fmt,
    io::{self, IsTerminal},
};

use kd3b_device::{
    DiscoveredHidInterface, enumerate_target_hid_interfaces, prepare_configuration_interface_probe,
};

use devices_cli::render_interfaces;
use info_cli::render_info;
use probe_cli::ProbeSession;
use rgb_cli::RgbSession;

mod devices_cli;
mod info_cli;
mod probe_cli;
#[cfg(test)]
mod probe_cli_tests;
mod rgb_cli;

const USAGE: &str = "Usage: dpctl <devices|info|probe|rgb key F1 ff0000>\n";
const HELP: &str = "Usage: dpctl <devices|info|probe|rgb key F1 ff0000>\n\nCommands:\n  devices             Enumerate read-only HID metadata for Dark Project KD3B rev.2 (195d:2061).\n  info                Summarize current read-only target/interface status.\n  probe               Interactively prepare and open interface 2 without requesting HID report operations.\n  rgb key F1 ff0000   Hardware-gated first direct-RGB validation: F1 red, all other mapped keys black/off.\n";

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
    Info,
    Probe,
    RgbKeyF1Red,
    UsageError(String),
}

/// Dispatches arguments through the real device integrations selected by the command.
pub fn run<I, S>(arguments: I) -> CommandOutput
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    match parse_arguments(arguments) {
        Command::Probe => {
            let stdin = io::stdin();
            let is_interactive_stdin = stdin.is_terminal();
            let mut input = stdin.lock();
            let stdout = io::stdout();
            let mut output = stdout.lock();
            ProbeSession::new(&mut input, &mut output, is_interactive_stdin)
                .run(prepare_configuration_interface_probe)
        }
        Command::RgbKeyF1Red => {
            let stdin = io::stdin();
            let is_interactive_stdin = stdin.is_terminal();
            let mut input = stdin.lock();
            let stdout = io::stdout();
            let mut output = stdout.lock();
            RgbSession::new(&mut input, &mut output, is_interactive_stdin).run_real()
        }
        command => run_non_interactive(command, enumerate_target_hid_interfaces),
    }
}

/// Dispatches non-hardware-write arguments with an injected metadata enumerator for tests.
pub fn run_with_discovery<I, S, F, E>(arguments: I, discover: F) -> CommandOutput
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    F: FnOnce() -> Result<Vec<DiscoveredHidInterface>, E>,
    E: fmt::Display,
{
    run_non_interactive(parse_arguments(arguments), discover)
}

fn run_non_interactive<F, E>(command: Command, discover: F) -> CommandOutput
where
    F: FnOnce() -> Result<Vec<DiscoveredHidInterface>, E>,
    E: fmt::Display,
{
    match command {
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
            Err(error) => discovery_failure(error),
        },
        Command::Info => match discover() {
            Ok(interfaces) => match render_info(&interfaces) {
                Ok(stdout) => CommandOutput {
                    exit_code: 0,
                    stdout,
                    stderr: String::new(),
                },
                Err(error) => CommandOutput {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: format!("error: could not render KD3B status: {error}\n"),
                },
            },
            Err(error) => discovery_failure(error),
        },
        Command::Probe => interactive_dispatch_required("probe"),
        Command::RgbKeyF1Red => interactive_dispatch_required("RGB hardware write"),
        Command::UsageError(error) => CommandOutput {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("error: {error}\n{USAGE}"),
        },
    }
}

fn discovery_failure(error: impl fmt::Display) -> CommandOutput {
    CommandOutput {
        exit_code: 1,
        stdout: String::new(),
        stderr: format!(
            "error: could not enumerate KD3B HID interfaces: {error}\ndpctl made no application-level HID handle or report request. The approved HIDAPI/libusb backend may open visible HID devices while collecting metadata.\n"
        ),
    }
}

fn interactive_dispatch_required(operation: &str) -> CommandOutput {
    CommandOutput {
        exit_code: 2,
        stdout: String::new(),
        stderr: format!("error: {operation} requires the interactive dispatcher\n{USAGE}"),
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

    if command == "rgb" {
        let mode = arguments.next();
        let key = arguments.next();
        let color = arguments.next();
        let extra = arguments.next();
        if mode.as_ref().map(AsRef::as_ref) == Some("key")
            && key.as_ref().map(AsRef::as_ref) == Some("F1")
            && color.as_ref().map(AsRef::as_ref) == Some("ff0000")
            && extra.is_none()
        {
            return Command::RgbKeyF1Red;
        }

        return Command::UsageError(
            "first hardware-gated RGB validation currently accepts exactly 'rgb key F1 ff0000'"
                .to_owned(),
        );
    }

    let extra = arguments.next();
    match (command, extra) {
        ("--help" | "-h" | "help", None) => Command::Help,
        ("devices", None) => Command::Devices,
        ("info", None) => Command::Info,
        ("probe", None) => Command::Probe,
        ("devices", Some(extra)) => Command::UsageError(format!(
            "unexpected argument '{}' after 'devices'",
            extra.as_ref()
        )),
        ("info", Some(extra)) => Command::UsageError(format!(
            "unexpected argument '{}' after 'info'",
            extra.as_ref()
        )),
        ("probe", Some(extra)) => Command::UsageError(format!(
            "unexpected argument '{}' after 'probe'",
            extra.as_ref()
        )),
        (unknown, _) => Command::UsageError(format!("unknown command '{unknown}'")),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use kd3b_device::{BusType, DiscoveredHidInterface};

    use super::{Command, HELP, parse_arguments, run_with_discovery};

    #[test]
    fn gated_rgb_command_is_recognized_only_in_exact_first_write_form() {
        assert!(matches!(
            parse_arguments(["rgb", "key", "F1", "ff0000"]),
            Command::RgbKeyF1Red
        ));
        for invalid in [
            vec!["rgb"],
            vec!["rgb", "key", "F1"],
            vec!["rgb", "key", "F1", "FF0000"],
            vec!["rgb", "key", "F2", "ff0000"],
            vec!["rgb", "key", "F1", "ff0000", "extra"],
        ] {
            assert!(matches!(
                parse_arguments(invalid),
                Command::UsageError(_)
            ));
        }
    }

    #[test]
    fn info_and_probe_are_recognized_only_without_extra_arguments() {
        let info = parse_arguments(["info"]);
        let info_with_extra = parse_arguments(["info", "extra"]);
        let probe = parse_arguments(["probe"]);
        let probe_with_extra = parse_arguments(["probe", "extra"]);

        assert!(matches!(info, Command::Info));
        assert!(matches!(info_with_extra, Command::UsageError(_)));
        assert!(matches!(probe, Command::Probe));
        assert!(matches!(probe_with_extra, Command::UsageError(_)));
    }

    #[test]
    fn help_succeeds_without_discovery_when_help_form_is_used() {
        for arguments in [["--help"], ["-h"], ["help"]] {
            let discovery_called = Cell::new(false);

            let output = run_with_discovery(arguments, || {
                discovery_called.set(true);
                Ok::<_, &str>(Vec::new())
            });

            assert_eq!(output.exit_code, 0);
            assert_eq!(output.stdout, HELP);
            assert_eq!(output.stderr, "");
            assert!(!discovery_called.get());
        }
    }

    #[test]
    fn usage_fails_without_discovery_when_arguments_are_invalid() {
        let cases: &[(&[&str], &str)] = &[
            (
                &[],
                "error: missing command\nUsage: dpctl <devices|info|probe|rgb key F1 ff0000>\n",
            ),
            (
                &["unknown"],
                "error: unknown command 'unknown'\nUsage: dpctl <devices|info|probe|rgb key F1 ff0000>\n",
            ),
            (
                &["devices", "extra"],
                "error: unexpected argument 'extra' after 'devices'\nUsage: dpctl <devices|info|probe|rgb key F1 ff0000>\n",
            ),
            (
                &["info", "extra"],
                "error: unexpected argument 'extra' after 'info'\nUsage: dpctl <devices|info|probe|rgb key F1 ff0000>\n",
            ),
            (
                &["probe", "extra"],
                "error: unexpected argument 'extra' after 'probe'\nUsage: dpctl <devices|info|probe|rgb key F1 ff0000>\n",
            ),
        ];

        for (arguments, expected_stderr) in cases {
            let discovery_called = Cell::new(false);

            let output = run_with_discovery(arguments.iter().copied(), || {
                discovery_called.set(true);
                Ok::<_, &str>(Vec::new())
            });

            assert_eq!(output.exit_code, 2);
            assert_eq!(output.stdout, "");
            assert_eq!(output.stderr, *expected_stderr);
            assert!(!discovery_called.get());
        }
    }

    #[test]
    fn devices_reports_zero_matches_when_discovery_is_empty() {
        let expected = "Target: Dark Project KD3B rev.2 (VID:PID 195d:2061)\nMatches: 0\nNo matching HID interfaces found.\n";

        let output = run_with_discovery(["devices"], || Ok::<_, &str>(Vec::new()));

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, expected);
        assert_eq!(output.stderr, "");
    }

    #[test]
    fn info_reports_current_unique_candidate_without_opening_it() {
        let interfaces = vec![
            interface(0, "1-11:1.0"),
            interface(1, "1-11:1.1"),
            interface(2, "1-11:1.2"),
        ];

        let output = run_with_discovery(["info"], || Ok::<_, &str>(interfaces));

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("Matching HID interfaces: 3\n"));
        assert!(
            output
                .stdout
                .contains("Configuration interface selection: unique\n")
        );
        assert!(output.stdout.contains("Interface: 2\nPath: 1-11:1.2\n"));
        assert_eq!(output.stderr, "");
    }

    #[test]
    fn devices_renders_every_record_in_source_order_when_matches_exist() {
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
            "Path: /dev/hidraw2\nCandidate: unvalidated configuration-interface candidate; no application-level HID handle/report request by dpctl; HIDAPI/libusb may open visible HID devices for metadata; transport/writability not validated\n",
            "Interface: -1 (not reported)\nVID:PID: 195d:2061\nProduct: <unavailable>\n",
            "Manufacturer: Dark Project\nSerial: <unavailable>\nRelease: 0x0000\nBus: bluetooth\n",
            "Path: /safe\\path\\x1b\nCandidate: none\n",
        );

        let output = run_with_discovery(["devices"], || Ok::<_, &str>(interfaces));

        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, expected);
        assert_eq!(output.stderr, "");
    }

    #[test]
    fn devices_reports_safety_context_when_discovery_fails() {
        let expected = concat!(
            "error: could not enumerate KD3B HID interfaces: test enumeration failure\n",
            "dpctl made no application-level HID handle or report request. The approved HIDAPI/libusb backend may open visible HID devices while collecting metadata.\n",
        );

        let output = run_with_discovery(["devices"], || {
            Err::<Vec<DiscoveredHidInterface>, _>("test enumeration failure")
        });

        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stdout, "");
        assert_eq!(output.stderr, expected);
        assert_eq!(
            output.stderr.lines().nth(1),
            Some(
                "dpctl made no application-level HID handle or report request. The approved HIDAPI/libusb backend may open visible HID devices while collecting metadata."
            )
        );
    }

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
}
