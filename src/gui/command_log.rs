use crate::emulator::EmulatorState;
use egui::{Color32, Frame, Margin, RichText, Rounding, ScrollArea, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CommandLog {
    show_timestamps: bool,
    show_raw_data: bool,
    max_display_lines: usize,
    filter_text: String,
}

impl Default for CommandLog {
    fn default() -> Self {
        Self {
            show_timestamps: true,
            show_raw_data: false,
            max_display_lines: 1000,
            filter_text: String::new(),
        }
    }
}

impl CommandLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut Ui, emulator_state: &Arc<Mutex<EmulatorState>>) {
        // Toolbar
        ui.horizontal(|ui| {
            ui.heading("Command Log");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Clear").clicked() {
                    if let Ok(mut state) = emulator_state.try_lock() {
                        state.clear_history();
                    }
                }
                ui.checkbox(&mut self.show_raw_data, "Raw hex");
                ui.checkbox(&mut self.show_timestamps, "Timestamps");
            });
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.filter_text)
                    .hint_text("Filter commands…")
                    .desired_width(240.0),
            );
            if !self.filter_text.is_empty() && ui.button("✕").clicked() {
                self.filter_text.clear();
            }
        });

        ui.separator();
        ui.add_space(4.0);

        // Log area
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Ok(state) = emulator_state.try_lock() {
                    self.render_command_list(ui, &state);
                } else {
                    ui.label("Cannot load emulator state");
                }
            });
    }

    fn render_command_list(&self, ui: &mut Ui, state: &EmulatorState) {
        let history = state.get_command_history();

        if history.is_empty() {
            let weak = ui.visuals().weak_text_color();
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("No commands received yet").color(weak));
            });
            return;
        }

        // Apply filter
        let filtered_commands: Vec<_> = history
            .iter()
            .filter(|entry| {
                if self.filter_text.is_empty() {
                    return true;
                }

                match &entry.command {
                    crate::escpos::commands::EscPosCommand::Text(text) => {
                        text.to_lowercase().contains(&self.filter_text.to_lowercase())
                    }
                    _ => format!("{:?}", entry.command)
                        .to_lowercase()
                        .contains(&self.filter_text.to_lowercase()),
                }
            })
            .collect();

        // Limit displayed lines
        let display_commands: Vec<_> = filtered_commands
            .iter()
            .rev() // Most recent first
            .take(self.max_display_lines)
            .collect();

        let display_count = display_commands.len();
        for entry in &display_commands {
            self.render_command_entry(ui, entry);
        }

        // Statistics
        ui.add_space(6.0);
        ui.separator();
        ui.label(
            RichText::new(format!(
                "Total: {}  •  Displayed: {}  •  Filtered: {}",
                history.len(),
                display_count,
                filtered_commands.len()
            ))
            .small()
            .color(ui.visuals().weak_text_color()),
        );
    }

    fn render_command_entry(&self, ui: &mut Ui, entry: &crate::emulator::CommandEntry) {
        let default_text = ui.visuals().text_color();
        let weak = ui.visuals().weak_text_color();
        let (text, color) = describe_command(&entry.command, default_text);
        let fill = ui.visuals().faint_bg_color;
        let border = ui.visuals().widgets.noninteractive.bg_stroke;

        Frame::none()
            .fill(fill)
            .rounding(Rounding::same(6.0))
            .stroke(border)
            .inner_margin(Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.show_timestamps {
                        if let Ok(duration) =
                            entry.timestamp.duration_since(std::time::UNIX_EPOCH)
                        {
                            let secs = duration.as_secs();
                            let time_str = if secs < 60 {
                                format!("{}s", secs)
                            } else if secs < 3600 {
                                format!("{}m{}s", secs / 60, secs % 60)
                            } else {
                                format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
                            };
                            ui.label(
                                RichText::new(format!("⏰ {}", time_str))
                                    .small()
                                    .color(weak),
                            );
                        }
                    }
                    ui.label(RichText::new(text).color(color));
                });

                if self.show_raw_data && !entry.raw_data.is_empty() {
                    let hex_data: String = entry
                        .raw_data
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    ui.label(RichText::new(hex_data).monospace().small().color(weak));
                }
            });
        ui.add_space(4.0);
    }
}

/// Map a command to a human-readable label and a colour for the log entry.
fn describe_command(
    command: &crate::escpos::commands::EscPosCommand,
    text: Color32,
) -> (String, Color32) {
    use crate::escpos::commands::EscPosCommand as C;

    let control = Color32::from_rgb(120, 175, 255);
    let toggle = Color32::from_rgb(120, 205, 150);
    let media = Color32::from_rgb(190, 150, 250);
    let danger = Color32::from_rgb(240, 130, 120);
    let muted = Color32::from_gray(150);

    match command {
        C::Text(t) => (format!("📝 {}", t), text),
        C::NewLine => ("↵ New line".to_string(), muted),
        C::SetFont(font) => (format!("🔤 Font: {:?}", font), control),
        C::SetJustification(just) => (format!("📐 Justification: {:?}", just), control),
        C::SetEmphasis(on) => (
            format!("💪 Emphasis: {}", if *on { "ON" } else { "OFF" }),
            toggle,
        ),
        C::SetUnderline(on) => (
            format!("➖ Underline: {}", if *on { "ON" } else { "OFF" }),
            toggle,
        ),
        C::SetItalic(on) => (
            format!("✒ Italic: {}", if *on { "ON" } else { "OFF" }),
            toggle,
        ),
        C::CutPaper => ("✂️ Paper cut".to_string(), danger),
        C::PrintImage(_) => ("🖼️ Bit Image (ESC *)".to_string(), media),
        C::PrintRasterImage {
            width_bytes,
            height,
            ..
        } => (
            format!("🖼️ Raster Image (GS v 0) {}×{}", width_bytes * 8, height),
            media,
        ),
        C::SetCodepage(cp) => (format!("🌐 Codepage: {}", cp), control),
        C::SetLineHeight(h) => (format!("📏 Line height: {}", h), control),
        C::SetFontSize(s) => (format!("🔤 Font size: {}", s), control),
        C::Unknown(_) => ("❓ Unknown command".to_string(), muted),
        other => (format!("⚙️ {:?}", other), muted),
    }
}
