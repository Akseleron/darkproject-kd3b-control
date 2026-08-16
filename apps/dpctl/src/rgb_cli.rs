use std::{
    fmt,
    io::{BufRead, Write},
};

use kd3b_device::{
    DiscoveredHidInterface, HidPacketWriteError, OpenConfigurationInterfaceTransportError,
    enumerate_target_hid_interfaces, open_configuration_interface_transport,
    select_configuration_interface,
};
use kd3b_protocol::{Key, LOGICAL_KEY_COUNT, Rgb8};

use crate::CommandOutput;

const CONFIRMATION: &str = "WRITE F1 FF0000";
const EXPECTED_RESULT: &str = "Expected visible result if the documented direct-RGB behavior matches this keyboard: F1 becomes red and every other mapped key becomes black/off.";

#[derive(Debug)]
enum FirstRgbWriteError {
    Open(OpenConfigurationInterfaceTransportError),
    SelectionChanged {
        expected: Box<DiscoveredHidInterface>,
        actual: Box<DiscoveredHidInterface>,
    },
    Write(HidPacketWriteError),
}

impl fmt::Display for FirstRgbWriteError {
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

fn perform_f1_red_write(expected: &DiscoveredHidInterface) -> Result<(), FirstRgbWriteError> {
    let mut transport =
        open_configuration_interface_transport().map_err(FirstRgbWriteError::Open)?;
    if transport.selected_metadata() != expected {
        return Err(FirstRgbWriteError::SelectionChanged {
            expected: Box::new(expected.clone()),
            actual: Box::new(transport.selected_metadata().clone()),
        });
    }

    let mut frame = [Rgb8::new(0, 0, 0); LOGICAL_KEY_COUNT];
    frame[Key::F1.index()] = Rgb8::new(255, 0, 0);
    transport
        .set_direct_rgb(&frame)
        .map_err(FirstRgbWriteError::Write)
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

    pub(crate) fn run_real(self) -> CommandOutput {
        self.run(enumerate_target_hid_interfaces, perform_f1_red_write)
    }

    fn run<D, F, DE, WE>(self, discover: D, write_rgb: F) -> CommandOutput
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

        if let Err(error) = write_disclosure(self.output, &selected) {
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
        if confirmation != CONFIRMATION {
            return failure("confirmation did not exactly match; no RGB packet was written");
        }

        if let Err(error) = write_rgb(&selected) {
            return failure(&format!("RGB operation did not complete: {error}"));
        }

        if let Err(error) = writeln!(
            self.output,
            "HIDAPI accepted the complete encoded direct-RGB packet pair. Verify the keyboard visually.\n{EXPECTED_RESULT}"
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
) -> Result<(), std::io::Error> {
    write!(
        output,
        "FIRST HARDWARE RGB WRITE\nSelected configuration interface:\nInterface: {}\nVID:PID: {:04x}:{:04x}\nPath: {}\nRelease: 0x{:04x}\nBus: {}\nThis is a volatile lighting operation. After confirmation, dpctl will re-enumerate and open the same unique interface-2 metadata record, abort if that record changed, then request exactly two 256-byte HIDAPI writes in packet-A then packet-B order. No retry or fallback is performed. No profile, EEPROM, macro, remap, firmware, or bootloader operation is requested.\n{EXPECTED_RESULT}\nType {CONFIRMATION} and press Enter to perform the write: ",
        metadata.interface_number,
        metadata.vendor_id,
        metadata.product_id,
        metadata.path,
        metadata.release_number,
        metadata.bus_type,
    )?;
    output.flush()
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
    use std::{
        cell::{Cell, RefCell},
        io::Cursor,
        rc::Rc,
    };

    use kd3b_device::{BusType, DiscoveredHidInterface};

    use super::RgbSession;

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
    fn non_interactive_input_fails_before_discovery_or_write() {
        let discovered = Cell::new(false);
        let written = Cell::new(false);
        let mut input = Cursor::new(b"WRITE F1 FF0000\n".to_vec());
        let mut output = Vec::new();

        let result = RgbSession::new(&mut input, &mut output, false).run(
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
        let calls = Rc::new(Cell::new(0));
        let selected_paths = Rc::new(RefCell::new(Vec::new()));
        let calls_for_writer = Rc::clone(&calls);
        let paths_for_writer = Rc::clone(&selected_paths);
        let mut input = Cursor::new(b"WRITE F1 FF0000\n".to_vec());
        let mut output = Vec::new();

        let result = RgbSession::new(&mut input, &mut output, true).run(
            || Ok::<_, &str>(unique_interfaces()),
            move |selected| {
                calls_for_writer.set(calls_for_writer.get() + 1);
                paths_for_writer.borrow_mut().push(selected.path.clone());
                Ok::<_, &str>(())
            },
        );

        assert_eq!(result.exit_code, 0);
        assert_eq!(calls.get(), 1);
        assert_eq!(selected_paths.borrow().as_slice(), ["1-11:1.2"]);
        let output = String::from_utf8(output).expect("test output is UTF-8");
        assert!(output.contains("exactly two 256-byte HIDAPI writes"));
        assert!(output.contains("F1 becomes red"));
    }

    #[test]
    fn ambiguous_interface_two_fails_before_writer() {
        let written = Cell::new(false);
        let mut input = Cursor::new(b"WRITE F1 FF0000\n".to_vec());
        let mut output = Vec::new();

        let result = RgbSession::new(&mut input, &mut output, true).run(
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
        let mut input = Cursor::new(b"WRITE F1 FF0000\n".to_vec());
        let mut output = Vec::new();

        let result = RgbSession::new(&mut input, &mut output, true).run(
            || Ok::<_, &str>(unique_interfaces()),
            |_| Err::<(), _>("synthetic write failure"),
        );

        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("synthetic write failure"));
        let output = String::from_utf8(output).expect("test output is UTF-8");
        assert!(!output.contains("HIDAPI accepted the complete"));
    }
}
