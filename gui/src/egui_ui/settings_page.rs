use eframe::egui;
use crate::config::Config;

pub fn settings_page(ui: &mut egui::Ui, config: &mut Config) {
    ui.heading("Settings");

    if ui.button("Save Settings").clicked() {
        let _ = config.save();
    }

    ui.collapsing("Appearance", |ui| {
        egui::ComboBox::from_label("Theme")
            .selected_text(format!("{:?}", config.data.theme))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut config.data.theme, tuxedo_common::types::Theme::Auto, "Auto");
                ui.selectable_value(&mut config.data.theme, tuxedo_common::types::Theme::Light, "Light");
                ui.selectable_value(&mut config.data.theme, tuxedo_common::types::Theme::Dark, "Dark");
            });
    });

    ui.collapsing("Startup", |ui| {
        ui.checkbox(&mut config.data.start_minimized, "Start minimized");
        ui.checkbox(&mut config.data.autostart, "Enable autostart");
    });
}
