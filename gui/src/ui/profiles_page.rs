use gtk::prelude::*;
use gtk::{Box, Button, CheckButton, Label, Orientation, ScrolledWindow, Entry};
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
    window: gtk::Window,
) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .build();
    
    let content = build_profiles_content(config.clone(), dbus_client.clone(), window.clone());
    scrolled.set_child(Some(&content));
    
    let scrolled_weak = scrolled.downgrade();
    let config_clone = config.clone();
    let dbus_clone = dbus_client.clone();
    let window_clone = window.clone();
    
    gtk::glib::timeout_add_seconds_local(1, move || {
        if let Some(scrolled) = scrolled_weak.upgrade() {
            let new_content = build_profiles_content(config_clone.clone(), dbus_clone.clone(), window_clone.clone());
            scrolled.set_child(Some(&new_content));
            gtk::glib::ControlFlow::Continue
        } else {
            gtk::glib::ControlFlow::Break
        }
    });
    
    scrolled
}

fn build_profiles_content(
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
    window: gtk::Window,
) -> Box {
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);
    
    let current_profile = &config.borrow().data.current_profile;
    let indicator = Label::new(Some(&format!("Current Profile: {}", current_profile)));
    indicator.set_css_classes(&["title-2"]);
    main_box.append(&indicator);
    
    let mut radio_group: Option<CheckButton> = None;
    
    let default_group = adw::PreferencesGroup::builder()
        .title("Default Profile")
        .description("System default profile (can be edited, cannot be deleted)")
        .build();
    
    let default_profile = config.borrow().data.profiles.iter()
        .find(|p| p.is_default)
        .cloned();
    
    if let Some(profile) = default_profile {
        let (profile_row, radio) = create_profile_row(
            &profile,
            false,
            config.clone(),
            dbus_client.clone(),
            radio_group.clone(),
            window.clone(),
        );
        radio_group = Some(radio);
        default_group.add(&profile_row);
    }
    
    main_box.append(&default_group);
    
    let custom_group = adw::PreferencesGroup::builder()
        .title("Custom Profiles")
        .build();
    
    let custom_profiles: Vec<_> = config.borrow().data.profiles.iter()
        .filter(|p| !p.is_default)
        .cloned()
        .collect();
    
    for profile in custom_profiles {
        let (profile_row, radio) = create_profile_row(
            &profile,
            true,
            config.clone(),
            dbus_client.clone(),
            radio_group.clone(),
            window.clone(),
        );
        radio_group = Some(radio);
        custom_group.add(&profile_row);
    }
    
    main_box.append(&custom_group);
    
    let button_box = Box::new(Orientation::Horizontal, 6);
    button_box.set_halign(gtk::Align::Start);
    button_box.set_margin_top(12);
    
    let new_button = Button::with_label("➕ New Profile");
    new_button.add_css_class("suggested-action");
    
    let config_clone = config.clone();
    let window_clone = window.clone();
    new_button.connect_clicked(move |_| {
        show_new_profile_dialog(config_clone.clone(), window_clone.clone());
    });
    
    button_box.append(&new_button);
    main_box.append(&button_box);
    
    main_box
}

fn create_profile_row(
    profile: &Profile,
    deletable: bool,
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
    radio_group: Option<CheckButton>,
    window: gtk::Window,
) -> (adw::ExpanderRow, CheckButton) {
    let expander = adw::ExpanderRow::builder()
        .title(&profile.name)
        .build();
    
    let radio = if let Some(group) = radio_group {
        CheckButton::builder()
            .group(&group)
            .build()
    } else {
        CheckButton::new()
    };
    
    radio.set_active(&config.borrow().data.current_profile == &profile.name);
    expander.add_prefix(&radio);
    
    let profile_name = profile.name.clone();
    let config_clone = config.clone();
    let dbus_clone = dbus_client.clone();
    let profile_clone = profile.clone();
    radio.connect_toggled(move |rb| {
        if rb.is_active() {
            config_clone.borrow_mut().data.current_profile = profile_name.clone();
            let _ = config_clone.borrow().save();
            
            if let Some(client) = dbus_clone.borrow().as_ref() {
                let _ = client.apply_profile(&profile_clone);
            }
        }
    });
    
    let cpu_info = if let Some(ref gov) = profile.cpu_settings.governor {
        format!("Governor: {}", gov)
    } else {
        "Governor: Auto".to_string()
    };
    
    let cpu_extra = vec![
        profile.cpu_settings.boost.map(|b| format!("Boost: {}", if b { "On" } else { "Off" })),
        profile.cpu_settings.smt.map(|s| format!("SMT: {}", if s { "On" } else { "Off" })),
        profile.cpu_settings.min_frequency.map(|f| format!("Min: {} MHz", f / 1000)),
        profile.cpu_settings.max_frequency.map(|f| format!("Max: {} MHz", f / 1000)),
    ].into_iter().flatten().collect::<Vec<_>>().join(", ");
    
    let cpu_subtitle = if !cpu_extra.is_empty() {
        format!("{}, {}", cpu_info, cpu_extra)
    } else {
        cpu_info
    };
    
    let cpu_row = adw::ActionRow::builder()
        .title("CPU Settings")
        .subtitle(&cpu_subtitle)
        .build();
    expander.add_row(&cpu_row);
    
    let keyboard_info = if profile.keyboard_settings.control_enabled {
        "Keyboard: Manual control"
    } else {
        "Keyboard: Auto (system control)"
    };
    let keyboard_row = adw::ActionRow::builder()
        .title("Keyboard Settings")
        .subtitle(keyboard_info)
        .build();
    expander.add_row(&keyboard_row);
    
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
    
    let screen_info = if profile.screen_settings.system_control {
        "Screen: System control"
    } else {
        &format!("Screen: {}% brightness", profile.screen_settings.brightness)
    };
    let screen_row = adw::ActionRow::builder()
        .title("Screen Settings")
        .subtitle(screen_info)
        .build();
    expander.add_row(&screen_row);
    
    let edit_button = Button::with_label("✏️ Edit");
    edit_button.set_valign(gtk::Align::Center);
    
    let config_clone = config.clone();
    let profile_name = profile.name.clone();
    let window_clone = window.clone();
    edit_button.connect_clicked(move |_| {
        config_clone.borrow_mut().data.current_profile = profile_name.clone();
        let _ = config_clone.borrow().save();
        
        if let Some(app_window) = window_clone.downcast_ref::<adw::ApplicationWindow>() {
            if let Some(content) = app_window.content() {
                if let Some(vbox) = content.downcast_ref::<Box>() {
                    let mut child = vbox.first_child();
                    child = child.and_then(|c| c.next_sibling());
                    
                    if let Some(view_stack) = child.and_then(|c| c.downcast::<adw::ViewStack>().ok()) {
                        view_stack.set_visible_child_name("tuning");
                    }
                }
            }
        }
    });
    
    expander.add_suffix(&edit_button);
    
    if deletable {
        let delete_button = Button::with_label("🗑️ Delete");
        delete_button.set_valign(gtk::Align::Center);
        delete_button.add_css_class("destructive-action");
        
        let config_clone = config.clone();
        let profile_name = profile.name.clone();
        let window_clone = window.clone();
        delete_button.connect_clicked(move |_| {
            show_delete_confirmation(&profile_name, config_clone.clone(), window_clone.clone());
        });
        
        expander.add_suffix(&delete_button);
    } else if profile.is_default {
        let reset_button = Button::with_label("↺ Reset to Stock");
        reset_button.set_valign(gtk::Align::Center);
        
        let config_clone = config.clone();
        let dbus_clone = dbus_client.clone();
        reset_button.connect_clicked(move |_| {
            let mut cfg = config_clone.borrow_mut();
            if let Some(default_prof) = cfg.data.profiles.iter_mut().find(|p| p.is_default) {
                let reset_profile = default_prof.clone();  // Clone before modifying
               *default_prof = Profile::default();
                default_prof.is_default = true;
        
                drop(cfg);  // Explicitly drop mutable borrow
        
                let _ = config_clone.borrow().save();  // Now safe immutable borrow
        
                if let Some(client) = dbus_clone.borrow().as_ref() {
                   let _ = client.apply_profile(&reset_profile);
                }
            }
        });
        
        expander.add_suffix(&reset_button);
    }
    
    (expander, radio)
}

fn show_new_profile_dialog(config: Rc<RefCell<Config>>, window: gtk::Window) {
    let dialog = adw::MessageDialog::builder()
        .heading("Create New Profile")
        .body("Enter a name for the new profile")
        .transient_for(&window)
        .build();
    
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("create", "Create");
    dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("create"));
    dialog.set_close_response("cancel");
    
    let entry = Entry::builder()
        .placeholder_text("Profile name")
        .build();
    
    let content_box = Box::new(Orientation::Vertical, 12);
    content_box.set_margin_top(12);
    content_box.set_margin_bottom(12);
    content_box.set_margin_start(12);
    content_box.set_margin_end(12);
    content_box.append(&entry);
    
    dialog.set_extra_child(Some(&content_box));
    
    let config_clone = config.clone();
    let entry_clone = entry.clone();
    let window_clone = window.clone();
    dialog.connect_response(None, move |dialog, response| {
        if response == "create" {
            let name = entry_clone.text().to_string();
            if !name.is_empty() {
                let mut cfg = config_clone.borrow_mut();
                
                if cfg.data.profiles.iter().any(|p| p.name == name) {
                    drop(cfg);
                    show_error_dialog("Profile Already Exists", &format!("A profile named '{}' already exists.", name), &window_clone);
                    return;
                }
                
                let default_settings = cfg.data.profiles.iter()
                    .find(|p| p.is_default)
                    .cloned()
                    .unwrap_or_default();
                
                let mut new_profile = default_settings;
                new_profile.name = name.clone();
                new_profile.is_default = false;
                
                cfg.data.profiles.push(new_profile);
                let _ = cfg.save();
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

fn show_delete_confirmation(profile_name: &str, config: Rc<RefCell<Config>>, window: gtk::Window) {
    let dialog = adw::MessageDialog::builder()
        .heading("Delete Profile?")
        .body(&format!("Are you sure you want to delete the profile '{}'?", profile_name))
        .transient_for(&window)
        .build();
    
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");
    
    let config_clone = config.clone();
    let profile_name = profile_name.to_string();
    dialog.connect_response(None, move |dialog, response| {
        if response == "delete" {
            let mut cfg = config_clone.borrow_mut();
            cfg.data.profiles.retain(|p| p.name != profile_name);
            let _ = cfg.save();
        }
        dialog.close();
    });
    
    dialog.present();
}

fn show_error_dialog(heading: &str, body: &str, window: &gtk::Window) {
    let dialog = adw::MessageDialog::builder()
        .heading(heading)
        .body(body)
        .transient_for(window)
        .build();
    
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.set_close_response("ok");
    
    dialog.present();
}
