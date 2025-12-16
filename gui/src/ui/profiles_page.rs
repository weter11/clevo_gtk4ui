use gtk::prelude::*;
use gtk::{Box, Button, CheckButton, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::dbus_client::DbusClient;

pub fn create_page(
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);
    
    // Current profile indicator
    let current_profile = &config.borrow().data.current_profile;
    let indicator = Label::new(Some(&format!("Current Profile: {}", current_profile)));
    indicator.set_css_classes(&["title-2"]);
    main_box.append(&indicator);
    
    // Default Profile Group
    let default_group = adw::PreferencesGroup::builder()
        .title("Default Profile")
        .description("System default profile (cannot be edited or deleted)")
        .build();
    
    let default_profile = config.borrow().data.profiles.iter()
        .find(|p| p.is_default)
        .cloned();
    
    if let Some(profile) = default_profile {
        let profile_row = create_profile_row(&profile, false, config.clone(), dbus_client.clone());
        default_group.add(&profile_row);
    }
    
    main_box.append(&default_group);
    
    // Custom Profiles Group
    let custom_group = adw::PreferencesGroup::builder()
        .title("Custom Profiles")
        .build();
    
    let custom_profiles: Vec<_> = config.borrow().data.profiles.iter()
        .filter(|p| !p.is_default)
        .cloned()
        .collect();
    
    for profile in custom_profiles {
        let profile_row = create_profile_row(&profile, true, config.clone(), dbus_client.clone());
        custom_group.add(&profile_row);
    }
    
    main_box.append(&custom_group);
    
    // Action buttons
    let button_box = Box::new(Orientation::Horizontal, 6);
    button_box.set_halign(gtk::Align::Start);
    button_box.set_margin_top(12);
    
    let new_button = Button::with_label("New Profile");
    let delete_button = Button::with_label("Delete Profile");
    let move_up_button = Button::with_label("⬆️ Move Up");
    let move_down_button = Button::with_label("⬇️ Move Down");
    
    button_box.append(&new_button);
    button_box.append(&delete_button);
    button_box.append(&move_up_button);
    button_box.append(&move_down_button);
    
    main_box.append(&button_box);
    
    scrolled.set_child(Some(&main_box));
    scrolled
}

fn create_profile_row(
    profile: &tuxedo_common::types::Profile,
    editable: bool,
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> adw::ExpanderRow {
    let expander = adw::ExpanderRow::builder()
        .title(&profile.name)
        .build();
    
    // Checkbox to select and apply profile
    let checkbox = CheckButton::new();
    checkbox.set_active(&config.borrow().data.current_profile == &profile.name);
    expander.add_prefix(&checkbox);
    
    let profile_name = profile.name.clone();
    let config_clone = config.clone();
    let dbus_clone = dbus_client.clone();
    let profile_clone = profile.clone();
    checkbox.connect_toggled(move |cb| {
        if cb.is_active() {
            config_clone.borrow_mut().data.current_profile = profile_name.clone();
            let _ = config_clone.borrow().save();
            
            // Apply profile via DBus
            let dbus = dbus_clone.clone();
            let profile = profile_clone.clone();
            gtk::glib::spawn_future_local(async move {
                if let Some(client) = dbus.borrow().as_ref() {
                    let _ = client.apply_profile(&profile);
                }
            });
        }
    });
    
    // Show profile settings
    let cpu_info = format!("CPU: {}", 
        profile.cpu_settings.governor.as_ref().unwrap_or(&"Auto".to_string()));
    let cpu_row = adw::ActionRow::builder()
        .title("CPU Settings")
        .subtitle(&cpu_info)
        .build();
    expander.add_row(&cpu_row);
    
    let fan_info = if profile.fan_settings.control_enabled {
        format!("Fans: Custom ({} curves)", profile.fan_settings.curves.len())
    } else {
        "Fans: Auto".to_string()
    };
    let fan_row = adw::ActionRow::builder()
        .title("Fan Settings")
        .subtitle(&fan_info)
        .build();
    expander.add_row(&fan_row);
    
    // Edit button (if editable)
    if editable {
        let edit_button = Button::with_label("Edit");
        edit_button.set_valign(gtk::Align::Center);
        expander.add_suffix(&edit_button);
    }
    
    expander
}