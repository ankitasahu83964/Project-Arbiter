#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arbiter_core::protocol::{LogEntry, PIPE_TELEMETRY};
use eframe::{egui, epaint};
use futures::StreamExt;
use globset::Glob;
use std::sync::{Arc, Mutex};
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

struct Palette;

impl Palette {
    const SUCCESS: egui::Color32 = egui::Color32::from_rgb(16, 185, 129);
    const WARN: egui::Color32 = egui::Color32::from_rgb(245, 158, 11);
    const ERROR: egui::Color32 = egui::Color32::from_rgb(244, 63, 94);
    //const SYSTEM: egui::Color32 = egui::Color32::from_rgb(99, 102, 241);
}

struct InquisitorApp {
    logs: Arc<Mutex<Vec<LogEntry>>>,
    test_path: String,
    glob_pattern: String,
    is_match: bool,
    match_error: Option<String>,
}

impl InquisitorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let logs = Arc::new(Mutex::new(Vec::new()));
        let logs_clone = logs.clone();
        let ctx = cc.egui_ctx.clone();

        // UI Theme
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = egui::Rounding::ZERO;
        visuals.menu_rounding = egui::Rounding::ZERO;
        visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
        visuals.window_shadow = epaint::Shadow::NONE;
        ctx.set_visuals(visuals);

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

        Self {
            logs,
            test_path: String::new(),
            glob_pattern: String::new(),
            is_match: false,
            match_error: None,
        }
    }

    fn update_match_status(&mut self) {
        self.match_error = None;

        if self.glob_pattern.is_empty() || self.test_path.is_empty() {
            self.is_match = false;
            return;
        }

        match Glob::new(&self.glob_pattern) {
            Ok(glob) => {
                let matcher = glob.compile_matcher();
                self.is_match = matcher.is_match(&self.test_path);
            }
            Err(err) => {
                self.is_match = false;
                self.match_error = Some(err.to_string());
            }
        }
    }
}

impl eframe::App for InquisitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ===================== SANDBOX (RIGHT PANEL) =====================
        egui::SidePanel::right("sandbox_panel")
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading(
                    egui::RichText::new("INQUISITOR SANDBOX")
                        .strong()
                        .color(egui::Color32::from_rgb(120, 200, 255)),
                );

                ui.separator();
                ui.add_space(6.0);

                ui.label("Test Path");
                if ui.text_edit_singleline(&mut self.test_path).changed() {
                    self.update_match_status();
                }

                ui.add_space(4.0);

                ui.label("Glob Pattern");
                ui.small("Example: src/**/*.rs");

                if ui.text_edit_singleline(&mut self.glob_pattern).changed() {
                    self.update_match_status();
                }

                ui.add_space(10.0);

                let status_text = if self.is_match { "MATCH" } else { "NO MATCH" };

                let status_color = if self.match_error.is_some() {
                    Palette::WARN
                } else if self.is_match {
                    Palette::SUCCESS
                } else {
                    Palette::ERROR
                };

                ui.label(
                    egui::RichText::new(status_text)
                        .strong()
                        .color(status_color),
                );

                if let Some(err) = &self.match_error {
                    ui.label(
                        egui::RichText::new(format!("Invalid glob: {}", err))
                            .color(Palette::WARN)
                            .small(),
                    );
                }
            });

        // ===================== LOGS (CENTER PANEL) =====================
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("ARBITER INQUISITOR")
                        .strong()
                        .color(egui::Color32::from_rgb(100, 100, 255)),
                );

                if ui.button("CLEAR").clicked() {
                    self.logs.lock().unwrap().clear();
                }
            });

            ui.separator();

            let logs = self.logs.lock().unwrap();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    use egui_extras::{Column, TableBuilder};

                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .column(Column::initial(100.0))
                        .column(Column::initial(80.0))
                        .column(Column::remainder())
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("TIME");
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
                                    ui.label(&log.time);
                                });

                                row.col(|ui| {
                                    ui.label(&log.tag);
                                });

                                row.col(|ui| {
                                    ui.label(&log.message);
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
    )?;

    Ok(())
}
