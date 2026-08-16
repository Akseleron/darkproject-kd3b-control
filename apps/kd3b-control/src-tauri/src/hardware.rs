use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use kd3b_device::open_configuration_interface_transport;
use kd3b_effects::{Direction, EffectConfig, FrameContext, render};
use kd3b_protocol::{LOGICAL_KEY_COUNT, Rgb8};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{parse_color, parse_effect};

const ARM_PHRASE: &[u8] = b"ENABLE VOLATILE RGB";
const FRAME_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Default)]
pub(crate) struct HardwareController {
    shared: Arc<SharedState>,
    worker: Mutex<Option<EffectWorker>>,
}

#[derive(Default)]
struct SharedState {
    armed: AtomicBool,
    runtime: Mutex<RuntimeState>,
}

#[derive(Default)]
struct RuntimeState {
    running: bool,
    frames_written: u64,
    detail: String,
    last_error: Option<String>,
}

struct EffectWorker {
    stop: Arc<AtomicBool>,
    join: JoinHandle<()>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EffectOutputRequest {
    kind: String,
    primary: String,
    secondary: String,
    speed: f32,
    brightness_percent: u8,
    direction: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HardwareOutputStatusDto {
    armed: bool,
    running: bool,
    frames_written: u64,
    detail: String,
    last_error: Option<String>,
}

impl HardwareController {
    fn ensure_armed(&self) -> Result<(), String> {
        if self.shared.armed.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err("hardware output is not armed for this application session".to_owned())
        }
    }

    fn status(&self) -> Result<HardwareOutputStatusDto, String> {
        let runtime = self
            .shared
            .runtime
            .lock()
            .map_err(|_| "hardware runtime status lock is poisoned".to_owned())?;
        Ok(HardwareOutputStatusDto {
            armed: self.shared.armed.load(Ordering::Acquire),
            running: runtime.running,
            frames_written: runtime.frames_written,
            detail: runtime.detail.clone(),
            last_error: runtime.last_error.clone(),
        })
    }

    fn update_status(
        &self,
        running: bool,
        detail: impl Into<String>,
        last_error: Option<String>,
    ) -> Result<(), String> {
        let mut runtime = self
            .shared
            .runtime
            .lock()
            .map_err(|_| "hardware runtime status lock is poisoned".to_owned())?;
        runtime.running = running;
        runtime.detail = detail.into();
        runtime.last_error = last_error;
        Ok(())
    }

    fn stop_worker(&self, detail: &str) -> Result<(), String> {
        let worker = self
            .worker
            .lock()
            .map_err(|_| "hardware worker lock is poisoned".to_owned())?
            .take();

        if let Some(worker) = worker {
            worker.stop.store(true, Ordering::Release);
            if worker.join.join().is_err() {
                self.update_status(
                    false,
                    "hardware effect worker terminated unexpectedly",
                    Some("hardware effect worker thread panicked".to_owned()),
                )?;
                return Err("hardware effect worker thread panicked".to_owned());
            }
        }

        self.update_status(false, detail, None)
    }
}

#[tauri::command]
pub(crate) fn arm_hardware_output(
    controller: State<'_, HardwareController>,
    confirmation: String,
) -> Result<HardwareOutputStatusDto, String> {
    let confirmation = confirmation.into_bytes();
    if confirmation.as_slice() != ARM_PHRASE {
        return Err("confirmation did not exactly match ENABLE VOLATILE RGB".to_owned());
    }

    controller.shared.armed.store(true, Ordering::Release);
    controller.update_status(
        false,
        "volatile RGB output armed for this application session",
        None,
    )?;
    controller.status()
}

#[tauri::command]
pub(crate) fn disarm_hardware_output(
    controller: State<'_, HardwareController>,
) -> Result<HardwareOutputStatusDto, String> {
    controller.stop_worker("hardware output disarmed")?;
    controller.shared.armed.store(false, Ordering::Release);
    controller.status()
}

#[tauri::command]
pub(crate) fn get_hardware_output_status(
    controller: State<'_, HardwareController>,
) -> Result<HardwareOutputStatusDto, String> {
    controller.status()
}

#[tauri::command]
pub(crate) fn apply_static_frame(
    controller: State<'_, HardwareController>,
    colors: Vec<String>,
) -> Result<HardwareOutputStatusDto, String> {
    controller.ensure_armed()?;
    let frame = parse_frame(colors)?;
    controller.stop_worker("preparing one-shot direct RGB frame")?;

    let mut transport =
        open_configuration_interface_transport().map_err(|error| error.to_string())?;
    let selected = transport.selected_metadata().clone();
    transport
        .set_direct_rgb(&frame)
        .map_err(|error| error.to_string())?;

    {
        let mut runtime = controller
            .shared
            .runtime
            .lock()
            .map_err(|_| "hardware runtime status lock is poisoned".to_owned())?;
        runtime.frames_written = runtime.frames_written.saturating_add(1);
        runtime.running = false;
        runtime.detail = format!(
            "one volatile frame written to interface {} at {}",
            selected.interface_number, selected.path
        );
        runtime.last_error = None;
    }

    controller.status()
}

#[tauri::command]
pub(crate) fn start_effect_output(
    controller: State<'_, HardwareController>,
    request: EffectOutputRequest,
) -> Result<HardwareOutputStatusDto, String> {
    controller.ensure_armed()?;
    let config = parse_effect_output_request(request)?;
    controller.stop_worker("starting hardware effect worker")?;

    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let shared = Arc::clone(&controller.shared);
    let join = thread::Builder::new()
        .name("kd3b-rgb-worker".to_owned())
        .spawn(move || run_effect_worker(shared, worker_stop, config))
        .map_err(|error| format!("failed to start hardware effect worker: {error}"))?;

    let mut worker_slot = controller
        .worker
        .lock()
        .map_err(|_| "hardware worker lock is poisoned".to_owned())?;
    *worker_slot = Some(EffectWorker { stop, join });
    drop(worker_slot);

    controller.status()
}

#[tauri::command]
pub(crate) fn stop_effect_output(
    controller: State<'_, HardwareController>,
) -> Result<HardwareOutputStatusDto, String> {
    controller.stop_worker("hardware effect stopped by user")?;
    controller.status()
}

fn parse_effect_output_request(request: EffectOutputRequest) -> Result<EffectConfig, String> {
    let EffectOutputRequest {
        kind,
        primary,
        secondary,
        speed,
        brightness_percent,
        direction,
    } = request;

    let mut config = EffectConfig::new(parse_effect(&kind)?);
    config.primary = parse_color(&primary)?;
    config.secondary = parse_color(&secondary)?;
    config.speed = speed;
    config.brightness_percent = brightness_percent.min(100);
    config.direction = match direction.as_str() {
        "forward" => Direction::Forward,
        "reverse" => Direction::Reverse,
        other => return Err(format!("unknown direction '{other}'")),
    };
    Ok(config)
}

fn parse_frame(colors: Vec<String>) -> Result<[Rgb8; LOGICAL_KEY_COUNT], String> {
    if colors.len() != LOGICAL_KEY_COUNT {
        return Err(format!(
            "direct RGB frame must contain exactly {LOGICAL_KEY_COUNT} colors, got {}",
            colors.len()
        ));
    }

    let mut frame = [Rgb8::default(); LOGICAL_KEY_COUNT];
    for (slot, color) in frame.iter_mut().zip(colors) {
        *slot = parse_color(&color)?;
    }
    Ok(frame)
}

fn run_effect_worker(shared: Arc<SharedState>, stop: Arc<AtomicBool>, config: EffectConfig) {
    let mut transport = match open_configuration_interface_transport() {
        Ok(transport) => transport,
        Err(error) => {
            record_worker_error(
                &shared,
                format!("failed to open configuration interface: {error}"),
            );
            return;
        }
    };

    let selected = transport.selected_metadata().clone();
    update_worker_runtime(
        &shared,
        true,
        format!(
            "streaming volatile RGB to interface {} at {}",
            selected.interface_number, selected.path
        ),
        None,
    );

    let started = Instant::now();
    let mut previous = None;

    while !stop.load(Ordering::Acquire) {
        let frame = render(&config, FrameContext::new(started.elapsed().as_secs_f32()));
        if previous.as_ref() != Some(&frame) {
            if let Err(error) = transport.set_direct_rgb(&frame) {
                record_worker_error(&shared, format!("hardware RGB stream stopped: {error}"));
                return;
            }
            previous = Some(frame);
            increment_written_frames(&shared);
        }
        thread::sleep(FRAME_INTERVAL);
    }

    update_worker_runtime(
        &shared,
        false,
        "hardware effect worker stopped cleanly".to_owned(),
        None,
    );
}

fn increment_written_frames(shared: &SharedState) {
    if let Ok(mut runtime) = shared.runtime.lock() {
        runtime.frames_written = runtime.frames_written.saturating_add(1);
    }
}

fn record_worker_error(shared: &SharedState, error: String) {
    update_worker_runtime(shared, false, error.clone(), Some(error));
}

fn update_worker_runtime(
    shared: &SharedState,
    running: bool,
    detail: String,
    last_error: Option<String>,
) {
    if let Ok(mut runtime) = shared.runtime.lock() {
        runtime.running = running;
        runtime.detail = detail;
        runtime.last_error = last_error;
    }
}

#[cfg(test)]
mod tests {
    use kd3b_protocol::{LOGICAL_KEY_COUNT, Rgb8};

    use super::{EffectOutputRequest, parse_effect_output_request, parse_frame};

    #[test]
    fn direct_frame_parser_requires_exact_keyboard_length() {
        let colors = vec!["#000000".to_owned(); LOGICAL_KEY_COUNT - 1];
        assert!(parse_frame(colors).is_err());
    }

    #[test]
    fn direct_frame_parser_preserves_key_order() {
        let mut colors = vec!["#000000".to_owned(); LOGICAL_KEY_COUNT];
        colors[0] = "#ff0000".to_owned();
        colors[LOGICAL_KEY_COUNT - 1] = "#0000ff".to_owned();

        let frame = parse_frame(colors).expect("synthetic frame is valid");
        assert_eq!(frame[0], Rgb8::new(255, 0, 0));
        assert_eq!(frame[LOGICAL_KEY_COUNT - 1], Rgb8::new(0, 0, 255));
    }

    #[test]
    fn effect_output_request_clamps_brightness() {
        let request = EffectOutputRequest {
            kind: "wave".to_owned(),
            primary: "#112233".to_owned(),
            secondary: "#445566".to_owned(),
            speed: 1.0,
            brightness_percent: 200,
            direction: "forward".to_owned(),
        };

        let config = parse_effect_output_request(request).expect("synthetic request is valid");
        assert_eq!(config.brightness_percent, 100);
    }
}
