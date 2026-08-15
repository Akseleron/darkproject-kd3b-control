use kd3b_protocol::{ALL_KEYS, Key, LOGICAL_KEY_COUNT};

#[test]
fn crate_root_layout_api_matches_documented_representative_keys() {
    // Given the public layout catalogue imported only from the crate root.
    let expected = [
        (Key::Esc, 0, 5),
        (Key::F1, 1, 11),
        (Key::A, 51, 8),
        (Key::Space, 79, 34),
        (Key::Right, 86, 100),
    ];

    // When an external consumer inspects its length and representative entries.
    // Then the catalogue size and each documented index/offset pair are stable.
    assert_eq!(ALL_KEYS.len(), LOGICAL_KEY_COUNT);
    for (key, expected_index, expected_offset) in expected {
        assert_eq!(ALL_KEYS[expected_index], key);
        assert_eq!(key.index(), expected_index);
        assert_eq!(key.offset(), expected_offset);
    }
}
