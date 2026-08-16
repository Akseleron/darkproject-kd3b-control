use kd3b_device::{
    BusType, ConfigurationInterfaceIndex, ConfigurationInterfaceSelectionError,
    DiscoveredHidInterface, select_configuration_interface,
};

fn target_interface(
    interface_number: i32,
    serial_number: &str,
    path: &str,
) -> DiscoveredHidInterface {
    DiscoveredHidInterface {
        interface_number,
        vendor_id: 0x195d,
        product_id: 0x2061,
        product_string: Some("Turing Gaming Keyboard".to_owned()),
        manufacturer_string: Some("Dark Project".to_owned()),
        serial_number: Some(serial_number.to_owned()),
        release_number: 0x1234,
        bus_type: BusType::Usb,
        path: path.to_owned(),
    }
}

#[test]
fn selection_reports_target_not_found_when_input_is_empty() {
    // Given
    let interfaces = [];

    // When
    let selection = select_configuration_interface(&interfaces);

    // Then
    assert_eq!(
        selection,
        Err(ConfigurationInterfaceSelectionError::TargetNotFound)
    );
}

#[test]
fn selection_reports_configuration_interface_not_found_when_interface_two_is_absent() {
    // Given
    let interfaces = [
        target_interface(0, "serial-1", "/synthetic/interface-0"),
        target_interface(1, "serial-1", "/synthetic/interface-1"),
    ];

    // When
    let selection = select_configuration_interface(&interfaces);

    // Then
    assert_eq!(
        selection,
        Err(ConfigurationInterfaceSelectionError::ConfigurationInterfaceNotFound)
    );
}

#[test]
fn selection_returns_original_slice_index_for_the_only_interface_two() {
    // Given
    let interfaces = [
        target_interface(0, "serial-1", "/synthetic/interface-0"),
        target_interface(1, "serial-1", "/synthetic/interface-1"),
        target_interface(2, "serial-1", "/synthetic/interface-2"),
    ];

    // When
    let selection = select_configuration_interface(&interfaces);

    // Then
    assert_eq!(selection, Ok(ConfigurationInterfaceIndex::new(2)));
}

#[test]
fn selection_reports_every_interface_two_record_as_an_ambiguous_candidate() {
    // Given
    let interfaces = [
        target_interface(2, "serial-a", "/synthetic/keyboard-a/interface-2"),
        target_interface(1, "serial-a", "/synthetic/keyboard-a/interface-1"),
        target_interface(2, "serial-b", "/synthetic/keyboard-b/interface-2"),
    ];

    // When
    let selection = select_configuration_interface(&interfaces);

    // Then
    assert_eq!(
        selection,
        Err(
            ConfigurationInterfaceSelectionError::AmbiguousConfigurationInterface {
                candidate_count: 2,
            }
        )
    );
}
