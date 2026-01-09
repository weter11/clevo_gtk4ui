use egui::{Ui, ScrollArea, RichText, Frame};
use crate::app::{AppState, Page};
use crate::dbus_client::DbusClient;

pub fn draw(ui: &mut Ui, state: &mut AppState, dbus_client: Option<&DbusClient>) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(8.0);
            
            // Current profile indicator
            ui.heading(format!("Current Profile: {}", state.config.current_profile));
            ui.add_space(12.0);
            
            // Profile list
            let mut profiles_to_remove = Vec::new();
            let mut profile_to_activate = None;
            
            for (idx, profile) in state.config.profiles.iter().enumerate() {
                let is_current = profile.name == state.config.current_profile;
                
                // Frame with highlight for current profile
                let frame = if is_current {
                    Frame::none()
                        .fill(ui.style().visuals.selection.bg_fill.gamma_multiply(0.3))
                        .stroke(ui.style().visuals.selection.stroke)
                        .rounding(6.0)
                        .inner_margin(12.0)
                } else {
                    Frame::none()
                        .fill(ui.style().visuals.faint_bg_color)
                        .rounding(6.0)
                        .inner_margin(12.0)
                };
                
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Radio button style indicator
                        ui.label(if is_current { "●" } else { "○" });
                        
                        // Profile name
                        let name_text = if profile.is_default {
                            RichText::new(&profile.name).strong()
                        } else {
                            RichText::new(&profile.name)
                        };
                        
                        if ui.selectable_label(is_current, name_text).clicked() && !is_current {
                            profile_to_activate = Some(idx);
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Delete button (only for non-default profiles)
                            if !profile.is_default {
                                if ui.button("🗑️ Delete").clicked() {
                                    profiles_to_remove.push(idx);
                                }
                            }
                            
                            // Edit button
                            if ui.button("✏️ Edit").clicked() {
                                state.config.current_profile = profile.name.clone();
                                state.current_page = Page::Tuning;
                            }
                            
                            // Apply button
                            if ui.button("▶️ Apply").clicked() {
                                profile_to_activate = Some(idx);
                            }
                        });
                    });
                    
                    // Profile details
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        // CPU settings summary
                        if let Some(ref gov) = profile.cpu_settings.governor {
                            ui.label(RichText::new(format!("Governor: {}", gov)).small());
                            ui.label(RichText::new("|").small());
                        }
                        
                        if let Some(boost) = profile.cpu_settings.boost {
                            ui.label(RichText::new(format!("Boost: {}", if boost { "On" } else { "Off" })).small());
                            ui.label(RichText::new("|").small());
                        }
                        
                        // Keyboard settings
                        if profile.keyboard_settings.control_enabled {
                            ui.label(RichText::new("Keyboard: Manual").small());
                        } else {
                            ui.label(RichText::new("Keyboard: Auto").small());
                        }
                        
                        ui.label(RichText::new("|").small());
                        
                        // Fan settings
                        if profile.fan_settings.control_enabled {
                            ui.label(RichText::new(format!("Fans: Custom ({})", profile.fan_settings.curves.len())).small());
                        } else {
                            ui.label(RichText::new("Fans: Auto").small());
                        }
                    });
                });
                
                ui.add_space(8.0);
            }
            
            // Handle profile activation
            if let Some(idx) = profile_to_activate {
                state.config.current_profile = state.config.profiles[idx].name.clone();
                state.config_dirty = true;
                
                // Apply to hardware
                if let Some(client) = dbus_client {
                    let profile_clone = state.config.profiles[idx].clone();
                    let _rx = client.apply_profile(profile_clone);
                    state.show_message(format!("Applying profile '{}'", state.config.profiles[idx].name), false);
                }
            }
            
            // Handle profile deletion
            for idx in profiles_to_remove.into_iter().rev() {
                let name = state.config.profiles[idx].name.clone();
                state.config.profiles.remove(idx);
                state.config_dirty = true;
                state.show_message(format!("Profile '{}' deleted", name), false);
            }
            
            // Add new profile section
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("Create New Profile:").strong());
                
                let _text_edit_id = ui.make_persistent_id("new_profile_name");
                let mut new_name = state.editing_profile_name.clone().unwrap_or_default();
                
                ui.text_edit_singleline(&mut new_name);
                state.editing_profile_name = Some(new_name.clone());
                
                if ui.button("➕ Create").clicked() && !new_name.is_empty() {
                    if state.config.profiles.iter().any(|p| p.name == new_name) {
                        state.show_message(format!("Profile '{}' already exists", new_name), true);
                    } else {
                        // Create new profile based on default
                        let default_profile = state.config.profiles.iter()
                            .find(|p| p.is_default)
                            .cloned()
                            .unwrap_or_default();
                        
                        let mut new_profile = default_profile;
                        new_profile.name = new_name.clone();
                        new_profile.is_default = false;
                        
                        state.config.profiles.push(new_profile);
                        state.config.current_profile = new_name.clone();
                        state.config_dirty = true;
                        state.editing_profile_name = None;
                        state.show_message(format!("Profile '{}' created", new_name), false);
                    }
                }
            });
        });
}
