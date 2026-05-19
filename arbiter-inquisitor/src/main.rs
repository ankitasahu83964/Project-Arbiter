#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arbiter_core::protocol::{LogEntry, PIPE_TELEMETRY};
use eframe::egui;
use futures::StreamExt;
use std::sync::{Arc, Mutex};
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

struct Palette;
impl Palette {
    const SUCCESS: egui::Color32 = egui::Color32::from_rgb(16, 185, 129);
    const WARN: egui::Color32 = egui::Color32::from_rgb(245, 158, 11);
    const ERROR: egui::Color32 = egui::Color32::from_rgb(244, 63, 94);
    const SYSTEM: egui::Color32 = egui::Color32::from_rgb(99, 102, 241);
}

struct InquisitorApp {
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl InquisitorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let logs = Arc::new(Mutex::new(Vec::new()));
        let logs_clone = logs.clone();
        let ctx = cc.egui_ctx.clone();

        // Brutalist Aesthetic
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = egui::Rounding::ZERO;
        visuals.menu_rounding = egui::Rounding::ZERO;
        visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(15, 15, 15);
        visuals.widgets.inactive.rounding = egui::Rounding::ZERO;
        visuals.widgets.hovered.rounding = egui::Rounding::ZERO;
        visuals.widgets.active.rounding = egui::Rounding::ZERO;
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::monospace(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::monospace(18.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::monospace(14.0));
        ctx.set_style(style);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                loop {
                    use tokio::net::windows::named_pipe::ClientOptions;
                    if let Ok(client) = ClientOptions::new().open(PIPE_TELEMETRY) {
                        let mut framed = FramedRead::new(client, LengthDelimitedCodec::new());
                        while let Some(Ok(bytes)) = framed.next().await {
                            if let Ok(mut entry) = rmp_serde::from_slice::<LogEntry>(&bytes) {
                                if entry.time.is_empty() {
                                    entry.time =
                                        chrono::Local::now().format("%H:%M:%S").to_string();
                                } else if let Ok(dt) =
                                    chrono::DateTime::parse_from_rfc3339(&entry.time)
                                {
                                    entry.time = dt
                                        .with_timezone(&chrono::Local)
                                        .format("%H:%M:%S")
                                        .to_string();
                                }

                                let mut logs = logs_clone.lock().unwrap();
                                logs.push(entry);
                                if logs.len() > 2000 {
                                    logs.remove(0);
                                }
                                ctx.request_repaint();
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            });
        });

        Self { logs }
    }
}

impl eframe::App for InquisitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("ARBITER INQUISITOR // VIVISECTION TABLE")
                        .strong()
                        .color(egui::Color32::from_rgb(100, 100, 255)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("CLEAR").strong()).clicked() {
                        self.logs.lock().unwrap().clear();
                    }
                });
            });
            ui.add_space(8.0);

            let logs = self.logs.lock().unwrap();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    use egui_extras::{Column, TableBuilder};

                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::initial(100.0).at_least(80.0))
                        .column(Column::initial(80.0).at_least(60.0))
                        .column(Column::remainder().at_least(200.0))
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("TIMESTAMP");
                            });
                            header.col(|ui| {
                                ui.strong("TAG");
                            });
                            header.col(|ui| {
                                ui.strong("MESSAGE");
                            });
                        })
                        .body(|body| {
                            body.rows(18.0, logs.len(), |mut row| {
                                let log = &logs[row.index()];
                                row.col(|ui| {
                                    ui.label(
                                        egui::RichText::new(&log.time)
                                            .color(egui::Color32::GRAY)
                                            .small(),
                                    );
                                });
                                row.col(|ui| {
                                    let color = match log.tag.as_str() {
                                        "ATLAS" => Palette::WARN,
                                        "VIGIL" | "Vigil-fs" => Palette::SYSTEM,
                                        "RUNNER" | "Runner" => Palette::SUCCESS,
                                        "PRESN" => Palette::ERROR,
                                        _ => egui::Color32::LIGHT_GRAY,
                                    };
                                    ui.label(
                                        egui::RichText::new(&log.tag).color(color).strong().small(),
                                    );
                                });
                                row.col(|ui| {
                                    let text_color = if log.is_error {
                                        Palette::ERROR
                                    } else {
                                        egui::Color32::LIGHT_GRAY
                                    };
                                    ui.label(
                                        egui::RichText::new(&log.message).color(text_color).small(),
                                    );
                                });
                            });
                        });
                });
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_title("Arbiter Inquisitor"),
        ..Default::default()
    };
    eframe::run_native(
        "Arbiter Inquisitor",
        native_options,
        Box::new(|cc| Ok(Box::new(InquisitorApp::new(cc)))),
    )
}
