use egui::{Ui, ScrollArea, CollapsingHeader, Grid, ProgressBar, RichText};
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
            
            if state.config.statistics_sections.show_fans {
                draw_fan_info(ui, state);
                ui.add_space(12.0);
            }
        });
}

fn draw_system_info(ui: &mut Ui, state: &AppState) {
    CollapsingHeader::new(RichText::new("📊 System Information").heading())
        .default_open(false)
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
                        // Processor name
                        ui.label("Processor:");
                        ui.label(&cpu.name);
                        ui.end_row();
                        
                        // Frequency with real-time update
                        ui.label("Median Frequency:");
                        ui.label(RichText::new(format!("{} MHz", cpu.median_frequency / 1000))
                            .monospace());
                        ui.end_row();
                        
                        // Load with progress bar and color coding
                        ui.label("Median Load:");
                        ui.horizontal(|ui| {
                            ui.add(
                                ProgressBar::new(cpu.median_load / 100.0)
                                    .text(format!("{:.1}%", cpu.median_load))
                                    .fill(load_color(cpu.median_load))
                            );
                        });
                        ui.end_row();
                        
                        // Temperature with color coding
                        ui.label("Package Temperature:");
                        ui.colored_label(
                            temp_color(cpu.package_temp),
                            RichText::new(format!("{:.1}°C", cpu.package_temp))
                                .strong()
                                .monospace()
                        );
                        ui.end_row();
                        
                        // Power draw if available
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
                        
                        // Additional power sources if available
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
                        
                        // CPU Settings
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
                
                // Per-core details (collapsible)
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
        .default_open(false)
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
                        // Capacity with large progress bar
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
                        
                        // Power draw (calculated)
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
                        
                        // Charge thresholds if available
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
