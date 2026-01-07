use eframe::egui;
use crate::dbus_client::DbusClient;
use std::cell::RefCell;
use std::rc::Rc;
use tuxedo_common::types::Profile;

pub fn profiles_page(
    ui: &mut egui::Ui,
    profiles: &[Profile],
    current_profile: &str,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    ui.heading("Profiles");
    ui.label(format!("Current Profile: {}", current_profile));

    for profile in profiles {
        let is_current = profile.name == current_profile;
        if ui.selectable_label(is_current, &profile.name).clicked() {
            if let Some(client) = dbus_client.borrow().as_ref() {
                let _ = client.apply_profile(profile);
            }
        }
    }
}
