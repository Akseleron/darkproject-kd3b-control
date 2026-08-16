//! Hardware-independent host-rendered lighting effects for Dark Project KD3B rev.2.
//!
//! The engine produces logical 87-key RGB frames. It never opens a device and never writes USB.

use std::f32::consts::{PI, TAU};

use kd3b_protocol::{ALL_KEYS, Key, LOGICAL_KEY_COUNT, Rgb8};

pub type Frame = [Rgb8; LOGICAL_KEY_COUNT];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    Gradient,
    Wave,
    ConicBand,
    Spiral,
    Cycle,
    LinearWave,
    Ripple,
    Breathing,
    Rain,
    Fire,
}

impl EffectKind {
    pub const ALL: [Self; 10] = [
        Self::Gradient,
        Self::Wave,
        Self::ConicBand,
        Self::Spiral,
        Self::Cycle,
        Self::LinearWave,
        Self::Ripple,
        Self::Breathing,
        Self::Rain,
        Self::Fire,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Gradient => "gradient",
            Self::Wave => "wave",
            Self::ConicBand => "conic-band",
            Self::Spiral => "spiral",
            Self::Cycle => "cycle",
            Self::LinearWave => "linear-wave",
            Self::Ripple => "ripple",
            Self::Breathing => "breathing",
            Self::Rain => "rain",
            Self::Fire => "fire",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    Reverse,
    #[default]
    Forward,
}

impl Direction {
    const fn sign(self) -> f32 {
        match self {
            Self::Forward => 1.0,
            Self::Reverse => -1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectConfig {
    pub kind: EffectKind,
    pub primary: Rgb8,
    pub secondary: Rgb8,
    pub speed: f32,
    pub brightness_percent: u8,
    pub direction: Direction,
    pub seed: u64,
}

impl EffectConfig {
    #[must_use]
    pub const fn new(kind: EffectKind) -> Self {
        Self {
            kind,
            primary: Rgb8::new(0x40, 0x80, 0xff),
            secondary: Rgb8::new(0xff, 0x30, 0x00),
            speed: 1.0,
            brightness_percent: 100,
            direction: Direction::Forward,
            seed: 0x4b44_3342,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameContext {
    pub elapsed_seconds: f32,
}

impl FrameContext {
    #[must_use]
    pub const fn new(elapsed_seconds: f32) -> Self {
        Self { elapsed_seconds }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyPosition {
    pub column: u8,
    pub row: u8,
    pub x: f32,
    pub y: f32,
}

/// Converts the protocol's six-lane offset grid into an approximate physical key position.
///
/// The protocol offset is column-major. Lane 5 is the function-key row, followed by lanes 0-4
/// from the number row down to the modifier row. Wide keys still have one logical point.
#[must_use]
pub fn key_position(key: Key) -> KeyPosition {
    let offset = key.offset();
    let column = offset / 6;
    let protocol_lane = offset % 6;
    let row = match protocol_lane {
        5 => 0,
        lane => lane + 1,
    };

    KeyPosition {
        column,
        row,
        x: f32::from(column) / 16.0,
        y: f32::from(row) / 5.0,
    }
}

/// Renders one hardware-independent RGB frame.
#[must_use]
pub fn render(config: &EffectConfig, context: FrameContext) -> Frame {
    let speed = sanitized_speed(config.speed);
    let phase = context.elapsed_seconds.max(0.0) * speed * config.direction.sign();
    let mut frame = [Rgb8::default(); LOGICAL_KEY_COUNT];

    for key in ALL_KEYS {
        let position = key_position(key);
        frame[key.index()] = match config.kind {
            EffectKind::Gradient => render_gradient(config, position),
            EffectKind::Wave => render_wave(position, phase),
            EffectKind::ConicBand => render_conic(position, phase),
            EffectKind::Spiral => render_spiral(position, phase),
            EffectKind::Cycle => rainbow(fract01(phase * 0.16)),
            EffectKind::LinearWave => render_linear_wave(config.primary, position, phase),
            EffectKind::Ripple => render_ripple(config.primary, position, phase),
            EffectKind::Breathing => render_breathing(config.primary, phase),
            EffectKind::Rain => render_rain(config, key, position, phase),
            EffectKind::Fire => render_fire(config, key, position, phase),
        };
    }

    apply_brightness(&mut frame, config.brightness_percent);
    frame
}

/// Applies host-side brightness scaling to an already rendered frame.
pub fn apply_brightness(frame: &mut Frame, brightness_percent: u8) {
    let brightness = f32::from(brightness_percent.min(100)) / 100.0;
    for color in frame {
        *color = scale_color(*color, brightness);
    }
}

fn render_gradient(config: &EffectConfig, position: KeyPosition) -> Rgb8 {
    mix(config.primary, config.secondary, position.x)
}

fn render_wave(position: KeyPosition, phase: f32) -> Rgb8 {
    let hue = fract01(position.x + position.y * 0.08 + phase * 0.18);
    rainbow(hue)
}

fn render_conic(position: KeyPosition, phase: f32) -> Rgb8 {
    let dx = position.x - 0.5;
    let dy = (position.y - 0.5) * 0.65;
    let angle = dy.atan2(dx) / TAU + 0.5;
    rainbow(fract01(angle + phase * 0.18))
}

fn render_spiral(position: KeyPosition, phase: f32) -> Rgb8 {
    let dx = position.x - 0.5;
    let dy = (position.y - 0.5) * 0.65;
    let angle = dy.atan2(dx) / TAU + 0.5;
    let radius = (dx.mul_add(dx, dy * dy)).sqrt();
    rainbow(fract01(angle + radius * 1.6 + phase * 0.2))
}

fn render_linear_wave(primary: Rgb8, position: KeyPosition, phase: f32) -> Rgb8 {
    let center = fract01(phase * 0.35);
    let distance = cyclic_distance(position.x, center);
    let intensity = pulse(distance, 0.18);
    scale_color(primary, intensity)
}

fn render_ripple(primary: Rgb8, position: KeyPosition, phase: f32) -> Rgb8 {
    let dx = position.x - 0.5;
    let dy = (position.y - 0.5) * 0.65;
    let radius = (dx.mul_add(dx, dy * dy)).sqrt();
    let ring = fract01(phase * 0.35);
    let distance = cyclic_distance(fract01(radius), ring);
    scale_color(primary, pulse(distance, 0.12))
}

fn render_breathing(primary: Rgb8, phase: f32) -> Rgb8 {
    let intensity = ((phase * PI).sin() + 1.0) * 0.5;
    scale_color(primary, intensity * intensity)
}

fn render_rain(
    config: &EffectConfig,
    key: Key,
    position: KeyPosition,
    phase: f32,
) -> Rgb8 {
    let epoch = phase.floor() as i64;
    let local = fract01(phase);
    let spawn = unit_hash(config.seed, u64::from(position.column), epoch as u64);
    if spawn > 0.34 {
        return Rgb8::default();
    }

    let head_y = local * 1.45 - 0.2;
    let distance = (position.y - head_y).abs();
    let trail = pulse(distance, 0.22);
    let twinkle = 0.65 + 0.35 * unit_hash(config.seed ^ 0xa55a, key.index() as u64, epoch as u64);
    scale_color(mix(config.primary, config.secondary, 0.22), trail * twinkle)
}

fn render_fire(
    config: &EffectConfig,
    key: Key,
    position: KeyPosition,
    phase: f32,
) -> Rgb8 {
    let frame_tick = (phase * 12.0).floor() as i64;
    let noise = unit_hash(config.seed, key.index() as u64, frame_tick as u64);
    let upward_heat = (1.0 - position.y * 0.78).clamp(0.0, 1.0);
    let flicker = (upward_heat * (0.45 + noise * 0.75)).clamp(0.0, 1.0);
    let hot = mix(config.secondary, Rgb8::new(0xff, 0xf0, 0x70), flicker * 0.45);
    scale_color(hot, flicker)
}

fn sanitized_speed(speed: f32) -> f32 {
    if speed.is_finite() {
        speed.abs().clamp(0.05, 8.0)
    } else {
        1.0
    }
}

fn cyclic_distance(left: f32, right: f32) -> f32 {
    let distance = (left - right).abs();
    distance.min(1.0 - distance)
}

fn pulse(distance: f32, width: f32) -> f32 {
    if distance >= width {
        0.0
    } else {
        let normalized = 1.0 - distance / width;
        normalized * normalized
    }
}

fn fract01(value: f32) -> f32 {
    value.rem_euclid(1.0)
}

fn scale_color(color: Rgb8, factor: f32) -> Rgb8 {
    let factor = factor.clamp(0.0, 1.0);
    Rgb8::new(
        scale_channel(color.red, factor),
        scale_channel(color.green, factor),
        scale_channel(color.blue, factor),
    )
}

fn scale_channel(channel: u8, factor: f32) -> u8 {
    (f32::from(channel) * factor).round().clamp(0.0, 255.0) as u8
}

fn mix(left: Rgb8, right: Rgb8, amount: f32) -> Rgb8 {
    let amount = amount.clamp(0.0, 1.0);
    let inverse = 1.0 - amount;
    Rgb8::new(
        (f32::from(left.red) * inverse + f32::from(right.red) * amount).round() as u8,
        (f32::from(left.green) * inverse + f32::from(right.green) * amount).round() as u8,
        (f32::from(left.blue) * inverse + f32::from(right.blue) * amount).round() as u8,
    )
}

fn rainbow(hue: f32) -> Rgb8 {
    let hue = fract01(hue) * 6.0;
    let sector = hue.floor() as u8;
    let fraction = hue - f32::from(sector);
    let rising = scale_channel(255, fraction);
    let falling = 255_u8.saturating_sub(rising);

    match sector {
        0 => Rgb8::new(255, rising, 0),
        1 => Rgb8::new(falling, 255, 0),
        2 => Rgb8::new(0, 255, rising),
        3 => Rgb8::new(0, falling, 255),
        4 => Rgb8::new(rising, 0, 255),
        _ => Rgb8::new(255, 0, falling),
    }
}

fn unit_hash(seed: u64, a: u64, b: u64) -> f32 {
    let mut value = seed
        ^ a.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ b.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    let upper = (value >> 40) as u32;
    upper as f32 / 16_777_215.0
}

#[cfg(test)]
mod tests {
    use kd3b_protocol::{ALL_KEYS, Key, LOGICAL_KEY_COUNT, Rgb8};

    use super::{
        Direction, EffectConfig, EffectKind, FrameContext, apply_brightness, key_position, render,
    };

    #[test]
    fn protocol_offset_grid_maps_function_and_number_rows_in_physical_order() {
        let f1 = key_position(Key::F1);
        let digit1 = key_position(Key::Digit1);
        let q = key_position(Key::Q);
        let a = key_position(Key::A);

        assert_eq!((f1.column, f1.row), (1, 0));
        assert_eq!((digit1.column, digit1.row), (1, 1));
        assert_eq!((q.column, q.row), (1, 2));
        assert_eq!((a.column, a.row), (1, 3));
    }

    #[test]
    fn cycle_is_uniform_across_every_key() {
        let config = EffectConfig::new(EffectKind::Cycle);
        let frame = render(&config, FrameContext::new(1.25));

        assert!(frame.iter().all(|color| *color == frame[0]));
        assert_ne!(frame[0], Rgb8::default());
    }

    #[test]
    fn brightness_zero_blacks_frame_and_hundred_preserves_it() {
        let original = [Rgb8::new(100, 150, 200); LOGICAL_KEY_COUNT];
        let mut zero = original;
        apply_brightness(&mut zero, 0);
        assert_eq!(zero, [Rgb8::default(); LOGICAL_KEY_COUNT]);

        let mut full = original;
        apply_brightness(&mut full, 100);
        assert_eq!(full, original);
    }

    #[test]
    fn brightness_is_clamped_at_one_hundred_percent() {
        let original = [Rgb8::new(10, 20, 30); LOGICAL_KEY_COUNT];
        let mut frame = original;
        apply_brightness(&mut frame, 200);
        assert_eq!(frame, original);
    }

    #[test]
    fn deterministic_effects_repeat_for_same_context_and_seed() {
        for kind in [EffectKind::Rain, EffectKind::Fire] {
            let mut config = EffectConfig::new(kind);
            config.seed = 42;
            let left = render(&config, FrameContext::new(12.345));
            let right = render(&config, FrameContext::new(12.345));
            assert_eq!(left, right);
        }
    }

    #[test]
    fn forward_and_reverse_wave_diverge_after_time_advances() {
        let forward = EffectConfig::new(EffectKind::Wave);
        let mut reverse = forward;
        reverse.direction = Direction::Reverse;

        assert_ne!(
            render(&forward, FrameContext::new(2.0)),
            render(&reverse, FrameContext::new(2.0))
        );
    }

    #[test]
    fn every_effect_renders_a_complete_frame_without_panics() {
        for kind in EffectKind::ALL {
            let config = EffectConfig::new(kind);
            let frame = render(&config, FrameContext::new(3.75));
            assert_eq!(frame.len(), LOGICAL_KEY_COUNT);
            for key in ALL_KEYS {
                let _ = frame[key.index()];
            }
        }
    }

    #[test]
    fn non_finite_speed_falls_back_to_a_stable_value() {
        let mut config = EffectConfig::new(EffectKind::Wave);
        config.speed = f32::NAN;
        let frame = render(&config, FrameContext::new(1.0));
        assert_ne!(frame, [Rgb8::default(); LOGICAL_KEY_COUNT]);
    }
}
