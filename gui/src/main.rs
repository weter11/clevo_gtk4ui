mod config;
mod dbus_client;
mod egui_ui;

use eframe::NativeOptions;
use egui_ui::app::TuxedoControlCenterApp;

fn main() -> eframe::Result<()> {
    // Initialize logging
    env_logger::init();

    // Run the egui application
    eframe::run_native(
        "TUXEDO Control Center",
        NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(TuxedoControlCenterApp::new()))),
    )
}
