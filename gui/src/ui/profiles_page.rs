use gtk::prelude::*;
use gtk::{Box, Button, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::dbus_client::DbusClient;
use tuxedo_common::types::Profile;

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
    let indicator = gtk::Label::new(Some(&format!("Current Profile: {}", current_profile)));
    indicator.set_css_classes(&["title-2"]);
    main_box.append(&indicator);
    
    // Profiles list
    let profiles_list = create_profiles_list(config.clone(), dbus_client.clone());
    main_box.append(&profiles_list);
    
    // Action buttons
    let button_box = Box::new(Orientation::Horizontal, 6);
    button_box.set_halign(gtk::Align::Start);
    button_box.set_margin_top(12);
    
    let new_button = Button::with_label("➕ New Profile");
    let new_config = config.clone();
    let new_dbus = dbus_client.clone();
    new_button.connect_clicked(move |_| {
        create_new_profile(new_config.clone(), new_dbus.clone());
    });
    button_box.append(&new_button);
    
    main_box.append(&button_box);
    
    scrolled.set_child(Some(&main_box));
    scrolled
}

fn create_profiles_list(
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> Box {
    let list_box = Box::new(Orientation::Vertical, 12);
    
    let profiles = config.borrow().data.profiles.clone();
    
    for profile in profiles {
        let profile_card = create_profile_card(&profile, config.clone(), dbus_client.clone());
        list_box.append(&profile_card);
    }
    
    list_box
}

fn create_profile_card(
    profile: &Profile,
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> adw::ExpanderRow {
    let expander = adw::ExpanderRow::builder()
        .title(&profile.name)
        .subtitle(if profile.is_default { "Default Profile" } else { "Custom Profile" })
        .build();
    
    // Radio button for selection (toggle behavior)
    let radio = gtk::CheckButton::new();
    radio.set_active(&config.borrow().data.current_profile == &profile.name);
    expander.add_prefix(&radio);
    
    // Connect radio button to apply profile IMMEDIATELY
    let profile_name = profile.name.clone();
    let config_clone = config.clone();
    let dbus_clone = dbus_client.clone();
    let profile_clone = profile.clone();
    radio.connect_toggled(move |rb| {
        if rb.is_active() {
            config_clone.borrow_mut().data.current_profile = profile_name.clone();
            let _ = config_clone.borrow().save();
            
            // Apply profile IMMEDIATELY via DBus
            if let Some(client) = dbus_clone.borrow().as_ref() {
                match client.apply_profile(&profile_clone) {
                    Ok(_) => println!("Profile '{}' applied successfully", profile_name),
                    Err(e) => eprintln!("Failed to apply profile '{}': {}", profile_name, e),
                }
            }
        }
    });
    
    // CPU Settings
    let cpu_info = if let Some(ref gov) = profile.cpu_settings.governor {
        format!("CPU Governor: {}", gov)
    } else {
        "CPU: Auto".to_string()
    };
    let cpu_row = adw::ActionRow::builder()
        .title("CPU Settings")
        .subtitle(&cpu_info)
        .build();
    expander.add_row(&cpu_row);
    
    // Fan Settings
    let fan_info = if profile.fan_settings.control_enabled {
        format!("Fans: Manual ({} curves)", profile.fan_settings.curves.len())
    } else {
        "Fans: Auto".to_string()
    };
    let fan_row = adw::ActionRow::builder()
        .title("Fan Settings")
        .subtitle(&fan_info)
        .build();
    expander.add_row(&fan_row);
    
    // Keyboard Settings
    let kbd_info = if profile.keyboard_settings.control_enabled {
        "Keyboard: Manual"
    } else {
        "Keyboard: Auto"
    };
    let kbd_row = adw::ActionRow::builder()
        .title("Keyboard Backlight")
        .subtitle(kbd_info)
        .build();
    expander.add_row(&kbd_row);
    
    // Edit and Delete buttons
    let actions_box = Box::new(Orientation::Horizontal, 6);
    
    let edit_button = Button::with_label("Edit");
    let edit_profile = profile.clone();
    edit_button.connect_clicked(move |_| {
        // TODO: Switch to tuning tab with this profile
        println!("Edit profile: {}", edit_profile.name);
    });
    actions_box.append(&edit_button);
    
    // Only allow deletion for non-default profiles
    if !profile.is_default {
        let delete_button = Button::with_label("Delete");
        let del_profile_name = profile.name.clone();
        let del_config = config.clone();
        delete_button.connect_clicked(move |_| {
            delete_profile(&del_profile_name, del_config.clone());
        });
        actions_box.append(&delete_button);
    } else {
        // For default profile, add reset button
        let reset_button = Button::with_label("Reset to Stock");
        let reset_config = config.clone();
        reset_button.connect_clicked(move |_| {
            reset_default_profile(reset_config.clone());
        });
        actions_box.append(&reset_button);
    }
    
    expander.add_suffix(&actions_box);
    
    expander
}

fn create_new_profile(config: Rc<RefCell<Config>>, _dbus_client: Rc<RefCell<Option<DbusClient>>>) {
    // Create dialog to get profile name
    let dialog = adw::MessageDialog::builder()
        .heading("Create New Profile")
        .body("Enter a name for the new profile:")
        .build();
    
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("create", "Create");
    dialog.set_default_response(Some("create"));
    dialog.set_close_response("cancel");
    
    // TODO: Add text entry for profile name
    // For now, create with default name
    let config_clone = config.clone();
    dialog.connect_response(Some("create"), move |_, _| {
        let profile_count = config_clone.borrow().data.profiles.len();
        let new_name = format!("Profile {}", profile_count);
        
        // Create new profile based on default
        let default_profile = config_clone.borrow().data.profiles.iter()
            .find(|p| p.is_default)
            .cloned()
            .unwrap_or_default();
        
        let mut new_profile = default_profile;
        new_profile.name = new_name.clone();
        new_profile.is_default = false;
        
        // Add to config
        config_clone.borrow_mut().data.profiles.push(new_profile);
        let _ = config_clone.borrow().save();
        
        println!("Created new profile: {}", new_name);
        // TODO: Refresh profiles list
    });
    
    dialog.present();
}

fn delete_profile(profile_name: &str, config: Rc<RefCell<Config>>) {
    config.borrow_mut().data.profiles.retain(|p| p.name != profile_name);
    
    // If deleted profile was active, switch to default
    if config.borrow().data.current_profile == profile_name {
        config.borrow_mut().data.current_profile = "Default".to_string();
    }
    
    let _ = config.borrow().save();
    println!("Deleted profile: {}", profile_name);
    // TODO: Refresh profiles list
}

fn reset_default_profile(config: Rc<RefCell<Config>>) {
    // Reset default profile to stock settings
    if let Some(profile) = config.borrow_mut().data.profiles.iter_mut().find(|p| p.is_default) {
        *profile = Profile::default();
    }
    
    let _ = config.borrow().save();
    println!("Reset default profile to stock settings");
    // TODO: Refresh profiles list
}
