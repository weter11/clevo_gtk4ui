mod application;
mod config;
mod dbus_client;
mod ui;

use gtk::glib;

const APP_ID: &str = "com.tuxedo.ControlCenter";

fn main() -> glib::ExitCode {
    // Initialize logging
    env_logger::init();
    
    // Initialize GTK
    gtk::init().expect("Failed to initialize GTK");
    
    // Create application
    let app = application::TuxedoApplication::new(APP_ID);
    
    // Run
    app.run()
}