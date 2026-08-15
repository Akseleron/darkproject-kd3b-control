//! Typed logical-key catalogue sourced from `docs/protocol/key-offsets.csv`.
//!
//! Protocol offsets locate key color channels as documented in
//! `docs/protocol/direct-rgb.md`; this module defines offsets only and does not
//! construct direct-RGB packets.
// allow: SIZE_OK — the authoritative 87-key enum, order, exhaustive index map, and golden order fixture are indivisible protocol catalogue data.

use crate::LOGICAL_KEY_COUNT;

/// A KD3B logical protocol key.
///
/// Variants identify entries in the KD3B protocol catalogue; they are not
/// generic HID usages or claims about physical key legends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Esc,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    Backtick,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    Minus,
    Equal,
    Backspace,
    Insert,
    Home,
    PageUp,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBracket,
    RightBracket,
    Backslash,
    Delete,
    End,
    PageDown,
    CapsLock,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Quote,
    Enter,
    LeftShift,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    Slash,
    RightShift,
    Up,
    LeftCtrl,
    LeftMeta,
    LeftAlt,
    Space,
    RightAlt,
    Fn,
    Menu,
    RightCtrl,
    Left,
    Down,
    Right,
}

/// All KD3B logical protocol keys in CSV index order.
pub const ALL_KEYS: [Key; LOGICAL_KEY_COUNT] = [
    Key::Esc,
    Key::F1,
    Key::F2,
    Key::F3,
    Key::F4,
    Key::F5,
    Key::F6,
    Key::F7,
    Key::F8,
    Key::F9,
    Key::F10,
    Key::F11,
    Key::F12,
    Key::PrintScreen,
    Key::ScrollLock,
    Key::Pause,
    Key::Backtick,
    Key::Digit1,
    Key::Digit2,
    Key::Digit3,
    Key::Digit4,
    Key::Digit5,
    Key::Digit6,
    Key::Digit7,
    Key::Digit8,
    Key::Digit9,
    Key::Digit0,
    Key::Minus,
    Key::Equal,
    Key::Backspace,
    Key::Insert,
    Key::Home,
    Key::PageUp,
    Key::Tab,
    Key::Q,
    Key::W,
    Key::E,
    Key::R,
    Key::T,
    Key::Y,
    Key::U,
    Key::I,
    Key::O,
    Key::P,
    Key::LeftBracket,
    Key::RightBracket,
    Key::Backslash,
    Key::Delete,
    Key::End,
    Key::PageDown,
    Key::CapsLock,
    Key::A,
    Key::S,
    Key::D,
    Key::F,
    Key::G,
    Key::H,
    Key::J,
    Key::K,
    Key::L,
    Key::Semicolon,
    Key::Quote,
    Key::Enter,
    Key::LeftShift,
    Key::Z,
    Key::X,
    Key::C,
    Key::V,
    Key::B,
    Key::N,
    Key::M,
    Key::Comma,
    Key::Period,
    Key::Slash,
    Key::RightShift,
    Key::Up,
    Key::LeftCtrl,
    Key::LeftMeta,
    Key::LeftAlt,
    Key::Space,
    Key::RightAlt,
    Key::Fn,
    Key::Menu,
    Key::RightCtrl,
    Key::Left,
    Key::Down,
    Key::Right,
];

impl Key {
    /// Returns this key's stable index in the documented KD3B catalogue.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Esc => 0,
            Self::F1 => 1,
            Self::F2 => 2,
            Self::F3 => 3,
            Self::F4 => 4,
            Self::F5 => 5,
            Self::F6 => 6,
            Self::F7 => 7,
            Self::F8 => 8,
            Self::F9 => 9,
            Self::F10 => 10,
            Self::F11 => 11,
            Self::F12 => 12,
            Self::PrintScreen => 13,
            Self::ScrollLock => 14,
            Self::Pause => 15,
            Self::Backtick => 16,
            Self::Digit1 => 17,
            Self::Digit2 => 18,
            Self::Digit3 => 19,
            Self::Digit4 => 20,
            Self::Digit5 => 21,
            Self::Digit6 => 22,
            Self::Digit7 => 23,
            Self::Digit8 => 24,
            Self::Digit9 => 25,
            Self::Digit0 => 26,
            Self::Minus => 27,
            Self::Equal => 28,
            Self::Backspace => 29,
            Self::Insert => 30,
            Self::Home => 31,
            Self::PageUp => 32,
            Self::Tab => 33,
            Self::Q => 34,
            Self::W => 35,
            Self::E => 36,
            Self::R => 37,
            Self::T => 38,
            Self::Y => 39,
            Self::U => 40,
            Self::I => 41,
            Self::O => 42,
            Self::P => 43,
            Self::LeftBracket => 44,
            Self::RightBracket => 45,
            Self::Backslash => 46,
            Self::Delete => 47,
            Self::End => 48,
            Self::PageDown => 49,
            Self::CapsLock => 50,
            Self::A => 51,
            Self::S => 52,
            Self::D => 53,
            Self::F => 54,
            Self::G => 55,
            Self::H => 56,
            Self::J => 57,
            Self::K => 58,
            Self::L => 59,
            Self::Semicolon => 60,
            Self::Quote => 61,
            Self::Enter => 62,
            Self::LeftShift => 63,
            Self::Z => 64,
            Self::X => 65,
            Self::C => 66,
            Self::V => 67,
            Self::B => 68,
            Self::N => 69,
            Self::M => 70,
            Self::Comma => 71,
            Self::Period => 72,
            Self::Slash => 73,
            Self::RightShift => 74,
            Self::Up => 75,
            Self::LeftCtrl => 76,
            Self::LeftMeta => 77,
            Self::LeftAlt => 78,
            Self::Space => 79,
            Self::RightAlt => 80,
            Self::Fn => 81,
            Self::Menu => 82,
            Self::RightCtrl => 83,
            Self::Left => 84,
            Self::Down => 85,
            Self::Right => 86,
        }
    }

    /// Returns this key's protocol offset from `docs/protocol/key-offsets.csv`.
    ///
    /// `docs/protocol/direct-rgb.md` documents how packet encoders use this
    /// offset to locate color channels.
    #[must_use]
    pub const fn offset(self) -> u8 {
        match self {
            Self::Esc => 5,
            Self::F1 => 11,
            Self::F2 => 17,
            Self::F3 => 23,
            Self::F4 => 29,
            Self::F5 => 35,
            Self::F6 => 41,
            Self::F7 => 47,
            Self::F8 => 53,
            Self::F9 => 59,
            Self::F10 => 65,
            Self::F11 => 71,
            Self::F12 => 77,
            Self::PrintScreen => 83,
            Self::ScrollLock => 89,
            Self::Pause => 95,
            Self::Backtick => 0,
            Self::Digit1 => 6,
            Self::Digit2 => 12,
            Self::Digit3 => 18,
            Self::Digit4 => 24,
            Self::Digit5 => 30,
            Self::Digit6 => 36,
            Self::Digit7 => 42,
            Self::Digit8 => 48,
            Self::Digit9 => 54,
            Self::Digit0 => 60,
            Self::Minus => 66,
            Self::Equal => 72,
            Self::Backspace => 78,
            Self::Insert => 84,
            Self::Home => 90,
            Self::PageUp => 96,
            Self::Tab => 1,
            Self::Q => 7,
            Self::W => 13,
            Self::E => 19,
            Self::R => 25,
            Self::T => 31,
            Self::Y => 37,
            Self::U => 43,
            Self::I => 49,
            Self::O => 55,
            Self::P => 61,
            Self::LeftBracket => 67,
            Self::RightBracket => 73,
            Self::Backslash => 79,
            Self::Delete => 85,
            Self::End => 91,
            Self::PageDown => 97,
            Self::CapsLock => 2,
            Self::A => 8,
            Self::S => 14,
            Self::D => 20,
            Self::F => 26,
            Self::G => 32,
            Self::H => 38,
            Self::J => 44,
            Self::K => 50,
            Self::L => 56,
            Self::Semicolon => 62,
            Self::Quote => 68,
            Self::Enter => 80,
            Self::LeftShift => 3,
            Self::Z => 15,
            Self::X => 21,
            Self::C => 27,
            Self::V => 33,
            Self::B => 39,
            Self::N => 45,
            Self::M => 51,
            Self::Comma => 57,
            Self::Period => 63,
            Self::Slash => 69,
            Self::RightShift => 81,
            Self::Up => 93,
            Self::LeftCtrl => 4,
            Self::LeftMeta => 10,
            Self::LeftAlt => 16,
            Self::Space => 34,
            Self::RightAlt => 52,
            Self::Fn => 58,
            Self::Menu => 64,
            Self::RightCtrl => 76,
            Self::Left => 88,
            Self::Down => 94,
            Self::Right => 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ALL_KEYS, Key};
    use crate::{DIRECT_PACKET_LEN, LOGICAL_KEY_COUNT};

    const EXPECTED_LAYOUT: [(Key, usize, u8); LOGICAL_KEY_COUNT] = [
        (Key::Esc, 0, 5),
        (Key::F1, 1, 11),
        (Key::F2, 2, 17),
        (Key::F3, 3, 23),
        (Key::F4, 4, 29),
        (Key::F5, 5, 35),
        (Key::F6, 6, 41),
        (Key::F7, 7, 47),
        (Key::F8, 8, 53),
        (Key::F9, 9, 59),
        (Key::F10, 10, 65),
        (Key::F11, 11, 71),
        (Key::F12, 12, 77),
        (Key::PrintScreen, 13, 83),
        (Key::ScrollLock, 14, 89),
        (Key::Pause, 15, 95),
        (Key::Backtick, 16, 0),
        (Key::Digit1, 17, 6),
        (Key::Digit2, 18, 12),
        (Key::Digit3, 19, 18),
        (Key::Digit4, 20, 24),
        (Key::Digit5, 21, 30),
        (Key::Digit6, 22, 36),
        (Key::Digit7, 23, 42),
        (Key::Digit8, 24, 48),
        (Key::Digit9, 25, 54),
        (Key::Digit0, 26, 60),
        (Key::Minus, 27, 66),
        (Key::Equal, 28, 72),
        (Key::Backspace, 29, 78),
        (Key::Insert, 30, 84),
        (Key::Home, 31, 90),
        (Key::PageUp, 32, 96),
        (Key::Tab, 33, 1),
        (Key::Q, 34, 7),
        (Key::W, 35, 13),
        (Key::E, 36, 19),
        (Key::R, 37, 25),
        (Key::T, 38, 31),
        (Key::Y, 39, 37),
        (Key::U, 40, 43),
        (Key::I, 41, 49),
        (Key::O, 42, 55),
        (Key::P, 43, 61),
        (Key::LeftBracket, 44, 67),
        (Key::RightBracket, 45, 73),
        (Key::Backslash, 46, 79),
        (Key::Delete, 47, 85),
        (Key::End, 48, 91),
        (Key::PageDown, 49, 97),
        (Key::CapsLock, 50, 2),
        (Key::A, 51, 8),
        (Key::S, 52, 14),
        (Key::D, 53, 20),
        (Key::F, 54, 26),
        (Key::G, 55, 32),
        (Key::H, 56, 38),
        (Key::J, 57, 44),
        (Key::K, 58, 50),
        (Key::L, 59, 56),
        (Key::Semicolon, 60, 62),
        (Key::Quote, 61, 68),
        (Key::Enter, 62, 80),
        (Key::LeftShift, 63, 3),
        (Key::Z, 64, 15),
        (Key::X, 65, 21),
        (Key::C, 66, 27),
        (Key::V, 67, 33),
        (Key::B, 68, 39),
        (Key::N, 69, 45),
        (Key::M, 70, 51),
        (Key::Comma, 71, 57),
        (Key::Period, 72, 63),
        (Key::Slash, 73, 69),
        (Key::RightShift, 74, 81),
        (Key::Up, 75, 93),
        (Key::LeftCtrl, 76, 4),
        (Key::LeftMeta, 77, 10),
        (Key::LeftAlt, 78, 16),
        (Key::Space, 79, 34),
        (Key::RightAlt, 80, 52),
        (Key::Fn, 81, 58),
        (Key::Menu, 82, 64),
        (Key::RightCtrl, 83, 76),
        (Key::Left, 84, 88),
        (Key::Down, 85, 94),
        (Key::Right, 86, 100),
    ];

    #[test]
    fn catalogue_has_documented_logical_key_count() {
        // Given the canonical KD3B logical-key catalogue.
        // When its fixed-size collection is inspected.
        // Then it contains exactly the documented number of keys.
        assert_eq!(ALL_KEYS.len(), LOGICAL_KEY_COUNT);
    }

    #[test]
    fn catalogue_and_total_lookups_match_every_csv_triple() {
        // Given the exhaustive key, index, and offset triples from key-offsets.csv.
        // When every canonical key and both total lookups are invoked directly.
        for (position, &(expected_key, expected_index, expected_offset)) in
            EXPECTED_LAYOUT.iter().enumerate()
        {
            let key = ALL_KEYS[position];

            // Then canonical order and both lookup results match the same CSV row.
            assert_eq!(key, expected_key, "key mismatch at CSV row {position}");
            assert_eq!(
                (key, key.index(), key.offset()),
                (expected_key, expected_index, expected_offset),
                "layout mismatch at CSV row {position}"
            );
        }
    }

    #[test]
    fn catalogue_has_unique_keys_and_offsets() {
        // Given the canonical catalogue and its total offset lookup.
        // When each key and offset are compared with every later entry.
        for (position, &key) in ALL_KEYS.iter().enumerate() {
            for &later_key in &ALL_KEYS[position + 1..] {
                // Then neither logical keys nor protocol offsets are duplicated.
                assert_ne!(key, later_key, "duplicate logical key: {key:?}");
                assert_ne!(
                    key.offset(),
                    later_key.offset(),
                    "duplicate protocol offset: {}",
                    key.offset()
                );
            }
        }
    }

    #[test]
    fn every_offset_fits_documented_range_and_future_channel_positions() {
        // Given every documented key offset and the direct-RGB packet length.
        let mut minimum_offset = u8::MAX;
        let mut maximum_offset = u8::MIN;

        // When the range and future red/blue and green positions are calculated.
        for &(key, _, expected_offset) in &EXPECTED_LAYOUT {
            let offset = key.offset();
            minimum_offset = minimum_offset.min(offset);
            maximum_offset = maximum_offset.max(offset);
            let channel_offset = usize::from(offset);

            // Then each exact offset is documented and both positions remain in bounds.
            assert_eq!(offset, expected_offset);
            assert!(offset <= 100);
            assert!(5 + channel_offset < DIRECT_PACKET_LEN);
            assert!(107 + channel_offset < DIRECT_PACKET_LEN);
        }

        // Then the complete layout spans the documented inclusive offset range.
        assert_eq!(minimum_offset, 0);
        assert_eq!(maximum_offset, 100);
    }

    #[test]
    fn sentinel_keys_return_documented_offsets() {
        // Given sentinel keys spanning the documented layout.
        // When their protocol offsets are requested.
        // Then they match key-offsets.csv exactly.
        assert_eq!(Key::Esc.offset(), 5);
        assert_eq!(Key::F1.offset(), 11);
        assert_eq!(Key::A.offset(), 8);
        assert_eq!(Key::Space.offset(), 34);
        assert_eq!(Key::Right.offset(), 100);
    }
}
