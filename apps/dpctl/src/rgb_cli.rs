use std::{
    fmt,
    io::{BufRead, Write},
};

use kd3b_device::{
    DiscoveredHidInterface, HidPacketWriteError, OpenConfigurationInterfaceTransportError,
    enumerate_target_hid_interfaces, open_configuration_interface_transport,
    select_configuration_interface,
};
use kd3b_protocol::{ALL_KEYS, Key, LOGICAL_KEY_COUNT, Rgb8};

use crate::CommandOutput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RgbCommand {
    Key { key: Key, color: Rgb8 },
    Solid { color: Rgb8 },
    Off,
}

impl RgbCommand {
    pub(crate) fn parse(arguments: &[String]) -> Result<Self, String> {
        match arguments {
            [mode, key, color] if mode == "key" => {
                let key = parse_key(key).ok_or_else(|| {
                    format!(
                        "unknown KD3B key '{key}'; use an enum-style key name such as F1, A, Space, LeftShift, or PrintScreen"
                    )
                })?;
                let color = parse_color(color)?;
                Ok(Self::Key { key, color })
            }
            [mode, color] if mode == "solid" => Ok(Self::Solid {
                color: parse_color(color)?,
            }),
            [mode] if mode == "off" => Ok(Self::Off),
            _ => Err(
                "RGB usage: 'rgb key <KEY> <RRGGBB>', 'rgb solid <RRGGBB>', or 'rgb off'"
                    .to_owned(),
            ),
        }
    }

    fn frame(self) -> [Rgb8; LOGICAL_KEY_COUNT] {
        match self {
            Self::Key { key, color } => {
                let mut frame = [Rgb8::new(0, 0, 0); LOGICAL_KEY_COUNT];
                frame[key.index()] = color;
                frame
            }
            Self::Solid { color } => [color; LOGICAL_KEY_COUNT],
            Self::Off => [Rgb8::new(0, 0, 0); LOGICAL_KEY_COUNT],
        }
    }

    fn confirmation(self) -> String {
        match self {
            Self::Key { key, color } => {
                format!("WRITE RGB KEY {} {}", key_name(key), color_hex(color))
            }
            Self::Solid { color } => format!("WRITE RGB SOLID {}", color_hex(color)),
            Self::Off => "WRITE RGB OFF".to_owned(),
        }
    }

    fn expected_result(self) -> String {
        match self {
            Self::Key { key, color } => format!(
                "Expected visible result: {} becomes #{}, and every other mapped key becomes black/off.",
                key_name(key),
                color_hex(color)
            ),
            Self::Solid { color } => format!(
                "Expected visible result: all {LOGICAL_KEY_COUNT} mapped keys become #{}.",
                color_hex(color)
            ),
            Self::Off => format!(
                "Expected visible result: all {LOGICAL_KEY_COUNT} mapped keys become black/off."
            ),
        }
    }
}

#[derive(Debug)]
enum RgbWriteError {
    Open(OpenConfigurationInterfaceTransportError),
    SelectionChanged {
        expected: Box<DiscoveredHidInterface>,
        actual: Box<DiscoveredHidInterface>,
    },
    Write(HidPacketWriteError),
}

impl fmt::Display for RgbWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(source) => write!(
                formatter,
                "could not open configuration transport: {source}"
            ),
            Self::SelectionChanged { expected, actual } => write!(
                formatter,
                "selected interface changed after confirmation; expected interface {} at {}, got interface {} at {}; no RGB packet was written",
                expected.interface_number, expected.path, actual.interface_number, actual.path
            ),
            Self::Write(source) => write!(formatter, "direct-RGB write failed: {source}"),
        }
    }
}

fn perform_rgb_write(
    expected: &DiscoveredHidInterface,
    frame: &[Rgb8; LOGICAL_KEY_COUNT],
) -> Result<(), RgbWriteError> {
    let mut transport = open_configuration_interface_transport().map_err(RgbWriteError::Open)?;
    if transport.selected_metadata() != expected {
        return Err(RgbWriteError::SelectionChanged {
            expected: Box::new(expected.clone()),
            actual: Box::new(transport.selected_metadata().clone()),
        });
    }

    transport
        .set_direct_rgb(frame)
        .map_err(RgbWriteError::Write)
}

pub(crate) struct RgbSession<'a, R, W> {
    input: &'a mut R,
    output: &'a mut W,
    is_interactive_stdin: bool,
}

impl<'a, R, W> RgbSession<'a, R, W>
where
    R: BufRead,
    W: Write,
{
    pub(crate) const fn new(
        input: &'a mut R,
        output: &'a mut W,
        is_interactive_stdin: bool,
    ) -> Self {
        Self {
            input,
            output,
            is_interactive_stdin,
        }
    }

    pub(crate) fn run_real(self, command: RgbCommand) -> CommandOutput {
        let frame = command.frame();
        self.run(command, enumerate_target_hid_interfaces, move |selected| {
            perform_rgb_write(selected, &frame)
        })
    }

    fn run<D, F, DE, WE>(self, command: RgbCommand, discover: D, write_rgb: F) -> CommandOutput
    where
        D: FnOnce() -> Result<Vec<DiscoveredHidInterface>, DE>,
        F: FnOnce(&DiscoveredHidInterface) -> Result<(), WE>,
        DE: fmt::Display,
        WE: fmt::Display,
    {
        if !self.is_interactive_stdin {
            return failure("RGB hardware write requires interactive terminal stdin");
        }

        let interfaces = match discover() {
            Ok(interfaces) => interfaces,
            Err(error) => {
                return failure(&format!("could not enumerate target interfaces: {error}"));
            }
        };
        let selected_index = match select_configuration_interface(&interfaces) {
            Ok(selected_index) => selected_index,
            Err(error) => {
                return failure(&format!(
                    "could not select configuration interface: {error}"
                ));
            }
        };
        let selected = interfaces[selected_index.get()].clone();

        if let Err(error) = write_disclosure(self.output, &selected, command) {
            return failure(&format!("could not write RGB safety disclosure: {error}"));
        }

        let mut confirmation = String::new();
        let bytes_read = match self.input.read_line(&mut confirmation) {
            Ok(bytes_read) => bytes_read,
            Err(error) => return failure(&format!("could not read RGB confirmation: {error}")),
        };
        if bytes_read == 0 {
            return failure(
                "confirmation input ended before a line was read; no RGB packet was written",
            );
        }
        let confirmation = confirmation.strip_suffix('\n').unwrap_or(&confirmation);
        let confirmation = confirmation.strip_suffix('\r').unwrap_or(confirmation);
        if confirmation != command.confirmation() {
            return failure("confirmation did not exactly match; no RGB packet was written");
        }

        if let Err(error) = write_rgb(&selected) {
            return failure(&format!("RGB operation did not complete: {error}"));
        }

        if let Err(error) = writeln!(
            self.output,
            "HIDAPI accepted the complete encoded direct-RGB packet pair. Verify the keyboard visually.\n{}",
            command.expected_result()
        ) {
            return failure(&format!(
                "RGB operation completed, but the success message could not be written: {error}"
            ));
        }

        CommandOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

fn write_disclosure<W: Write>(
    output: &mut W,
    metadata: &DiscoveredHidInterface,
    command: RgbCommand,
) -> Result<(), std::io::Error> {
    let expected_result = command.expected_result();
    let confirmation = command.confirmation();
    write!(
        output,
        "DIRECT RGB HARDWARE WRITE\nSelected configuration interface:\nInterface: {}\nVID:PID: {:04x}:{:04x}\nPath: {}\nRelease: 0x{:04x}\nBus: {}\nThis is a volatile lighting operation. After confirmation, dpctl will re-enumerate and open the same unique interface-2 metadata record, abort if that record changed, then request exactly two 256-byte HIDAPI writes in packet-A then packet-B order. No retry or fallback is performed. No profile, EEPROM, macro, remap, firmware, or bootloader operation is requested.\n{expected_result}\nType {confirmation} and press Enter to perform the write: ",
        metadata.interface_number,
        metadata.vendor_id,
        metadata.product_id,
        metadata.path,
        metadata.release_number,
        metadata.bus_type,
    )?;
    output.flush()
}

fn parse_key(input: &str) -> Option<Key> {
    ALL_KEYS
        .iter()
        .copied()
        .find(|key| format!("{key:?}").eq_ignore_ascii_case(input))
}

fn parse_color(input: &str) -> Result<Rgb8, String> {
    if input.len() != 6 || !input.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "invalid RGB color '{input}'; expected exactly six hexadecimal digits RRGGBB"
        ));
    }

    let red = u8::from_str_radix(&input[0..2], 16).expect("validated hexadecimal color");
    let green = u8::from_str_radix(&input[2..4], 16).expect("validated hexadecimal color");
    let blue = u8::from_str_radix(&input[4..6], 16).expect("validated hexadecimal color");
    Ok(Rgb8::new(red, green, blue))
}

fn key_name(key: Key) -> String {
    format!("{key:?}")
}

fn color_hex(color: Rgb8) -> String {
    format!("{:02X}{:02X}{:02X}", color.red, color.green, color.blue)
}

fn failure(message: &str) -> CommandOutput {
    CommandOutput {
        exit_code: 1,
        stdout: String::new(),
        stderr: format!("error: {message}\n"),
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, io::Cursor};

    use kd3b_device::{BusType, DiscoveredHidInterface};
    use kd3b_protocol::{Key, LOGICAL_KEY_COUNT, Rgb8};

    use super::{RgbCommand, RgbSession, parse_color, parse_key};

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

    fn unique_interfaces() -> Vec<DiscoveredHidInterface> {
        vec![
            interface(0, "1-11:1.0"),
            interface(1, "1-11:1.1"),
            interface(2, "1-11:1.2"),
        ]
    }

    #[test]
    fn key_parser_accepts_catalogue_names_case_insensitively() {
        assert_eq!(parse_key("F1"), Some(Key::F1));
        assert_eq!(parse_key("f12"), Some(Key::F12));
        assert_eq!(parse_key("space"), Some(Key::Space));
        assert_eq!(parse_key("LeftShift"), Some(Key::LeftShift));
        assert_eq!(parse_key("printscreen"), Some(Key::PrintScreen));
        assert_eq!(parse_key("not-a-key"), None);
    }

    #[test]
    fn color_parser_requires_exact_rrggbb() {
        assert_eq!(parse_color("00ff7F"), Ok(Rgb8::new(0, 255, 127)));
        assert!(parse_color("#00ff7f").is_err());
        assert!(parse_color("fff").is_err());
        assert!(parse_color("gg0000").is_err());
    }

    #[test]
    fn key_command_builds_one_lit_key_frame() {
        let command = RgbCommand::parse(&[
            "key".to_owned(),
            "F1".to_owned(),
            "ff0000".to_owned(),
        ])
        .expect("valid key command");
        let frame = command.frame();

        assert_eq!(frame[Key::F1.index()], Rgb8::new(255, 0, 0));
        assert_eq!(
            frame
                .iter()
                .filter(|color| **color != Rgb8::new(0, 0, 0))
                .count(),
            1
        );
        assert_eq!(command.confirmation(), "WRITE RGB KEY F1 FF0000");
    }

    #[test]
    fn solid_and_off_commands_build_complete_frames() {
        let solid = RgbCommand::parse(&["solid".to_owned(), "123456".to_owned()])
            .expect("valid solid command");
        let off = RgbCommand::parse(&["off".to_owned()]).expect("valid off command");

        assert_eq!(solid.frame(), [Rgb8::new(0x12, 0x34, 0x56); LOGICAL_KEY_COUNT]);
        assert_eq!(off.frame(), [Rgb8::new(0, 0, 0); LOGICAL_KEY_COUNT]);
        assert_eq!(solid.confirmation(), "WRITE RGB SOLID 123456");
        assert_eq!(off.confirmation(), "WRITE RGB OFF");
    }

    #[test]
    fn non_interactive_input_fails_before_discovery_or_write() {
        let discovered = Cell::new(false);
        let written = Cell::new(false);
        let mut input = Cursor::new(b"WRITE RGB OFF\n".to_vec());
        let mut output = Vec::new();

        let result = RgbSession::new(&mut input, &mut output, false).run(
            RgbCommand::Off,
            || {
                discovered.set(true);
                Ok::<_, &str>(unique_interfaces())
            },
            |_| {
                written.set(true);
                Ok::<_, &str>(())
            },
        );

        assert_eq!(result.exit_code, 1);
        assert!(!discovered.get());
        assert!(!written.get());
    }

    #[test]
    fn wrong_confirmation_never_calls_writer() {
        let written = Cell::new(false);
        let mut input = Cursor::new(b"NO\n".to_vec());
        let mut output = Vec::new();

        let result = RgbSession::new(&mut input, &mut output, true).run(
            RgbCommand::Off,
            || Ok::<_, &str>(unique_interfaces()),
            |_| {
                written.set(true);
                Ok::<_, &str>(())
            },
        );

        assert_eq!(result.exit_code, 1);
        assert!(!written.get());
        assert!(result.stderr.contains("no RGB packet was written"));
    }

    #[test]
    fn exact_confirmation_calls_writer_once_with_selected_metadata() {
        let calls = Cell::new(0);
        let mut input = Cursor::new(b"WRITE RGB SOLID 112233\n".to_vec());
        let mut output = Vec::new();
        let command = RgbCommand::Solid {
            color: Rgb8::new(0x11, 0x22, 0x33),
        };

        let result = RgbSession::new(&mut input, &mut output, true).run(
            command,
            || Ok::<_, &str>(unique_interfaces()),
            |selected| {
                calls.set(calls.get() + 1);
                assert_eq!(selected.path, "1-11:1.2");
                Ok::<_, &str>(())
            },
        );

        assert_eq!(result.exit_code, 0);
        assert_eq!(calls.get(), 1);
        let output = String::from_utf8(output).expect("test output is UTF-8");
        assert!(output.contains("exactly two 256-byte HIDAPI writes"));
        assert!(output.contains("all 87 mapped keys become #112233"));
    }

    #[test]
    fn ambiguous_interface_two_fails_before_writer() {
        let written = Cell::new(false);
        let mut input = Cursor::new(b"WRITE RGB OFF\n".to_vec());
        let mut output = Vec::new();

        let result = RgbSession::new(&mut input, &mut output, true).run(
            RgbCommand::Off,
            || Ok::<_, &str>(vec![interface(2, "a"), interface(2, "b")]),
            |_| {
                written.set(true);
                Ok::<_, &str>(())
            },
        );

        assert_eq!(result.exit_code, 1);
        assert!(!written.get());
        assert!(result.stderr.contains("refusing to choose"));
    }

    #[test]
    fn writer_failure_is_reported_without_claiming_success() {
        let mut input = Cursor::new(b"WRITE RGB OFF\n".to_vec());
        let mut output = Vec::new();

        let result = RgbSession::new(&mut input, &mut output, true).run(
            RgbCommand::Off,
            || Ok::<_, &str>(unique_interfaces()),
            |_| Err::<(), _>("synthetic write failure"),
        );

        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("synthetic write failure"));
        let output = String::from_utf8(output).expect("test output is UTF-8");
        assert!(!output.contains("HIDAPI accepted the complete"));
    }
}
