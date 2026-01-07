use egui::{Ui, ScrollArea, RichText, Slider};
use crate::app::AppState;

pub fn draw(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);
            ui.heading("⚙️ Settings");
            ui.add_space(16.0);
            
            // Appearance
            ui.label(RichText::new("Appearance").strong().heading());
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("Theme:");
                
                use tuxedo_common::types::Theme;
                let mut current_theme = state.config.theme.clone();
                
                if ui.selectable_value(&mut current_theme, Theme::Auto, "Auto").clicked() {
                    state.config.theme = Theme::Auto;
                    state.config_dirty = true;
                }
                if ui.selectable_value(&mut current_theme, Theme::Light, "Light").clicked() {
                    state.config.theme = Theme::Light;
                    state.config_dirty = true;
                }
                if ui.selectable_value(&mut current_theme, Theme::Dark, "Dark").clicked() {
                    state.config.theme = Theme::Dark;
                    state.config_dirty = true;
                }
            });
            
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);
            
            // Startup
            ui.label(RichText::new("Startup").strong().heading());
            ui.add_space(8.0);
            
            if ui.checkbox(&mut state.config.start_minimized, "Start minimized").changed() {
                state.config_dirty = true;
            }
            
            if ui.checkbox(&mut state.config.autostart, "Enable autostart").changed() {
                state.config_dirty = true;
                // TODO: Create/remove autostart file
            }
            
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);
            
            // Daemon Controls
            ui.label(RichText::new("Daemon Controls").strong().heading());
            ui.add_space(8.0);
            
            if ui.checkbox(&mut state.config.fan_daemon_enabled, "Fan daemon").changed() {
                state.config_dirty = true;
            }
            ui.label(RichText::new("Monitor temperatures and apply fan curves").small().italics());
            ui.add_space(6.0);
            
            if ui.checkbox(&mut state.config.app_monitoring_enabled, "App monitoring").changed() {
                state.config_dirty = true;
            }
            ui.label(RichText::new("Enable automatic profile switching").small().italics());
            ui.add_space(6.0);
            
            if ui.checkbox(&mut state.config.auto_profile_switching, "Automatic profile switching").changed() {
                state.config_dirty = true;
            }
            ui.label(RichText::new("Switch profiles based on running applications").small().italics());
            
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);
            
            // Statistics Page Layout
            ui.label(RichText::new("Statistics Page Layout").strong().heading());
            ui.add_space(8.0);
            
            if ui.checkbox(&mut state.config.statistics_sections.show_system_info, "Show system info").changed() {
                state.config_dirty = true;
            }
            if ui.checkbox(&mut state.config.statistics_sections.show_cpu, "Show CPU").changed() {
                state.config_dirty = true;
            }
            if ui.checkbox(&mut state.config.statistics_sections.show_gpu, "Show GPU").changed() {
                state.config_dirty = true;
            }
            if ui.checkbox(&mut state.config.statistics_sections.show_battery, "Show battery").changed() {
                state.config_dirty = true;
            }
            if ui.checkbox(&mut state.config.statistics_sections.show_fans, "Show fans").changed() {
                state.config_dirty = true;
            }
            
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);
            
            // Polling Rates
            ui.label(RichText::new("Polling Rates").strong().heading());
            ui.add_space(8.0);
            ui.label(RichText::new("How often to update each section (in seconds)").small().italics());
            ui.add_space(6.0);
            
            let mut system_poll = (state.config.statistics_sections.system_info_poll_rate / 1000) as f32;
            ui.horizontal(|ui| {
                ui.label("System Info:");
                if ui.add(Slider::new(&mut system_poll, 10.0..=300.0).suffix(" s")).changed() {
                    state.config.statistics_sections.system_info_poll_rate = (system_poll * 1000.0) as u64;
                    state.config_dirty = true;
                }
            });
            
            let mut cpu_poll = (state.config.statistics_sections.cpu_poll_rate / 1000) as f32;
            ui.horizontal(|ui| {
                ui.label("CPU:");
                if ui.add(Slider::new(&mut cpu_poll, 1.0..=10.0).suffix(" s")).changed() {
                    state.config.statistics_sections.cpu_poll_rate = (cpu_poll * 1000.0) as u64;
                    state.config_dirty = true;
                }
            });
            
            let mut gpu_poll = (state.config.statistics_sections.gpu_poll_rate / 1000) as f32;
            ui.horizontal(|ui| {
                ui.label("GPU:");
                if ui.add(Slider::new(&mut gpu_poll, 1.0..=10.0).suffix(" s")).changed() {
                    state.config.statistics_sections.gpu_poll_rate = (gpu_poll * 1000.0) as u64;
                    state.config_dirty = true;
                }
            });
            
            let mut battery_poll = (state.config.statistics_sections.battery_poll_rate / 1000) as f32;
            ui.horizontal(|ui| {
                ui.label("Battery:");
                if ui.add(Slider::new(&mut battery_poll, 1.0..=30.0).suffix(" s")).changed() {
                    state.config.statistics_sections.battery_poll_rate = (battery_poll * 1000.0) as u64;
                    state.config_dirty = true;
                }
            });
            
            let mut fans_poll = (state.config.statistics_sections.fans_poll_rate / 1000) as f32;
            ui.horizontal(|ui| {
                ui.label("Fans:");
                if ui.add(Slider::new(&mut fans_poll, 1.0..=10.0).suffix(" s")).changed() {
                    state.config.statistics_sections.fans_poll_rate = (fans_poll * 1000.0) as u64;
                    state.config_dirty = true;
                }
            });
            
            ui.add_space(16.0);
            
            // Save button
            if state.config_dirty {
                ui.separator();
                ui.add_space(8.0);
                if ui.button("💾 Save Settings").clicked() {
                    let _ = state.save_config();
                }
            }
        });
}
