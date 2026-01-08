mod app;
mod dbus_client;
mod theme;
mod pages;
mod keyboard_shortcuts;

use app::TuxedoApp;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };
    
    eframe::run_native(
        "TUXEDO Control Center",
        options,
        Box::new(|cc| Ok(Box::new(TuxedoApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    egui::IconData::default()
}
