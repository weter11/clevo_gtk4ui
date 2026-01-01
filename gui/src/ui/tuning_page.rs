use gtk::prelude::*;
use gtk::{Box, Orientation, ScrolledWindow, Scale, Button};
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
        .vexpand(true)
        .hexpand(true)
        .build();
    
    let content = build_tuning_content(config.clone(), dbus_client.clone());
    scrolled.set_child(Some(&content));
    
    let scrolled_weak = scrolled.downgrade();
    let config_clone = config.clone();
    let dbus_clone = dbus_client.clone();
    let mut last_profile = config.borrow().data.current_profile.clone();
    
    gtk::glib::timeout_add_seconds_local(1, move || {
        let current_profile = config_clone.borrow().data.current_profile.clone();
        
        if current_profile != last_profile {
            last_profile = current_profile;
            
            if let Some(scrolled) = scrolled_weak.upgrade() {
                let new_content = build_tuning_content(config_clone.clone(), dbus_clone.clone());
                scrolled.set_child(Some(&new_content));
            }
        }
        
        gtk::glib::ControlFlow::Continue
    });
    
    scrolled
}

fn build_tuning_content(
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> Box {
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);
    
    let current_profile = {
        let cfg = config.borrow();
        let profile_name = cfg.data.current_profile.clone();
        cfg.data.profiles.iter()
            .find(|p| p.name == profile_name)
            .cloned()
    };
    
    if let Some(profile) = current_profile {
        let header = gtk::Label::new(Some(&format!("Editing Profile: {}", profile.name)));
        header.set_css_classes(&["title-2"]);
        main_box.append(&header);
        
        let cpu_info = dbus_client.borrow().as_ref()
            .and_then(|client| client.get_cpu_info().ok());
        
        if dbus_client.borrow().is_some() {
            if let Ok(tdp_profiles) = dbus_client.borrow().as_ref().unwrap().get_tdp_profiles() {
                if !tdp_profiles.is_empty() {
                    let tdp_section = create_tdp_section(&profile, &tdp_profiles, config.clone());
                    main_box.append(&tdp_section);
                }
            }
        }
        
        let cpu_section = create_cpu_tuning_section(&profile, config.clone(), dbus_client.clone(), cpu_info);
        main_box.append(&cpu_section);
        
        let keyboard_section = create_keyboard_tuning_section(&profile, config.clone(), dbus_client.clone());
        main_box.append(&keyboard_section);
        
        let screen_section = create_screen_tuning_section(&profile, config.clone());
        main_box.append(&screen_section);
        
        let fans_section = create_fans_tuning_section(&profile, config.clone());
        main_box.append(&fans_section);
        
        let button_box = Box::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk::Align::Center);
        button_box.set_margin_top(24);
        
        let apply_button = Button::with_label("💾 Apply & Save Profile");
        apply_button.add_css_class("suggested-action");
        
        let config_clone = config.clone();
        let dbus_clone = dbus_client.clone();
        let profile_name = profile.name.clone();
        apply_button.connect_clicked(move |_| {
            let prof = {
                let cfg = config_clone.borrow();
                cfg.data.profiles.iter()
                    .find(|p| p.name == profile_name)
                    .cloned()
            };
            
            if let Some(prof) = prof {
                let _ = config_clone.borrow().save();
                
                if let Some(client) = dbus_clone.borrow().as_ref() {
                    let _ = client.apply_profile(&prof);
                }
            }
        });
        
        let reset_button = Button::with_label("↺ Reset to Saved");
        
        button_box.append(&apply_button);
        button_box.append(&reset_button);
        main_box.append(&button_box);
    } else {
        let error_label = gtk::Label::new(Some("No profile selected"));
        main_box.append(&error_label);
    }
    
    main_box
}

fn create_tdp_section(
    profile: &Profile,
    tdp_profiles: &[String],
    config: Rc<RefCell<Config>>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("TDP Performance Profile")
        .build();
    
    let tdp_row = adw::ComboRow::builder()
        .title("TDP Profile")
        .build();
    
    let tdp_strs: Vec<&str> = tdp_profiles.iter().map(|s| s.as_str()).collect();
    let model = gtk::StringList::new(&tdp_strs);
    tdp_row.set_model(Some(&model));
    
    if let Some(ref current) = profile.cpu_settings.tdp_profile {
        if let Some(idx) = tdp_profiles.iter().position(|p| p == current) {
            tdp_row.set_selected(idx as u32);
        }
    }
    
    let config_clone = config.clone();
    let profile_name = profile.name.clone();
    let profiles_clone = tdp_profiles.to_vec();
    tdp_row.connect_selected_notify(move |row| {
        let idx = row.selected() as usize;
        if idx < profiles_clone.len() {
            let tdp_profile = &profiles_clone[idx];
            let mut cfg = config_clone.borrow_mut();
            if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                prof.cpu_settings.tdp_profile = Some(tdp_profile.clone());
            }
        }
    });
    
    group.add(&tdp_row);
    group
}

fn create_cpu_tuning_section(
    profile: &Profile,
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
    cpu_info: Option<tuxedo_common::types::CpuInfo>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("CPU Tuning")
        .description("Available CPU controls based on current pstate mode")
        .build();
    
    let Some(info) = cpu_info else {
        let error_row = adw::ActionRow::builder()
            .title("Error")
            .subtitle("CPU information not available")
            .build();
        group.add(&error_row);
        return group;
    };
    
    let caps = &info.capabilities;
    
    if caps.has_amd_pstate {
        if let Some(ref pstate_mode) = info.amd_pstate_status {
            let mode_row = adw::ActionRow::builder()
                .title("AMD Pstate Mode")
                .subtitle(&format!("Current mode: {}", pstate_mode))
                .build();
            group.add(&mode_row);
            
            let pstate_selector = adw::ComboRow::builder()
                .title("Change Pstate Mode")
                .subtitle("Restart may be required")
                .build();
            
            let modes = vec!["passive", "active", "guided"];
            let model = gtk::StringList::new(&modes);
            pstate_selector.set_model(Some(&model));
            
            if let Some(idx) = modes.iter().position(|m| *m == pstate_mode.as_str()) {
                pstate_selector.set_selected(idx as u32);
            }
            
            let config_clone = config.clone();
            let profile_name = profile.name.clone();
            pstate_selector.connect_selected_notify(move |row| {
                let idx = row.selected() as usize;
                let modes = vec!["passive", "active", "guided"];
                if idx < modes.len() {
                    let mode = modes[idx];
                    let mut cfg = config_clone.borrow_mut();
                    if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                        prof.cpu_settings.amd_pstate_status = Some(mode.to_string());
                    }
                }
            });
            
            group.add(&pstate_selector);
        }
    }
    
    if caps.has_energy_performance_preference && !info.available_epp_options.is_empty() {
        let epp_row = adw::ComboRow::builder()
            .title("Energy Performance Preference")
            .subtitle("Balance between performance and power")
            .build();
        
        let epp_strs: Vec<&str> = info.available_epp_options.iter().map(|s| s.as_str()).collect();
        let model = gtk::StringList::new(&epp_strs);
        epp_row.set_model(Some(&model));
        
        if let Some(ref current_epp) = info.energy_performance_preference {
            if let Some(idx) = info.available_epp_options.iter().position(|e| e == current_epp) {
                epp_row.set_selected(idx as u32);
            }
        }
        
        let config_clone = config.clone();
        let profile_name = profile.name.clone();
        let epp_options = info.available_epp_options.clone();
        epp_row.connect_selected_notify(move |row| {
            let idx = row.selected() as usize;
            if idx < epp_options.len() {
                let epp = &epp_options[idx];
                let mut cfg = config_clone.borrow_mut();
                if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                    prof.cpu_settings.energy_performance_preference = Some(epp.clone());
                }
            }
        });
        
        group.add(&epp_row);
    }
    
    if caps.has_scaling_governor && !info.available_governors.is_empty() {
        let governor_row = adw::ComboRow::builder()
            .title("CPU Governor")
            .subtitle("Performance profile for the CPU")
            .build();
        
        let gov_strs: Vec<&str> = info.available_governors.iter().map(|s| s.as_str()).collect();
        let model = gtk::StringList::new(&gov_strs);
        governor_row.set_model(Some(&model));
        
        if let Some(ref gov) = profile.cpu_settings.governor {
            if let Some(idx) = info.available_governors.iter().position(|g| g == gov) {
                governor_row.set_selected(idx as u32);
            }
        }
        
        let config_clone = config.clone();
        let profile_name = profile.name.clone();
        let governors = info.available_governors.clone();
        governor_row.connect_selected_notify(move |row| {
            let idx = row.selected() as usize;
            if idx < governors.len() {
                let governor = &governors[idx];
                let mut cfg = config_clone.borrow_mut();
                if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                    prof.cpu_settings.governor = Some(governor.clone());
                }
            }
        });
        
        group.add(&governor_row);
    } else if !caps.has_scaling_governor {
        let info_row = adw::ActionRow::builder()
            .title("Governor Control")
            .subtitle("Not available in current pstate mode")
            .build();
        group.add(&info_row);
    }
    
    if caps.has_scaling_min_freq && caps.has_scaling_max_freq {
        let min_freq_row = adw::ActionRow::builder()
            .title("Minimum Frequency")
            .build();
        
        let min_scale = Scale::with_range(
            gtk::Orientation::Horizontal,
            info.hw_min_freq as f64,
            info.hw_max_freq as f64,
            100000.0,
        );
        
        if let Some(min) = profile.cpu_settings.min_frequency {
            min_scale.set_value(min as f64);
        } else {
            min_scale.set_value(info.hw_min_freq as f64);
        }
        
        min_scale.set_hexpand(true);
        min_scale.set_draw_value(true);
        min_scale.set_value_pos(gtk::PositionType::Right);
        min_scale.set_format_value_func(|_, val| format!("{} MHz", (val as u64) / 1000));
        
        let config_clone = config.clone();
        let profile_name = profile.name.clone();
        min_scale.connect_value_changed(move |scale| {
            let value = scale.value() as u64;
            let mut cfg = config_clone.borrow_mut();
            if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                prof.cpu_settings.min_frequency = Some(value);
            }
        });
        
        min_freq_row.add_suffix(&min_scale);
        group.add(&min_freq_row);
        
        let max_freq_row = adw::ActionRow::builder()
            .title("Maximum Frequency")
            .build();
        
        let max_scale = Scale::with_range(
            gtk::Orientation::Horizontal,
            info.hw_min_freq as f64,
            info.hw_max_freq as f64,
            100000.0,
        );
        
        if let Some(max) = profile.cpu_settings.max_frequency {
            max_scale.set_value(max as f64);
        } else {
            max_scale.set_value(info.hw_max_freq as f64);
        }
        
        max_scale.set_hexpand(true);
        max_scale.set_draw_value(true);
        max_scale.set_value_pos(gtk::PositionType::Right);
        max_scale.set_format_value_func(|_, val| format!("{} MHz", (val as u64) / 1000));
        
        let config_clone = config.clone();
        let profile_name = profile.name.clone();
        max_scale.connect_value_changed(move |scale| {
            let value = scale.value() as u64;
            let mut cfg = config_clone.borrow_mut();
            if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                prof.cpu_settings.max_frequency = Some(value);
            }
        });
        
        max_freq_row.add_suffix(&max_scale);
        group.add(&max_freq_row);
    } else {
        let info_row = adw::ActionRow::builder()
            .title("Frequency Control")
            .subtitle("Not available in current pstate mode")
            .build();
        group.add(&info_row);
    }
    
    if caps.has_boost {
        let boost_row = adw::SwitchRow::builder()
            .title("CPU Boost / Turbo")
            .subtitle("Enable CPU turbo frequencies")
            .build();
        
        boost_row.set_active(profile.cpu_settings.boost.unwrap_or(info.boost_enabled));
        
        let config_clone = config.clone();
        let profile_name = profile.name.clone();
        boost_row.connect_active_notify(move |row| {
            let mut cfg = config_clone.borrow_mut();
            if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                prof.cpu_settings.boost = Some(row.is_active());
            }
        });
        
        group.add(&boost_row);
    }
    
    if caps.has_smt {
        let smt_row = adw::SwitchRow::builder()
            .title("SMT / Hyperthreading")
            .subtitle("Enable simultaneous multithreading")
            .build();
        
        smt_row.set_active(profile.cpu_settings.smt.unwrap_or(info.smt_enabled));
        
        let config_clone = config.clone();
        let profile_name = profile.name.clone();
        smt_row.connect_active_notify(move |row| {
            let mut cfg = config_clone.borrow_mut();
            if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                prof.cpu_settings.smt = Some(row.is_active());
            }
        });
        
        group.add(&smt_row);
    }
    
    group
}

fn create_keyboard_tuning_section(
    profile: &Profile,
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> adw::PreferencesGroup {
    use tuxedo_common::types::KeyboardMode;
    
    let group = adw::PreferencesGroup::builder()
        .title("Keyboard Backlight")
        .build();
    
    let control_row = adw::SwitchRow::builder()
        .title("Control keyboard backlight")
        .build();
    
    control_row.set_active(profile.keyboard_settings.control_enabled);
    
    let config_clone = config.clone();
    let profile_name = profile.name.clone();
    control_row.connect_active_notify(move |row| {
        let mut cfg = config_clone.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
            prof.keyboard_settings.control_enabled = row.is_active();
        }
    });
    
    group.add(&control_row);
    
    // Mode selector
    let mode_row = adw::ComboRow::builder()
        .title("Backlight Mode")
        .build();
    
    let modes = vec![
        "Single Color",
        "Breathe",
        "Cycle", 
        "Dance",
        "Flash",
        "Random Color",
        "Tempo",
        "Wave",
    ];
    let model = gtk::StringList::new(&modes.iter().map(|s| *s).collect::<Vec<_>>());
    mode_row.set_model(Some(&model));
    
    // Set current mode
    let current_mode_idx = match &profile.keyboard_settings.mode {
        KeyboardMode::SingleColor { .. } => 0,
        KeyboardMode::Breathe { .. } => 1,
        KeyboardMode::Cycle { .. } => 2,
        KeyboardMode::Dance { .. } => 3,
        KeyboardMode::Flash { .. } => 4,
        KeyboardMode::RandomColor { .. } => 5,
        KeyboardMode::Tempo { .. } => 6,
        KeyboardMode::Wave { .. } => 7,
    };
    mode_row.set_selected(current_mode_idx);
    
    let config_clone = config.clone();
    let profile_name = profile.name.clone();
    let dbus_clone = dbus_client.clone();
    mode_row.connect_selected_notify(move |row| {
        let mode_idx = row.selected();
        let mut cfg = config_clone.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
            // Update mode - keep existing values where possible
            prof.keyboard_settings.mode = match mode_idx {
                0 => KeyboardMode::SingleColor { r: 255, g: 255, b: 255, brightness: 50 },
                1 => KeyboardMode::Breathe { r: 255, g: 255, b: 255, brightness: 50, speed: 50 },
                2 => KeyboardMode::Cycle { brightness: 50, speed: 50 },
                3 => KeyboardMode::Dance { brightness: 50, speed: 50 },
                4 => KeyboardMode::Flash { r: 255, g: 255, b: 255, brightness: 50, speed: 50 },
                5 => KeyboardMode::RandomColor { brightness: 50, speed: 50 },
                6 => KeyboardMode::Tempo { brightness: 50, speed: 50 },
                7 => KeyboardMode::Wave { brightness: 50, speed: 50 },
                _ => KeyboardMode::SingleColor { r: 255, g: 255, b: 255, brightness: 50 },
            };
            
            // REAL-TIME PREVIEW: Apply immediately via DBus
            if let Some(client) = dbus_clone.borrow().as_ref() {
                let _ = client.preview_keyboard_settings(&prof.keyboard_settings);
            }
        }
    });
    
    group.add(&mode_row);
    
    // Add controls based on current mode
    match &profile.keyboard_settings.mode {
        KeyboardMode::SingleColor { r, g, b, brightness } => {
            add_rgb_controls(&group, *r, *g, *b, *brightness, config.clone(), profile.name.clone(), dbus_client.clone());
        }
        KeyboardMode::Breathe { r, g, b, brightness, speed } => {
            add_rgb_controls(&group, *r, *g, *b, *brightness, config.clone(), profile.name.clone(), dbus_client.clone());
            add_speed_control(&group, *speed, config.clone(), profile.name.clone(), dbus_client.clone());
        }
        KeyboardMode::Flash { r, g, b, brightness, speed } => {
            add_rgb_controls(&group, *r, *g, *b, *brightness, config.clone(), profile.name.clone(), dbus_client.clone());
            add_speed_control(&group, *speed, config.clone(), profile.name.clone(), dbus_client.clone());
        }
        KeyboardMode::Cycle { brightness, speed } |
        KeyboardMode::Dance { brightness, speed } |
        KeyboardMode::RandomColor { brightness, speed } |
        KeyboardMode::Tempo { brightness, speed } |
        KeyboardMode::Wave { brightness, speed } => {
            add_brightness_control(&group, *brightness, config.clone(), profile.name.clone(), dbus_client.clone());
            add_speed_control(&group, *speed, config.clone(), profile.name.clone(), dbus_client.clone());
        }
    }
    
    group
}

fn add_rgb_controls(
    group: &adw::PreferencesGroup,
    r: u8, g: u8, b: u8, brightness: u8,
    config: Rc<RefCell<Config>>,
    profile_name: String,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    let r_row = adw::ActionRow::builder().title("Red").build();
    let r_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 255.0, 1.0);
    r_scale.set_value(r as f64);
    r_scale.set_hexpand(true);
    r_scale.set_draw_value(true);
    r_scale.set_value_pos(gtk::PositionType::Right);
    r_row.add_suffix(&r_scale);
    group.add(&r_row);
    
    let g_row = adw::ActionRow::builder().title("Green").build();
    let g_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 255.0, 1.0);
    g_scale.set_value(g as f64);
    g_scale.set_hexpand(true);
    g_scale.set_draw_value(true);
    g_scale.set_value_pos(gtk::PositionType::Right);
    g_row.add_suffix(&g_scale);
    group.add(&g_row);
    
    let b_row = adw::ActionRow::builder().title("Blue").build();
    let b_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 255.0, 1.0);
    b_scale.set_value(b as f64);
    b_scale.set_hexpand(true);
    b_scale.set_draw_value(true);
    b_scale.set_value_pos(gtk::PositionType::Right);
    b_row.add_suffix(&b_scale);
    group.add(&b_row);
    
    let bright_row = adw::ActionRow::builder().title("Brightness").build();
    let bright_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    bright_scale.set_value(brightness as f64);
    bright_scale.set_hexpand(true);
    bright_scale.set_draw_value(true);
    bright_scale.set_value_pos(gtk::PositionType::Right);
    bright_row.add_suffix(&bright_scale);
    group.add(&bright_row);
    
    // Connect value changed handlers
    let r_clone = r_scale.clone();
    let g_clone = g_scale.clone();
    let b_clone = b_scale.clone();
    let bright_clone = bright_scale.clone();
    
    for scale in [&r_scale, &g_scale, &b_scale, &bright_scale] {
        let cfg = config.clone();
        let pname = profile_name.clone();
        let r_s = r_clone.clone();
        let g_s = g_clone.clone();
        let b_s = b_clone.clone();
        let br_s = bright_clone.clone();
        let dbus_clone = dbus_client.clone();
        
        scale.connect_value_changed(move |_| {
            use tuxedo_common::types::KeyboardMode;
            let r_val = r_s.value() as u8;
            let g_val = g_s.value() as u8;
            let b_val = b_s.value() as u8;
            let br_val = br_s.value() as u8;
            
            let mut config = cfg.borrow_mut();
            if let Some(prof) = config.data.profiles.iter_mut().find(|p| p.name == pname) {
                // Update the RGB values in the current mode
                match &mut prof.keyboard_settings.mode {
                    KeyboardMode::SingleColor { r, g, b, brightness } => {
                        *r = r_val;
                        *g = g_val;
                        *b = b_val;
                        *brightness = br_val;
                    }
                    KeyboardMode::Breathe { r, g, b, brightness, .. } |
                    KeyboardMode::Flash { r, g, b, brightness, .. } => {
                        *r = r_val;
                        *g = g_val;
                        *b = b_val;
                        *brightness = br_val;
                    }
                    _ => {}
                }
                
                // REAL-TIME PREVIEW: Apply immediately via DBus
                if let Some(client) = dbus_clone.borrow().as_ref() {
                    let _ = client.preview_keyboard_settings(&prof.keyboard_settings);
                }
            }
        });
    }
}

fn add_brightness_control(
    group: &adw::PreferencesGroup,
    brightness: u8,
    config: Rc<RefCell<Config>>,
    profile_name: String,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    let bright_row = adw::ActionRow::builder().title("Brightness").build();
    let bright_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    bright_scale.set_value(brightness as f64);
    bright_scale.set_hexpand(true);
    bright_scale.set_draw_value(true);
    bright_scale.set_value_pos(gtk::PositionType::Right);
    bright_row.add_suffix(&bright_scale);
    group.add(&bright_row);
    
    bright_scale.connect_value_changed(move |scale| {
        use tuxedo_common::types::KeyboardMode;
        let br_val = scale.value() as u8;
        
        let mut cfg = config.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
            match &mut prof.keyboard_settings.mode {
                KeyboardMode::Cycle { brightness, .. } |
                KeyboardMode::Dance { brightness, .. } |
                KeyboardMode::RandomColor { brightness, .. } |
                KeyboardMode::Tempo { brightness, .. } |
                KeyboardMode::Wave { brightness, .. } => {
                    *brightness = br_val;
                }
                _ => {}
            }
        }
    });
}

fn add_speed_control(
    group: &adw::PreferencesGroup,
    speed: u8,
    config: Rc<RefCell<Config>>,
    profile_name: String,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    let speed_row = adw::ActionRow::builder().title("Speed").build();
    let speed_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    speed_scale.set_value(speed as f64);
    speed_scale.set_hexpand(true);
    speed_scale.set_draw_value(true);
    speed_scale.set_value_pos(gtk::PositionType::Right);
    speed_row.add_suffix(&speed_scale);
    group.add(&speed_row);
    
    speed_scale.connect_value_changed(move |scale| {
        use tuxedo_common::types::KeyboardMode;
        let speed_val = scale.value() as u8;
        
        let mut cfg = config.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
            match &mut prof.keyboard_settings.mode {
                KeyboardMode::Breathe { speed, .. } |
                KeyboardMode::Cycle { speed, .. } |
                KeyboardMode::Dance { speed, .. } |
                KeyboardMode::Flash { speed, .. } |
                KeyboardMode::RandomColor { speed, .. } |
                KeyboardMode::Tempo { speed, .. } |
                KeyboardMode::Wave { speed, .. } => {
                    *speed = speed_val;
                }
                _ => {}
            }
            
            // REAL-TIME PREVIEW: Apply immediately via DBus
            if let Some(client) = dbus_client.borrow().as_ref() {
                let _ = client.preview_keyboard_settings(&prof.keyboard_settings);
            }
        }
    });
}

fn create_screen_tuning_section(profile: &Profile, config: Rc<RefCell<Config>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Screen")
        .build();
    
    let system_control_row = adw::SwitchRow::builder()
        .title("System control")
        .build();
    
    system_control_row.set_active(profile.screen_settings.system_control);
    
    let config_clone = config.clone();
    let profile_name = profile.name.clone();
    system_control_row.connect_active_notify(move |row| {
        let mut cfg = config_clone.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
            prof.screen_settings.system_control = row.is_active();
        }
    });
    
    group.add(&system_control_row);
    
    let brightness_row = adw::ActionRow::builder()
        .title("Brightness")
        .build();
    
    let brightness_scale = Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    brightness_scale.set_value(profile.screen_settings.brightness as f64);
    brightness_scale.set_hexpand(true);
    brightness_scale.set_draw_value(true);
    brightness_scale.set_value_pos(gtk::PositionType::Right);
    brightness_row.add_suffix(&brightness_scale);
    group.add(&brightness_row);
    
    let config_clone = config.clone();
    let profile_name = profile.name.clone();
    brightness_scale.connect_value_changed(move |scale| {
        let value = scale.value() as u8;
        let mut cfg = config_clone.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
            prof.screen_settings.brightness = value;
        }
    });
    
    group
}

fn create_fans_tuning_section(profile: &Profile, config: Rc<RefCell<Config>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Fans")
        .build();
    
    let control_row = adw::SwitchRow::builder()
        .title("Control fans")
        .build();
    
    control_row.set_active(profile.fan_settings.control_enabled);
    
    let config_clone = config.clone();
    let profile_name = profile.name.clone();
    control_row.connect_active_notify(move |row| {
        let mut cfg = config_clone.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
            prof.fan_settings.control_enabled = row.is_active();
        }
    });
    
    group.add(&control_row);
    
    group
}

// Add to GUI for fan curve editor
pub fn create_fan_curve_editor(
    fan_id: u32,
    current_curve: &FanCurve,
    config: Rc<RefCell<Config>>,
) -> gtk4::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 12);
    
    let title = gtk::Label::new(Some(&format!("Fan {} Curve", fan_id)));
    title.add_css_class("title-3");
    container.append(&title);
    
    // Add visual curve editor here
    // Users can add/remove points
    // Show temperature on X axis, speed on Y axis
    
    let points_list = gtk::Box::new(gtk::Orientation::Vertical, 6);
    
    for (i, (temp, speed)) in current_curve.points.iter().enumerate() {
        let point_row = adw::ActionRow::builder()
            .title(&format!("Point {}", i + 1))
            .build();
        
        let temp_spin = gtk::SpinButton::with_range(0.0, 100.0, 1.0);
        temp_spin.set_value(*temp as f64);
        temp_spin.set_suffix(Some("°C"));
        
        let speed_spin = gtk::SpinButton::with_range(0.0, 100.0, 1.0);
        speed_spin.set_value(*speed as f64);
        speed_spin.set_suffix(Some("%"));
        
        let remove_btn = gtk::Button::from_icon_name("user-trash-symbolic");
        remove_btn.add_css_class("destructive-action");
        
        point_row.add_suffix(&temp_spin);
        point_row.add_suffix(&speed_spin);
        point_row.add_suffix(&remove_btn);
        
        points_list.append(&point_row);
    }
    
    container.append(&points_list);
    
    let add_point_btn = gtk::Button::with_label("Add Point");
    add_point_btn.add_css_class("suggested-action");
    container.append(&add_point_btn);
    
    container
}
