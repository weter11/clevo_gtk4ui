use eframe::egui;
use tuxedo_common::types::{CpuInfo, FanInfo, SystemInfo};

pub fn statistics_page(
    ui: &mut egui::Ui,
    system_info: &Option<SystemInfo>,
    cpu_info: &Option<CpuInfo>,
    fan_info: &[FanInfo],
) {
    ui.heading("System Information");
    if let Some(info) = system_info {
        ui.label(format!("Notebook Model: {}", info.product_name));
        ui.label(format!("Manufacturer: {}", info.manufacturer));
        ui.label(format!("BIOS Version: {}", info.bios_version));
    } else {
        ui.label("Loading system information...");
    }

    ui.separator();

    ui.heading("CPU");
    if let Some(info) = cpu_info {
        ui.label(format!("Processor: {}", info.name));
        ui.label(format!("Median Frequency: {} MHz", info.median_frequency / 1000));
        ui.label(format!("Median Load: {:.1}%", info.median_load));
        ui.label(format!("Package Temperature: {:.1}°C", info.package_temp));
        if let Some(pwr) = info.package_power {
            ui.label(format!("Package Power: {:.1} W", pwr));
        }
    } else {
        ui.label("Loading CPU information...");
    }

    ui.separator();

    ui.heading("Fans");
    for (i, fan) in fan_info.iter().enumerate() {
        let speed = if fan.is_rpm {
            format!("{} RPM", fan.rpm_or_percent)
        } else {
            format!("{}%", fan.rpm_or_percent)
        };
        if let Some(temp) = fan.temperature {
            ui.label(format!("Fan {}: {} - {}°C", i, speed, temp));
        } else {
            ui.label(format!("Fan {}: {}", i, speed));
        }
    }
}
