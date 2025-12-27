use gtk::prelude::*;
use gtk::{Box, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use gtk::glib;

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
    
    let sections = &config.borrow().data.statistics_sections;
    
    // System Info
    if sections.show_system_info {
        let system_info = create_system_info_section(dbus_client.clone());
        main_box.append(&system_info);
    }
    
    // CPU
    if sections.show_cpu {
        let cpu_section = create_cpu_section(dbus_client.clone());
        main_box.append(&cpu_section);
        setup_cpu_realtime_updates(cpu_section.clone(), dbus_client.clone());
    }
    
    // GPU
    if sections.show_gpu {
        let gpu_section = create_gpu_section(dbus_client.clone());
        main_box.append(&gpu_section);
        setup_gpu_realtime_updates(gpu_section.clone(), dbus_client.clone());
    }
    
    // Battery
    if sections.show_battery {
        let battery_section = create_battery_section(dbus_client.clone());
        main_box.append(&battery_section);
        setup_battery_realtime_updates(battery_section.clone(), dbus_client.clone());
    }
    
    // WiFi
    if sections.show_wifi {
        let wifi_section = create_wifi_section(dbus_client.clone());
        main_box.append(&wifi_section);
        setup_wifi_realtime_updates(wifi_section.clone(), dbus_client.clone());
    }
    
    // Fans
    if sections.show_fans {
        let fans_section = create_fans_section(dbus_client.clone());
        main_box.append(&fans_section);
        setup_fans_realtime_updates(fans_section.clone(), dbus_client.clone());
    }
    
    scrolled.set_child(Some(&main_box));
    scrolled
}

// ... (previous system_info and cpu sections remain same) ...

fn create_gpu_section(dbus_client: Rc<RefCell<Option<DbusClient>>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("GPU Information")
        .build();
    
    if let Some(client) = dbus_client.borrow().as_ref() {
        match client.get_gpu_info() {
            Ok(gpus) => {
                for gpu in gpus {
                    let status_text = match gpu.status {
                        tuxedo_common::types::GpuStatus::Active => "🟢 Active",
                        tuxedo_common::types::GpuStatus::Suspended => "🔴 Suspended",
                        tuxedo_common::types::GpuStatus::Unknown => "⚪ Unknown",
                    };
                    
                    let gpu_row = adw::ActionRow::builder()
                        .title(&gpu.name)
                        .subtitle(&format!("Status: {} | Driver: {}", 
                            status_text,
                            gpu.driver.as_ref().unwrap_or(&"Unknown".to_string())))
                        .build();
                    
                    if let Some(ref pci_id) = gpu.pci_id {
                        let detail = gtk::Label::new(Some(&format!("PCI: {}", pci_id)));
                        detail.set_css_classes(&["dim-label", "caption"]);
                        gpu_row.add_suffix(&detail);
                    }
                    
                    group.add(&gpu_row);
                }
            }
            Err(e) => {
                let error_row = adw::ActionRow::builder()
                    .title("Error loading GPU info")
                    .subtitle(&e.to_string())
                    .build();
                group.add(&error_row);
            }
        }
    }
    
    group
}

fn create_battery_section(dbus_client: Rc<RefCell<Option<DbusClient>>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Battery")
        .build();
    
    // Charge level
    let charge_row = adw::ActionRow::builder()
        .title("Charge Level")
        .subtitle("Loading...")
        .build();
    group.add(&charge_row);
    
    // Status
    let status_row = adw::ActionRow::builder()
        .title("Status")
        .subtitle("Loading...")
        .build();
    group.add(&status_row);
    
    // Power draw
    let power_row = adw::ActionRow::builder()
        .title("Power Draw")
        .subtitle("Loading...")
        .build();
    group.add(&power_row);
    
    // Capacity
    let capacity_row = adw::ActionRow::builder()
        .title("Capacity")
        .subtitle("Loading...")
        .build();
    group.add(&capacity_row);
    
    // Load initial data
    update_battery_info(&charge_row, &status_row, &power_row, &capacity_row, dbus_client);
    
    group
}

fn update_battery_info(
    charge_row: &adw::ActionRow,
    status_row: &adw::ActionRow,
    power_row: &adw::ActionRow,
    capacity_row: &adw::ActionRow,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    if let Some(client) = dbus_client.borrow().as_ref() {
        match client.get_battery_info() {
            Ok(info) => {
                charge_row.set_subtitle(&format!("{}%", info.charge_percent));
                
                let status_text = match info.status {
                    tuxedo_common::types::BatteryStatus::Charging => "⚡ Charging",
                    tuxedo_common::types::BatteryStatus::Discharging => "🔋 Discharging",
                    tuxedo_common::types::BatteryStatus::Full => "✓ Full",
                    tuxedo_common::types::BatteryStatus::NotCharging => "⏸ Not Charging",
                    tuxedo_common::types::BatteryStatus::Unknown => "❓ Unknown",
                };
                status_row.set_subtitle(status_text);
                
                // Power draw: N/A on AC, actual value on battery
                if info.on_ac_power {
                    power_row.set_subtitle("N/A (on AC power)");
                } else if let Some(draw) = info.power_draw_w {
                    power_row.set_subtitle(&format!("{:.1} W", draw));
                } else {
                    power_row.set_subtitle("Unknown");
                }
                
                capacity_row.set_subtitle(&format!("{} mAh", info.capacity_mah));
            }
            Err(e) => {
                charge_row.set_subtitle(&format!("Error: {}", e));
            }
        }
    }
}

fn create_wifi_section(dbus_client: Rc<RefCell<Option<DbusClient>>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("WiFi")
        .build();
    
    if let Some(client) = dbus_client.borrow().as_ref() {
        match client.get_wifi_info() {
            Ok(interfaces) => {
                if interfaces.is_empty() {
                    let no_wifi = adw::ActionRow::builder()
                        .title("No WiFi interfaces found")
                        .build();
                    group.add(&no_wifi);
                } else {
                    for iface in interfaces {
                        let expander = adw::ExpanderRow::builder()
                            .title(&iface.interface)
                            .subtitle(&format!("Driver: {}", iface.driver))
                            .build();
                        
                        // Chip model
                        let chip_row = adw::ActionRow::builder()
                            .title("Chip Model")
                            .subtitle(&iface.chip_model)
                            .build();
                        expander.add_row(&chip_row);
                        
                        // Link speed
                        if let Some(speed) = iface.link_speed_mbps {
                            let speed_row = adw::ActionRow::builder()
                                .title("Link Speed")
                                .subtitle(&format!("{} Mbps", speed))
                                .build();
                            expander.add_row(&speed_row);
                        }
                        
                        // Signal strength
                        if let Some(signal) = iface.signal_strength {
                            let signal_row = adw::ActionRow::builder()
                                .title("Signal Strength")
                                .subtitle(&format!("{} dBm", signal))
                                .build();
                            expander.add_row(&signal_row);
                        }
                        
                        // SSID
                        if let Some(ref ssid) = iface.ssid {
                            let ssid_row = adw::ActionRow::builder()
                                .title("Connected to")
                                .subtitle(ssid)
                                .build();
                            expander.add_row(&ssid_row);
                        }
                        
                        // Temperature
                        if let Some(temp) = iface.temperature {
                            let temp_row = adw::ActionRow::builder()
                                .title("Temperature")
                                .subtitle(&format!("{:.1}°C", temp))
                                .build();
                            expander.add_row(&temp_row);
                        }
                        
                        group.add(&expander);
                    }
                }
            }
            Err(e) => {
                let error_row = adw::ActionRow::builder()
                    .title("Error loading WiFi info")
                    .subtitle(&e.to_string())
                    .build();
                group.add(&error_row);
            }
        }
    }
    
    group
}

fn create_fans_section(dbus_client: Rc<RefCell<Option<DbusClient>>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Fans")
        .build();
    
    if let Some(client) = dbus_client.borrow().as_ref() {
        match client.get_fan_info() {
            Ok(fans) => {
                if fans.is_empty() {
                    let no_fans = adw::ActionRow::builder()
                        .title("No fan information available")
                        .subtitle("Fan control may not be supported on this hardware")
                        .build();
                    group.add(&no_fans);
                } else {
                    // Show number of detected fans
                    let count_row = adw::ActionRow::builder()
                        .title("Detected Fans")
                        .subtitle(&format!("{} fan(s)", fans.len()))
                        .build();
                    group.add(&count_row);
                    
                    // Show each fan
                    for fan in fans {
                        let fan_row = adw::ActionRow::builder()
                            .title(&fan.name)
                            .subtitle(&format!(
                                "Speed: {}% | RPM: {}",
                                fan.duty_percent,
                                fan.rpm.map(|r| r.to_string()).unwrap_or_else(|| "N/A".to_string())
                            ))
                            .build();
                        
                        if let Some(temp) = fan.temperature {
                            let temp_label = gtk::Label::new(Some(&format!("{:.1}°C", temp)));
                            temp_label.set_css_classes(&["dim-label"]);
                            fan_row.add_suffix(&temp_label);
                        }
                        
                        group.add(&fan_row);
                    }
                }
            }
            Err(e) => {
                let error_row = adw::ActionRow::builder()
                    .title("Error loading fan info")
                    .subtitle(&e.to_string())
                    .build();
                group.add(&error_row);
            }
        }
    }
    
    group
}

// Real-time update functions

fn setup_gpu_realtime_updates(
    group: adw::PreferencesGroup,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    glib::timeout_add_seconds_local(3, move || {
        // Update GPU status
        if let Some(client) = dbus_client.borrow().as_ref() {
            if let Ok(_gpus) = client.get_gpu_info() {
                // Refresh GPU info
                // (would need to update individual rows)
            }
        }
        glib::ControlFlow::Continue
    });
}

fn setup_battery_realtime_updates(
    group: adw::PreferencesGroup,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    // Find rows
    let charge_row = find_row_by_title(&group, "Charge Level");
    let status_row = find_row_by_title(&group, "Status");
    let power_row = find_row_by_title(&group, "Power Draw");
    let capacity_row = find_row_by_title(&group, "Capacity");
    
    if let (Some(cr), Some(sr), Some(pr), Some(cap_r)) = 
           (charge_row, status_row, power_row, capacity_row) {
        
        glib::timeout_add_seconds_local(2, move || {
            update_battery_info(&cr, &sr, &pr, &cap_r, dbus_client.clone());
            glib::ControlFlow::Continue
        });
    }
}

fn setup_wifi_realtime_updates(
    _group: adw::PreferencesGroup,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    glib::timeout_add_seconds_local(5, move || {
        // Update WiFi info (signal strength, link speed)
        if let Some(client) = dbus_client.borrow().as_ref() {
            let _ = client.get_wifi_info();
            // Would need to update individual rows
        }
        glib::ControlFlow::Continue
    });
}

fn setup_fans_realtime_updates(
    _group: adw::PreferencesGroup,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
) {
    glib::timeout_add_seconds_local(2, move || {
        // Update fan speeds and RPM
        if let Some(client) = dbus_client.borrow().as_ref() {
            let _ = client.get_fan_info();
            // Would need to update individual rows
        }
        glib::ControlFlow::Continue
    });
}

fn find_row_by_title(group: &adw::PreferencesGroup, title: &str) -> Option<adw::ActionRow> {
    let mut child = group.first_child();
    while let Some(widget) = child {
        if let Ok(row) = widget.downcast::<adw::ActionRow>() {
            if row.title() == title {
                return Some(row);
            }
            child = row.next_sibling();
        } else {
            child = widget.next_sibling();
        }
    }
    None
}

fn find_expander_by_title(group: &adw::PreferencesGroup, title: &str) -> Option<adw::ExpanderRow> {
    let mut child = group.first_child();
    while let Some(widget) = child {
        if let Ok(expander) = widget.downcast::<adw::ExpanderRow>() {
            if expander.title() == title {
                return Some(expander);
            }
            child = expander.next_sibling();
        } else {
            child = widget.next_sibling();
        }
    }
    None
}
