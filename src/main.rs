use eframe::egui;
use escpos_emulator::emulator::EmulatorState;
use escpos_emulator::gui::EscPosEmulatorApp;
use escpos_emulator::networking::server;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting ESC/POS Emulator...");

    // Create shared emulator state
    let emulator_state = Arc::new(Mutex::new(EmulatorState::new()));

    // Start network server in background
    let server_state = emulator_state.clone();
    tokio::spawn(async move {
        if let Err(e) = server::start_server(server_state).await {
            eprintln!("❌ Server error: {}", e);
        }
    });

    info!("✅ Emulator initialized successfully");

    // Launch GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 560.0])
            .with_min_inner_size([520.0, 400.0])
            .with_title("ESC/POS Virtual Printer Emulator"),
        ..Default::default()
    };

    eframe::run_native(
        "ESC/POS Virtual Printer Emulator",
        options,
        Box::new(|cc| {
            use escpos_emulator::gui::theme;
            theme::install_fonts(&cc.egui_ctx);
            theme::apply_style(&cc.egui_ctx);
            theme::apply_theme(&cc.egui_ctx, true); // start in dark mode
            Box::new(EscPosEmulatorApp::new(emulator_state))
        }),
    )
}

