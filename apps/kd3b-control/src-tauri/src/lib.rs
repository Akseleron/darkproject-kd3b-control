mod hardware;

use kd3b_device::{
    DiscoveredHidInterface, enumerate_target_hid_interfaces, select_configuration_interface,
};
use kd3b_effects::{Direction, EffectConfig, EffectKind, FrameContext, key_position, render};
use kd3b_protocol::{ALL_KEYS, Rgb8};
use serde::{Deserialize, Serialize};

use hardware::{
    HardwareController, apply_static_frame, arm_hardware_output, disarm_hardware_output,
    get_hardware_output_status, start_effect_output, stop_effect_output,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InterfaceDto {
    interface_number: i32,
    vendor_id: u16,
    product_id: u16,
    product: Option<String>,
    manufacturer: Option<String>,
    release_number: u16,
    bus: String,
    path: String,
}

impl From<&DiscoveredHidInterface> for InterfaceDto {
    fn from(value: &DiscoveredHidInterface) -> Self {
        Self {
            interface_number: value.interface_number,
            vendor_id: value.vendor_id,
            product_id: value.product_id,
            product: value.product_string.clone(),
            manufacturer: value.manufacturer_string.clone(),
            release_number: value.release_number,
            bus: value.bus_type.to_string(),
            path: value.path.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceStatusDto {
    present: bool,
    matching_interfaces: usize,
    configuration_state: String,
    selected: Option<InterfaceDto>,
    detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyDto {
    index: usize,
    name: String,
    column: u8,
    row: u8,
    x: f32,
    y: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EffectDto {
    id: &'static str,
    label: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreviewRequest {
    kind: String,
    primary: String,
    secondary: String,
    speed: f32,
    brightness_percent: u8,
    direction: String,
    elapsed_seconds: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewFrameDto {
    colors: Vec<String>,
}

#[tauri::command]
fn get_device_status() -> Result<DeviceStatusDto, String> {
    let interfaces = enumerate_target_hid_interfaces().map_err(|error| error.to_string())?;
    let matching_interfaces = interfaces.len();
    if interfaces.is_empty() {
        return Ok(DeviceStatusDto {
            present: false,
            matching_interfaces,
            configuration_state: "missing".to_owned(),
            selected: None,
            detail: "KD3B rev.2 (195d:2061) не обнаружена".to_owned(),
        });
    }

    match select_configuration_interface(&interfaces) {
        Ok(index) => {
            let selected = interfaces
                .get(index.get())
                .ok_or_else(|| "selector returned an invalid interface index".to_owned())?;
            Ok(DeviceStatusDto {
                present: true,
                matching_interfaces,
                configuration_state: "ready".to_owned(),
                selected: Some(InterfaceDto::from(selected)),
                detail: "Интерфейс 2 однозначно выбран. Предпросмотр не открывает устройство и не пишет HID-пакеты."
                    .to_owned(),
            })
        }
        Err(error) => Ok(DeviceStatusDto {
            present: true,
            matching_interfaces,
            configuration_state: "blocked".to_owned(),
            selected: None,
            detail: error.to_string(),
        }),
    }
}

#[tauri::command]
fn get_layout() -> Vec<KeyDto> {
    ALL_KEYS
        .into_iter()
        .map(|key| {
            let position = key_position(key);
            KeyDto {
                index: key.index(),
                name: format!("{key:?}"),
                column: position.column,
                row: position.row,
                x: position.x,
                y: position.y,
            }
        })
        .collect()
}

#[tauri::command]
fn get_effect_catalog() -> Vec<EffectDto> {
    EffectKind::ALL
        .into_iter()
        .map(|kind| EffectDto {
            id: kind.name(),
            label: effect_label(kind),
        })
        .collect()
}

#[tauri::command]
fn preview_effect(request: PreviewRequest) -> Result<PreviewFrameDto, String> {
    let PreviewRequest {
        kind,
        primary,
        secondary,
        speed,
        brightness_percent,
        direction,
        elapsed_seconds,
    } = request;

    let kind = parse_effect(&kind)?;
    let primary = parse_color(&primary)?;
    let secondary = parse_color(&secondary)?;
    let direction = match direction.as_str() {
        "forward" => Direction::Forward,
        "reverse" => Direction::Reverse,
        other => return Err(format!("unknown direction '{other}'")),
    };

    let mut config = EffectConfig::new(kind);
    config.primary = primary;
    config.secondary = secondary;
    config.speed = speed;
    config.brightness_percent = brightness_percent.min(100);
    config.direction = direction;

    let frame = render(&config, FrameContext::new(elapsed_seconds));
    Ok(PreviewFrameDto {
        colors: frame.into_iter().map(color_hex).collect(),
    })
}

fn parse_effect(value: &str) -> Result<EffectKind, String> {
    EffectKind::ALL
        .into_iter()
        .find(|kind| kind.name() == value)
        .ok_or_else(|| format!("unknown effect '{value}'"))
}

fn parse_color(value: &str) -> Result<Rgb8, String> {
    let value = value.strip_prefix('#').unwrap_or(value);
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("color must contain exactly six hexadecimal digits".to_owned());
    }

    let red = u8::from_str_radix(&value[0..2], 16).map_err(|error| error.to_string())?;
    let green = u8::from_str_radix(&value[2..4], 16).map_err(|error| error.to_string())?;
    let blue = u8::from_str_radix(&value[4..6], 16).map_err(|error| error.to_string())?;
    Ok(Rgb8::new(red, green, blue))
}

fn color_hex(color: Rgb8) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
}

const fn effect_label(kind: EffectKind) -> &'static str {
    match kind {
        EffectKind::Gradient => "Градиент",
        EffectKind::Wave => "Волна",
        EffectKind::ConicBand => "Коническая полоса",
        EffectKind::Spiral => "Спираль",
        EffectKind::Cycle => "Цикл цвета",
        EffectKind::LinearWave => "Линейная волна",
        EffectKind::Ripple => "Круговая волна",
        EffectKind::Breathing => "Дыхание",
        EffectKind::Rain => "Дождь",
        EffectKind::Fire => "Огонь",
    }
}

/// Runs the KD3B Control desktop application.
///
/// # Panics
///
/// Panics if Tauri cannot start the application or its event loop fails.
pub fn run() {
    tauri::Builder::default()
        .manage(HardwareController::default())
        .invoke_handler(tauri::generate_handler![
            get_device_status,
            get_layout,
            get_effect_catalog,
            preview_effect,
            arm_hardware_output,
            disarm_hardware_output,
            get_hardware_output_status,
            apply_static_frame,
            start_effect_output,
            stop_effect_output
        ])
        .run(tauri::generate_context!())
        .expect("failed to run KD3B Control desktop application");
}

#[cfg(test)]
mod tests {
    use kd3b_protocol::Rgb8;

    use super::{color_hex, parse_color, parse_effect};

    #[test]
    fn color_parser_accepts_hash_or_plain_hex() {
        assert_eq!(parse_color("#12aBef"), Ok(Rgb8::new(0x12, 0xab, 0xef)));
        assert_eq!(parse_color("12abef"), Ok(Rgb8::new(0x12, 0xab, 0xef)));
    }

    #[test]
    fn color_parser_rejects_invalid_values() {
        for value in ["fff", "gg0000", "00112233"] {
            assert!(parse_color(value).is_err());
        }
    }

    #[test]
    fn effect_parser_matches_engine_catalog() {
        assert!(parse_effect("wave").is_ok());
        assert!(parse_effect("not-an-effect").is_err());
    }

    #[test]
    fn color_formatter_is_css_compatible() {
        assert_eq!(color_hex(Rgb8::new(0, 17, 255)), "#0011ff");
    }
}
