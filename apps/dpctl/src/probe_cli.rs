use std::{
    fmt,
    io::{BufRead, Write},
};

use kd3b_device::{
    ConfigurationInterfaceOpenError, DiscoveredHidInterface, PreparedConfigurationInterfaceProbe,
};

use crate::CommandOutput;

const CONFIRMATION: &str = "OPEN INTERFACE 2";
const ENUMERATION_DISCLOSURE: &str = "Enumeration disclosure: preparing this probe causes HIDAPI/libusb to enumerate metadata for all visible HID devices before target filtering; the backend may open visible HID devices while collecting metadata.\n";
const OPEN_DISCLOSURE: &str = "Selected-open disclosure: after confirmation, HIDAPI open_path may internally rescan and path-match before opening; it may claim/release the interface, may detach/reattach a kernel driver where applicable, and may establish backend-managed interrupt-IN activity where applicable. These are possible backend effects, not claims that they occurred.\n";
const SUCCESS: &str = "No HID report read/write/feature operation was requested.\nThis statement covers application-level HID report APIs only; the HIDAPI/libusb backend may exhibit the disclosed metadata/open/interface/interrupt behavior where applicable.\n";

pub(crate) trait ProbeOperation {
    type OpenError: fmt::Display;

    fn selected_metadata(&self) -> &DiscoveredHidInterface;
    fn open_and_drop(self) -> Result<(), Self::OpenError>;
}

impl ProbeOperation for PreparedConfigurationInterfaceProbe {
    type OpenError = ConfigurationInterfaceOpenError;

    fn selected_metadata(&self) -> &DiscoveredHidInterface {
        self.selected_metadata()
    }

    fn open_and_drop(self) -> Result<(), Self::OpenError> {
        self.open_and_drop().map(|_| ())
    }
}

pub(crate) struct ProbeSession<'a, R, W> {
    input: &'a mut R,
    output: &'a mut W,
    is_interactive_stdin: bool,
}

impl<'a, R, W> ProbeSession<'a, R, W>
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

    pub(crate) fn run<P, F, E>(self, prepare: F) -> CommandOutput
    where
        P: ProbeOperation,
        F: FnOnce() -> Result<P, E>,
        E: fmt::Display,
    {
        if !self.is_interactive_stdin {
            return failure("probe requires interactive terminal stdin");
        }

        let prepared = match prepare() {
            Ok(prepared) => prepared,
            Err(error) => {
                return failure(&format!(
                    "could not prepare configuration-interface probe: {error}"
                ));
            }
        };

        if let Err(error) = write_disclosure(self.output, prepared.selected_metadata()) {
            return failure(&format!("could not write probe disclosure: {error}"));
        }

        let mut confirmation = String::new();
        let bytes_read = match self.input.read_line(&mut confirmation) {
            Ok(bytes_read) => bytes_read,
            Err(error) => return failure(&format!("could not read probe confirmation: {error}")),
        };
        if bytes_read == 0 {
            return failure(
                "confirmation input ended before a line was read; interface was not opened",
            );
        }
        let confirmation = confirmation.strip_suffix('\n').unwrap_or(&confirmation);
        let confirmation = confirmation.strip_suffix('\r').unwrap_or(confirmation);
        if confirmation != CONFIRMATION {
            return failure("confirmation did not exactly match; interface was not opened");
        }

        if let Err(error) = prepared.open_and_drop() {
            return failure(&format!(
                "could not open selected configuration interface: {error}"
            ));
        }
        if let Err(error) = self.output.write_all(SUCCESS.as_bytes()) {
            return failure(&format!("could not write probe result: {error}"));
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
        "Selected configuration interface:\nInterface: {}\nVID:PID: {:04x}:{:04x}\nProduct: {}\nManufacturer: {}\nSerial: {}\nRelease: 0x{:04x}\nBus: {}\nPath: {}\n{ENUMERATION_DISCLOSURE}{OPEN_DISCLOSURE}Type {CONFIRMATION} and press Enter to continue: ",
        metadata.interface_number,
        metadata.vendor_id,
        metadata.product_id,
        metadata
            .product_string
            .as_deref()
            .unwrap_or("<unavailable>"),
        metadata
            .manufacturer_string
            .as_deref()
            .unwrap_or("<unavailable>"),
        metadata.serial_number.as_deref().unwrap_or("<unavailable>"),
        metadata.release_number,
        metadata.bus_type,
        metadata.path,
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
