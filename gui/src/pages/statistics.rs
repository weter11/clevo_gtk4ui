use egui::{Ui, ScrollArea, CollapsingHeader, Grid, ProgressBar, RichText};
use egui::Color32;
use crate::app::AppState;
use crate::theme::{temp_color, load_color, power_color};

pub fn draw(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);
            
            if state.config.statistics_sections.show_system_info {
                draw_system_info(ui, state);
                ui.add_space(12.0);
            }
            
            if state.config.statistics_sections.show_cpu {
                draw_cpu_info(ui, state);
                ui.add_space(12.0);
            }
            
            if state.config.statistics_sections.show_gpu {
                draw_gpu_info(ui, state);
                ui.add_space(12.0);
            }
            
            if state.config.statistics_sections.show_battery {
                draw_battery_info(ui, state);
                ui.add_space(12.0);
            }

            if state.config.statistics_sections.show_wifi {
                draw_wifi_info(ui, state);
                ui.add_space(12.0);
            }

            if state.config.statistics_sections.show_storage {
                draw_storage_info(ui, state);
                ui.add_space(12.0);
            }
            
            if state.config.statistics_sections.show_fans {
                draw_fan_info(ui, state);
                ui.add_space(12.0);
            }
        });
}

fn draw_system_info(ui: &mut Ui, state: &AppState) {
    CollapsingHeader::new(RichText::new("📊 System Information").heading())
        .default_open(true)  // Changed to true
        .show(ui, |ui| {
            if let Some(ref info) = state.system_info {
                Grid::new("system_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Notebook Model:");
                        ui.label(&info.product_name);
                        ui.end_row();
                        
                        ui.label("Manufacturer:");
                        ui.label(&info.manufacturer);
                        ui.end_row();
                        
                        ui.label("BIOS Version:");
                        ui.label(&info.bios_version);
                        ui.end_row();
                    });
            } else {
                ui.spinner();
                ui.label("Loading system information...");
            }
        });
}

fn draw_cpu_info(ui: &mut Ui, state: &AppState) {
    CollapsingHeader::new(RichText::new("🖥️ CPU").heading())
        .default_open(true)
        .show(ui, |ui| {
            if let Some(ref cpu) = state.cpu_info {
                Grid::new("cpu_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Processor:");
                        ui.label(&cpu.name);
                        ui.end_row();
                        
                        ui.label("Median Frequency:");
                        ui.label(RichText::new(format!("{} MHz", cpu.median_frequency / 1000))
                            .monospace());
                        ui.end_row();
                        
                        ui.label("Median Load:");
                        ui.horizontal(|ui| {
                            ui.add(
                                ProgressBar::new(cpu.median_load / 100.0)
                                    .text(format!("{:.1}%", cpu.median_load))
                                    .fill(load_color(cpu.median_load))
                            );
                        });
                        ui.end_row();
                        
                        ui.label("Package Temperature:");
                        ui.colored_label(
                            temp_color(cpu.package_temp),
                            RichText::new(format!("{:.1}°C", cpu.package_temp))
                                .strong()
                                .monospace()
                        );
                        ui.end_row();
                        
                        if let Some(power) = cpu.package_power {
                            ui.label("Package Power:");
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    power_color(power),
                                    RichText::new(format!("{:.1} W", power))
                                        .strong()
                                        .monospace()
                                );
                                
                                if let Some(ref source) = cpu.power_source {
                                    ui.label(RichText::new(format!("({})", source))
                                        .small()
                                        .italics());
                                }
                            });
                            ui.end_row();
                        }
                        
                        if !cpu.all_power_sources.is_empty() && cpu.all_power_sources.len() > 1 {
                            ui.label("All Power Sources:");
                            ui.vertical(|ui| {
                                for source in &cpu.all_power_sources {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&source.name).small());
                                        ui.label(RichText::new(format!("{:.1} W", source.value))
                                            .small()
                                            .monospace());
                                    });
                                }
                            });
                            ui.end_row();
                        }
                        
                        ui.label("");
                        ui.separator();
                        ui.end_row();
                        
                        if cpu.capabilities.has_scaling_driver {
                            ui.label("Scaling Driver:");
                            ui.label(&cpu.scaling_driver);
                            ui.end_row();
                        }
                        
                        if cpu.capabilities.has_scaling_governor {
                            ui.label("Governor:");
                            ui.label(RichText::new(&cpu.governor).monospace());
                            ui.end_row();
                        }
                        
                        if cpu.capabilities.has_energy_performance_preference {
                            if let Some(ref epp) = cpu.energy_performance_preference {
                                ui.label("EPP:");
                                ui.label(epp);
                                ui.end_row();
                            }
                        }
                        
                        if cpu.capabilities.has_boost {
                            ui.label("CPU Boost:");
                            ui.label(if cpu.boost_enabled { "✅ Enabled" } else { "❌ Disabled" });
                            ui.end_row();
                        }
                        
                        if cpu.capabilities.has_smt {
                            ui.label("SMT / Hyperthreading:");
                            ui.label(if cpu.smt_enabled { "✅ Enabled" } else { "❌ Disabled" });
                            ui.end_row();
                        }
                        
                        if cpu.capabilities.has_amd_pstate {
                            if let Some(ref status) = cpu.amd_pstate_status {
                                ui.label("AMD P-State:");
                                ui.label(format!("{} mode", status));
                                ui.end_row();
                            }
                        }
                    });
                
                // Per-core details (still collapsed by default)
                ui.add_space(8.0);
                CollapsingHeader::new(format!("Core Details ({} cores)", cpu.cores.len()))
                    .default_open(false)
                    .show(ui, |ui| {
                        Grid::new("cores_grid")
                            .num_columns(4)
                            .spacing([20.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Core").strong());
                                ui.label(RichText::new("Frequency").strong());
                                ui.label(RichText::new("Load").strong());
                                ui.label(RichText::new("Temp").strong());
                                ui.end_row();
                                
                                for core in &cpu.cores {
                                    ui.label(format!("CPU {}", core.id));
                                    ui.label(RichText::new(format!("{} MHz", core.frequency / 1000))
                                        .monospace());
                                    ui.add(
                                        ProgressBar::new(core.load / 100.0)
                                            .text(format!("{:.0}%", core.load))
                                            .desired_width(80.0)
                                    );
                                    ui.colored_label(
                                        temp_color(core.temperature),
                                        format!("{:.0}°C", core.temperature)
                                    );
                                    ui.end_row();
                                }
                            });
                    });
            } else {
                ui.spinner();
                ui.label("Loading CPU information...");
            }
        });
}

fn draw_gpu_info(ui: &mut Ui, state: &AppState) {
    CollapsingHeader::new(RichText::new("🎮 GPU").heading())
        .default_open(true)  // Changed to true
        .show(ui, |ui| {
            if !state.gpu_info.is_empty() {
                for (idx, gpu) in state.gpu_info.iter().enumerate() {
                    if idx > 0 {
                        ui.separator();
                        ui.add_space(6.0);
                    }
                    
                    ui.label(RichText::new(&gpu.name).strong());
                    Grid::new(format!("gpu_grid_{}", idx))
                        .num_columns(2)
                        .spacing([40.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Type:");
                            ui.label(if gpu.gpu_type == tuxedo_common::types::GpuType::Integrated {
                                "Integrated"
                            } else {
                                "Discrete"
                            });
                            ui.end_row();
                            
                            ui.label("Status:");
                            ui.label(&gpu.status);
                            ui.end_row();
                            
                            if let Some(freq) = gpu.frequency {
                                ui.label("Frequency:");
                                ui.label(format!("{} MHz", freq));
                                ui.end_row();
                            }
                            
                            if let Some(temp) = gpu.temperature {
                                ui.label("Temperature:");
                                ui.colored_label(
                                    temp_color(temp),
                                    format!("{:.1}°C", temp)
                                );
                                ui.end_row();
                            }
                            
                            if let Some(load) = gpu.load {
                                ui.label("Load:");
                                ui.add(ProgressBar::new(load / 100.0)
                                    .text(format!("{:.1}%", load)));
                                ui.end_row();
                            }
                            
                            if let Some(power) = gpu.power {
                                ui.label("Power:");
                                ui.colored_label(
                                    power_color(power),
                                    format!("{:.1} W", power)
                                );
                                ui.end_row();
                            }
                        });
                }
            } else {
                ui.label("No GPU detected");
            }
        });
}

fn draw_battery_info(ui: &mut Ui, state: &AppState) {
    CollapsingHeader::new(RichText::new("🔋 Battery").heading())
        .default_open(true)
        .show(ui, |ui| {
            if let Some(ref battery) = state.battery_info {
                Grid::new("battery_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Capacity:");
                        ui.horizontal(|ui| {
                            ui.add(
                                ProgressBar::new(battery.charge_percent as f32 / 100.0)
                                    .text(format!("{}%", battery.charge_percent))
                                    .desired_width(200.0)
                            );
                        });
                        ui.end_row();
                        
                        ui.label("Voltage:");
                        ui.label(format!("{:.2} V", battery.voltage_mv as f64 / 1000.0));
                        ui.end_row();
                        
                        ui.label("Current:");
                        let current_a = battery.current_ma as f64 / 1000.0;
                        ui.label(format!("{:.2} A", current_a.abs()));
                        ui.end_row();
                        
                        let power_w = (battery.voltage_mv as f64 * battery.current_ma as f64) / 1_000_000.0;
                        if power_w.abs() > 0.1 {
                            ui.label("Power:");
                            ui.colored_label(
                                power_color(power_w.abs() as f32),
                                format!("{:.1} W {}", 
                                    power_w.abs(),
                                    if power_w > 0.0 { "(charging)" } else { "(discharging)" }
                                )
                            );
                            ui.end_row();
                        }
                        
                        ui.label("Manufacturer:");
                        ui.label(&battery.manufacturer);
                        ui.end_row();
                        
                        ui.label("Model:");
                        ui.label(&battery.model);
                        ui.end_row();
                        
                        if let Some(start) = battery.charge_start_threshold {
                            ui.label("Charge Start:");
                            ui.label(format!("{}%", start));
                            ui.end_row();
                        }
                        
                        if let Some(end) = battery.charge_end_threshold {
                            ui.label("Charge End:");
                            ui.label(format!("{}%", end));
                            ui.end_row();
                        }
                    });
            } else {
                ui.label("No battery detected");
            }
        });
}

fn draw_wifi_info(ui: &mut Ui, state: &AppState) {
    CollapsingHeader::new(RichText::new("📶 WiFi").heading())
        .default_open(true)  // Changed to true
        .show(ui, |ui| {
            if !state.wifi_info.is_empty() {
                for wifi in &state.wifi_info {
                    ui.label(RichText::new(format!("Interface: {}", wifi.interface)).strong());
                    
                    Grid::new(format!("wifi_grid_{}", wifi.interface))
                        .num_columns(2)
                        .spacing([40.0, 6.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Driver:");
                            ui.label(&wifi.driver);
                            ui.end_row();
                            
                            if let Some(signal) = wifi.signal_level {
                                ui.label("Signal Level:");
                                ui.horizontal(|ui| {
                                    let signal_percent = ((signal + 90) as f32 / 60.0).clamp(0.0, 1.0);
                                    
                                    let color = if signal_percent > 0.7 {
                                        Color32::from_rgb(100, 200, 120)
                                    } else if signal_percent > 0.4 {
                                        Color32::from_rgb(255, 200, 60)
                                    } else {
                                        Color32::from_rgb(255, 100, 80)
                                    };
                                    
                                    ui.add(
                                        ProgressBar::new(signal_percent)
                                            .text(format!("{} dBm", signal))
                                            .fill(color)
                                            .desired_width(150.0)
                                    );
                                });
                                ui.end_row();
                            }
                            
                            if let Some(channel) = wifi.channel {
                                ui.label("Channel:");
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}", channel));
                                    
                                    if let Some(width) = wifi.channel_width {
                                        ui.label(RichText::new(format!("({} MHz)", width))
                                            .small()
                                            .italics());
                                    }
                                });
                                ui.end_row();
                            }
                            
                            if let Some(tx_rate) = wifi.tx_rate {
                                ui.label("TX Rate:");
                                ui.label(RichText::new(format!("{:.1} Mbps", tx_rate))
                                    .monospace());
                                ui.end_row();
                            }
                            
                            if let Some(rx_rate) = wifi.rx_rate {
                                ui.label("RX Rate:");
                                ui.label(RichText::new(format!("{:.1} Mbps", rx_rate))
                                    .monospace());
                                ui.end_row();
                            }
                            
                            if let Some(temp) = wifi.temperature {
                                ui.label("Temperature:");
                                ui.colored_label(
                                    temp_color(temp),
                                    format!("{:.1}°C", temp)
                                );
                                ui.end_row();
                            }
                        });
                    
                    ui.add_space(8.0);
                }
            } else {
                ui.label("No WiFi interface detected");
            }
        });
}

fn draw_storage_info(ui: &mut Ui, state: &AppState) {
    CollapsingHeader::new(RichText::new("💾 Storage").heading())
        .default_open(true)  // Changed to true
        .show(ui, |ui| {
            if !state.storage_info.is_empty() {
                Grid::new("storage_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for storage in &state.storage_info {
                            ui.label(RichText::new(&storage.model).strong());
                            ui.label(format!("{} GB total", storage.size_gb));
                            ui.end_row();
                            
                            ui.label("Device:");
                            ui.label(RichText::new(&storage.device).monospace());
                            ui.end_row();
                            
                            // Try to get free space using statvfs
                            if let Ok(free_gb) = get_free_space(&storage.device) {
                                let used_gb = storage.size_gb.saturating_sub(free_gb);
                                let used_percent = if storage.size_gb > 0 {
                                    (used_gb as f32 / storage.size_gb as f32) * 100.0
                                } else {
                                    0.0
                                };
                                
                                ui.label("Usage:");
                                ui.horizontal(|ui| {
                                    ui.add(
                                        ProgressBar::new(used_percent / 100.0)
                                            .text(format!("{} GB / {} GB ({:.1}%)", 
                                                used_gb, storage.size_gb, used_percent))
                                            .desired_width(200.0)
                                    );
                                });
                                ui.end_row();
                                
                                ui.label("Free:");
                                ui.label(format!("{} GB", free_gb));
                                ui.end_row();
                            }
                            
                            if let Some(temp) = storage.temperature {
                                ui.label("Temperature:");
                                ui.colored_label(
                                    temp_color(temp),
                                    format!("{:.1}°C", temp)
                                );
                                ui.end_row();
                            }
                            
                            ui.label("");
                            ui.separator();
                            ui.end_row();
                        }
                    });
            } else {
                ui.label("No storage devices detected");
            }
        });
}

fn draw_fan_info(ui: &mut Ui, state: &AppState) {
    CollapsingHeader::new(RichText::new("💨 Fans").heading())
        .default_open(true)
        .show(ui, |ui| {
            if !state.fan_info.is_empty() {
                Grid::new("fans_grid")
                    .num_columns(3)
                    .spacing([40.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Fan").strong());
                        ui.label(RichText::new("Speed").strong());
                        ui.label(RichText::new("Temperature").strong());
                        ui.end_row();
                        
                        for fan in &state.fan_info {
                            ui.label(&fan.name);
                            
                            ui.horizontal(|ui| {
                                let speed_pct = if fan.is_rpm {
                                    (fan.rpm_or_percent as f32 / 5000.0).min(1.0)
                                } else {
                                    fan.rpm_or_percent as f32 / 100.0
                                };
                                
                                ui.add(
                                    ProgressBar::new(speed_pct)
                                        .text(if fan.is_rpm {
                                            format!("{} RPM", fan.rpm_or_percent)
                                        } else {
                                            format!("{}%", fan.rpm_or_percent)
                                        })
                                        .desired_width(120.0)
                                );
                            });
                            
                            if let Some(temp) = fan.temperature {
                                ui.colored_label(
                                    temp_color(temp),
                                    format!("{:.1}°C", temp)
                                );
                            } else {
                                ui.label("—");
                            }
                            
                            ui.end_row();
                        }
                    });
            } else {
                ui.label("No fan information available");
            }
        });
}

fn get_free_space(device: &str) -> Result<u64, std::io::Error> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    
    // Try to find mount point for this device
    let output = std::process::Command::new("findmnt")
        .args(&["-n", "-o", "TARGET", "--source", device])
        .output()?;
    
    if !output.status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not mounted"));
    }
    
    let mount_point = String::from_utf8_lossy(&output.stdout);
    let mount_point = mount_point.trim();
    
    if mount_point.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No mount point"));
    }
    
    // Use statvfs to get space info
    let path = CString::new(mount_point)?;
    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
    
    unsafe {
        if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let stat = stat.assume_init();
            let free_bytes = stat.f_bavail * stat.f_bsize;
            Ok(free_bytes / 1_000_000_000)
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}
