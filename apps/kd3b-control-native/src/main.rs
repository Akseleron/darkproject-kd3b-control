use std::time::{Duration, Instant};

use iced::mouse;
use iced::time;
use iced::widget::canvas::{self, Geometry, Path};
use iced::widget::{Column, canvas, container, scrollable, text};
use iced::{Color, Element, Fill, Length, Point, Rectangle, Renderer, Size, Subscription, Theme};
use kd3b_device::{enumerate_target_hid_interfaces, select_configuration_interface};
use kd3b_effects::{EffectConfig, EffectKind, Frame, FrameContext, key_position, render};
use kd3b_protocol::{ALL_KEYS, Rgb8};

const FRAME_INTERVAL: Duration = Duration::from_micros(16_667);
const METRIC_WINDOW: Duration = Duration::from_millis(750);

pub fn main() -> iced::Result {
    iced::application(NativeApp::new, NativeApp::update, NativeApp::view)
        .subscription(NativeApp::subscription)
        .theme(|_| Theme::Dark)
        .run()
}

struct NativeApp {
    started_at: Instant,
    frame: Frame,
    effect: EffectConfig,
    metric_started_at: Instant,
    metric_ticks: u16,
    measured_update_hz: f32,
    device_summary: String,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(Instant),
}

impl NativeApp {
    fn new() -> Self {
        let now = Instant::now();
        let effect = EffectConfig::new(EffectKind::Wave);
        Self {
            started_at: now,
            frame: render(&effect, FrameContext::new(0.0)),
            effect,
            metric_started_at: now,
            metric_ticks: 0,
            measured_update_hz: 0.0,
            device_summary: read_only_device_summary(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                let elapsed = now.duration_since(self.started_at).as_secs_f32();
                self.frame = render(&self.effect, FrameContext::new(elapsed));
                self.metric_ticks = self.metric_ticks.saturating_add(1);

                let metric_elapsed = now.duration_since(self.metric_started_at);
                if metric_elapsed >= METRIC_WINDOW {
                    self.measured_update_hz =
                        f32::from(self.metric_ticks) / metric_elapsed.as_secs_f32();
                    self.metric_ticks = 0;
                    self.metric_started_at = now;
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(FRAME_INTERVAL).map(Message::Tick)
    }

    fn view(&self) -> Element<'_, Message> {
        let heading = Column::new()
            .spacing(6)
            .push(text("KD3B Control · native renderer checkpoint").size(28))
            .push(text(format!(
                "Iced 0.14 / wgpu · update {:.1} Hz · {}",
                self.measured_update_hz, self.device_summary
            )));

        let preview = canvas(self).width(Fill).height(Length::Fixed(390.0));

        let mut content = Column::new()
            .spacing(18)
            .padding(24)
            .push(heading)
            .push(container(preview).width(Fill).padding(16));

        for index in 1..=8 {
            content = content.push(
                container(
                    Column::new()
                        .spacing(8)
                        .push(text(format!("Панель прокрутки {index}")).size(20))
                        .push(text(
                            "Эти блоки намеренно создают длинную страницу. Активно прокручивай её, пока RGB-preview продолжает обновляться.",
                        )),
                )
                .width(Fill)
                .height(Length::Fixed(150.0))
                .padding(18),
            );
        }

        scrollable(content).height(Fill).into()
    }
}

impl canvas::Program<Message> for NativeApp {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut canvas_frame = canvas::Frame::new(renderer, bounds.size());
        let background = Path::rectangle(Point::ORIGIN, bounds.size());
        canvas_frame.fill(&background, Color::from_rgb8(9, 12, 17));

        let margin = 18.0;
        let usable_width = (bounds.width - margin * 2.0).max(1.0);
        let usable_height = (bounds.height - margin * 2.0).max(1.0);
        let key_width = (usable_width / 18.5).max(3.0);
        let key_height = (usable_height / 6.4).max(3.0);

        for key in ALL_KEYS {
            let position = key_position(key);
            let x = margin + position.x * (usable_width - key_width);
            let y = margin + position.y * (usable_height - key_height);
            let rect = Path::rectangle(
                Point::new(x, y),
                Size::new((key_width - 3.0).max(1.0), (key_height - 3.0).max(1.0)),
            );
            canvas_frame.fill(&rect, iced_color(self.frame[key.index()]));
        }

        vec![canvas_frame.into_geometry()]
    }
}

fn iced_color(color: Rgb8) -> Color {
    Color::from_rgb8(color.red, color.green, color.blue)
}

fn read_only_device_summary() -> String {
    match enumerate_target_hid_interfaces() {
        Err(error) => format!("device metadata error: {error}"),
        Ok(interfaces) => match select_configuration_interface(&interfaces) {
            Ok(index) => interfaces.get(index.get()).map_or_else(
                || "interface selector returned an invalid index".to_owned(),
                |selected| {
                    format!(
                        "KD3B ready · interface {} · {}",
                        selected.interface_number, selected.path
                    )
                },
            ),
            Err(error) => format!("KD3B not ready: {error}"),
        },
    }
}
