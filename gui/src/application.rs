use gtk::prelude::*;
use gtk::{glib, Application};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::dbus_client::DbusClient;
use crate::ui::window::TuxedoWindow;

pub struct TuxedoApplication {
    app: Application,
    config: Rc<RefCell<Config>>,
    dbus_client: Rc<RefCell<Option<DbusClient>>>,
}

impl TuxedoApplication {
    pub fn new(app_id: &str) -> Self {
        let app = Application::builder()
            .application_id(app_id)
            .build();
        
        let config = Rc::new(RefCell::new(Config::load().unwrap_or_default()));
        let dbus_client = Rc::new(RefCell::new(None));
        
        let instance = Self {
            app: app.clone(),
            config: config.clone(),
            dbus_client: dbus_client.clone(),
        };
        
        // Setup application
        let config_for_startup = config.clone();
        app.connect_startup(move |_app| {
            // Initialize libadwaita
            adw::init().expect("Failed to initialize libadwaita");
            
            // Load CSS
            Self::load_css();
            
            // Apply theme
            Self::apply_theme(&config_for_startup.borrow().data.theme);
        });
        
        let config_clone = config.clone();
        let dbus_clone = dbus_client.clone();
        app.connect_activate(move |app| {
            // Initialize DBus client BEFORE creating window
            if dbus_clone.borrow().is_none() {
                println!("Connecting to DBus...");
                match DbusClient::new() {
                    Ok(client) => {
                        *dbus_clone.borrow_mut() = Some(client);
                        println!("✅ DBus client connected successfully");
                        log::info!("DBus client connected successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to connect to daemon: {}", e);
                        
                        // Show error using Adwaita MessageDialog
                        let dialog = adw::MessageDialog::builder()
                            .heading("Failed to Connect")
                            .body(&format!("Could not connect to TUXEDO daemon:\n\n{}\n\nPlease ensure the package is installed correctly.", e))
                            .build();
                        dialog.add_response("ok", "OK");
                        dialog.set_default_response(Some("ok"));
                        dialog.set_close_response("ok");
                        dialog.present();
                        return; // Don't create window if connection failed
                    }
                }
            } else {
                println!("DBus client already connected");
            }
            
            // Verify client is set
            if dbus_clone.borrow().is_some() {
                println!("Creating window with connected DBus client...");
            } else {
                println!("⚠️ WARNING: Creating window without DBus client!");
            }
            
            // Create window
            let window = TuxedoWindow::new(app, config_clone.clone(), dbus_clone.clone());
            window.present();
        });
        
        instance
    }
    
    pub fn run(&self) -> glib::ExitCode {
        self.app.run()
    }
    
    fn load_css() {
        let provider = gtk::CssProvider::new();
        let css = "
/* TUXEDO Control Center Custom Styles */
.section-header {
    font-weight: bold;
    font-size: 1.2em;
    margin-top: 12px;
    margin-bottom: 6px;
}
        ";
        provider.load_from_string(css);
        
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    
    fn apply_theme(theme: &tuxedo_common::types::Theme) {
        use tuxedo_common::types::Theme;
        
        let style_manager = adw::StyleManager::default();
        match theme {
            Theme::Auto => style_manager.set_color_scheme(adw::ColorScheme::Default),
            Theme::Light => style_manager.set_color_scheme(adw::ColorScheme::ForceLight),
            Theme::Dark => style_manager.set_color_scheme(adw::ColorScheme::ForceDark),
        }
    }
}
