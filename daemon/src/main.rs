mod dbus_interface;
mod fan_daemon;
mod hardware_control;
mod hardware_detection;
mod tuxedo_io;
mod battery_control;

use anyhow::Result;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting TUXEDO Control Center Daemon");

    // Check if running as root
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("Error: Daemon must run as root");
        std::process::exit(1);
    }

    // Check hardware interface
    if tuxedo_io::TuxedoIo::is_available() {
        match tuxedo_io::TuxedoIo::new() {
            Ok(io) => {
                let interface = match io.get_interface() {
                    tuxedo_io::HardwareInterface::Clevo => "Clevo",
                    tuxedo_io::HardwareInterface::Uniwill => "Uniwill",
                    tuxedo_io::HardwareInterface::None => "None",
                };
                log::info!("Detected hardware interface: {}", interface);
                log::info!("Number of fans: {}", io.get_fan_count());
            }
            Err(e) => {
                log::warn!("Failed to initialize tuxedo_io: {}", e);
            }
        }
    } else {
        log::warn!("/dev/tuxedo_io not available - some features will be disabled");
    }

    // Check battery charge control
    if battery_control::BatteryControl::is_available() {
        log::info!("Battery charge control (flexicharger) is available");
    } else {
        log::info!("Battery charge control not available");
    }

    // Start DBus service
    let connection = zbus::Connection::system().await?;
    let _service = dbus_interface::start_service(connection.clone()).await?;

    log::info!("DBus service started");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    log::info!("Shutting down daemon");

    Ok(())
}
