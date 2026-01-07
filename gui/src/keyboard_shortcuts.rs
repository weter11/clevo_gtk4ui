use egui::{Context, Key, Modifiers};
use crate::app::{AppState, Page};

pub struct KeyboardShortcuts {
    show_help: bool,
}

impl KeyboardShortcuts {
    pub fn new() -> Self {
        Self { show_help: false }
    }
    
    pub fn handle_shortcuts(&mut self, ctx: &Context, state: &mut AppState) -> bool {
        let mut handled = false;
        
        ctx.input(|i| {
            // Ctrl+S - Save config
            if i.modifiers.command && i.key_pressed(Key::S) {
                let _ = state.save_config();
                handled = true;
            }
            
            // Ctrl+1-4 - Switch pages
            if i.modifiers.command {
                if i.key_pressed(Key::Num1) {
                    state.current_page = Page::Statistics;
                    handled = true;
                }
                if i.key_pressed(Key::Num2) {
                    state.current_page = Page::Profiles;
                    handled = true;
                }
                if i.key_pressed(Key::Num3) {
                    state.current_page = Page::Tuning;
                    handled = true;
                }
                if i.key_pressed(Key::Num4) {
                    state.current_page = Page::Settings;
                    handled = true;
                }
            }
            
            // Ctrl+N - New profile
            if i.modifiers.command && i.key_pressed(Key::N) {
                if state.current_page == Page::Profiles {
                    state.editing_profile_name = Some("New Profile".to_string());
                    handled = true;
                }
            }
            
            // Ctrl+R - Reload config
            if i.modifiers.command && i.key_pressed(Key::R) {
                state.load_config();
                state.show_message("Configuration reloaded", false);
                handled = true;
            }
            
            // F1 or ? - Show help
            if i.key_pressed(Key::F1) || 
               (i.modifiers.shift && i.key_pressed(Key::Questionmark)) {
                self.show_help = !self.show_help;
                handled = true;
            }
            
            // Escape - Close help
            if i.key_pressed(Key::Escape) && self.show_help {
                self.show_help = false;
                handled = true;
            }
            
            // Ctrl+Q - Quit
            if i.modifiers.command && i.key_pressed(Key::Q) {
                std::process::exit(0);
            }
            
            // Profile shortcuts (1-9 without modifiers on Profiles page)
            if state.current_page == Page::Profiles && !i.modifiers.any() {
                for (idx, key) in [
                    Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5,
                    Key::Num6, Key::Num7, Key::Num8, Key::Num9
                ].iter().enumerate() {
                    if i.key_pressed(*key) && idx < state.config.profiles.len() {
                        state.config.current_profile = state.config.profiles[idx].name.clone();
                        handled = true;
                    }
                }
            }
        });
        
        // Show help window if requested
        if self.show_help {
            self.draw_help_window(ctx);
        }
        
        handled
    }
    
    fn draw_help_window(&mut self, ctx: &Context) {
        egui::Window::new("⌨️ Keyboard Shortcuts")
            .open(&mut self.show_help)
            .default_width(400.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("Global Shortcuts");
                ui.add_space(8.0);
                
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        self.shortcut_row(ui, "Ctrl+S", "Save configuration");
                        self.shortcut_row(ui, "Ctrl+R", "Reload configuration");
                        self.shortcut_row(ui, "Ctrl+Q", "Quit application");
                        self.shortcut_row(ui, "F1 or ?", "Show this help");
                        self.shortcut_row(ui, "Escape", "Close help");
                        
                        ui.end_row();
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        
                        ui.label(egui::RichText::new("Navigation").strong());
                        ui.label("");
                        ui.end_row();
                        
                        self.shortcut_row(ui, "Ctrl+1", "Statistics page");
                        self.shortcut_row(ui, "Ctrl+2", "Profiles page");
                        self.shortcut_row(ui, "Ctrl+3", "Tuning page");
                        self.shortcut_row(ui, "Ctrl+4", "Settings page");
                        
                        ui.end_row();
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        
                        ui.label(egui::RichText::new("Profiles Page").strong());
                        ui.label("");
                        ui.end_row();
                        
                        self.shortcut_row(ui, "1-9", "Switch to profile 1-9");
                        self.shortcut_row(ui, "Ctrl+N", "Create new profile");
                    });
                
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("💡 Tip:").strong());
                    ui.label("Use Tab to navigate between fields");
                });
            });
    }
    
    fn shortcut_row(&self, ui: &mut egui::Ui, keys: &str, description: &str) {
        ui.label(egui::RichText::new(keys).monospace().strong());
        ui.label(description);
        ui.end_row();
    }
}
