use std::{error::Error, fmt};

use hidapi::{HidApi, HidDevice, HidError};

use crate::{
    ConfigurationInterfaceIndex, ConfigurationInterfaceSelectionError, DeviceDiscoveryError,
    DiscoveredHidInterface, PacketTransport,
    discovery::enumerate_target_runtime_hid_interfaces, select_configuration_interface,
};

/// Failure to open the selected configuration interface as a retained transport.
#[derive(Debug)]
pub enum OpenConfigurationInterfaceTransportError {
    Discovery(DeviceDiscoveryError),
    Selection(ConfigurationInterfaceSelectionError),
    SelectedRecordUnavailable {
        selected_index: ConfigurationInterfaceIndex,
        record_count: usize,
    },
    Open {
        metadata: Box<DiscoveredHidInterface>,
        source: HidError,
    },
}

impl fmt::Display for OpenConfigurationInterfaceTransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discovery(source) => {
                write!(formatter, "cannot open configuration transport: {source}")
            }
            Self::Selection(source) => {
                write!(formatter, "cannot open configuration transport: {source}")
            }
            Self::SelectedRecordUnavailable {
                selected_index,
                record_count,
            } => write!(
                formatter,
                "selected interface index {} is unavailable in {record_count} runtime records",
                selected_index.get()
            ),
            Self::Open { metadata, source } => write!(
                formatter,
                "failed to open selected interface {} at presentation path {}: {source}",
                metadata.interface_number, metadata.path
            ),
        }
    }
}

impl Error for OpenConfigurationInterfaceTransportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Discovery(source) => Some(source),
            Self::Selection(source) => Some(source),
            Self::Open { source, .. } => Some(source),
            Self::SelectedRecordUnavailable { .. } => None,
        }
    }
}

/// Failure while forwarding one already-encoded packet through HIDAPI.
#[derive(Debug)]
pub enum HidPacketWriteError {
    Backend(HidError),
    ShortWrite { expected: usize, actual: usize },
}

impl fmt::Display for HidPacketWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(source) => write!(formatter, "HIDAPI write failed: {source}"),
            Self::ShortWrite { expected, actual } => write!(
                formatter,
                "HIDAPI reported a short write: expected {expected} bytes, wrote {actual}"
            ),
        }
    }
}

impl Error for HidPacketWriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Backend(source) => Some(source),
            Self::ShortWrite { .. } => None,
        }
    }
}

/// Retained interface-2 HIDAPI transport.
///
/// Construction is deliberately separate from packet encoding. The transport
/// accepts only already-encoded packet bytes through the internal
/// [`PacketTransport`] boundary; normal CLI/UI code should use typed higher-level
/// RGB operations rather than exposing arbitrary packet sending.
pub struct HidPacketTransport {
    _api: HidApi,
    device: HidDevice,
    metadata: DiscoveredHidInterface,
}

impl HidPacketTransport {
    /// Returns copied presentation metadata for the exact opened interface.
    #[must_use]
    pub const fn selected_metadata(&self) -> &DiscoveredHidInterface {
        &self.metadata
    }
}

impl PacketTransport for HidPacketTransport {
    type Error = HidPacketWriteError;

    fn write_packet(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        match write_exact(&self.device, bytes) {
            Ok(()) => Ok(()),
            Err(ExactWriteError::Backend(source)) => Err(HidPacketWriteError::Backend(source)),
            Err(ExactWriteError::ShortWrite { expected, actual }) => {
                Err(HidPacketWriteError::ShortWrite { expected, actual })
            }
        }
    }
}

/// Enumerates the target once, selects exactly one interface-2 record, and opens
/// that record by its retained original HIDAPI path.
///
/// This function does not write any HID report. A returned transport becomes
/// capable of writes only when a typed higher-level operation later calls the
/// [`PacketTransport`] implementation.
///
/// # Errors
/// Returns a typed discovery, selection, pairing, or HIDAPI open failure.
pub fn open_configuration_interface_transport()
-> Result<HidPacketTransport, OpenConfigurationInterfaceTransportError> {
    let api = HidApi::new()
        .map_err(DeviceDiscoveryError::from_source)
        .map_err(OpenConfigurationInterfaceTransportError::Discovery)?;
    let runtime_interfaces = enumerate_target_runtime_hid_interfaces(&api);
    let metadata: Vec<_> = runtime_interfaces
        .iter()
        .map(|interface| interface.metadata.clone())
        .collect();
    let selected_index = select_configuration_interface(&metadata)
        .map_err(OpenConfigurationInterfaceTransportError::Selection)?;
    let record_count = runtime_interfaces.len();
    let runtime = runtime_interfaces
        .into_iter()
        .nth(selected_index.get())
        .ok_or(
            OpenConfigurationInterfaceTransportError::SelectedRecordUnavailable {
                selected_index,
                record_count,
            },
        )?;

    let selected_metadata = runtime.metadata.clone();
    let device = api.open_path(runtime.raw_path()).map_err(|source| {
        OpenConfigurationInterfaceTransportError::Open {
            metadata: Box::new(selected_metadata),
            source,
        }
    })?;

    Ok(HidPacketTransport {
        _api: api,
        device,
        metadata: runtime.into_metadata(),
    })
}

trait OutputWriter {
    type Error;

    fn write_output(&self, bytes: &[u8]) -> Result<usize, Self::Error>;
}

impl OutputWriter for HidDevice {
    type Error = HidError;

    fn write_output(&self, bytes: &[u8]) -> Result<usize, Self::Error> {
        self.write(bytes)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ExactWriteError<E> {
    Backend(E),
    ShortWrite { expected: usize, actual: usize },
}

fn write_exact<W: OutputWriter>(writer: &W, bytes: &[u8]) -> Result<(), ExactWriteError<W::Error>> {
    let actual = writer
        .write_output(bytes)
        .map_err(ExactWriteError::Backend)?;
    if actual == bytes.len() {
        Ok(())
    } else {
        Err(ExactWriteError::ShortWrite {
            expected: bytes.len(),
            actual,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::{ExactWriteError, OutputWriter, write_exact};

    struct RecordingWriter {
        result: Result<usize, &'static str>,
        writes: RefCell<Vec<Vec<u8>>>,
    }

    impl OutputWriter for RecordingWriter {
        type Error = &'static str;

        fn write_output(&self, bytes: &[u8]) -> Result<usize, Self::Error> {
            self.writes.borrow_mut().push(bytes.to_vec());
            self.result
        }
    }

    #[test]
    fn exact_writer_accepts_only_full_length_success() {
        let writer = RecordingWriter {
            result: Ok(4),
            writes: RefCell::new(Vec::new()),
        };

        let result = write_exact(&writer, &[1, 2, 3, 4]);

        assert_eq!(result, Ok(()));
        assert_eq!(writer.writes.borrow().as_slice(), [vec![1, 2, 3, 4]]);
    }

    #[test]
    fn exact_writer_rejects_short_write_without_retry() {
        let writer = RecordingWriter {
            result: Ok(3),
            writes: RefCell::new(Vec::new()),
        };

        let result = write_exact(&writer, &[1, 2, 3, 4]);

        assert_eq!(
            result,
            Err(ExactWriteError::ShortWrite {
                expected: 4,
                actual: 3,
            })
        );
        assert_eq!(writer.writes.borrow().len(), 1);
    }

    #[test]
    fn exact_writer_forwards_backend_error_without_retry() {
        let writer = RecordingWriter {
            result: Err("synthetic write failure"),
            writes: RefCell::new(Vec::new()),
        };

        let result = write_exact(&writer, &[1, 2, 3, 4]);

        assert_eq!(
            result,
            Err(ExactWriteError::Backend("synthetic write failure"))
        );
        assert_eq!(writer.writes.borrow().len(), 1);
    }
}
