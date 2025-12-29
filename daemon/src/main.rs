mod dbus_interface;
mod fan_daemon;
mod hardware_control;
mod hardware_detection;
mod tuxedo_io;
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

    // Start DBus service
    let connection = zbus::Connection::system().await?;
    let _service = dbus_interface::start_service(connection.clone()).await?;

    log::info!("DBus service started");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    log::info!("Shutting down daemon");

    Ok(())
}
