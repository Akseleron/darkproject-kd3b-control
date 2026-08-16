//! Device orchestration and transport abstractions for KD3B.

mod discovery;
mod mock;
mod probe;
mod transport;

pub use discovery::{
    BusType, DeviceDiscoveryError, DiscoveredHidInterface, enumerate_target_hid_interfaces,
    escape_hid_path, filter_target_interfaces,
};
pub use mock::{MockTransport, MockTransportError};
pub use probe::{
    ConfigurationInterfaceIndex, ConfigurationInterfaceOpenError,
    ConfigurationInterfaceSelectionError, PrepareConfigurationInterfaceProbeError,
    PreparedConfigurationInterfaceProbe, prepare_configuration_interface_probe,
    select_configuration_interface,
};
pub use transport::{PacketTransport, write_direct_rgb};

use kd3b_protocol::{USB_PRODUCT_ID, USB_VENDOR_ID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetIdentity {
    pub vendor_id: u16,
    pub product_id: u16,
}

impl Default for TargetIdentity {
    fn default() -> Self {
        Self {
            vendor_id: USB_VENDOR_ID,
            product_id: USB_PRODUCT_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        ffi::{CStr, CString},
        fmt,
        rc::Rc,
    };

    use super::*;
    use crate::{
        discovery::RuntimeHidInterface,
        probe::{ConfigurationPathOpener, PreparedProbe},
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct SyntheticOpenError;

    impl fmt::Display for SyntheticOpenError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("synthetic open failure")
        }
    }

    struct RecordingOpener {
        forwarded_paths: Rc<RefCell<Vec<Vec<u8>>>>,
        result: Result<(), SyntheticOpenError>,
    }

    impl ConfigurationPathOpener for RecordingOpener {
        type Error = SyntheticOpenError;

        fn open_and_drop(&self, path: &CStr) -> Result<(), Self::Error> {
            self.forwarded_paths
                .borrow_mut()
                .push(path.to_bytes().to_vec());
            self.result
        }
    }

    fn synthetic_runtime(raw_path: CString) -> RuntimeHidInterface {
        RuntimeHidInterface::from_parts(
            DiscoveredHidInterface {
                interface_number: 2,
                vendor_id: 0x195d,
                product_id: 0x2061,
                product_string: Some("Turing Gaming Keyboard".to_owned()),
                manufacturer_string: Some("Dark Project".to_owned()),
                serial_number: Some("synthetic".to_owned()),
                release_number: 0x1234,
                bus_type: BusType::Usb,
                path: "presentation-only-path".to_owned(),
            },
            raw_path,
        )
    }

    #[test]
    fn target_identity_matches_protocol_crate() {
        assert_eq!(
            TargetIdentity::default(),
            TargetIdentity {
                vendor_id: 0x195d,
                product_id: 0x2061,
            }
        );
    }

    #[test]
    fn prepared_probe_forwards_exact_raw_c_string_once_when_open_succeeds() {
        // Given
        let raw_path = CString::new(b"/dev/hidraw-synthetic-\xff".as_slice())
            .expect("synthetic path has no interior NUL");
        let forwarded_paths = Rc::new(RefCell::new(Vec::new()));
        let prepared = PreparedProbe::new(
            RecordingOpener {
                forwarded_paths: Rc::clone(&forwarded_paths),
                result: Ok(()),
            },
            synthetic_runtime(raw_path.clone()),
        );

        // When
        let opened = prepared.open_and_drop();

        // Then
        assert_eq!(
            opened.expect("synthetic opener succeeds").path,
            "presentation-only-path"
        );
        assert_eq!(
            forwarded_paths.borrow().as_slice(),
            [raw_path.as_bytes().to_vec()]
        );
    }

    #[test]
    fn prepared_probe_returns_selected_metadata_after_one_failed_open_without_fallback() {
        // Given
        let raw_path =
            CString::new("/dev/hidraw-selected").expect("synthetic path has no interior NUL");
        let forwarded_paths = Rc::new(RefCell::new(Vec::new()));
        let prepared = PreparedProbe::new(
            RecordingOpener {
                forwarded_paths: Rc::clone(&forwarded_paths),
                result: Err(SyntheticOpenError),
            },
            synthetic_runtime(raw_path.clone()),
        );

        // When
        let error = prepared
            .open_and_drop()
            .expect_err("synthetic opener fails");

        // Then
        assert_eq!(error.metadata.path, "presentation-only-path");
        assert_eq!(error.source, SyntheticOpenError);
        assert_eq!(
            forwarded_paths.borrow().as_slice(),
            [raw_path.as_bytes().to_vec()]
        );
    }

    #[test]
    fn dropping_prepared_probe_does_not_call_opener() {
        // Given
        let raw_path =
            CString::new("/dev/hidraw-unopened").expect("synthetic path has no interior NUL");
        let forwarded_paths = Rc::new(RefCell::new(Vec::new()));
        let prepared = PreparedProbe::new(
            RecordingOpener {
                forwarded_paths: Rc::clone(&forwarded_paths),
                result: Ok(()),
            },
            synthetic_runtime(raw_path),
        );

        // When
        drop(prepared);

        // Then
        assert!(forwarded_paths.borrow().is_empty());
    }
}
