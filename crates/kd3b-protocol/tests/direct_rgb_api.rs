use kd3b_protocol::{
    ALL_KEYS, DIRECT_PACKET_LEN, DirectRgbPackets, Key, LOGICAL_KEY_COUNT, Rgb8, encode_direct_rgb,
};

const PACKET_A_HEADER: [u8; 5] = [0x08, 0x07, 0x00, 0x00, 0x00];
const PACKET_B_HEADER: [u8; 5] = [0x08, 0x07, 0x00, 0x01, 0x00];

fn frame_with(colors: &[(Key, Rgb8)]) -> [Rgb8; LOGICAL_KEY_COUNT] {
    let mut frame = [Rgb8::new(0, 0, 0); LOGICAL_KEY_COUNT];
    for &(key, color) in colors {
        frame[key.index()] = color;
    }
    frame
}

fn expected_packets(colors: &[(Key, Rgb8)]) -> [[u8; DIRECT_PACKET_LEN]; 2] {
    let mut packet_a = [0_u8; DIRECT_PACKET_LEN];
    let mut packet_b = [0_u8; DIRECT_PACKET_LEN];
    packet_a[..PACKET_A_HEADER.len()].copy_from_slice(&PACKET_A_HEADER);
    packet_b[..PACKET_B_HEADER.len()].copy_from_slice(&PACKET_B_HEADER);

    for &(key, color) in colors {
        let offset = usize::from(key.offset());
        packet_a[5 + offset] = color.red;
        packet_a[107 + offset] = color.green;
        packet_b[5 + offset] = color.blue;
    }

    [packet_a, packet_b]
}

fn assert_complete_packets(packets: &DirectRgbPackets, expected: &[[u8; DIRECT_PACKET_LEN]; 2]) {
    assert_eq!(packets.packet_a(), &expected[0]);
    assert_eq!(packets.packet_b(), &expected[1]);
}

#[test]
fn packet_lengths_and_headers_match_documented_format() {
    // Given an all-black canonical frame.
    let frame = [Rgb8::new(0, 0, 0); LOGICAL_KEY_COUNT];

    // When it is encoded through the crate-root API.
    let packets = encode_direct_rgb(&frame);

    // Then both fixed packets have the documented length and exact headers.
    assert_eq!(packets.packet_a().len(), DIRECT_PACKET_LEN);
    assert_eq!(packets.packet_b().len(), DIRECT_PACKET_LEN);
    assert_eq!(packets.packet_a()[..5], PACKET_A_HEADER);
    assert_eq!(packets.packet_b()[..5], PACKET_B_HEADER);
}

#[test]
fn all_black_frame_matches_complete_zero_payload_goldens() {
    // Given an all-black canonical frame and independently assembled full packets.
    let frame = [Rgb8::new(0, 0, 0); LOGICAL_KEY_COUNT];
    let expected = expected_packets(&[]);

    // When the frame is encoded.
    let packets = encode_direct_rgb(&frame);

    // Then every byte in both 256-byte packets matches the header-only goldens.
    assert_complete_packets(&packets, &expected);
}

#[test]
fn sentinel_primary_channels_match_complete_packet_goldens() {
    // Given each required sentinel and each independent primary color channel.
    let sentinels = [Key::Esc, Key::F1, Key::A, Key::Space, Key::Right];
    let primaries = [
        Rgb8::new(0x31, 0, 0),
        Rgb8::new(0, 0x52, 0),
        Rgb8::new(0, 0, 0x73),
    ];

    for key in sentinels {
        for color in primaries {
            let selected = [(key, color)];
            let frame = frame_with(&selected);
            let expected = expected_packets(&selected);

            // When exactly one sentinel primary channel is encoded.
            let packets = encode_direct_rgb(&frame);

            // Then both complete packets match the documented channel placement.
            assert_complete_packets(&packets, &expected);
        }
    }
}

#[test]
fn multi_key_frame_matches_complete_packet_goldens() {
    // Given distinct RGB triples at multiple sentinel offsets.
    let selected = [
        (Key::Esc, Rgb8::new(0x11, 0x12, 0x13)),
        (Key::A, Rgb8::new(0x21, 0x22, 0x23)),
        (Key::Space, Rgb8::new(0x31, 0x32, 0x33)),
        (Key::Right, Rgb8::new(0x41, 0x42, 0x43)),
    ];
    let frame = frame_with(&selected);
    let expected = expected_packets(&selected);

    // When all selected keys are encoded together.
    let packets = encode_direct_rgb(&frame);

    // Then both complete packets preserve every distinct triple.
    assert_complete_packets(&packets, &expected);
}

#[test]
fn all_undocumented_packet_bytes_remain_zero() {
    // Given every canonical frame slot populated with nonzero channels.
    let frame = [Rgb8::new(0x41, 0x52, 0x63); LOGICAL_KEY_COUNT];
    let mut red_green_bytes = [false; DIRECT_PACKET_LEN];
    let mut blue_bytes = [false; DIRECT_PACKET_LEN];
    red_green_bytes[..5].fill(true);
    blue_bytes[..5].fill(true);
    for key in ALL_KEYS {
        let offset = usize::from(key.offset());
        red_green_bytes[5 + offset] = true;
        red_green_bytes[107 + offset] = true;
        blue_bytes[5 + offset] = true;
    }

    // When the complete frame is encoded.
    let packets = encode_direct_rgb(&frame);

    // Then every byte outside headers and documented channel locations is zero.
    for index in 0..DIRECT_PACKET_LEN {
        if !red_green_bytes[index] {
            assert_eq!(packets.packet_a()[index], 0, "Packet A byte {index}");
        }
        if !blue_bytes[index] {
            assert_eq!(packets.packet_b()[index], 0, "Packet B byte {index}");
        }
    }
}

#[test]
fn channel_and_key_changes_are_isolated() {
    // Given two frames differing only in F1's green channel.
    let baseline_selected = [(Key::A, Rgb8::new(0x21, 0x22, 0x23))];
    let changed_selected = [
        (Key::A, Rgb8::new(0x21, 0x22, 0x23)),
        (Key::F1, Rgb8::new(0, 0x7f, 0)),
    ];
    let baseline = encode_direct_rgb(&frame_with(&baseline_selected));
    let changed = encode_direct_rgb(&frame_with(&changed_selected));
    let baseline_expected = expected_packets(&baseline_selected);
    let changed_expected = expected_packets(&changed_selected);

    // When the resulting complete packets are compared with independent goldens.
    assert_complete_packets(&baseline, &baseline_expected);
    assert_complete_packets(&changed, &changed_expected);

    // Then only F1's documented Packet A green location changed.
    let changed_index = 107 + usize::from(Key::F1.offset());
    for index in 0..DIRECT_PACKET_LEN {
        if index != changed_index {
            assert_eq!(changed.packet_a()[index], baseline.packet_a()[index]);
        }
        assert_eq!(changed.packet_b()[index], baseline.packet_b()[index]);
    }
}

#[test]
fn encode_all_documented_keys_stays_in_bounds_and_uses_canonical_slots() {
    // Given every frame slot populated from its canonical catalogue position.
    let mut frame = [Rgb8::new(0, 0, 0); LOGICAL_KEY_COUNT];
    let mut expected = expected_packets(&[]);
    for (position, key) in ALL_KEYS.into_iter().enumerate() {
        assert_eq!(key.index(), position);
        let value = u8::try_from(position + 1).expect("87 keys fit in u8");
        let color = Rgb8::new(value, value.wrapping_add(87), value.wrapping_add(174));
        let offset = usize::from(key.offset());
        assert!(5 + offset < DIRECT_PACKET_LEN);
        assert!(107 + offset < DIRECT_PACKET_LEN);
        frame[key.index()] = color;
        expected[0][5 + offset] = color.red;
        expected[0][107 + offset] = color.green;
        expected[1][5 + offset] = color.blue;
    }

    // When all 87 keys are encoded.
    let packets = encode_direct_rgb(&frame);

    // Then every canonical slot contributes to complete in-bounds packet goldens.
    assert_eq!(ALL_KEYS.len(), LOGICAL_KEY_COUNT);
    assert_complete_packets(&packets, &expected);
}

#[test]
fn ordered_accessor_is_packet_a_then_packet_b() {
    // Given a packet pair whose A and B contents are observably different.
    let selected = [(Key::Esc, Rgb8::new(0x12, 0x34, 0x56))];
    let packets = encode_direct_rgb(&frame_with(&selected));

    // When the documented write order is requested.
    let ordered = packets.write_order();

    // Then it returns immutable references to Packet A followed by Packet B.
    assert_eq!(ordered[0], packets.packet_a());
    assert_eq!(ordered[1], packets.packet_b());
}
