use gtk::prelude::*;
use gtk::{Box, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use gtk::glib;

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
    
    // Create sections with background polling
    
    if config.borrow().data.statistics_sections.show_system_info {
        let (group, refs) = create_system_info_section();
        main_box.append(&group);
        
        let dbus_clone = dbus_client.clone();
        let poll_rate = config.borrow().data.statistics_sections.system_info_poll_rate;
        start_background_poll_system(refs, dbus_clone, poll_rate);
    }
    
    if config.borrow().data.statistics_sections.show_cpu {
        let (group, refs) = create_cpu_section();
        main_box.append(&group);
        
        let dbus_clone = dbus_client.clone();
        let poll_rate = config.borrow().data.statistics_sections.cpu_poll_rate;
        start_background_poll_cpu(refs, dbus_clone, poll_rate);
    }
    
    if config.borrow().data.statistics_sections.show_gpu {
        let group = create_gpu_section();
        main_box.append(&group);
    }
    
    if config.borrow().data.statistics_sections.show_battery {
        let (group, refs) = create_battery_section(dbus_client.clone());
        main_box.append(&group);
        
        let poll_rate = config.borrow().data.statistics_sections.battery_poll_rate;
        start_background_poll_battery(refs, dbus_client.clone(), poll_rate);
    }
    
    if config.borrow().data.statistics_sections.show_wifi {
        let group = create_wifi_section();
        main_box.append(&group);
    }
    
    if config.borrow().data.statistics_sections.show_storage {
        let group = create_storage_section();
        main_box.append(&group);
    }
    
    if config.borrow().data.statistics_sections.show_fans {
        let (group, refs) = create_fans_section();
        main_box.append(&group);
        
        let dbus_clone = dbus_client.clone();
        let poll_rate = config.borrow().data.statistics_sections.fans_poll_rate;
        start_background_poll_fans(refs, dbus_clone, poll_rate);
    }
    
    scrolled.set_child(Some(&main_box));
    scrolled
}

// Background polling for CPU info
fn start_background_poll_cpu(
    refs: WidgetRefs,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
    poll_rate_ms: u64,
) {
    // Clone for the closure
    let refs_clone = refs.clone();
    let dbus_clone = dbus_client.clone();
    
    // Do an immediate first update
    update_cpu_info(&refs, dbus_client.clone());
    
    // Schedule periodic updates
    glib::timeout_add_local(
        std::time::Duration::from_millis(poll_rate_ms),
        move || {
            if let Some(client) = dbus_clone.borrow().as_ref() {
                if let Ok(cpu_info) = client.get_cpu_info() {
                    update_cpu_info_with_data(&refs_clone, cpu_info);
                }
            }
            glib::ControlFlow::Continue
        }
    );
}

// Background polling for system info
fn start_background_poll_system(
    refs: WidgetRefs,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
    poll_rate_ms: u64,
) {
    // Clone for the closure
    let refs_clone = refs.clone();
    let dbus_clone = dbus_client.clone();
    
    // Do an immediate first update
    update_system_info(&refs, dbus_client.clone());
    
    // Schedule periodic updates
    glib::timeout_add_local(
        std::time::Duration::from_millis(poll_rate_ms),
        move || {
            if let Some(client) = dbus_clone.borrow().as_ref() {
                if let Ok(system_info) = client.get_system_info() {
                    update_system_info_with_data(&refs_clone, system_info);
                }
            }
            glib::ControlFlow::Continue
        }
    );
}

// Background polling for battery info
fn start_background_poll_battery(
    refs: WidgetRefs,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
    poll_rate_ms: u64,
) {
    // Battery info is read from sysfs directly in the UI process, no DBus needed
    // But we also check battery charge control status via DBus
    let refs_clone = refs.clone();
    let dbus_clone = dbus_client.clone();
    
    // Initial update
    update_battery_info(&refs, &dbus_client);
    
    glib::timeout_add_seconds_local((poll_rate_ms / 1000).max(1) as u32, move || {
        update_battery_info(&refs_clone, &dbus_clone);
        glib::ControlFlow::Continue
    });
}

// Background polling for fans info
fn start_background_poll_fans(
    refs: WidgetRefs,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
    poll_rate_ms: u64,
) {
    // Clone for the closure
    let refs_clone = refs.clone();
    let dbus_clone = dbus_client.clone();
    
    // Schedule periodic updates
    glib::timeout_add_local(
        std::time::Duration::from_millis(poll_rate_ms),
        move || {
            if let Some(client) = dbus_clone.borrow().as_ref() {
                if let Ok(fans) = client.get_fan_info() {
                    update_fans_info_with_data(&refs_clone, fans);
                }
            }
            glib::ControlFlow::Continue
        }
    );
}

fn update_fans_info_with_data(refs: &WidgetRefs, fans: Vec<tuxedo_common::types::FanInfo>) {
    // Hide all rows first
    for (_, row) in refs {
        row.set_visible(false);
    }
    
    // Update with actual fan data
    for (idx, fan) in fans.iter().enumerate() {
        if idx < refs.len() {
            let (_, row) = &refs[idx];
            
            if let Some(temp) = fan.temperature {
                if fan.is_rpm {
                    row.set_subtitle(&format!("{} RPM - {}°C", fan.rpm_or_percent, temp));
                } else {
                    row.set_subtitle(&format!("{}% - {}°C", fan.rpm_or_percent, temp));
                }
            } else {
                if fan.is_rpm {
                    row.set_subtitle(&format!("{} RPM", fan.rpm_or_percent));
                } else {
                    row.set_subtitle(&format!("{}%", fan.rpm_or_percent));
                }
            }
            
            row.set_visible(true);
        }
    }
}

type WidgetRefs = Vec<(String, adw::ActionRow)>;

fn create_system_info_section() -> (adw::PreferencesGroup, WidgetRefs) {
    let group = adw::PreferencesGroup::builder()
        .title("System Information")
        .build();
    
    let model_row = adw::ActionRow::builder()
        .title("Notebook Model")
        .subtitle("Loading...")
        .build();
    
    let manufacturer_row = adw::ActionRow::builder()
        .title("Manufacturer")
        .subtitle("Loading...")
        .build();
    
    let bios_row = adw::ActionRow::builder()
        .title("BIOS Version")
        .subtitle("Loading...")
        .build();
    
    group.add(&model_row);
    group.add(&manufacturer_row);
    group.add(&bios_row);
    
    let refs = vec![
        ("model".to_string(), model_row),
        ("manufacturer".to_string(), manufacturer_row),
        ("bios".to_string(), bios_row),
    ];
    
    (group, refs)
}

fn update_system_info(refs: &WidgetRefs, dbus_client: Rc<RefCell<Option<DbusClient>>>) {
    if let Some(client) = dbus_client.borrow().as_ref() {
        if let Ok(info) = client.get_system_info() {
            update_system_info_with_data(refs, info);
        }
    }
}

fn update_system_info_with_data(refs: &WidgetRefs, info: tuxedo_common::types::SystemInfo) {
    for (key, row) in refs {
        match key.as_str() {
            "model" => {
                row.set_subtitle(&info.product_name);
            }
            "manufacturer" => {
                row.set_subtitle(&info.manufacturer);
            }
            "bios" => {
                row.set_subtitle(&info.bios_version);
            }
            _ => {}
        }
    }
}


fn create_cpu_section() -> (adw::PreferencesGroup, WidgetRefs) {
    let group = adw::PreferencesGroup::builder()
        .title("CPU")
        .build();
    
    let name_row = adw::ActionRow::builder()
        .title("Processor")
        .subtitle("Loading...")
        .build();
    group.add(&name_row);
    
    let freq_row = adw::ActionRow::builder()
        .title("Median Frequency")
        .subtitle("Loading...")
        .build();
    group.add(&freq_row);
    
    let load_row = adw::ActionRow::builder()
        .title("Median Load")
        .subtitle("Loading...")
        .build();
    group.add(&load_row);
    
    let temp_row = adw::ActionRow::builder()
        .title("Package Temperature")
        .subtitle("Loading...")
        .build();
    group.add(&temp_row);
    
    let power_row = adw::ActionRow::builder()
        .title("Package Power")
        .subtitle("Loading...")
        .build();
    group.add(&power_row);
    
    let scaling_driver_row = adw::ActionRow::builder()
        .title("Scaling Driver")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&scaling_driver_row);
    
    let governor_row = adw::ActionRow::builder()
        .title("CPU Governor")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&governor_row);
    
    let epp_row = adw::ActionRow::builder()
        .title("Energy Performance Preference")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&epp_row);
    
    let boost_row = adw::ActionRow::builder()
        .title("CPU Boost")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&boost_row);
    
    let smt_row = adw::ActionRow::builder()
        .title("SMT / Hyperthreading")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&smt_row);
    
    let amd_pstate_row = adw::ActionRow::builder()
        .title("AMD Pstate Mode")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&amd_pstate_row);
    
    let refs = vec![
        ("name".to_string(), name_row),
        ("freq".to_string(), freq_row),
        ("load".to_string(), load_row),
        ("temp".to_string(), temp_row),
        ("power".to_string(), power_row),
        ("scaling_driver".to_string(), scaling_driver_row),
        ("governor".to_string(), governor_row),
        ("epp".to_string(), epp_row),
        ("boost".to_string(), boost_row),
        ("smt".to_string(), smt_row),
        ("amd_pstate".to_string(), amd_pstate_row),
    ];
    
    (group, refs)
}

fn update_cpu_info(refs: &WidgetRefs, dbus_client: Rc<RefCell<Option<DbusClient>>>) {
    if let Some(client) = dbus_client.borrow().as_ref() {
        if let Ok(info) = client.get_cpu_info() {
            update_cpu_info_with_data(refs, info);
        }
    }
}

// Version that takes data directly for background threading
fn update_cpu_info_with_data(refs: &WidgetRefs, info: tuxedo_common::types::CpuInfo) {
    let caps = &info.capabilities;
    
    for (key, row) in refs {
        match key.as_str() {
            "name" => {
                row.set_subtitle(&info.name);
                row.set_visible(true);
            }
            "freq" => {
                row.set_subtitle(&format!("{} MHz", info.median_frequency / 1000));
                row.set_visible(true);
            }
            "load" => {
                row.set_subtitle(&format!("{:.1}%", info.median_load));
                row.set_visible(true);
            }
            "temp" => {
                row.set_subtitle(&format!("{:.1}°C", info.package_temp));
                row.set_visible(true);
            }
            "power" => {
                if let Some(pwr) = info.package_power {
                    if let Some(ref src) = info.power_source {
                        row.set_subtitle(&format!("{:.1} W ({})", pwr, src));
                    } else {
                        row.set_subtitle(&format!("{:.1} W", pwr));
                    }
                    row.set_visible(true);
                } else {
                    row.set_visible(false);
                }
            }
            "scaling_driver" => {
                if caps.has_scaling_driver && info.scaling_driver != "not_available" {
                    row.set_subtitle(&info.scaling_driver);
                    row.set_visible(true);
                } else {
                    row.set_visible(false);
                }
            }
            "governor" => {
                if caps.has_scaling_governor && info.governor != "not_available" {
                    row.set_subtitle(&info.governor);
                    row.set_visible(true);
                } else {
                    row.set_visible(false);
                }
            }
            "epp" => {
                if caps.has_energy_performance_preference {
                    if let Some(ref epp) = info.energy_performance_preference {
                        row.set_subtitle(epp);
                        row.set_visible(true);
                    } else {
                        row.set_visible(false);
                    }
                } else {
                    row.set_visible(false);
                }
            }
            "boost" => {
                if caps.has_boost {
                    row.set_subtitle(if info.boost_enabled { "Enabled" } else { "Disabled" });
                    row.set_visible(true);
                } else {
                    row.set_visible(false);
                }
            }
            "smt" => {
                if caps.has_smt {
                    row.set_subtitle(if info.smt_enabled { "Enabled" } else { "Disabled" });
                    row.set_visible(true);
                } else {
                    row.set_visible(false);
                }
            }
            "amd_pstate" => {
                if caps.has_amd_pstate {
                    if let Some(ref status) = info.amd_pstate_status {
                        row.set_subtitle(&format!("{} mode", status));
                        row.set_visible(true);
                    } else {
                        row.set_visible(false);
                    }
                } else {
                    row.set_visible(false);
                }
            }
            _ => {}
        }
    }
}

fn create_gpu_section() -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("GPU")
        .build();
    
    let mut gpus_found = false;
    
    for i in 128..132 {
        let render_path = format!("/sys/class/drm/renderD{}", i);
        if std::path::Path::new(&render_path).exists() {
            if let Some(gpu_info) = detect_gpu_info(&render_path, i - 128) {
                let gpu_row = adw::ActionRow::builder()
                    .title(&gpu_info.name)
                    .subtitle(&gpu_info.status)
                    .build();
                group.add(&gpu_row);
                gpus_found = true;
            }
        }
    }
    
    if !gpus_found {
        for i in 0..4 {
            let card_path = format!("/sys/class/drm/card{}", i);
            if std::path::Path::new(&card_path).exists() {
                if let Some(gpu_info) = detect_gpu_info(&card_path, i) {
                    let gpu_row = adw::ActionRow::builder()
                        .title(&gpu_info.name)
                        .subtitle(&gpu_info.status)
                        .build();
                    group.add(&gpu_row);
                    gpus_found = true;
                }
            }
        }
    }
    
    if !gpus_found {
        let no_gpu_row = adw::ActionRow::builder()
            .title("No GPU detected")
            .build();
        group.add(&no_gpu_row);
    }
    
    group
}

struct SimpleGpuInfo {
    name: String,
    status: String,
}

fn detect_gpu_info(path: &str, id: u32) -> Option<SimpleGpuInfo> {
    let device_path = format!("{}/device", path);
    
    let vendor_path = format!("{}/vendor", device_path);
    let vendor_id = std::fs::read_to_string(&vendor_path)
        .ok()?
        .trim()
        .to_string();
    
    let vendor_name = match vendor_id.as_str() {
        "0x1002" => "AMD",
        "0x10de" => "NVIDIA",
        "0x8086" => "Intel",
        _ => "Unknown",
    };
    
    let device_name = format!("{} Graphics {}", vendor_name, id);
    
    let gpu_type = if id == 0 {
        "Integrated"
    } else {
        "Discrete"
    };
    
    let power_status_path = format!("{}/power/runtime_status", device_path);
    let power_status = std::fs::read_to_string(&power_status_path)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    let status_text = match power_status.as_str() {
        "active" => format!("{} - Active", gpu_type),
        "suspended" => format!("{} - Suspended", gpu_type),
        _ => format!("{} - {}", gpu_type, power_status),
    };
    
    Some(SimpleGpuInfo {
        name: device_name,
        status: status_text,
    })
}

fn create_battery_section(dbus_client: Rc<RefCell<Option<DbusClient>>>) -> (adw::PreferencesGroup, WidgetRefs) {
    let group = adw::PreferencesGroup::builder()
        .title("Battery")
        .build();
    
    let status_row = adw::ActionRow::builder()
        .title("Status")
        .subtitle("Loading...")
        .build();
    group.add(&status_row);
    
    let capacity_row = adw::ActionRow::builder()
        .title("Capacity")
        .subtitle("Loading...")
        .build();
    group.add(&capacity_row);
    
    let voltage_row = adw::ActionRow::builder()
        .title("Voltage")
        .subtitle("Loading...")
        .build();
    group.add(&voltage_row);
    
    let current_row = adw::ActionRow::builder()
        .title("Current")
        .subtitle("Loading...")
        .build();
    group.add(&current_row);
    
    let power_row = adw::ActionRow::builder()
        .title("Power Draw")
        .subtitle("Loading...")
        .build();
    group.add(&power_row);
    
    // Add battery charge control info
    let charge_type_row = adw::ActionRow::builder()
        .title("Charge Type")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&charge_type_row);
    
    let charge_start_row = adw::ActionRow::builder()
        .title("Charge Start Threshold")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&charge_start_row);
    
    let charge_end_row = adw::ActionRow::builder()
        .title("Charge End Threshold")
        .subtitle("Loading...")
        .visible(false)
        .build();
    group.add(&charge_end_row);
    
    // Load battery charge control info if available
    if let Some(client) = dbus_client.borrow().as_ref() {
        if let Ok(charge_type) = client.get_battery_charge_type() {
            let display_type = match charge_type.as_str() {
                "standard" => "Standard",
                "express" => "Express",
                "primarily_ac" => "Primarily AC",
                _ => &charge_type,
            };
            charge_type_row.set_subtitle(display_type);
            charge_type_row.set_visible(true);
        }
        
        if let Ok(start_threshold) = client.get_battery_charge_start_threshold() {
            charge_start_row.set_subtitle(&format!("{}%", start_threshold));
            charge_start_row.set_visible(true);
        }
        
        if let Ok(end_threshold) = client.get_battery_charge_end_threshold() {
            charge_end_row.set_subtitle(&format!("{}%", end_threshold));
            charge_end_row.set_visible(true);
        }
    }
    
    let refs = vec![
        ("status".to_string(), status_row),
        ("capacity".to_string(), capacity_row),
        ("voltage".to_string(), voltage_row),
        ("current".to_string(), current_row),
        ("power".to_string(), power_row),
        ("charge_type".to_string(), charge_type_row),
        ("charge_start".to_string(), charge_start_row),
        ("charge_end".to_string(), charge_end_row),
    ];
    
    (group, refs)
}

fn update_battery_info(refs: &WidgetRefs, dbus_client: &Rc<RefCell<Option<DbusClient>>>) {
    let battery_path = if std::path::Path::new("/sys/class/power_supply/BAT0").exists() {
        "/sys/class/power_supply/BAT0"
    } else if std::path::Path::new("/sys/class/power_supply/BAT1").exists() {
        "/sys/class/power_supply/BAT1"
    } else {
        for (_, row) in refs {
            row.set_subtitle("No battery detected");
        }
        return;
    };
    
    // Check battery status first
    let status = std::fs::read_to_string(format!("{}/status", battery_path))
        .unwrap_or_else(|_| "Unknown".to_string())
        .trim()
        .to_string();
    
    let on_ac = status == "Charging" || status == "Full" || status == "Not charging";
    
    for (key, row) in refs {
        match key.as_str() {
            "status" => {
                row.set_subtitle(&status);
            }
            "capacity" => {
                if let Ok(capacity) = std::fs::read_to_string(format!("{}/capacity", battery_path)) {
                    row.set_subtitle(&format!("{}%", capacity.trim()));
                }
            }
            "voltage" => {
                if let Ok(voltage) = std::fs::read_to_string(format!("{}/voltage_now", battery_path)) {
                    if let Ok(v) = voltage.trim().parse::<f64>() {
                        row.set_subtitle(&format!("{:.2} V", v / 1_000_000.0));
                    }
                }
            }
            "current" => {
                if let Ok(current) = std::fs::read_to_string(format!("{}/current_now", battery_path)) {
                    if let Ok(c) = current.trim().parse::<f64>() {
                        row.set_subtitle(&format!("{:.2} A", c / 1_000_000.0));
                    }
                }
            }
            "power" => {
                if on_ac {
                    // Don't show power draw when on AC
                    row.set_visible(false);
                } else {
                    row.set_visible(true);
                    if let Ok(power) = std::fs::read_to_string(format!("{}/power_now", battery_path)) {
                        if let Ok(p) = power.trim().parse::<f64>() {
                            row.set_subtitle(&format!("{:.2} W", p / 1_000_000.0));
                        }
                    }
                }
            }
            "charge_type" => {
                if let Some(client) = dbus_client.borrow().as_ref() {
                    if let Ok(charge_type) = client.get_battery_charge_type() {
                        let display_type = match charge_type.as_str() {
                            "standard" => "Standard",
                            "express" => "Express",
                            "primarily_ac" => "Primarily AC",
                            _ => &charge_type,
                        };
                        row.set_subtitle(display_type);
                        row.set_visible(true);
                    } else {
                        row.set_visible(false);
                    }
                } else {
                    row.set_visible(false);
                }
            }
            "charge_start" => {
                if let Some(client) = dbus_client.borrow().as_ref() {
                    if let Ok(threshold) = client.get_battery_charge_start_threshold() {
                        row.set_subtitle(&format!("{}%", threshold));
                        row.set_visible(true);
                    } else {
                        row.set_visible(false);
                    }
                } else {
                    row.set_visible(false);
                }
            }
            "charge_end" => {
                if let Some(client) = dbus_client.borrow().as_ref() {
                    if let Ok(threshold) = client.get_battery_charge_end_threshold() {
                        row.set_subtitle(&format!("{}%", threshold));
                        row.set_visible(true);
                    } else {
                        row.set_visible(false);
                    }
                } else {
                    row.set_visible(false);
                }
            }
            _ => {}
        }
    }
}

fn create_wifi_section() -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("WiFi")
        .build();
    
    let mut wifi_found = false;
    
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if !path.join("wireless").exists() {
                continue;
            }
            
            wifi_found = true;
            
            let iface_name = entry.file_name().to_string_lossy().to_string();
            
            let iface_row = adw::ActionRow::builder()
                .title(&format!("Interface: {}", iface_name))
                .build();
            group.add(&iface_row);
            
            let device_path = path.join("device");
            let uevent_path = device_path.join("uevent");
            if let Ok(uevent) = std::fs::read_to_string(&uevent_path) {
                for line in uevent.lines() {
                    if line.starts_with("DRIVER=") {
                        let driver = line.trim_start_matches("DRIVER=");
                        let driver_row = adw::ActionRow::builder()
                            .title("Driver")
                            .subtitle(driver)
                            .build();
                        group.add(&driver_row);
                    }
                }
            }
            
            let modalias_path = device_path.join("modalias");
            if let Ok(modalias) = std::fs::read_to_string(&modalias_path) {
                if modalias.starts_with("pci:") {
                    if let Some(vendor_device) = extract_pci_ids(&modalias) {
                        let chip_name = get_wifi_chip_name(&vendor_device);
                        let chip_row = adw::ActionRow::builder()
                            .title("Chip")
                            .subtitle(&chip_name)
                            .build();
                        group.add(&chip_row);
                    }
                }
            }
            
            let operstate_path = path.join("operstate");
            if let Ok(state) = std::fs::read_to_string(&operstate_path) {
                let state = state.trim();
                let status_row = adw::ActionRow::builder()
                    .title("Status")
                    .subtitle(state)
                    .build();
                group.add(&status_row);
            }
        }
    }
    
    if !wifi_found {
        let no_wifi_row = adw::ActionRow::builder()
            .title("No WiFi interface detected")
            .build();
        group.add(&no_wifi_row);
    }
    
    group
}

fn extract_pci_ids(modalias: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = modalias.split('d').collect();
    if parts.len() < 2 {
        return None;
    }
    
    let vendor = parts[0].trim_start_matches("pci:v");
    let device = parts[1].split('s').next()?;
    
    Some((vendor.to_string(), device.to_string()))
}

fn get_wifi_chip_name(vendor_device: &(String, String)) -> String {
    let (vendor, device) = vendor_device;
    
    match vendor.as_str() {
        "00008086" => {
            match device.as_str() {
                "00002723" => "Intel Wi-Fi 6 AX200".to_string(),
                "000024FD" => "Intel Wi-Fi 6 AX210".to_string(),
                "00002725" => "Intel Wi-Fi 6E AX211".to_string(),
                "000051F0" => "Intel Wi-Fi 6E AX211".to_string(),
                "00007AF0" => "Intel Wi-Fi 7 BE200".to_string(),
                _ => format!("Intel WiFi ({}:{})", vendor, device),
            }
        }
        "000010EC" => {
            match device.as_str() {
                "0000C821" => "Realtek RTL8821CE".to_string(),
                "0000C822" => "Realtek RTL8822CE".to_string(),
                "0000B822" => "Realtek RTL8822BE".to_string(),
                _ => format!("Realtek WiFi ({}:{})", vendor, device),
            }
        }
        "000014E4" => format!("Broadcom WiFi ({}:{})", vendor, device),
        "0000168C" => format!("Qualcomm Atheros WiFi ({}:{})", vendor, device),
        _ => format!("Unknown WiFi ({}:{})", vendor, device),
    }
}

fn create_storage_section() -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Storage")
        .build();
    
    if let Ok(entries) = std::fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let dev_name = entry.file_name().to_string_lossy().to_string();
            
            if dev_name.starts_with("loop") || dev_name.starts_with("ram") {
                continue;
            }
            
            let path = entry.path();
            if let Ok(model) = std::fs::read_to_string(path.join("device/model")) {
                let model = model.trim();
                if let Ok(size_str) = std::fs::read_to_string(path.join("size")) {
                    if let Ok(size_sectors) = size_str.trim().parse::<u64>() {
                        let size_gb = (size_sectors * 512) / 1_000_000_000;
                        let device_row = adw::ActionRow::builder()
                            .title(&format!("{} - {} GB", model, size_gb))
                            .subtitle(&format!("/dev/{}", dev_name))
                            .build();
                        group.add(&device_row);
                    }
                }
            }
        }
    }
    
    group
}

fn create_fans_section() -> (adw::PreferencesGroup, WidgetRefs) {
    let group = adw::PreferencesGroup::builder()
        .title("Fans")
        .build();
    
    let mut refs = Vec::new();
    for i in 0..4 {
        let fan_row = adw::ActionRow::builder()
            .title(&format!("Fan {}", i))
            .subtitle("Not detected")
            .visible(false)
            .build();
        group.add(&fan_row);
        refs.push((format!("fan{}", i), fan_row));
    }
    
    (group, refs)
}

fn update_fans_info(refs: &WidgetRefs, dbus_client: Rc<RefCell<Option<DbusClient>>>) {
    for (_, row) in refs {
        row.set_visible(false);
    }

    if let Some(client) = dbus_client.borrow().as_ref() {
        // Try new get_fan_info method first (includes temperature)
        match client.get_fan_info() {
            Ok(fans) => {
                for (idx, fan) in fans.iter().enumerate() {
                    if idx < refs.len() {
                        let (_, row) = &refs[idx];

                        if let Some(temp) = fan.temperature {
                            if fan.is_rpm {
                                row.set_subtitle(&format!(
                                    "{} RPM - {}°C",
                                    fan.rpm_or_percent, temp
                                ));
                            } else {
                                row.set_subtitle(&format!(
                                    "{}% - {}°C",
                                    fan.rpm_or_percent, temp
                                ));
                            }
                        } else if fan.is_rpm {
                            row.set_subtitle(&format!("{} RPM", fan.rpm_or_percent));
                        } else {
                            row.set_subtitle(&format!("{}%", fan.rpm_or_percent));
                        }

                        row.set_visible(true);
                    }
                }
            }

            Err(_) => {
                // Fallback to old method
                match client.get_fan_speeds() {
                    Ok(fans) => {
                        for (idx, (fan_id, speed)) in fans.iter().enumerate() {
                            if idx < refs.len() {
                                let (_, row) = &refs[idx];

                                let is_rpm = *speed > 200;

                                match client.get_fan_temperature(*fan_id) {
                                    Ok(temp) => {
                                        if is_rpm {
                                            row.set_subtitle(&format!(
                                                "{} RPM - {}°C",
                                                speed, temp
                                            ));
                                        } else {
                                            row.set_subtitle(&format!(
                                                "{}% - {}°C",
                                                speed, temp
                                            ));
                                        }
                                    }
                                    Err(_) => {
                                        if is_rpm {
                                            row.set_subtitle(&format!("{} RPM", speed));
                                        } else {
                                            row.set_subtitle(&format!("{}%", speed));
                                        }
                                    }
                                }

                                row.set_visible(true);
                            }
                        }
                    }

                    Err(_) => {
                        // Fallback to reading from sysfs
                        if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
                            let mut fan_idx = 0;

                            for entry in entries.flatten() {
                                for i in 1..10 {
                                    let fan_path =
                                        entry.path().join(format!("fan{}_input", i));

                                    if let Ok(speed) = std::fs::read_to_string(&fan_path) {
                                        if let Ok(rpm) = speed.trim().parse::<u32>() {
                                            if rpm > 0 && fan_idx < refs.len() {
                                                let (_, row) = &refs[fan_idx];

                                                let temp_path = entry
                                                    .path()
                                                    .join(format!("temp{}_input", i));

                                                if let Ok(temp_str) =
                                                    std::fs::read_to_string(&temp_path)
                                                {
                                                    if let Ok(temp_millideg) =
                                                        temp_str.trim().parse::<i32>()
                                                    {
                                                        let temp_c =
                                                            temp_millideg as f32 / 1000.0;
                                                        row.set_subtitle(&format!(
                                                            "{} RPM - {:.1}°C",
                                                            rpm, temp_c
                                                        ));
                                                    } else {
                                                        row.set_subtitle(&format!("{} RPM", rpm));
                                                    }
                                                } else {
                                                    row.set_subtitle(&format!("{} RPM", rpm));
                                                }

                                                row.set_visible(true);
                                                fan_idx += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
