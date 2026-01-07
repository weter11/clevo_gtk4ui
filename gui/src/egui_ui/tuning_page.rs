use eframe::egui;
use crate::dbus_client::DbusClient;
use std::cell::RefCell;
use std::rc::Rc;
use tuxedo_common::types::Profile;

pub fn tuning_page(
    ui: &mut egui::Ui,
    profile: &mut Profile,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    ui.heading(format!("Editing Profile: {}", profile.name));

    if ui.button("Apply & Save Profile").clicked() {
        if let Some(client) = dbus_client.borrow().as_ref() {
            let _ = client.apply_profile(profile);
        }
    }

    // CPU Tuning
    ui.collapsing("CPU Tuning", |ui| {
        // Boost
        ui.checkbox(&mut profile.cpu_settings.boost.unwrap_or(false), "CPU Boost / Turbo");
        // SMT
        ui.checkbox(&mut profile.cpu_settings.smt.unwrap_or(false), "SMT / Hyperthreading");
    });
}
