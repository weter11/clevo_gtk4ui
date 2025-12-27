use gtk::prelude::*;
use gtk::{Box, Orientation, Scale, ScrolledWindow};
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
    
    // Get current profile
    let current_profile_name = config.borrow().data.current_profile.clone();
    let current_profile = config.borrow().data.profiles.iter()
        .find(|p| p.name == current_profile_name)
        .cloned();
    
    if let Some(profile) = current_profile {
        // Add sections based on order
        for section_name in &config.borrow().data.tuning_section_order {
            match section_name.as_str() {
                "Keyboard" => {
                    let section = create_keyboard_section(&profile, config.clone(), dbus_client.clone());
                    main_box.append(&section);
                }
                "CPU" => {
                    let section = create_cpu_section(&profile, config.clone(), dbus_client.clone());
                    main_box.append(&section);
                }
                "GPU" => {
                    let section = create_gpu_section(&profile);
                    main_box.append(&section);
                }
                "Screen" => {
                    let section = create_screen_section(&profile);
                    main_box.append(&section);
                }
                "Fans" => {
                    let section = create_fans_section(&profile, config.clone());
                    main_box.append(&section);
                }
                _ => {}
            }
        }
        
        // Apply and Save button
        let button_box = Box::new(Orientation::Horizontal, 6);
        button_box.set_halign(gtk::Align::End);
        button_box.set_margin_top(12);
        
        let apply_button = gtk::Button::with_label("Apply & Save");
        apply_button.set_css_classes(&["suggested-action"]);
        
        let conf_clone = config.clone();
        let dbus_clone = dbus_client.clone();
        apply_button.connect_clicked(move |_| {
            // Save config
            let _ = conf_clone.borrow().save();
            
            // Apply current profile via DBus
            let profile_name = conf_clone.borrow().data.current_profile.clone();
            if let Some(profile) = conf_clone.borrow().data.profiles.iter()
                .find(|p| p.name == profile_name) {
                
                if let Some(client) = dbus_clone.borrow().as_ref() {
                    match client.apply_profile(&profile) {
                        Ok(_) => println!("Profile applied successfully"),
                        Err(e) => eprintln!("Failed to apply profile: {}", e),
                    }
                }
            }
        });
        
        button_box.append(&apply_button);
        main_box.append(&button_box);
    }
    
    scrolled.set_child(Some(&main_box));
    scrolled
}

fn create_keyboard_section(
    profile: &tuxedo_common::types::Profile,
    _config: Rc<RefCell<Config>>,
    _dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Keyboard Backlight")
        .build();
    
    let control_row = adw::SwitchRow::builder()
        .title("Manual control")
        .subtitle("When disabled, system controls the backlight")
        .build();
    control_row.set_active(profile.keyboard_settings.control_enabled);
    group.add(&control_row);
    
    // RGB Controls
    if let tuxedo_common::types::KeyboardMode::SingleColor { r, g, b, brightness } = profile.keyboard_settings.mode {
        // Red component
        let red_row = adw::ActionRow::builder()
            .title("Red")
            .subtitle(&format!("{}", r))
            .build();
        
        let red_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 255.0, 1.0);
        red_scale.set_value(r as f64);
        red_scale.set_hexpand(true);
        red_row.add_suffix(&red_scale);
        group.add(&red_row);
        
        // Green component
        let green_row = adw::ActionRow::builder()
            .title("Green")
            .subtitle(&format!("{}", g))
            .build();
        
        let green_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 255.0, 1.0);
        green_scale.set_value(g as f64);
        green_scale.set_hexpand(true);
        green_row.add_suffix(&green_scale);
        group.add(&green_row);
        
        // Blue component
        let blue_row = adw::ActionRow::builder()
            .title("Blue")
            .subtitle(&format!("{}", b))
            .build();
        
        let blue_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 255.0, 1.0);
        blue_scale.set_value(b as f64);
        blue_scale.set_hexpand(true);
        blue_row.add_suffix(&blue_scale);
        group.add(&blue_row);
        
        // Brightness
        let brightness_row = adw::ActionRow::builder()
            .title("Brightness")
            .subtitle(&format!("{}%", brightness))
            .build();
        
        let brightness_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        brightness_scale.set_value(brightness as f64);
        brightness_scale.set_hexpand(true);
        brightness_row.add_suffix(&brightness_scale);
        group.add(&brightness_row);
    }
    
    group
}

fn create_cpu_section(
    profile: &tuxedo_common::types::Profile,
    _config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("CPU Settings")
        .description("Profile-specific CPU configuration")
        .build();
    
    // Get current CPU info to determine available controls
    let available_controls = if let Some(client) = dbus_client.borrow().as_ref() {
        if let Ok(cpu_info) = client.get_cpu_info() {
            cpu_info.available_pstate_controls
        } else {
            vec![]
        }
    } else {
        vec![]
    };
    
    // Governor (always available)
    if available_controls.contains(&"scaling_governor".to_string()) {
        let cpu_info = if let Some(client) = dbus_client.borrow().as_ref() {
            client.get_cpu_info().ok()
        } else {
            None
        };
        
        if let Some(info) = cpu_info {
            let governor_row = adw::ComboRow::builder()
                .title("CPU Governor")
                .build();
            
            let governors: Vec<&str> = info.available_governors.iter().map(|s| s.as_str()).collect();
            let governor_model = gtk::StringList::new(&governors);
            governor_row.set_model(Some(&governor_model));
            
            if let Some(ref current_gov) = profile.cpu_settings.governor {
                if let Some(idx) = info.available_governors.iter().position(|g| g == current_gov) {
                    governor_row.set_selected(idx as u32);
                }
            }
            
            group.add(&governor_row);
        }
    }
    
    // Boost control
    if available_controls.contains(&"boost".to_string()) {
        let boost_row = adw::SwitchRow::builder()
            .title("CPU Boost")
            .subtitle("Turbo / Precision Boost")
            .build();
        
        if let Some(boost) = profile.cpu_settings.boost {
            boost_row.set_active(boost);
        }
        
        group.add(&boost_row);
    }
    
    // SMT control
    if available_controls.contains(&"smt".to_string()) {
        let smt_row = adw::SwitchRow::builder()
            .title("SMT / Hyperthreading")
            .subtitle("Simultaneous Multithreading")
            .build();
        
        if let Some(smt) = profile.cpu_settings.smt {
            smt_row.set_active(smt);
        }
        
        group.add(&smt_row);
    }
    
    // Frequency limits (if available)
    if available_controls.contains(&"cpuinfo_min_freq".to_string()) 
        && available_controls.contains(&"cpuinfo_max_freq".to_string()) {
        
        if let Some(client) = dbus_client.borrow().as_ref() {
            if let Ok(cpu_info) = client.get_cpu_info() {
                // Min frequency
                let min_freq_row = adw::ActionRow::builder()
                    .title("Minimum Frequency")
                    .subtitle(&format!("{} MHz", 
                        profile.cpu_settings.min_frequency.unwrap_or(cpu_info.hw_min_freq) / 1000))
                    .build();
                
                let min_scale = Scale::with_range(
                    gtk::Orientation::Horizontal,
                    cpu_info.hw_min_freq as f64,
                    cpu_info.hw_max_freq as f64,
                    100000.0
                );
                min_scale.set_value(profile.cpu_settings.min_frequency.unwrap_or(cpu_info.hw_min_freq) as f64);
                min_scale.set_hexpand(true);
                min_freq_row.add_suffix(&min_scale);
                group.add(&min_freq_row);
                
                // Max frequency
                let max_freq_row = adw::ActionRow::builder()
                    .title("Maximum Frequency")
                    .subtitle(&format!("{} MHz", 
                        profile.cpu_settings.max_frequency.unwrap_or(cpu_info.hw_max_freq) / 1000))
                    .build();
                
                let max_scale = Scale::with_range(
                    gtk::Orientation::Horizontal,
                    cpu_info.hw_min_freq as f64,
                    cpu_info.hw_max_freq as f64,
                    100000.0
                );
                max_scale.set_value(profile.cpu_settings.max_frequency.unwrap_or(cpu_info.hw_max_freq) as f64);
                max_scale.set_hexpand(true);
                max_freq_row.add_suffix(&max_scale);
                group.add(&max_freq_row);
            }
        }
    }
    
    // Energy Performance Preference (EPP) - for active/passive modes
    if available_controls.contains(&"energy_performance_preference".to_string()) {
        if let Some(client) = dbus_client.borrow().as_ref() {
            if let Ok(cpu_info) = client.get_cpu_info() {
                if !cpu_info.available_epp_preferences.is_empty() {
                    let epp_row = adw::ComboRow::builder()
                        .title("Energy Performance Preference")
                        .subtitle("Balance between performance and power saving")
                        .build();
                    
                    let epp_prefs: Vec<&str> = cpu_info.available_epp_preferences.iter()
                        .map(|s| s.as_str()).collect();
                    let epp_model = gtk::StringList::new(&epp_prefs);
                    epp_row.set_model(Some(&epp_model));
                    
                    if let Some(ref current_epp) = profile.cpu_settings.energy_performance_preference {
                        if let Some(idx) = cpu_info.available_epp_preferences.iter()
                            .position(|e| e == current_epp) {
                            epp_row.set_selected(idx as u32);
                        }
                    }
                    
                    group.add(&epp_row);
                }
            }
        }
    }
    
    // AMD pstate mode selector (if available)
    if let Some(client) = dbus_client.borrow().as_ref() {
        if let Ok(cpu_info) = client.get_cpu_info() {
            if cpu_info.amd_pstate_status.is_some() {
                let pstate_row = adw::ComboRow::builder()
                    .title("AMD Pstate Mode")
                    .subtitle("Changes available CPU controls")
                    .build();
                
                let pstate_modes = gtk::StringList::new(&["passive", "active", "guided"]);
                pstate_row.set_model(Some(&pstate_modes));
                
                if let Some(ref status) = cpu_info.amd_pstate_status {
                    let idx = match status.as_str() {
                        "passive" => 0,
                        "active" => 1,
                        "guided" => 2,
                        _ => 0,
                    };
                    pstate_row.set_selected(idx);
                }
                
                group.add(&pstate_row);
            }
        }
    }
    
    // Show info about available controls
    let info_label = gtk::Label::new(Some(&format!(
        "Available controls for current pstate mode: {}",
        available_controls.join(", ")
    )));
    info_label.set_wrap(true);
    info_label.set_css_classes(&["dim-label", "caption"]);
    info_label.set_halign(gtk::Align::Start);
    
    let info_box = Box::new(Orientation::Vertical, 0);
    info_box.append(&info_label);
    
    let info_row = adw::ActionRow::builder()
        .title("Available Controls")
        .build();
    info_row.set_child(Some(&info_box));
    group.add(&info_row);
    
    group
}

fn create_gpu_section(profile: &tuxedo_common::types::Profile) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("GPU Settings")
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
        .title("Screen Brightness")
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
    
    let brightness_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    brightness_scale.set_value(profile.screen_settings.brightness as f64);
    brightness_scale.set_hexpand(true);
    brightness_scale.set_sensitive(!profile.screen_settings.system_control);
    brightness_row.add_suffix(&brightness_scale);
    group.add(&brightness_row);
    
    group
}

fn create_fans_section(
    profile: &tuxedo_common::types::Profile,
    _config: Rc<RefCell<Config>>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Fan Control")
        .build();
    
    let control_row = adw::SwitchRow::builder()
        .title("Manual fan control")
        .subtitle("Use custom fan curves (disable for auto mode)")
        .build();
    
    control_row.set_active(profile.fan_settings.control_enabled);
    group.add(&control_row);
    
    if profile.fan_settings.control_enabled {
        // Show configured curves
        for curve in &profile.fan_settings.curves {
            let curve_expander = adw::ExpanderRow::builder()
                .title(&format!("Fan {} Curve", curve.fan_id))
                .subtitle(&format!("{} points configured", curve.points.len()))
                .build();
            
            // Display curve points
            for (idx, (temp, speed)) in curve.points.iter().enumerate() {
                let point_row = adw::ActionRow::builder()
                    .title(&format!("Point {}", idx + 1))
                    .subtitle(&format!("{}°C → {}% speed", temp, speed))
                    .build();
                curve_expander.add_row(&point_row);
            }
            
            group.add(&curve_expander);
        }
        
        // Add new curve button
        let add_curve_button = gtk::Button::with_label("➕ Add Fan Curve");
        let add_box = Box::new(Orientation::Horizontal, 0);
        add_box.set_halign(gtk::Align::Start);
        add_box.append(&add_curve_button);
        
        let button_row = adw::ActionRow::new();
        button_row.set_child(Some(&add_box));
        group.add(&button_row);
    }
    
    group
}
