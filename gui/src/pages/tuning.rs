use egui::{Ui, ScrollArea, RichText, Slider, ComboBox};
use crate::app::AppState;
use crate::dbus_client::DbusClient;
use tuxedo_common::types::{KeyboardMode, Profile, FanCurve};  // ADD FanCurve
use crate::widgets::fan_curve_editor::FanCurveEditor;  // ADD this

pub fn draw(ui: &mut Ui, state: &mut AppState, dbus_client: Option<&DbusClient>) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);
            
            let profile_name = state.config.current_profile.clone();
            let profile_idx = state.config.profiles.iter()
                .position(|p| p.name == profile_name);
            
            if let Some(idx) = profile_idx {
                ui.heading(format!("Editing Profile: {}", profile_name));
                ui.add_space(16.0);
                
                // CPU tuning
                let cpu_info_clone = state.cpu_info.clone();
                if let Some(cpu_info) = &cpu_info_clone {
                    let cpu_caps = Some(&cpu_info.capabilities);
                    draw_cpu_tuning(ui, &mut state.config.profiles[idx], cpu_caps, cpu_info);
                } else {
                    ui.heading("🖥️ CPU Tuning");
                    ui.add_space(8.0);
                    ui.label("CPU information not available");
                }
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);
                
                // Keyboard tuning
                draw_keyboard_tuning(ui, &mut state.config.profiles[idx], dbus_client);
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);
                
                // Screen tuning
                draw_screen_tuning(ui, &mut state.config.profiles[idx]);
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);
                
                // Fan tuning
                let fan_count = state.fan_info.len().max(2);
                draw_fan_tuning(ui, &mut state.config.profiles[idx], fan_count);
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                // Battery tuning
                let start_thresholds = state.available_start_thresholds.clone();
                let end_thresholds = state.available_end_thresholds.clone();
                draw_battery_tuning(
                    ui,
                    &mut state.config.profiles[idx],
                    &start_thresholds,
                    &end_thresholds,
                );
                ui.add_space(16.0);
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("💾 Save & Apply Profile").clicked() {
                        state.config_dirty = false;
                        let _ = state.save_config();
                        
                        if let Some(client) = dbus_client {
                            let profile_clone = state.config.profiles[idx].clone();
                            let rx = client.apply_profile(profile_clone);
                            // Result handled in background
                        }
                    }
                    
                    if ui.button("↺ Reset to Saved").clicked() {
                        state.load_config();
                        state.show_message("Profile reset to saved state", false);
                    }
                });
                
                state.config_dirty = true;
            } else {
                ui.label("No profile selected");
            }
        });
}

fn draw_cpu_tuning(
    ui: &mut Ui,
    profile: &mut Profile,
    cpu_caps: Option<&tuxedo_common::types::CpuCapabilities>,
    cpu_info: &tuxedo_common::types::CpuInfo,
) {
    ui.heading("🖥️ CPU Tuning");
    ui.add_space(8.0);
    
    let caps = match cpu_caps {
        Some(c) => c,
        None => {
            ui.label("CPU information not available");
            return;
        }
    };
    
    // Governor
    if caps.has_scaling_governor && !cpu_info.available_governors.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Governor:");
            
            let mut current_gov = profile.cpu_settings.governor
                .clone()
                .unwrap_or_else(|| "auto".to_string());
            
            ComboBox::from_id_source("governor_combo")
                .selected_text(&current_gov)
                .show_ui(ui, |ui| {
                    for gov in &cpu_info.available_governors {
                        ui.selectable_value(&mut current_gov, gov.clone(), gov);
                    }
                });
            
            profile.cpu_settings.governor = Some(current_gov);
        });
        ui.add_space(6.0);
    }
    
    // EPP
    if caps.has_energy_performance_preference && !cpu_info.available_epp_options.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Energy Performance:");
            
            let mut current_epp = profile.cpu_settings.energy_performance_preference
                .clone()
                .unwrap_or_else(|| "balance_performance".to_string());
            
            ComboBox::from_id_source("epp_combo")
                .selected_text(&current_epp)
                .show_ui(ui, |ui| {
                    for epp in &cpu_info.available_epp_options {
                        ui.selectable_value(&mut current_epp, epp.clone(), epp);
                    }
                });
            
            profile.cpu_settings.energy_performance_preference = Some(current_epp);
        });
        ui.add_space(6.0);
    }
    
    // Frequency sliders
    if caps.has_scaling_min_freq && caps.has_scaling_max_freq {
        ui.label(RichText::new("Frequency Limits:").strong());
        
        let mut min_freq = profile.cpu_settings.min_frequency
            .unwrap_or(cpu_info.hw_min_freq) as f64 / 1000.0;
        let mut max_freq = profile.cpu_settings.max_frequency
            .unwrap_or(cpu_info.hw_max_freq) as f64 / 1000.0;
        
        ui.horizontal(|ui| {
            ui.label("Min:");
            ui.add(Slider::new(&mut min_freq, 
                (cpu_info.hw_min_freq / 1000) as f64..=(cpu_info.hw_max_freq / 1000) as f64)
                .suffix(" MHz"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Max:");
            ui.add(Slider::new(&mut max_freq,
                (cpu_info.hw_min_freq / 1000) as f64..=(cpu_info.hw_max_freq / 1000) as f64)
                .suffix(" MHz"));
        });
        
        profile.cpu_settings.min_frequency = Some((min_freq * 1000.0) as u64);
        profile.cpu_settings.max_frequency = Some((max_freq * 1000.0) as u64);
        
        ui.add_space(6.0);
    }
    
    // Checkboxes
    if caps.has_boost {
        let mut boost = profile.cpu_settings.boost.unwrap_or(true);
        ui.checkbox(&mut boost, "CPU Boost / Turbo");
        profile.cpu_settings.boost = Some(boost);
    }
    
    if caps.has_smt {
        let mut smt = profile.cpu_settings.smt.unwrap_or(true);
        ui.checkbox(&mut smt, "SMT / Hyperthreading");
        profile.cpu_settings.smt = Some(smt);
    }
}

fn draw_keyboard_tuning(
    ui: &mut Ui,
    profile: &mut Profile,
    dbus_client: Option<&DbusClient>,
) {
    ui.heading("⌨️ Keyboard Backlight");
    ui.add_space(8.0);
    
    ui.checkbox(&mut profile.keyboard_settings.control_enabled, "Control keyboard backlight");
    ui.add_space(6.0);
    
    if profile.keyboard_settings.control_enabled {
        // Mode selector
        ui.horizontal(|ui| {
            ui.label("Mode:");
            
            let current_mode_name = match &profile.keyboard_settings.mode {
                KeyboardMode::SingleColor { .. } => "Single Color",
                KeyboardMode::Breathe { .. } => "Breathe",
                KeyboardMode::Cycle { .. } => "Cycle",
                KeyboardMode::Wave { .. } => "Wave",
                _ => "Other",
            };
            
            ComboBox::from_id_source("keyboard_mode")
                .selected_text(current_mode_name)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(current_mode_name == "Single Color", "Single Color").clicked() {
                        profile.keyboard_settings.mode = KeyboardMode::SingleColor { r: 255, g: 255, b: 255, brightness: 50 };
                    }
                    if ui.selectable_label(current_mode_name == "Breathe", "Breathe").clicked() {
                        profile.keyboard_settings.mode = KeyboardMode::Breathe { r: 255, g: 255, b: 255, brightness: 50, speed: 50 };
                    }
                    if ui.selectable_label(current_mode_name == "Cycle", "Cycle").clicked() {
                        profile.keyboard_settings.mode = KeyboardMode::Cycle { brightness: 50, speed: 50 };
                    }
                    if ui.selectable_label(current_mode_name == "Wave", "Wave").clicked() {
                        profile.keyboard_settings.mode = KeyboardMode::Wave { brightness: 50, speed: 50 };
                    }
                });
        });
        ui.add_space(6.0);
        
        // Mode-specific controls
        match &mut profile.keyboard_settings.mode {
            KeyboardMode::SingleColor { r, g, b, brightness } => {
                ui.horizontal(|ui| {
                    ui.label("Red:");
                    ui.add(Slider::new(r, 0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Green:");
                    ui.add(Slider::new(g, 0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Blue:");
                    ui.add(Slider::new(b, 0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Brightness:");
                    ui.add(Slider::new(brightness, 0..=100).suffix("%"));
                });
                
                // Color preview
                let color = egui::Color32::from_rgb(*r, *g, *b);
                ui.horizontal(|ui| {
                    ui.label("Preview:");
                    ui.colored_label(color, "■■■■■");
                });
            }
            _ => {}
        }
        
        // Preview button
        if ui.button("👁️ Preview").clicked() {
            if let Some(client) = dbus_client {
                let _ = client.preview_keyboard_settings(profile.keyboard_settings.clone());
            }
        }
    }
}

fn draw_battery_tuning(
    ui: &mut Ui,
    profile: &mut Profile,
    available_start_thresholds: &[u8],
    available_end_thresholds: &[u8],
) {
    ui.heading("🔋 Battery Charge Control");
    ui.add_space(8.0);

    ui.checkbox(&mut profile.battery_settings.control_enabled, "Enable charge thresholds");
    ui.add_space(6.0);

    if profile.battery_settings.control_enabled {
        // Start Threshold
        ui.horizontal(|ui| {
            ui.label("Start Threshold:");
            ComboBox::from_id_source("start_threshold_combo")
                .selected_text(format!("{}%", profile.battery_settings.charge_start_threshold))
                .show_ui(ui, |ui| {
                    for &threshold in available_start_thresholds {
                        ui.selectable_value(
                            &mut profile.battery_settings.charge_start_threshold,
                            threshold,
                            format!("{}%", threshold),
                        );
                    }
                });
        });

        // End Threshold
        ui.horizontal(|ui| {
            ui.label("End Threshold:");
            ComboBox::from_id_source("end_threshold_combo")
                .selected_text(format!("{}%", profile.battery_settings.charge_end_threshold))
                .show_ui(ui, |ui| {
                    for &threshold in available_end_thresholds {
                        ui.selectable_value(
                            &mut profile.battery_settings.charge_end_threshold,
                            threshold,
                            format!("{}%", threshold),
                        );
                    }
                });
        });

        if profile.battery_settings.charge_start_threshold >= profile.battery_settings.charge_end_threshold {
            if let Some(valid_start) = available_start_thresholds.iter()
                .filter(|&&t| t < profile.battery_settings.charge_end_threshold)
                .last()
            {
                profile.battery_settings.charge_start_threshold = *valid_start;
            }
        }
    }
}

fn draw_screen_tuning(ui: &mut Ui, profile: &mut Profile) {
    ui.heading("🖥️ Screen");
    ui.add_space(8.0);
    
    ui.checkbox(&mut profile.screen_settings.system_control, "Use system brightness control");
    ui.add_space(6.0);
    
    if !profile.screen_settings.system_control {
        ui.horizontal(|ui| {
            ui.label("Brightness:");
            ui.add(Slider::new(&mut profile.screen_settings.brightness, 0..=100).suffix("%"));
        });
    }
}

fn draw_fan_tuning(ui: &mut Ui, profile: &mut Profile, fan_count: usize) {
    ui.heading("💨 Fan Control");
    ui.add_space(8.0);
    
    ui.checkbox(&mut profile.fan_settings.control_enabled, "Enable custom fan curves");
    ui.add_space(6.0);
    
    if profile.fan_settings.control_enabled {
        // Ensure curves exist
        while profile.fan_settings.curves.len() < fan_count {
            let fan_id = profile.fan_settings.curves.len() as u32;
            profile.fan_settings.curves.push(FanCurve {
                fan_id,
                points: vec![(0, 0), (50, 50), (70, 75), (85, 100)],
            });
        }
        
        // Show editor for each fan
        for curve in profile.fan_settings.curves.iter_mut() {
            if (curve.fan_id as usize) < fan_count {
                ui.separator();
                ui.add_space(8.0);
                
                egui::CollapsingHeader::new(format!("Fan {} Configuration", curve.fan_id))
                    .default_open(curve.fan_id == 0)
                    .show(ui, |ui| {
                        let mut editor = FanCurveEditor::new(curve.fan_id, curve.clone());
                        editor.show(ui);
                        *curve = editor.get_curve();
                    });
            }
        }
    }
}
