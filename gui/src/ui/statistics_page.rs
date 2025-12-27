use gtk::prelude::*;
use gtk::{Box, Orientation, ScrolledWindow, Switch};
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
        .vexpand(true)
        .hexpand(true)
        .build();
    
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(24);
    main_box.set_margin_start(24);
    main_box.set_margin_end(24);
    
// System Info Section
    if config.borrow().data.statistics_sections.show_system_info {
        // Add the println statement here
        println!("create_system_info_section: DBus client available: {}", dbus_client.borrow().is_some());
        
        let system_info = create_system_info_section(dbus_client.clone());
        main_box.append(&system_info);
    }
    
    // CPU Section
    if config.borrow().data.statistics_sections.show_cpu {
        let cpu_section = create_cpu_section(dbus_client.clone());
        main_box.append(&cpu_section);
    }
    
    scrolled.set_child(Some(&main_box));
    scrolled
}

fn create_system_info_section(dbus_client: Rc<RefCell<Option<DbusClient>>>) -> adw::PreferencesGroup {
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
    
    group.add(&model_row);
    group.add(&manufacturer_row);
    
    // Load data - must stay on main thread with GTK widgets
    if let Some(client) = dbus_client.borrow().as_ref() {
        match client.get_system_info() {
            Ok(info) => {
                model_row.set_subtitle(&info.product_name);
                manufacturer_row.set_subtitle(&info.manufacturer);
            }
            Err(e) => {
                eprintln!("Failed to get system info: {}", e);
                model_row.set_subtitle("Error loading");
                manufacturer_row.set_subtitle("Error loading");
            }
        }
    }
    
    group
}

fn create_cpu_section(dbus_client: Rc<RefCell<Option<DbusClient>>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("CPU")
        .build();
    
    // CPU Name
    let name_row = adw::ActionRow::builder()
        .title("Processor")
        .subtitle("Loading...")
        .build();
    group.add(&name_row);
    
    // Median Frequency
    let freq_row = adw::ActionRow::builder()
        .title("Median Frequency")
        .subtitle("Loading...")
        .build();
    group.add(&freq_row);
    
    // Package Temperature
    let temp_row = adw::ActionRow::builder()
        .title("Package Temperature")
        .subtitle("Loading...")
        .build();
    group.add(&temp_row);
    
    // Package Power
    let power_row = adw::ActionRow::builder()
        .title("Package Power")
        .subtitle("Loading...")
        .build();
    group.add(&power_row);
    
    // Power Sources
    let power_sources_expander = adw::ExpanderRow::builder()
        .title("Power Sources")
        .subtitle("Available power monitoring sources")
        .build();
    group.add(&power_sources_expander);
    
    // CPU Governor
    let governor_row = adw::ComboRow::builder()
        .title("CPU Governor")
        .build();
    let governor_model = gtk::StringList::new(&["Loading..."]);
    governor_row.set_model(Some(&governor_model));
    group.add(&governor_row);
    
    // AMD pstate
    let pstate_row = adw::ComboRow::builder()
        .title("AMD pstate Status")
        .build();
    let pstate_model = gtk::StringList::new(&["passive", "active", "guided"]);
    pstate_row.set_model(Some(&pstate_model));
    pstate_row.set_visible(false);
    group.add(&pstate_row);
    
    // Boost Toggle
    let boost_row = adw::ActionRow::builder()
        .title("CPU Boost")
        .subtitle("Turbo / Precision Boost")
        .build();
    let boost_switch = Switch::new();
    boost_switch.set_valign(gtk::Align::Center);
    boost_row.add_suffix(&boost_switch);
    boost_row.set_activatable_widget(Some(&boost_switch));
    group.add(&boost_row);
    
    // SMT Toggle
    let smt_row = adw::ActionRow::builder()
        .title("SMT / Hyperthreading")
        .subtitle("Simultaneous Multithreading")
        .build();
    let smt_switch = Switch::new();
    smt_switch.set_valign(gtk::Align::Center);
    smt_row.add_suffix(&smt_switch);
    smt_row.set_activatable_widget(Some(&smt_switch));
    group.add(&smt_row);
    
    // Per-core details
    let expander = adw::ExpanderRow::builder()
        .title("Show per-core details")
        .build();
    group.add(&expander);
    
    // Load CPU info - blocking call is fine here, it's fast (< 50ms)
    if let Some(client) = dbus_client.borrow().as_ref() {
        match client.get_cpu_info() {
            Ok(info) => {
                name_row.set_subtitle(&info.name);
                freq_row.set_subtitle(&format!("{} MHz", info.median_frequency / 1000));
                temp_row.set_subtitle(&format!("{:.1}°C", info.package_temp));
                
                // Power
                if let Some(pwr) = info.package_power {
                    if let Some(ref src) = info.power_source {
                        power_row.set_subtitle(&format!("{:.1} W ({})", pwr, src));
                    } else {
                        power_row.set_subtitle(&format!("{:.1} W", pwr));
                    }
                }
                
                // Power sources
                for source in &info.all_power_sources {
                    let source_row = adw::ActionRow::builder()
                        .title(&source.name)
                        .subtitle(&format!("{:.1} W - {}", source.value, source.description))
                        .build();
                    power_sources_expander.add_row(&source_row);
                }
                
                if info.all_power_sources.is_empty() {
                    power_sources_expander.set_visible(false);
                }
                
                // Governors
                let gov_strs: Vec<&str> = info.available_governors.iter().map(|s| s.as_str()).collect();
                let new_model = gtk::StringList::new(&gov_strs);
                governor_row.set_model(Some(&new_model));
                
                if let Some(idx) = info.available_governors.iter().position(|g| g == &info.governor) {
                    governor_row.set_selected(idx as u32);
                }
                
                // AMD pstate
                if let Some(ref status) = info.amd_pstate_status {
                    pstate_row.set_visible(true);
                    let statuses = vec!["passive", "active", "guided"];
                    let status_str = status.as_str();
                    if let Some(idx) = statuses.iter().position(|s| *s == status_str) {
                        pstate_row.set_selected(idx as u32);
                    }
                }
                
                // Switches
                boost_switch.set_active(info.boost_enabled);
                smt_switch.set_active(info.smt_enabled);
                
                // Core details
                for core in &info.cores {
                    let core_row = adw::ActionRow::builder()
                        .title(&format!("Core {}", core.id))
                        .subtitle(&format!("{} MHz, {:.1}°C", core.frequency / 1000, core.temperature))
                        .build();
                    expander.add_row(&core_row);
                }
            }
            Err(e) => {
                eprintln!("Failed to get CPU info: {}", e);
                name_row.set_subtitle("Error loading");
                freq_row.set_subtitle("Error");
                temp_row.set_subtitle("Error");
                power_row.set_subtitle("Error");
            }
        }
    }
    
    group
}
