use gtk::prelude::*;
use gtk::{Box, Orientation, ScrolledWindow, Scale, Button, Label};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::dbus_client::DbusClient;
use tuxedo_common::types::{FanCurve, Profile};

pub fn create_page(
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> ScrolledWindow {
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .build();
    
    let content = build_tuning_content(config.clone(), dbus_client.clone(), scrolled.clone());
    scrolled.set_child(Some(&content));
    
    // Rebuild content when profile changes
    let scrolled_weak = scrolled.downgrade();
    let config_clone = config.clone();
    let dbus_clone = dbus_client.clone();
    let mut last_profile = config.borrow().data.current_profile.clone();
    
    gtk::glib::timeout_add_seconds_local(1, move || {
        let current_profile = config_clone.borrow().data.current_profile.clone();
        
        if current_profile != last_profile {
            last_profile = current_profile.clone();
            
            // Rebuild the entire content with new profile
            if let Some(scrolled) = scrolled_weak.upgrade() {
                let new_content = build_tuning_content(config_clone.clone(), dbus_clone.clone(), scrolled.clone());
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
    scrolled: ScrolledWindow,
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
        let header = Label::new(Some(&format!("Editing Profile: {}", profile.name)));
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
        
        let keyboard_section = create_keyboard_tuning_section(
            &profile, 
            config.clone(), 
            dbus_client.clone(),
            scrolled.clone()
        );
        main_box.append(&keyboard_section);
        
        let screen_section = create_screen_tuning_section(&profile, config.clone(), dbus_client.clone());
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
        
        let config_clone = config.clone();
        let dbus_clone = dbus_client.clone();
        let profile_name = profile.name.clone();
        reset_button.connect_clicked(move |_| {
            // Reload config from disk
            if let Ok(loaded_config) = Config::load() {
                let mut cfg = config_clone.borrow_mut();
                cfg.data = loaded_config.data;
                drop(cfg);
                
                // Apply the reloaded profile
                if let Some(prof) = config_clone.borrow().data.profiles.iter()
                    .find(|p| p.name == profile_name).cloned() {
                    if let Some(client) = dbus_clone.borrow().as_ref() {
                        let _ = client.apply_profile(&prof);
                    }
                }
            }
        });
        
        button_box.append(&apply_button);
        button_box.append(&reset_button);
        main_box.append(&button_box);
    } else {
        let error_label = Label::new(Some("No profile selected"));
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
    _dbus_client: Rc<RefCell<Option<DbusClient>>>,
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
    scrolled: ScrolledWindow,
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
    let scrolled_clone = scrolled.clone();
    mode_row.connect_selected_notify(move |row| {
        let mode_idx = row.selected();
        let mut cfg = config_clone.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
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
            
            if let Some(client) = dbus_clone.borrow().as_ref() {
                let _ = client.preview_keyboard_settings(&prof.keyboard_settings);
            }
        }
        drop(cfg);
        
        // Rebuild the entire tuning page to show new controls
        let new_content = build_tuning_content(config_clone.clone(), dbus_clone.clone(), scrolled_clone.clone());
        scrolled_clone.set_child(Some(&new_content));
    });
    
    group.add(&mode_row);
    
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
            
            if let Some(client) = dbus_client.borrow().as_ref() {
                let _ = client.preview_keyboard_settings(&prof.keyboard_settings);
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
            
            if let Some(client) = dbus_client.borrow().as_ref() {
                let _ = client.preview_keyboard_settings(&prof.keyboard_settings);
            }
        }
    });
}

fn create_screen_tuning_section(
    profile: &Profile, 
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) -> adw::PreferencesGroup {
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
    let dbus_clone = dbus_client.clone();
    brightness_scale.connect_value_changed(move |scale| {
        let value = scale.value() as u8;
        let mut cfg = config_clone.borrow_mut();
        if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
            prof.screen_settings.brightness = value;
            
            // Apply brightness immediately if not under system control
            if !prof.screen_settings.system_control {
                drop(cfg);
                
                // Apply brightness to system
                if let Err(e) = apply_brightness_to_system(value) {
                    eprintln!("Failed to apply brightness: {}", e);
                }
            }
        }
    });
    
    group
}

fn apply_brightness_to_system(brightness: u8) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Write;
    
    // Find backlight device
    let backlight_base = "/sys/class/backlight";
    let entries = fs::read_dir(backlight_base)?;
    
    for entry in entries.flatten() {
        let path = entry.path();
        let max_brightness_path = path.join("max_brightness");
        let brightness_path = path.join("brightness");
        
        if let Ok(max_str) = fs::read_to_string(&max_brightness_path) {
            if let Ok(max_brightness) = max_str.trim().parse::<u32>() {
                let target_brightness = (max_brightness as f64 * brightness as f64 / 100.0) as u32;
                
                // Write brightness value
                if let Ok(mut file) = fs::OpenOptions::new()
                    .write(true)
                    .open(&brightness_path) 
                {
                    let _ = write!(file, "{}", target_brightness);
                    return Ok(());
                }
            }
        }
    }
    
    Err("No backlight device found".into())
}

fn create_fans_tuning_section(profile: &Profile, config: Rc<RefCell<Config>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Fans")
        .build();
    
    let control_row = adw::SwitchRow::builder()
        .title("Control fans")
        .subtitle("Enable custom fan curves")
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
    
    if profile.fan_settings.control_enabled {
        let fan_count = if profile.fan_settings.curves.is_empty() {
            2
        } else {
            profile.fan_settings.curves.len().max(1)
        };
        
        for fan_id in 0..fan_count {
            let curve = profile.fan_settings.curves
                .iter()
                .find(|c| c.fan_id == fan_id as u32)
                .cloned()
                .unwrap_or_else(|| {
                    FanCurve {
                        fan_id: fan_id as u32,
                        points: vec![(0, 0), (50, 50), (80, 100)],
                    }
                });
            
            let expander = adw::ExpanderRow::builder()
                .title(&format!("Fan {} Curve", fan_id))
                .subtitle(&format!("{} points", curve.points.len()))
                .build();
            
            for (i, (temp, speed)) in curve.points.iter().enumerate() {
                let point_row = adw::ActionRow::builder()
                    .title(&format!("Point {}", i + 1))
                    .build();
                
                let temp_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                let temp_label = gtk::Label::new(Some("Temp (°C):"));
                let temp_spin = gtk::SpinButton::with_range(0.0, 100.0, 1.0);
                temp_spin.set_value(*temp as f64);
                temp_spin.set_width_chars(5);
                temp_box.append(&temp_label);
                temp_box.append(&temp_spin);
                
                let speed_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                let speed_label = gtk::Label::new(Some("Speed (%):"));
                let speed_spin = gtk::SpinButton::with_range(0.0, 100.0, 1.0);
                speed_spin.set_value(*speed as f64);
                speed_spin.set_width_chars(5);
                speed_box.append(&speed_label);
                speed_box.append(&speed_spin);
                
                point_row.add_suffix(&temp_box);
                point_row.add_suffix(&speed_box);
                
                // Add delete button
                let delete_btn = Button::from_icon_name("user-trash-symbolic");
                delete_btn.add_css_class("destructive-action");
                delete_btn.set_valign(gtk::Align::Center);
                delete_btn.set_tooltip_text(Some("Delete point"));
                
                let config_clone = config.clone();
                let profile_name = profile.name.clone();
                let fan_id_copy = fan_id as u32;
                let point_idx = i;
                delete_btn.connect_clicked(move |_| {
                    let mut cfg = config_clone.borrow_mut();
                    if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
                        if let Some(curve) = prof.fan_settings.curves.iter_mut().find(|c| c.fan_id == fan_id_copy) {
                            if curve.points.len() > 2 && point_idx < curve.points.len() {
                                curve.points.remove(point_idx);
                            }
                        }
                    }
                });
                
                point_row.add_suffix(&delete_btn);
                
                let config_clone = config.clone();
                let profile_name_clone = profile.name.clone();
                let fan_id_copy = fan_id as u32;
                let point_idx = i;
                
                let config_for_temp = config_clone.clone();
                let profile_for_temp = profile_name_clone.clone();
                let speed_spin_clone = speed_spin.clone();
                temp_spin.connect_value_changed(move |temp_spin| {
                    let temp_val = temp_spin.value() as u8;
                    let speed_val = speed_spin_clone.value() as u8;
                    update_fan_curve_point(&config_for_temp, &profile_for_temp, fan_id_copy, point_idx, temp_val, speed_val);
                });
                
                let config_for_speed = config_clone.clone();
                let profile_for_speed = profile_name_clone.clone();
                let temp_spin_clone = temp_spin.clone();
                speed_spin.connect_value_changed(move |speed_spin| {
                    let temp_val = temp_spin_clone.value() as u8;
                    let speed_val = speed_spin.value() as u8;
                    update_fan_curve_point(&config_for_speed, &profile_for_speed, fan_id_copy, point_idx, temp_val, speed_val);
                });
                
                expander.add_row(&point_row);
            }
            
            let add_point_row = adw::ActionRow::builder()
                .title("Add Point")
                .build();
            
            let add_button = Button::with_label("➕ Add");
            add_button.add_css_class("suggested-action");
            add_button.set_valign(gtk::Align::Center);
            
            let config_clone = config.clone();
            let profile_name_clone = profile.name.clone();
            let fan_id_copy = fan_id as u32;
            add_button.connect_clicked(move |_| {
                add_fan_curve_point(&config_clone, &profile_name_clone, fan_id_copy);
            });
            
            add_point_row.add_suffix(&add_button);
            expander.add_row(&add_point_row);
            
            group.add(&expander);
        }
    }
    
    group
}

fn update_fan_curve_point(
    config: &Rc<RefCell<Config>>,
    profile_name: &str,
    fan_id: u32,
    point_idx: usize,
    temp: u8,
    speed: u8,
) {
    let mut cfg = config.borrow_mut();
    if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
        if let Some(curve) = prof.fan_settings.curves.iter_mut().find(|c| c.fan_id == fan_id) {
            if point_idx < curve.points.len() {
                curve.points[point_idx] = (temp, speed);
            }
        } else {
            prof.fan_settings.curves.push(FanCurve {
                fan_id,
                points: vec![(temp, speed)],
            });
        }
    }
}

fn add_fan_curve_point(
    config: &Rc<RefCell<Config>>,
    profile_name: &str,
    fan_id: u32,
) {
    let mut cfg = config.borrow_mut();
    if let Some(prof) = cfg.data.profiles.iter_mut().find(|p| p.name == profile_name) {
        if let Some(curve) = prof.fan_settings.curves.iter_mut().find(|c| c.fan_id == fan_id) {
            let new_temp = if let Some((last_temp, _)) = curve.points.last() {
                (*last_temp + 10).min(100)
            } else {
                50
            };
            curve.points.push((new_temp, 50));
        } else {
            prof.fan_settings.curves.push(FanCurve {
                fan_id,
                points: vec![(50, 50)],
            });
        }
    }
}
