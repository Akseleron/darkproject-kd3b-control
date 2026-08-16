use std::{error::Error, ffi::CStr, fmt};

use hidapi::{HidApi, HidError};

use crate::{
    DeviceDiscoveryError, DiscoveredHidInterface,
    discovery::{RuntimeHidInterface, enumerate_target_runtime_hid_interfaces},
};

/// Original-slice index of the sole configuration-interface candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigurationInterfaceIndex(usize);

impl ConfigurationInterfaceIndex {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Failure to select exactly one configuration interface from target metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationInterfaceSelectionError {
    TargetNotFound,
    ConfigurationInterfaceNotFound,
    AmbiguousConfigurationInterface { candidate_count: usize },
}

impl fmt::Display for ConfigurationInterfaceSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetNotFound => formatter.write_str("target HID device was not found"),
            Self::ConfigurationInterfaceNotFound => {
                formatter.write_str("target HID device has no interface-2 record")
            }
            Self::AmbiguousConfigurationInterface { candidate_count } => write!(
                formatter,
                "target HID discovery returned {candidate_count} interface-2 records; refusing to choose"
            ),
        }
    }
}

impl Error for ConfigurationInterfaceSelectionError {}

/// Failure to prepare one selected configuration interface for an explicit open.
#[derive(Debug)]
pub enum PrepareConfigurationInterfaceProbeError {
    Discovery(DeviceDiscoveryError),
    Selection(ConfigurationInterfaceSelectionError),
    SelectedRecordUnavailable {
        selected_index: ConfigurationInterfaceIndex,
        record_count: usize,
    },
}

impl fmt::Display for PrepareConfigurationInterfaceProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discovery(source) => {
                write!(formatter, "cannot prepare interface probe: {source}")
            }
            Self::Selection(source) => {
                write!(formatter, "cannot prepare interface probe: {source}")
            }
            Self::SelectedRecordUnavailable {
                selected_index,
                record_count,
            } => write!(
                formatter,
                "selected interface index {} is unavailable in {record_count} runtime records",
                selected_index.get()
            ),
        }
    }
}

impl Error for PrepareConfigurationInterfaceProbeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Discovery(source) => Some(source),
            Self::Selection(source) => Some(source),
            Self::SelectedRecordUnavailable { .. } => None,
        }
    }
}

/// Failure to open the selected interface path.
#[derive(Debug)]
pub struct ConfigurationInterfaceOpenError {
    metadata: Box<DiscoveredHidInterface>,
    source: HidError,
}

impl ConfigurationInterfaceOpenError {
    /// Returns the copied metadata for the interface whose open failed.
    #[must_use]
    pub const fn selected_metadata(&self) -> &DiscoveredHidInterface {
        &self.metadata
    }
}

impl fmt::Display for ConfigurationInterfaceOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "failed to open selected interface {} at presentation path {}: {}",
            self.metadata.interface_number, self.metadata.path, self.source
        )
    }
}

impl Error for ConfigurationInterfaceOpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

pub(super) trait ConfigurationPathOpener {
    type Error;

    fn open_and_drop(&self, path: &CStr) -> Result<(), Self::Error>;
}

impl ConfigurationPathOpener for HidApi {
    type Error = HidError;

    fn open_and_drop(&self, path: &CStr) -> Result<(), Self::Error> {
        let handle = self.open_path(path)?;
        drop(handle);
        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct PreparedOpenError<E> {
    pub(super) metadata: DiscoveredHidInterface,
    pub(super) source: E,
}

pub(super) struct PreparedProbe<O> {
    opener: O,
    runtime: RuntimeHidInterface,
}

impl<O> PreparedProbe<O>
where
    O: ConfigurationPathOpener,
{
    pub(super) const fn new(opener: O, runtime: RuntimeHidInterface) -> Self {
        Self { opener, runtime }
    }

    fn metadata(&self) -> &DiscoveredHidInterface {
        &self.runtime.metadata
    }

    pub(super) fn open_and_drop(
        self,
    ) -> Result<DiscoveredHidInterface, PreparedOpenError<O::Error>> {
        let Self { opener, runtime } = self;
        match opener.open_and_drop(runtime.raw_path()) {
            Ok(()) => Ok(runtime.into_metadata()),
            Err(source) => Err(PreparedOpenError {
                metadata: runtime.into_metadata(),
                source,
            }),
        }
    }
}

/// Opaque selected interface state that does not open until consumed.
pub struct PreparedConfigurationInterfaceProbe {
    inner: PreparedProbe<HidApi>,
}

impl PreparedConfigurationInterfaceProbe {
    /// Returns copied presentation metadata for the selected interface.
    #[must_use]
    pub fn selected_metadata(&self) -> &DiscoveredHidInterface {
        self.inner.metadata()
    }

    /// Opens the selected original HIDAPI path once and immediately drops the handle.
    ///
    /// The backend may claim or release an interface, detach or reattach a kernel
    /// driver where applicable, or establish backend-managed interrupt-IN activity.
    ///
    /// # Errors
    /// Returns [`ConfigurationInterfaceOpenError`] with selected copied metadata when
    /// HIDAPI cannot open the retained path.
    pub fn open_and_drop(self) -> Result<DiscoveredHidInterface, ConfigurationInterfaceOpenError> {
        self.inner
            .open_and_drop()
            .map_err(|error| ConfigurationInterfaceOpenError {
                metadata: Box::new(error.metadata),
                source: error.source,
            })
    }
}

/// Enumerates once, selects the sole interface-2 record, and prepares a deferred open.
///
/// # Errors
/// Returns a typed discovery or selection failure, or an internal pairing failure if
/// the selected metadata index cannot be recovered from the paired runtime records.
pub fn prepare_configuration_interface_probe()
-> Result<PreparedConfigurationInterfaceProbe, PrepareConfigurationInterfaceProbeError> {
    let api = HidApi::new()
        .map_err(DeviceDiscoveryError::from_source)
        .map_err(PrepareConfigurationInterfaceProbeError::Discovery)?;
    let runtime_interfaces = enumerate_target_runtime_hid_interfaces(&api);
    let metadata: Vec<_> = runtime_interfaces
        .iter()
        .map(|interface| interface.metadata.clone())
        .collect();
    let selected_index = select_configuration_interface(&metadata)
        .map_err(PrepareConfigurationInterfaceProbeError::Selection)?;
    let record_count = runtime_interfaces.len();
    let runtime = runtime_interfaces
        .into_iter()
        .nth(selected_index.get())
        .ok_or(
            PrepareConfigurationInterfaceProbeError::SelectedRecordUnavailable {
                selected_index,
                record_count,
            },
        )?;

    Ok(PreparedConfigurationInterfaceProbe {
        inner: PreparedProbe::new(api, runtime),
    })
}

/// Selects interface 2 only when it occurs exactly once in the input slice.
///
/// # Errors
/// Returns a typed error when the target slice is empty, contains no interface 2,
/// or contains more than one interface-2 record.
pub fn select_configuration_interface(
    interfaces: &[DiscoveredHidInterface],
) -> Result<ConfigurationInterfaceIndex, ConfigurationInterfaceSelectionError> {
    if interfaces.is_empty() {
        return Err(ConfigurationInterfaceSelectionError::TargetNotFound);
    }

    let mut candidates = interfaces
        .iter()
        .enumerate()
        .filter(|(_, interface)| interface.is_unvalidated_configuration_interface_candidate());
    let Some((selected_index, _)) = candidates.next() else {
        return Err(ConfigurationInterfaceSelectionError::ConfigurationInterfaceNotFound);
    };
    let candidate_count = 1 + candidates.count();

    if candidate_count == 1 {
        Ok(ConfigurationInterfaceIndex::new(selected_index))
    } else {
        Err(
            ConfigurationInterfaceSelectionError::AmbiguousConfigurationInterface {
                candidate_count,
            },
        )
    }
}
