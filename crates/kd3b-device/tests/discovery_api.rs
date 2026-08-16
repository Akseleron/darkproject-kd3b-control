use kd3b_device::{
    BusType, DeviceDiscoveryError, DiscoveredHidInterface, TargetIdentity,
    enumerate_target_hid_interfaces, escape_hid_path, filter_target_interfaces,
};

#[test]
fn linux_discovery_entry_point_has_owned_domain_result() {
    // Given
    let discovery: fn() -> Result<Vec<DiscoveredHidInterface>, DeviceDiscoveryError> =
        enumerate_target_hid_interfaces;

    // When
    let pointer_size = std::mem::size_of_val(&discovery);

    // Then
    assert_eq!(pointer_size, std::mem::size_of::<fn()>());
}

fn target_interface(interface_number: i32, path: &str) -> DiscoveredHidInterface {
    DiscoveredHidInterface {
        interface_number,
        vendor_id: 0x195d,
        product_id: 0x2061,
        product_string: Some("Turing Gaming Keyboard".to_owned()),
        manufacturer_string: Some("Dark Project".to_owned()),
        serial_number: Some("serial-1".to_owned()),
        release_number: 0x1234,
        bus_type: BusType::Usb,
        path: path.to_owned(),
    }
}

#[test]
fn target_filter_retains_two_matching_interfaces_in_input_order() {
    // Given
    let interfaces = [
        target_interface(0, "/dev/hidraw0"),
        DiscoveredHidInterface {
            product_id: 0xffff,
            path: "/dev/non-target".to_owned(),
            ..target_interface(1, "/dev/unused")
        },
        target_interface(2, "/dev/hidraw2"),
    ];

    // When
    let matches = filter_target_interfaces(TargetIdentity::default(), &interfaces);

    // Then
    assert_eq!(matches, [interfaces[0].clone(), interfaces[2].clone()]);
}

#[test]
fn target_filter_rejects_wrong_vendor_and_product_ids() {
    // Given
    let interfaces = [
        DiscoveredHidInterface {
            vendor_id: 0xffff,
            ..target_interface(0, "/dev/wrong-vendor")
        },
        DiscoveredHidInterface {
            product_id: 0xffff,
            ..target_interface(1, "/dev/wrong-product")
        },
    ];

    // When
    let matches = filter_target_interfaces(TargetIdentity::default(), &interfaces);

    // Then
    assert!(matches.is_empty());
}

#[test]
fn target_filter_does_not_deduplicate_repeated_matching_records() {
    // Given
    let repeated = target_interface(2, "/dev/repeated");
    let interfaces = [
        repeated.clone(),
        repeated.clone(),
        target_interface(2, "/dev/other"),
    ];

    // When
    let matches = filter_target_interfaces(TargetIdentity::default(), &interfaces);

    // Then
    assert_eq!(matches, interfaces);
}

#[test]
fn target_filter_retains_distinct_logical_devices_with_the_same_interface_number() {
    // Given
    let first_device = target_interface(2, "/synthetic/keyboard-a/interface-2");
    let second_device = DiscoveredHidInterface {
        serial_number: Some("serial-2".to_owned()),
        path: "/synthetic/keyboard-b/interface-2".to_owned(),
        ..target_interface(2, "/synthetic/unused")
    };
    let interfaces = [first_device.clone(), second_device.clone()];

    // When
    let matches = filter_target_interfaces(TargetIdentity::default(), &interfaces);

    // Then
    assert_eq!(matches, [first_device, second_device]);
}

#[test]
fn discovery_record_preserves_optional_and_numeric_metadata() {
    // Given
    let interface = DiscoveredHidInterface {
        interface_number: 2,
        vendor_id: 0x195d,
        product_id: 0x2061,
        product_string: None,
        manufacturer_string: None,
        serial_number: None,
        release_number: 0x00af,
        bus_type: BusType::Unknown,
        path: "/dev/synthetic".to_owned(),
    };

    // When
    let is_candidate = interface.is_unvalidated_configuration_interface_candidate();

    // Then
    assert!(is_candidate);
    assert_eq!(interface.product_string, None);
    assert_eq!(interface.manufacturer_string, None);
    assert_eq!(interface.serial_number, None);
    assert_eq!(interface.release_number, 0x00af_u16);
}

#[test]
fn only_interface_two_is_an_unvalidated_configuration_candidate() {
    // Given
    let other_interface = target_interface(1, "/dev/other");

    // When
    let is_candidate = other_interface.is_unvalidated_configuration_interface_candidate();

    // Then
    assert!(!is_candidate);
}

#[test]
fn bus_types_have_project_owned_locked_lowercase_names() {
    // Given
    let bus_types = [
        BusType::Unknown,
        BusType::Usb,
        BusType::Bluetooth,
        BusType::I2c,
        BusType::Spi,
    ];

    // When
    let names = bus_types.map(|bus_type| bus_type.to_string());

    // Then
    assert_eq!(names, ["unknown", "usb", "bluetooth", "i2c", "spi"]);
}

#[test]
fn hid_path_escaping_preserves_ascii_and_escapes_unsafe_bytes() {
    // Given
    let path_bytes = b" /dev/a\\b~\x00\x1b\x7f\xff";

    // When
    let escaped = escape_hid_path(path_bytes);

    // Then
    assert_eq!(escaped, " /dev/a\\\\b~\\x00\\x1b\\x7f\\xff");
}
