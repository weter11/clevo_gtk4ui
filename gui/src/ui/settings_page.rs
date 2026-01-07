use gtk::prelude::*;
use gtk::{Box, Orientation, ScrolledWindow, Scale};
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
        .vexpand(true)
        .hexpand(true)
        .build();
    
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);
    
    // Appearance Group
    let appearance_group = adw::PreferencesGroup::builder()
        .title("Appearance")
        .build();
    
    let theme_row = adw::ComboRow::builder()
        .title("Theme")
        .build();
    
    let theme_model = gtk::StringList::new(&["Auto", "Light", "Dark"]);
    theme_row.set_model(Some(&theme_model));
    
    let current_theme = match config.borrow().data.theme {
        tuxedo_common::types::Theme::Auto => 0,
        tuxedo_common::types::Theme::Light => 1,
        tuxedo_common::types::Theme::Dark => 2,
    };
    theme_row.set_selected(current_theme);
    
    let config_clone = config.clone();
    theme_row.connect_selected_notify(move |row| {
        let theme = match row.selected() {
            0 => tuxedo_common::types::Theme::Auto,
            1 => tuxedo_common::types::Theme::Light,
            2 => tuxedo_common::types::Theme::Dark,
            _ => tuxedo_common::types::Theme::Auto,
        };
        
        config_clone.borrow_mut().data.theme = theme.clone();
        let _ = config_clone.borrow().save();
        
        // Apply theme immediately
        let style_manager = adw::StyleManager::default();
        match theme {
            tuxedo_common::types::Theme::Auto => {
                style_manager.set_color_scheme(adw::ColorScheme::Default);
            },
            tuxedo_common::types::Theme::Light => {
                style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
            },
            tuxedo_common::types::Theme::Dark => {
                style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
            },
        }
    });
    
    appearance_group.add(&theme_row);
    main_box.append(&appearance_group);
    
    // Battery Charge Control Group
    if dbus_client.borrow().is_some() {
        let battery_group = adw::PreferencesGroup::builder()
            .title("Battery Charge Control")
            .description("Configure battery charging thresholds to extend battery lifespan")
            .build();
        
        // Charge type
        let charge_type_row = adw::ComboRow::builder()
            .title("Charge Type")
            .subtitle("Select charging behavior")
            .build();
        
        let charge_types = vec!["Standard", "Express", "Primarily AC"];
        let charge_types_values = vec!["standard", "express", "primarily_ac"];
        let model = gtk::StringList::new(&charge_types);
        charge_type_row.set_model(Some(&model));
        
        if let Some(client) = dbus_client.borrow().as_ref() {
            if let Ok(current_type) = client.get_battery_charge_type() {
                if let Some(idx) = charge_types_values.iter().position(|t| *t == current_type.as_str()) {
                    charge_type_row.set_selected(idx as u32);
                }
            }
        }
        
        let dbus_clone = dbus_client.clone();
        charge_type_row.connect_selected_notify(move |row| {
            let idx = row.selected() as usize;
            let types = vec!["standard", "express", "primarily_ac"];
            if idx < types.len() {
                if let Some(client) = dbus_clone.borrow().as_ref() {
                    let _ = client.set_battery_charge_type(types[idx]);
                }
            }
        });
        
        battery_group.add(&charge_type_row);
        
        // Start threshold
        if let Some(client) = dbus_client.borrow().as_ref() {
            if let Ok(available_starts) = client.get_battery_available_start_thresholds() {
                if !available_starts.is_empty() {
                    let start_row = adw::ActionRow::builder()
                        .title("Charge Start Threshold")
                        .subtitle("Battery will start charging when below this level")
                        .build();
                    
                    let current_start = client.get_battery_charge_start_threshold().unwrap_or(0);
                    
                    let start_scale = Scale::with_range(
                        gtk::Orientation::Horizontal,
                        *available_starts.first().unwrap() as f64,
                        *available_starts.last().unwrap() as f64,
                        1.0,
                    );
                    start_scale.set_value(current_start as f64);
                    start_scale.set_hexpand(true);
                    start_scale.set_draw_value(true);
                    start_scale.set_value_pos(gtk::PositionType::Right);
                    start_scale.set_format_value_func(|_, val| format!("{}%", val as u8));
                    
                    let dbus_clone = dbus_client.clone();
                    start_scale.connect_value_changed(move |scale| {
                        let value = scale.value() as u8;
                        if let Some(client) = dbus_clone.borrow().as_ref() {
                            let _ = client.set_battery_charge_start_threshold(value);
                        }
                    });
                    
                    start_row.add_suffix(&start_scale);
                    battery_group.add(&start_row);
                }
            }
        }
        
        // End threshold
        if let Some(client) = dbus_client.borrow().as_ref() {
            if let Ok(available_ends) = client.get_battery_available_end_thresholds() {
                if !available_ends.is_empty() {
                    let end_row = adw::ActionRow::builder()
                        .title("Charge End Threshold")
                        .subtitle("Battery will stop charging when reaching this level")
                        .build();
                    
                    let current_end = client.get_battery_charge_end_threshold().unwrap_or(100);
                    
                    let end_scale = Scale::with_range(
                        gtk::Orientation::Horizontal,
                        *available_ends.first().unwrap() as f64,
                        *available_ends.last().unwrap() as f64,
                        1.0,
                    );
                    end_scale.set_value(current_end as f64);
                    end_scale.set_hexpand(true);
                    end_scale.set_draw_value(true);
                    end_scale.set_value_pos(gtk::PositionType::Right);
                    end_scale.set_format_value_func(|_, val| format!("{}%", val as u8));
                    
                    let dbus_clone = dbus_client.clone();
                    end_scale.connect_value_changed(move |scale| {
                        let value = scale.value() as u8;
                        if let Some(client) = dbus_clone.borrow().as_ref() {
                            let _ = client.set_battery_charge_end_threshold(value);
                        }
                    });
                    
                    end_row.add_suffix(&end_scale);
                    battery_group.add(&end_row);
                }
            }
        }
        
        main_box.append(&battery_group);
    }
    
    // Startup Group
    let startup_group = adw::PreferencesGroup::builder()
        .title("Startup")
        .build();
    
    let minimized_row = adw::SwitchRow::builder()
        .title("Start minimized")
        .subtitle("Start in system tray")
        .build();
    minimized_row.set_active(config.borrow().data.start_minimized);
    
    let config_clone = config.clone();
    minimized_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.start_minimized = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    startup_group.add(&minimized_row);
    
    let autostart_row = adw::SwitchRow::builder()
        .title("Enable autostart")
        .subtitle("Launch on system boot")
        .build();
    autostart_row.set_active(config.borrow().data.autostart);
    
    let config_clone = config.clone();
    autostart_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.autostart = row.is_active();
        let _ = config_clone.borrow().save();
        
        // Create or remove autostart file
        if row.is_active() {
            let _ = create_autostart_file();
        } else {
            let _ = remove_autostart_file();
        }
    });
    
    startup_group.add(&autostart_row);
    main_box.append(&startup_group);
    
    // Daemon Controls Group
    let daemon_group = adw::PreferencesGroup::builder()
        .title("Daemon Controls")
        .build();
    
    let fan_daemon_row = adw::SwitchRow::builder()
        .title("Fan daemon")
        .subtitle("Monitor temperatures and apply fan curves")
        .build();
    fan_daemon_row.set_active(config.borrow().data.fan_daemon_enabled);
    
    let config_clone = config.clone();
    fan_daemon_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.fan_daemon_enabled = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    daemon_group.add(&fan_daemon_row);
    
    let app_monitoring_row = adw::SwitchRow::builder()
        .title("App monitoring")
        .subtitle("Enable automatic profile switching")
        .build();
    app_monitoring_row.set_active(config.borrow().data.app_monitoring_enabled);
    
    let config_clone = config.clone();
    app_monitoring_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.app_monitoring_enabled = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    daemon_group.add(&app_monitoring_row);
    
    let auto_switch_row = adw::SwitchRow::builder()
        .title("Automatic profile switching")
        .subtitle("Switch profiles based on running applications")
        .build();
    auto_switch_row.set_active(config.borrow().data.auto_profile_switching);
    
    let config_clone = config.clone();
    auto_switch_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.auto_profile_switching = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    daemon_group.add(&auto_switch_row);
    main_box.append(&daemon_group);
    
    // CPU Scheduler Group (Global setting)
    let scheduler_group = adw::PreferencesGroup::builder()
        .title("CPU Scheduler")
        .description("Global scheduler setting (not per-profile)")
        .build();
    
    let scheduler_row = adw::ComboRow::builder()
        .title("Scheduler")
        .build();
    
    let scheduler_model = gtk::StringList::new(&["CFS", "EEVDF"]);
    scheduler_row.set_model(Some(&scheduler_model));
    scheduler_row.set_selected(if config.borrow().data.cpu_scheduler == "CFS" { 0 } else { 1 });
    
    let config_clone = config.clone();
    scheduler_row.connect_selected_notify(move |row| {
        let scheduler = if row.selected() == 0 { "CFS" } else { "EEVDF" };
        config_clone.borrow_mut().data.cpu_scheduler = scheduler.to_string();
        let _ = config_clone.borrow().save();
    });
    
    scheduler_group.add(&scheduler_row);
    main_box.append(&scheduler_group);
    
    // Statistics Page Layout Group
    let stats_layout_group = adw::PreferencesGroup::builder()
        .title("Statistics Page Layout")
        .build();
    
    let show_system_info_row = adw::SwitchRow::builder()
        .title("Show system info")
        .build();
    show_system_info_row.set_active(config.borrow().data.statistics_sections.show_system_info);
    
    let config_clone = config.clone();
    show_system_info_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.statistics_sections.show_system_info = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    stats_layout_group.add(&show_system_info_row);
    
    let show_cpu_row = adw::SwitchRow::builder()
        .title("Show CPU")
        .build();
    show_cpu_row.set_active(config.borrow().data.statistics_sections.show_cpu);
    
    let config_clone = config.clone();
    show_cpu_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.statistics_sections.show_cpu = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    stats_layout_group.add(&show_cpu_row);
    
    let show_gpu_row = adw::SwitchRow::builder()
        .title("Show GPU")
        .build();
    show_gpu_row.set_active(config.borrow().data.statistics_sections.show_gpu);
    
    let config_clone = config.clone();
    show_gpu_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.statistics_sections.show_gpu = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    stats_layout_group.add(&show_gpu_row);
    
    let show_battery_row = adw::SwitchRow::builder()
        .title("Show battery")
        .build();
    show_battery_row.set_active(config.borrow().data.statistics_sections.show_battery);
    
    let config_clone = config.clone();
    show_battery_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.statistics_sections.show_battery = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    stats_layout_group.add(&show_battery_row);
    
    let show_wifi_row = adw::SwitchRow::builder()
        .title("Show WiFi")
        .build();
    show_wifi_row.set_active(config.borrow().data.statistics_sections.show_wifi);
    
    let config_clone = config.clone();
    show_wifi_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.statistics_sections.show_wifi = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    stats_layout_group.add(&show_wifi_row);
    
    let show_storage_row = adw::SwitchRow::builder()
        .title("Show storage")
        .build();
    show_storage_row.set_active(config.borrow().data.statistics_sections.show_storage);
    
    let config_clone = config.clone();
    show_storage_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.statistics_sections.show_storage = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    stats_layout_group.add(&show_storage_row);
    
    let show_fans_row = adw::SwitchRow::builder()
        .title("Show fans")
        .build();
    show_fans_row.set_active(config.borrow().data.statistics_sections.show_fans);
    
    let config_clone = config.clone();
    show_fans_row.connect_active_notify(move |row| {
        config_clone.borrow_mut().data.statistics_sections.show_fans = row.is_active();
        let _ = config_clone.borrow().save();
    });
    
    stats_layout_group.add(&show_fans_row);
    main_box.append(&stats_layout_group);
    
    // Statistics Polling Rates Group
    let polling_group = adw::PreferencesGroup::builder()
        .title("Statistics Polling Rates")
        .description("How often to update each statistics section (in seconds)")
        .build();
    
    // System info polling
    let system_poll_row = adw::ActionRow::builder()
        .title("System Info Polling")
        .subtitle(&format!("{} seconds", config.borrow().data.statistics_sections.system_info_poll_rate / 1000))
        .build();
    let system_poll_scale = Scale::with_range(gtk::Orientation::Horizontal, 10.0, 300.0, 10.0);
    system_poll_scale.set_value((config.borrow().data.statistics_sections.system_info_poll_rate / 1000) as f64);
    system_poll_scale.set_hexpand(true);
    system_poll_scale.set_draw_value(false);
    system_poll_row.add_suffix(&system_poll_scale);
    let config_clone = config.clone();
    system_poll_scale.connect_value_changed(move |scale| {
        let val = (scale.value() as u64) * 1000;
        config_clone.borrow_mut().data.statistics_sections.system_info_poll_rate = val;
        let _ = config_clone.borrow().save();
        if let Some(row) = scale.parent().and_then(|p| p.downcast::<adw::ActionRow>().ok()) {
            row.set_subtitle(&format!("{} seconds", val / 1000));
        }
    });
    polling_group.add(&system_poll_row);
    
    // CPU polling
    let cpu_poll_row = adw::ActionRow::builder()
        .title("CPU Polling")
        .subtitle(&format!("{} seconds", config.borrow().data.statistics_sections.cpu_poll_rate / 1000))
        .build();
    let cpu_poll_scale = Scale::with_range(gtk::Orientation::Horizontal, 1.0, 10.0, 1.0);
    cpu_poll_scale.set_value((config.borrow().data.statistics_sections.cpu_poll_rate / 1000) as f64);
    cpu_poll_scale.set_hexpand(true);
    cpu_poll_scale.set_draw_value(false);
    cpu_poll_row.add_suffix(&cpu_poll_scale);
    let config_clone = config.clone();
    cpu_poll_scale.connect_value_changed(move |scale| {
        let val = (scale.value() as u64) * 1000;
        config_clone.borrow_mut().data.statistics_sections.cpu_poll_rate = val;
        let _ = config_clone.borrow().save();
        if let Some(row) = scale.parent().and_then(|p| p.downcast::<adw::ActionRow>().ok()) {
            row.set_subtitle(&format!("{} seconds", val / 1000));
        }
    });
    polling_group.add(&cpu_poll_row);
    
    // GPU polling
    let gpu_poll_row = adw::ActionRow::builder()
        .title("GPU Polling")
        .subtitle(&format!("{} seconds", config.borrow().data.statistics_sections.gpu_poll_rate / 1000))
        .build();
    let gpu_poll_scale = Scale::with_range(gtk::Orientation::Horizontal, 1.0, 10.0, 1.0);
    gpu_poll_scale.set_value((config.borrow().data.statistics_sections.gpu_poll_rate / 1000) as f64);
    gpu_poll_scale.set_hexpand(true);
    gpu_poll_scale.set_draw_value(false);
    gpu_poll_row.add_suffix(&gpu_poll_scale);
    let config_clone = config.clone();
    gpu_poll_scale.connect_value_changed(move |scale| {
        let val = (scale.value() as u64) * 1000;
        config_clone.borrow_mut().data.statistics_sections.gpu_poll_rate = val;
        let _ = config_clone.borrow().save();
        if let Some(row) = scale.parent().and_then(|p| p.downcast::<adw::ActionRow>().ok()) {
            row.set_subtitle(&format!("{} seconds", val / 1000));
        }
    });
    polling_group.add(&gpu_poll_row);
    
    // Battery polling
    let battery_poll_row = adw::ActionRow::builder()
        .title("Battery Polling")
        .subtitle(&format!("{} seconds", config.borrow().data.statistics_sections.battery_poll_rate / 1000))
        .build();
    let battery_poll_scale = Scale::with_range(gtk::Orientation::Horizontal, 1.0, 30.0, 1.0);
    battery_poll_scale.set_value((config.borrow().data.statistics_sections.battery_poll_rate / 1000) as f64);
    battery_poll_scale.set_hexpand(true);
    battery_poll_scale.set_draw_value(false);
    battery_poll_row.add_suffix(&battery_poll_scale);
    let config_clone = config.clone();
    battery_poll_scale.connect_value_changed(move |scale| {
        let val = (scale.value() as u64) * 1000;
        config_clone.borrow_mut().data.statistics_sections.battery_poll_rate = val;
        let _ = config_clone.borrow().save();
        if let Some(row) = scale.parent().and_then(|p| p.downcast::<adw::ActionRow>().ok()) {
            row.set_subtitle(&format!("{} seconds", val / 1000));
        }
    });
    polling_group.add(&battery_poll_row);
    
    // WiFi polling
    let wifi_poll_row = adw::ActionRow::builder()
        .title("WiFi Polling")
        .subtitle(&format!("{} seconds", config.borrow().data.statistics_sections.wifi_poll_rate / 1000))
        .build();
    let wifi_poll_scale = Scale::with_range(gtk::Orientation::Horizontal, 1.0, 30.0, 1.0);
    wifi_poll_scale.set_value((config.borrow().data.statistics_sections.wifi_poll_rate / 1000) as f64);
    wifi_poll_scale.set_hexpand(true);
    wifi_poll_scale.set_draw_value(false);
    wifi_poll_row.add_suffix(&wifi_poll_scale);
    let config_clone = config.clone();
    wifi_poll_scale.connect_value_changed(move |scale| {
        let val = (scale.value() as u64) * 1000;
        config_clone.borrow_mut().data.statistics_sections.wifi_poll_rate = val;
        let _ = config_clone.borrow().save();
        if let Some(row) = scale.parent().and_then(|p| p.downcast::<adw::ActionRow>().ok()) {
            row.set_subtitle(&format!("{} seconds", val / 1000));
        }
    });
    polling_group.add(&wifi_poll_row);
    
    // Storage polling
    let storage_poll_row = adw::ActionRow::builder()
        .title("Storage Polling")
        .subtitle(&format!("{} seconds", config.borrow().data.statistics_sections.storage_poll_rate / 1000))
        .build();
    let storage_poll_scale = Scale::with_range(gtk::Orientation::Horizontal, 10.0, 300.0, 10.0);
    storage_poll_scale.set_value((config.borrow().data.statistics_sections.storage_poll_rate / 1000) as f64);
    storage_poll_scale.set_hexpand(true);
    storage_poll_scale.set_draw_value(false);
    storage_poll_row.add_suffix(&storage_poll_scale);
    let config_clone = config.clone();
    storage_poll_scale.connect_value_changed(move |scale| {
        let val = (scale.value() as u64) * 1000;
        config_clone.borrow_mut().data.statistics_sections.storage_poll_rate = val;
        let _ = config_clone.borrow().save();
        if let Some(row) = scale.parent().and_then(|p| p.downcast::<adw::ActionRow>().ok()) {
            row.set_subtitle(&format!("{} seconds", val / 1000));
        }
    });
    polling_group.add(&storage_poll_row);
    
    // Fans polling
    let fans_poll_row = adw::ActionRow::builder()
        .title("Fans Polling")
        .subtitle(&format!("{} seconds", config.borrow().data.statistics_sections.fans_poll_rate / 1000))
        .build();
    let fans_poll_scale = Scale::with_range(gtk::Orientation::Horizontal, 1.0, 10.0, 1.0);
    fans_poll_scale.set_value((config.borrow().data.statistics_sections.fans_poll_rate / 1000) as f64);
    fans_poll_scale.set_hexpand(true);
    fans_poll_scale.set_draw_value(false);
    fans_poll_row.add_suffix(&fans_poll_scale);
    let config_clone = config.clone();
    fans_poll_scale.connect_value_changed(move |scale| {
        let val = (scale.value() as u64) * 1000;
        config_clone.borrow_mut().data.statistics_sections.fans_poll_rate = val;
        let _ = config_clone.borrow().save();
        if let Some(row) = scale.parent().and_then(|p| p.downcast::<adw::ActionRow>().ok()) {
            row.set_subtitle(&format!("{} seconds", val / 1000));
        }
    });
    polling_group.add(&fans_poll_row);
    
    main_box.append(&polling_group);
    
    scrolled.set_child(Some(&main_box));
    scrolled
}

fn create_autostart_file() -> std::io::Result<()> {
    use std::fs;
    use std::path::PathBuf;
    
    let home = std::env::var("HOME").unwrap_or_default();
    let autostart_dir = PathBuf::from(&home).join(".config/autostart");
    fs::create_dir_all(&autostart_dir)?;
    
    let desktop_content = r#"[Desktop Entry]
Type=Application
Name=TUXEDO Control Center
Exec=tuxedo-control-center --minimized
Icon=preferences-system
Comment=Hardware control for TUXEDO/Clevo laptops
X-GNOME-Autostart-enabled=true
"#;
    
    let autostart_file = autostart_dir.join("tuxedo-control-center.desktop");
    fs::write(autostart_file, desktop_content)?;
    
    Ok(())
}

fn remove_autostart_file() -> std::io::Result<()> {
    use std::fs;
    use std::path::PathBuf;
    
    let home = std::env::var("HOME").unwrap_or_default();
    let autostart_file = PathBuf::from(&home)
        .join(".config/autostart/tuxedo-control-center.desktop");
    
    if autostart_file.exists() {
        fs::remove_file(autostart_file)?;
    }
    
    Ok(())
}
