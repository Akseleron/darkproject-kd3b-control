use std::{
    error::Error,
    ffi::{CStr, CString},
    fmt,
};

use hidapi::{BusType as HidApiBusType, DeviceInfo, HidApi};

use crate::TargetIdentity;

/// Project-owned transport-bus classification for discovery output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    Unknown,
    Usb,
    Bluetooth,
    I2c,
    Spi,
}

/// Failure to initialize the read-only HID metadata discovery context.
#[derive(Debug)]
pub struct DeviceDiscoveryError {
    source: hidapi::HidError,
}

impl DeviceDiscoveryError {
    pub(super) const fn from_source(source: hidapi::HidError) -> Self {
        Self { source }
    }
}

impl fmt::Display for DeviceDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "failed to initialize HID metadata discovery: {}; no privileged recovery was attempted",
            self.source
        )
    }
}

impl Error for DeviceDiscoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl fmt::Display for BusType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Unknown => "unknown",
            Self::Usb => "usb",
            Self::Bluetooth => "bluetooth",
            Self::I2c => "i2c",
            Self::Spi => "spi",
        };

        formatter.write_str(name)
    }
}

/// Owned, HIDAPI-free metadata for one enumerated HID interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredHidInterface {
    pub interface_number: i32,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_string: Option<String>,
    pub manufacturer_string: Option<String>,
    pub serial_number: Option<String>,
    pub release_number: u16,
    pub bus_type: BusType,
    pub path: String,
}

#[derive(Debug)]
pub(super) struct RuntimeHidInterface {
    pub(super) metadata: DiscoveredHidInterface,
    raw_path: CString,
}

impl RuntimeHidInterface {
    pub(super) fn from_parts(metadata: DiscoveredHidInterface, raw_path: CString) -> Self {
        Self { metadata, raw_path }
    }

    pub(super) fn raw_path(&self) -> &CStr {
        self.raw_path.as_c_str()
    }

    pub(super) fn into_metadata(self) -> DiscoveredHidInterface {
        let Self { metadata, raw_path } = self;
        drop(raw_path);
        metadata
    }
}

impl DiscoveredHidInterface {
    /// Reports interface 2 only as an unvalidated configuration-interface candidate.
    #[must_use]
    pub const fn is_unvalidated_configuration_interface_candidate(&self) -> bool {
        self.interface_number == 2
    }
}

/// Enumerates target HID interface metadata without requesting an application-level
/// `HidDevice` or HID report operation. The HIDAPI/libusb metadata backend may open
/// visible HID devices while collecting metadata.
///
/// # Errors
/// Returns [`DeviceDiscoveryError`] when HIDAPI cannot initialize its metadata context.
pub fn enumerate_target_hid_interfaces() -> Result<Vec<DiscoveredHidInterface>, DeviceDiscoveryError>
{
    let api = HidApi::new().map_err(DeviceDiscoveryError::from_source)?;
    Ok(enumerate_target_runtime_hid_interfaces(&api)
        .into_iter()
        .map(RuntimeHidInterface::into_metadata)
        .collect())
}

pub(super) fn enumerate_target_runtime_hid_interfaces(api: &HidApi) -> Vec<RuntimeHidInterface> {
    let target = TargetIdentity::default();
    api.device_list()
        .map(map_device_info)
        .filter(|interface| is_target_interface(target, &interface.metadata))
        .collect()
}

fn map_device_info(device: &DeviceInfo) -> RuntimeHidInterface {
    let raw_path = device.path().to_owned();
    let metadata = DiscoveredHidInterface {
        interface_number: device.interface_number(),
        vendor_id: device.vendor_id(),
        product_id: device.product_id(),
        product_string: device.product_string().map(str::to_owned),
        manufacturer_string: device.manufacturer_string().map(str::to_owned),
        serial_number: device.serial_number().map(str::to_owned),
        release_number: device.release_number(),
        bus_type: map_bus_type(device.bus_type()),
        path: escape_hid_path(raw_path.as_bytes()),
    };

    RuntimeHidInterface::from_parts(metadata, raw_path)
}

const fn map_bus_type(bus_type: HidApiBusType) -> BusType {
    match bus_type {
        HidApiBusType::Unknown => BusType::Unknown,
        HidApiBusType::Usb => BusType::Usb,
        HidApiBusType::Bluetooth => BusType::Bluetooth,
        HidApiBusType::I2c => BusType::I2c,
        HidApiBusType::Spi => BusType::Spi,
    }
}

/// Copies every matching VID/PID record in source order without deduplication.
#[must_use]
pub fn filter_target_interfaces(
    target: TargetIdentity,
    interfaces: &[DiscoveredHidInterface],
) -> Vec<DiscoveredHidInterface> {
    interfaces
        .iter()
        .filter(|interface| is_target_interface(target, interface))
        .cloned()
        .collect()
}

const fn is_target_interface(target: TargetIdentity, interface: &DiscoveredHidInterface) -> bool {
    interface.vendor_id == target.vendor_id && interface.product_id == target.product_id
}

/// Escapes path bytes obtained without a C string's trailing NUL for terminal output.
#[must_use]
pub fn escape_hid_path(path_bytes: &[u8]) -> String {
    const LOWER_HEX: &[u8; 16] = b"0123456789abcdef";

    let mut escaped = String::with_capacity(path_bytes.len());
    for &byte in path_bytes {
        match byte {
            b'\\' => escaped.push_str("\\\\"),
            0x20..=0x7e => escaped.push(char::from(byte)),
            _ => {
                escaped.push_str("\\x");
                escaped.push(char::from(LOWER_HEX[usize::from(byte >> 4)]));
                escaped.push(char::from(LOWER_HEX[usize::from(byte & 0x0f)]));
            }
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::{BusType, DiscoveredHidInterface, RuntimeHidInterface, escape_hid_path};

    #[test]
    fn runtime_pair_preserves_escaped_metadata_and_original_raw_path() {
        // Given
        let raw_path = CString::new(b"/dev/hid\\raw\x1b\xff".as_slice())
            .expect("synthetic path has no interior NUL");
        let metadata = DiscoveredHidInterface {
            interface_number: 2,
            vendor_id: 0x195d,
            product_id: 0x2061,
            product_string: Some("Turing Gaming Keyboard".to_owned()),
            manufacturer_string: Some("Dark Project".to_owned()),
            serial_number: Some("synthetic".to_owned()),
            release_number: 0x1234,
            bus_type: BusType::Usb,
            path: escape_hid_path(raw_path.as_bytes()),
        };

        // When
        let runtime = RuntimeHidInterface::from_parts(metadata.clone(), raw_path.clone());

        // Then
        assert_eq!(runtime.metadata, metadata);
        assert_eq!(runtime.metadata.path, "/dev/hid\\\\raw\\x1b\\xff");
        assert_eq!(runtime.raw_path.as_bytes(), raw_path.as_bytes());
        assert_ne!(
            runtime.metadata.path.as_bytes(),
            runtime.raw_path.as_bytes()
        );
    }
}
