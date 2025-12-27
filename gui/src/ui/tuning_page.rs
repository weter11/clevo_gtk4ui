use gtk::prelude::*;
use gtk::{Box, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::dbus_client::DbusClient;

pub fn create_page(
    config: Rc<RefCell<Config>>,
    _dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .build();
    
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);
    
    // Get current profile
    let current_profile_name = config.borrow().data.current_profile.clone();
    let current_profile = config.borrow().data.profiles.iter()
        .find(|p| p.name == current_profile_name)
        .cloned();
    
    if let Some(profile) = current_profile {
        // Add sections based on order
        for section_name in &config.borrow().data.tuning_section_order {
            match section_name.as_str() {
                "Keyboard" => main_box.append(&create_keyboard_section(&profile)),
                "CPU" => main_box.append(&create_cpu_section(&profile)),
                "GPU" => main_box.append(&create_gpu_section(&profile)),
                "Screen" => main_box.append(&create_screen_section(&profile)),
                "Fans" => main_box.append(&create_fans_section(&profile)),
                _ => {}
            }
        }
    }
    
    scrolled.set_child(Some(&main_box));
    scrolled
}

fn create_keyboard_section(profile: &tuxedo_common::types::Profile) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Keyboard")
        .build();
    
    let control_row = adw::SwitchRow::builder()
        .title("Control keyboard backlight")
        .subtitle("When disabled, system controls the backlight")
        .build();
    
    control_row.set_active(profile.keyboard_settings.control_enabled);
    group.add(&control_row);
    
    // Color controls (only if control enabled)
    if let tuxedo_common::types::KeyboardMode::SingleColor { r, g, b, brightness } = profile.keyboard_settings.mode {
        let color_row = adw::ActionRow::builder()
            .title("Backlight Color")
            .subtitle(&format!("RGB({}, {}, {})", r, g, b))
            .build();
        group.add(&color_row);
        
        let brightness_row = adw::ActionRow::builder()
            .title("Brightness")
            .subtitle(&format!("{}%", brightness))
            .build();
        
        let brightness_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        brightness_scale.set_value(brightness as f64);
        brightness_scale.set_hexpand(true);
        brightness_row.add_suffix(&brightness_scale);
        group.add(&brightness_row);
    }
    
    group
}

fn create_cpu_section(profile: &tuxedo_common::types::Profile) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("CPU")
        .description("Profile-specific CPU settings")
        .build();
    
    if let Some(ref tdp) = profile.cpu_settings.tdp {
        let tdp_row = adw::ActionRow::builder()
            .title("TDP Limit")
            .subtitle(&format!("{} W", tdp))
            .build();
        group.add(&tdp_row);
    }
    
    group
}

fn create_gpu_section(profile: &tuxedo_common::types::Profile) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("GPU")
        .build();
    
    if let Some(ref tdp) = profile.gpu_settings.dgpu_tdp {
        let tdp_row = adw::ActionRow::builder()
            .title("dGPU TDP Limit")
            .subtitle(&format!("{} W", tdp))
            .build();
        group.add(&tdp_row);
    } else {
        let na_row = adw::ActionRow::builder()
            .title("dGPU TDP")
            .subtitle("Not supported on this hardware")
            .build();
        group.add(&na_row);
    }
    
    group
}

fn create_screen_section(profile: &tuxedo_common::types::Profile) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Screen")
        .build();
    
    let system_control_row = adw::SwitchRow::builder()
        .title("System control")
        .subtitle("Let the system manage brightness")
        .build();
    
    system_control_row.set_active(profile.screen_settings.system_control);
    group.add(&system_control_row);
    
    let brightness_row = adw::ActionRow::builder()
        .title("Brightness")
        .subtitle(&format!("{}%", profile.screen_settings.brightness))
        .build();
    
    let brightness_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    brightness_scale.set_value(profile.screen_settings.brightness as f64);
    brightness_scale.set_hexpand(true);
    brightness_scale.set_sensitive(!profile.screen_settings.system_control);
    brightness_row.add_suffix(&brightness_scale);
    group.add(&brightness_row);
    
    group
}

fn create_fans_section(profile: &tuxedo_common::types::Profile) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Fans")
        .build();
    
    let control_row = adw::SwitchRow::builder()
        .title("Control fans")
        .subtitle("Use custom fan curves (disable for auto mode)")
        .build();
    
    control_row.set_active(profile.fan_settings.control_enabled);
    group.add(&control_row);
    
    if profile.fan_settings.control_enabled {
        for curve in &profile.fan_settings.curves {
            let curve_row = adw::ActionRow::builder()
                .title(&format!("Fan {} Curve", curve.fan_id))
                .subtitle(&format!("{} points configured", curve.points.len()))
                .build();
            group.add(&curve_row);
        }
    }
    
    group
}
