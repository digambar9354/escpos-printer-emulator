//! Centralised fonts, type scale and light/dark theme setup.

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke,
    TextStyle, Visuals};

/// Refined indigo accent used across both themes.
const ACCENT: Color32 = Color32::from_rgb(108, 122, 255);

/// Install native system fonts (with broad symbol/emoji fallback) so the UI
/// matches the OS and no glyphs render as missing-glyph "tofu" boxes.
pub fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    // Primary proportional UI font.
    let ui_candidates = [
        "C:/Windows/Fonts/segoeui.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/Library/Fonts/Arial.ttf",
    ];
    if let Some(bytes) = load_font(&ui_candidates) {
        fonts.font_data.insert("ui".to_owned(), FontData::from_owned(bytes));
        if let Some(fam) = fonts.families.get_mut(&FontFamily::Proportional) {
            fam.insert(0, "ui".to_owned());
        }
    }

    // Monospace font for the receipt body.
    let mono_candidates = [
        "C:/Windows/Fonts/consola.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    ];
    if let Some(bytes) = load_font(&mono_candidates) {
        fonts.font_data.insert("mono".to_owned(), FontData::from_owned(bytes));
        if let Some(fam) = fonts.families.get_mut(&FontFamily::Monospace) {
            fam.insert(0, "mono".to_owned());
        }
    }

    // Symbol fallback (printer, receipt, etc.) appended to BOTH families so
    // pictographs resolve instead of showing as boxes.
    let symbol_candidates = [
        "C:/Windows/Fonts/seguisym.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
    ];
    if let Some(bytes) = load_font(&symbol_candidates) {
        fonts.font_data.insert("symbols".to_owned(), FontData::from_owned(bytes));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            if let Some(fam) = fonts.families.get_mut(&family) {
                fam.push("symbols".to_owned());
            }
        }
    }

    // Colour/emoji fallback (Segoe UI Emoji on Windows) for the remaining pictographs.
    let emoji_candidates = ["C:/Windows/Fonts/seguiemj.ttf"];
    if let Some(bytes) = load_font(&emoji_candidates) {
        fonts.font_data.insert("emoji".to_owned(), FontData::from_owned(bytes));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            if let Some(fam) = fonts.families.get_mut(&family) {
                fam.push("emoji".to_owned());
            }
        }
    }

    ctx.set_fonts(fonts);
}

/// Apply the compact, VS Code-like type scale and dense spacing (theme-independent).
pub fn apply_style(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(7.0, 5.0);
    style.spacing.button_padding = egui::vec2(9.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(10.0);
    ctx.set_style(style);
}

/// Apply the light or dark colour theme. Safe to call at runtime when toggling.
pub fn apply_theme(ctx: &Context, dark: bool) {
    let mut visuals = if dark { Visuals::dark() } else { Visuals::light() };

    if dark {
        visuals.panel_fill = Color32::from_rgb(21, 22, 27);
        visuals.window_fill = Color32::from_rgb(24, 25, 31);
        visuals.extreme_bg_color = Color32::from_rgb(15, 16, 20);
        visuals.faint_bg_color = Color32::from_rgb(31, 33, 41);
        visuals.selection.bg_fill = Color32::from_rgb(67, 71, 150);
    } else {
        visuals.panel_fill = Color32::from_rgb(243, 244, 247);
        visuals.window_fill = Color32::from_rgb(252, 252, 253);
        visuals.extreme_bg_color = Color32::from_rgb(255, 255, 255);
        visuals.faint_bg_color = Color32::from_rgb(234, 236, 240);
        visuals.selection.bg_fill = Color32::from_rgb(197, 207, 255);
    }

    visuals.hyperlink_color = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.window_rounding = Rounding::same(10.0);

    let r = Rounding::same(7.0);
    visuals.widgets.noninteractive.rounding = r;
    visuals.widgets.inactive.rounding = r;
    visuals.widgets.hovered.rounding = r;
    visuals.widgets.active.rounding = r;
    visuals.widgets.open.rounding = r;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT.linear_multiply(0.6));

    ctx.set_visuals(visuals);
}

/// Header background colour for the current theme.
pub fn header_fill(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(30, 33, 39)
    } else {
        Color32::from_rgb(232, 234, 240)
    }
}

/// Status-bar background colour for the current theme.
pub fn status_fill(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(20, 22, 26)
    } else {
        Color32::from_rgb(228, 230, 236)
    }
}

/// Primary heading/title text colour for the current theme.
pub fn title_color(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(235, 238, 245)
    } else {
        Color32::from_rgb(30, 33, 40)
    }
}

fn load_font(paths: &[&str]) -> Option<Vec<u8>> {
    for path in paths {
        if let Ok(bytes) = std::fs::read(path) {
            return Some(bytes);
        }
    }
    None
}
