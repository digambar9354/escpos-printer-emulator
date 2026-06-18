use crate::emulator::EmulatorState;
use crate::escpos::printer::{PrinterState, ReceiptLine};
use egui::{
    Color32, ColorImage, Frame, Margin, RichText, Rounding, ScrollArea, Slider, Stroke,
    TextureHandle, TextureOptions, Ui,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Colour of the simulated thermal paper.
const PAPER_COLOR: Color32 = Color32::from_rgb(250, 250, 247);
/// Ink colour for text rendered on the paper.
const INK_COLOR: Color32 = Color32::from_rgb(28, 28, 30);
/// Base monospace size on the paper at 1.0x zoom.
const BASE_TEXT_SIZE: f32 = 13.0;

pub struct ReceiptViewer {
    show_paper_edges: bool,
    show_grid: bool,
    /// Newest print job shown at the top when true.
    newest_first: bool,
    /// Preview zoom factor (applies to both text and printed images).
    zoom: f32,
    /// Cache of rendered bitmap textures (keyed by data hash)
    bitmap_cache: HashMap<u64, TextureHandle>,
}

impl Default for ReceiptViewer {
    fn default() -> Self {
        Self {
            show_paper_edges: true,
            show_grid: false,
            newest_first: true,
            zoom: 1.0,
            bitmap_cache: HashMap::new(),
        }
    }
}

fn hash_bytes(data: &[u8]) -> u64 {
    // Simple FNV-1a hash for cache key
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl ReceiptViewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut Ui, emulator_state: &Arc<Mutex<EmulatorState>>) {
        ui.heading("Receipt Preview");
        ui.add_space(4.0);

        // Single wrapping toolbar so controls never get clipped on narrow windows.
        ui.horizontal_wrapped(|ui| {
            ui.label("🔍 Zoom");
            ui.spacing_mut().slider_width = 90.0;
            ui.add(
                Slider::new(&mut self.zoom, 0.5..=3.0)
                    .fixed_decimals(1)
                    .suffix("x"),
            );
            if ui.button("Reset").clicked() {
                self.zoom = 1.0;
            }
            ui.separator();
            ui.checkbox(&mut self.newest_first, "Newest on top");
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.show_paper_edges, "Edges");
            if ui.button("🗑 Clear").clicked() {
                if let Ok(mut state) = emulator_state.try_lock() {
                    state.clear_printer_buffer();
                }
                self.bitmap_cache.clear();
            }
        });

        ui.separator();
        ui.add_space(6.0);

        // Receipt display area, centred on a darker stage.
        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Ok(state) = emulator_state.try_lock() {
                    ui.vertical_centered(|ui| {
                        self.render_receipt(ui, &state);
                    });
                } else {
                    ui.label("Cannot load emulator state");
                }
            });
    }

    fn render_receipt(&mut self, ui: &mut Ui, state: &EmulatorState) {
        let printer_state = state.get_printer_state();
        let buffer = printer_state.get_buffer();

        if buffer.is_empty() {
            let weak = ui.visuals().weak_text_color();
            ui.add_space(60.0);
            ui.label(RichText::new("🧾").size(48.0).color(weak));
            ui.add_space(8.0);
            ui.label(RichText::new("No receipt yet").size(16.0).color(weak));
            ui.label(
                RichText::new("Send ESC/POS data to 0.0.0.0:9100 to see it printed here")
                    .color(weak),
            );
            return;
        }

        // Metadata chips above the paper.
        ui.horizontal(|ui| {
            chip(ui, &format!("📄 {:?}", printer_state.paper_width));
            chip(ui, &format!("🔤 {:?}", printer_state.current_font));
            chip(ui, &format!("📐 {:?}", printer_state.justification));
            if printer_state.codepage != 0 {
                chip(ui, &format!("🌐 CP {}", printer_state.codepage));
            }
        });
        ui.add_space(10.0);

        // Split the flat buffer into separate print jobs at each paper cut (Separator).
        // Each job becomes its own paper sheet; the most recent can be shown on top.
        let mut jobs: Vec<(Vec<(usize, &ReceiptLine)>, bool)> = Vec::new();
        let mut current: Vec<(usize, &ReceiptLine)> = Vec::new();
        for (idx, line) in buffer.iter().enumerate() {
            match line {
                ReceiptLine::Separator => {
                    jobs.push((std::mem::take(&mut current), true));
                }
                other => current.push((idx, other)),
            }
        }
        if !current.is_empty() {
            jobs.push((current, false));
        }

        let total_jobs = jobs.len();
        let mut order: Vec<usize> = (0..total_jobs).collect();
        if self.newest_first {
            order.reverse();
        }

        for (pos, job_index) in order.iter().enumerate() {
            let (lines, was_cut) = &jobs[*job_index];

            // Small label so multiple jobs are distinguishable.
            if total_jobs > 1 {
                let display_no = job_index + 1;
                let tag = if self.newest_first && pos == 0 {
                    format!("● Receipt #{} (latest)", display_no)
                } else {
                    format!("Receipt #{}", display_no)
                };
                let weak = ui.visuals().weak_text_color();
                ui.label(RichText::new(tag).small().color(weak));
                ui.add_space(2.0);
            }

            self.render_sheet(ui, lines, *was_cut, printer_state);
            ui.add_space(18.0);
        }
    }

    fn render_sheet(
        &mut self,
        ui: &mut Ui,
        lines: &[(usize, &ReceiptLine)],
        was_cut: bool,
        printer_state: &PrinterState,
    ) {
        let zoom = self.zoom;
        let paper_width = printer_state.get_paper_width_dots();

        let mut paper = Frame::none()
            .fill(PAPER_COLOR)
            .inner_margin(Margin::symmetric(22.0, 18.0))
            .rounding(Rounding::same(3.0));
        if self.show_paper_edges {
            paper = paper.stroke(Stroke::new(1.0, Color32::from_gray(205)));
        }

        paper.show(ui, |ui| {
            ui.set_min_width(360.0 * zoom);
            ui.set_max_width(580.0 * zoom);

            // Render dark ink on the light paper regardless of the global dark theme.
            ui.visuals_mut().override_text_color = Some(INK_COLOR);
            ui.visuals_mut().widgets.noninteractive.bg_stroke =
                Stroke::new(1.0, Color32::from_gray(190));

            let default_gap = ui.spacing().item_spacing.y;
            let mut prev_bitmap = false;

            for (line_num, line) in lines {
                match line {
                    ReceiptLine::Text(text) => {
                        if text.is_empty() {
                            // Drop blank lines while we're in a run of image strips so a
                            // multi-band logo joins seamlessly. `prev_bitmap` stays sticky
                            // across blank lines, so several newlines between strips are all
                            // collapsed. Normal blank lines elsewhere keep their spacing.
                            if !prev_bitmap {
                                ui.spacing_mut().item_spacing.y = default_gap;
                                ui.add_space(6.0 * zoom);
                            }
                        } else {
                            ui.spacing_mut().item_spacing.y = default_gap;
                            self.render_text_line(
                                ui,
                                line_num + 1,
                                text,
                                printer_state.emphasis,
                                zoom,
                            );
                            prev_bitmap = false;
                        }
                    }
                    ReceiptLine::Bitmap {
                        width_px,
                        height_px,
                        data,
                    } => {
                        // Image strips of a logo arrive as several rasters; render them
                        // with no vertical gap so they join into one seamless image.
                        ui.spacing_mut().item_spacing.y = 0.0;
                        self.render_bitmap(ui, *width_px, *height_px, data, paper_width, zoom);
                        prev_bitmap = true;
                    }
                    ReceiptLine::Separator => {}
                }
            }

            ui.spacing_mut().item_spacing.y = default_gap;

            if was_cut {
                ui.add_space(10.0 * zoom);
                ui.label(
                    RichText::new("✂ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─")
                        .monospace()
                        .size(13.0 * zoom)
                        .color(Color32::from_gray(150)),
                );
            }
        });
    }

    fn render_text_line(&self, ui: &mut Ui, line_num: usize, text: &str, emphasis: bool, zoom: f32) {
        ui.horizontal(|ui| {
            if self.show_grid {
                ui.label(
                    RichText::new(format!("{:03}", line_num))
                        .monospace()
                        .size(11.0 * zoom)
                        .color(Color32::from_gray(170)),
                );
            }
            let mut rt = RichText::new(text)
                .monospace()
                .size(BASE_TEXT_SIZE * zoom)
                .color(INK_COLOR);
            if emphasis {
                rt = rt.strong();
            }
            ui.label(rt);
        });
    }

    fn render_bitmap(
        &mut self,
        ui: &mut Ui,
        width_px: u32,
        height_px: u32,
        data: &[u8],
        _paper_width: u32,
        zoom: f32,
    ) {
        let cache_key = hash_bytes(data);

        // Get or create texture
        let texture = self.bitmap_cache.entry(cache_key).or_insert_with(|| {
            let rgb_image = PrinterState::bitmap_to_rgb(width_px, height_px, data);
            let size = [rgb_image.width() as usize, rgb_image.height() as usize];
            let pixels: Vec<egui::Color32> = rgb_image
                .pixels()
                .map(|p| egui::Color32::from_rgb(p[0], p[1], p[2]))
                .collect();
            let color_image = ColorImage { size, pixels };
            ui.ctx().load_texture(
                format!("bitmap_{}", cache_key),
                color_image,
                TextureOptions::NEAREST,
            )
        });

        // Fit to ~400px base width, then apply the user's zoom factor.
        let fit = (400.0 / width_px as f32).min(1.0);
        let scale = fit * zoom;
        let display_size = egui::vec2(width_px as f32 * scale, height_px as f32 * scale);
        ui.image((texture.id(), display_size));
    }
}

/// A small rounded "chip" label used for receipt metadata (theme-aware).
fn chip(ui: &mut Ui, text: &str) {
    let fill = ui.visuals().faint_bg_color;
    let text_color = ui.visuals().text_color();
    Frame::none()
        .fill(fill)
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).color(text_color));
        });
}
