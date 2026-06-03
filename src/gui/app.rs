use crate::emulator::EmulatorState;
use crate::gui::{theme, CommandLog, ReceiptViewer, SettingsPanel};
use eframe::egui::{
    Align, CentralPanel, Color32, Frame, Layout, Margin, RichText, Stroke, TopBottomPanel,
};

/// Address the network server listens on (kept in sync with networking::server).
const LISTEN_ADDR: &str = "0.0.0.0:9100";

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Receipt,
    Commands,
    Settings,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Receipt
    }
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Receipt => "🖨  Receipt",
            Tab::Commands => "📋  Commands",
            Tab::Settings => "⚙  Settings",
        }
    }
}

pub struct EscPosEmulatorApp {
    pub emulator_state: std::sync::Arc<tokio::sync::Mutex<EmulatorState>>,
    selected_tab: Tab,
    dark_mode: bool,
    receipt_viewer: ReceiptViewer,
    command_log: CommandLog,
    settings_panel: SettingsPanel,
}

impl Default for EscPosEmulatorApp {
    fn default() -> Self {
        Self {
            emulator_state: std::sync::Arc::new(tokio::sync::Mutex::new(EmulatorState::new())),
            selected_tab: Tab::Receipt,
            dark_mode: true,
            receipt_viewer: ReceiptViewer::new(),
            command_log: CommandLog::new(),
            settings_panel: SettingsPanel::default(),
        }
    }
}

impl EscPosEmulatorApp {
    pub fn new(emulator_state: std::sync::Arc<tokio::sync::Mutex<EmulatorState>>) -> Self {
        Self {
            emulator_state,
            ..Default::default()
        }
    }
}

impl eframe::App for EscPosEmulatorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
        // The server fills the buffer from another thread; repaint regularly so
        // incoming receipts/commands appear live without needing mouse movement.
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}

impl EscPosEmulatorApp {
    fn show(&mut self, ctx: &eframe::egui::Context) {
        self.show_header(ctx);
        self.show_status_bar(ctx);

        CentralPanel::default().show(ctx, |ui| match self.selected_tab {
            Tab::Receipt => self.receipt_viewer.show(ui, &self.emulator_state),
            Tab::Commands => self.command_log.show(ui, &self.emulator_state),
            Tab::Settings => {
                if let Ok(mut state) = self.emulator_state.try_lock() {
                    self.settings_panel.show(ui, &mut state);
                }
            }
        });
    }

    fn show_header(&mut self, ctx: &eframe::egui::Context) {
        let dark = self.dark_mode;
        let header_frame = Frame::none()
            .fill(theme::header_fill(dark))
            .inner_margin(Margin::symmetric(14.0, 10.0));

        TopBottomPanel::top("header")
            .frame(header_frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("🖨  ESC/POS Virtual Printer")
                            .size(14.5)
                            .strong()
                            .color(theme::title_color(dark)),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Theme toggle
                        let toggle = if dark { "☀ Light" } else { "🌙 Dark" };
                        if ui.button(toggle).on_hover_text("Toggle light / dark theme").clicked() {
                            self.dark_mode = !dark;
                            theme::apply_theme(ui.ctx(), self.dark_mode);
                        }
                        ui.separator();
                        ui.label(
                            RichText::new(LISTEN_ADDR)
                                .monospace()
                                .color(Color32::from_gray(if dark { 170 } else { 110 })),
                        );
                        ui.label(
                            RichText::new("● Listening")
                                .strong()
                                .color(Color32::from_rgb(64, 190, 120)),
                        );
                    });
                });

                ui.add_space(8.0);

                // Tab bar
                ui.horizontal(|ui| {
                    for tab in [Tab::Receipt, Tab::Commands, Tab::Settings] {
                        let selected = self.selected_tab == tab;
                        if ui
                            .selectable_label(selected, RichText::new(tab.label()).size(13.0))
                            .clicked()
                        {
                            self.selected_tab = tab;
                        }
                    }
                });
            });
    }

    fn show_status_bar(&mut self, ctx: &eframe::egui::Context) {
        let dark = self.dark_mode;
        let muted = if dark {
            Color32::from_gray(160)
        } else {
            Color32::from_gray(90)
        };
        let status_frame = Frame::none()
            .fill(theme::status_fill(dark))
            .inner_margin(Margin::symmetric(12.0, 6.0))
            .stroke(Stroke::new(1.0, Color32::from_gray(if dark { 45 } else { 200 })));

        TopBottomPanel::bottom("status_bar")
            .frame(status_frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Ok(state) = self.emulator_state.try_lock() {
                        let summary = state.get_status_summary();

                        ui.label(RichText::new(format!("📄 {}", summary.paper_width)).color(muted));
                        ui.separator();
                        ui.label(
                            RichText::new(format!("📑 {} lines", summary.buffer_lines))
                                .color(muted),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("⚙ {} commands", summary.command_count))
                                .color(muted),
                        );

                        if let Ok(elapsed) = state.start_time.elapsed() {
                            ui.separator();
                            let secs = elapsed.as_secs();
                            let uptime = format!(
                                "⏱ {:02}:{:02}:{:02}",
                                secs / 3600,
                                (secs % 3600) / 60,
                                secs % 60
                            );
                            ui.label(RichText::new(uptime).color(muted));
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .small()
                                .color(muted),
                        );
                    });
                });
            });
    }
}
