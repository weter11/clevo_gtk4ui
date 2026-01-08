use egui::{Context, CentralPanel, TopBottomPanel};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tuxedo_common::types::*;
  // ADD THIS

use crate::dbus_client::DbusClient;
use crate::theme::TuxedoTheme;
use crate::pages::{statistics, profiles, tuning, settings};
use crate::keyboard_shortcuts::KeyboardShortcuts;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    Statistics,
    Profiles,
    Tuning,
    Settings,
}

pub struct AppState {
    // Core data
    pub config: AppConfig,
    
    // Hardware info (updated in background)
    pub system_info: Option<SystemInfo>,
    pub cpu_info: Option<CpuInfo>,
    pub gpu_info: Vec<GpuInfo>,
    pub battery_info: Option<BatteryInfo>,
    pub wifi_info: Vec<WiFiInfo>,
    pub fan_info: Vec<FanInfo>,
    pub storage_info: Vec<StorageInfo>,
    pub available_start_thresholds: Vec<u8>,
    pub available_end_thresholds: Vec<u8>,
    
    // UI state
    pub current_page: Page,
    pub config_dirty: bool,
    pub status_message: Option<StatusMessage>,
    
    // Polling control
    pub last_poll: PollTimers,
    
    // Profile editing
    pub editing_profile_name: Option<String>,
    
    // Async state
    pub pending_battery_update: Option<oneshot::Receiver<Result<(), anyhow::Error>>>,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
    pub shown_at: Instant,
}

pub struct PollTimers {
    pub system_info: Instant,
    pub cpu: Instant,
    pub gpu: Instant,
    pub battery: Instant,
    pub wifi: Instant,
    pub fans: Instant,
    pub storage: Instant,
}

impl Default for PollTimers {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            system_info: now,
            cpu: now,
            gpu: now,
            battery: now,
            wifi: now,
            fans: now,
            storage: now,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
            system_info: None,
            cpu_info: None,
            gpu_info: Vec::new(),
            battery_info: None,
            wifi_info: Vec::new(),
            fan_info: Vec::new(),
            storage_info: Vec::new(),
            available_start_thresholds: Vec::new(),
            available_end_thresholds: Vec::new(),
            current_page: Page::Statistics,
            config_dirty: false,
            status_message: None,
            last_poll: PollTimers::default(),
            editing_profile_name: None,
            pending_battery_update: None,
        }
    }
    
    pub fn load_config(&mut self) {
        if let Ok(config) = load_config_from_disk() {
            self.config = config;
        }
    }
    
    pub fn save_config(&mut self) -> anyhow::Result<()> {
        save_config_to_disk(&self.config)?;
        self.config_dirty = false;
        self.show_message("Configuration saved", false);
        Ok(())
    }
    
    pub fn show_message(&mut self, text: impl Into<String>, is_error: bool) {
        self.status_message = Some(StatusMessage {
            text: text.into(),
            is_error,
            shown_at: Instant::now(),
        });
    }
    
    pub fn current_profile(&self) -> Option<&Profile> {
        self.config.profiles.iter()
            .find(|p| p.name == self.config.current_profile)
    }
    
    pub fn current_profile_mut(&mut self) -> Option<&mut Profile> {
        let current = self.config.current_profile.clone();
        self.config.profiles.iter_mut()
            .find(|p| p.name == current)
    }
}

pub struct TuxedoApp {
    state: AppState,
    dbus_client: Option<DbusClient>,
    theme: TuxedoTheme,
    
    // Background update channel
    hw_update_rx: mpsc::UnboundedReceiver<HardwareUpdate>,
    
    // Keyboard shortcuts
    shortcuts: KeyboardShortcuts,
}

#[derive(Debug)]
pub enum HardwareUpdate {
    SystemInfo(SystemInfo),
    CpuInfo(CpuInfo),
    GpuInfo(Vec<GpuInfo>),
    BatteryInfo(BatteryInfo),
    WifiInfo(Vec<WiFiInfo>),
    FanInfo(Vec<FanInfo>),
    StorageInfo(Vec<StorageInfo>),
    AvailableThresholds(Vec<u8>, Vec<u8>),
    Error(String),
}

impl TuxedoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::new();
        state.load_config();
        
        // Create DBus client
        let dbus_client = match DbusClient::new() {
            Ok(client) => {
                log::info!("✅ Connected to TUXEDO daemon");
                Some(client)
            }
            Err(e) => {
                log::error!("❌ Failed to connect to daemon: {}", e);
                state.show_message(
                    format!("Failed to connect to daemon: {}", e),
                    true
                );
                None
            }
        };
        
        // Setup background polling
        let (hw_update_tx, hw_update_rx) = mpsc::unbounded_channel();
        if let Some(ref client) = dbus_client {
            start_background_polling(client.clone(), hw_update_tx.clone());

            // Fetch available thresholds
            let client_clone = client.clone();
            tokio::spawn(async move {
                let start_rx = client_clone.get_battery_available_start_thresholds();
                let end_rx = client_clone.get_battery_available_end_thresholds();

                match (start_rx.await, end_rx.await) {
                    (Ok(Ok(start)), Ok(Ok(end))) => {
                        let _ = hw_update_tx.send(HardwareUpdate::AvailableThresholds(start, end));
                    }
                    (Ok(Err(e)), _) => log::warn!("Failed to get start thresholds: {}", e),
                    (Err(e), _) => log::warn!("Failed to get start thresholds: {}", e),
                    (_, Ok(Err(e))) => log::warn!("Failed to get end thresholds: {}", e),
                    (_, Err(e)) => log::warn!("Failed to get end thresholds: {}", e),
                }
            });
        }
        
        // Apply theme
        let theme = TuxedoTheme::new(&state.config.theme);
        theme.apply(&cc.egui_ctx);
        
        Self {
            state,
            dbus_client,
            theme,
            hw_update_rx,
            shortcuts: KeyboardShortcuts::new(),
        }
    }
    
    fn handle_hardware_updates(&mut self) {
        // Process all pending updates (non-blocking)
        while let Ok(update) = self.hw_update_rx.try_recv() {
            match update {
                HardwareUpdate::SystemInfo(info) => {
                    self.state.system_info = Some(info);
                }
                HardwareUpdate::CpuInfo(info) => {
                    self.state.cpu_info = Some(info);
                }
                HardwareUpdate::GpuInfo(info) => {
                    self.state.gpu_info = info;
                }
                HardwareUpdate::BatteryInfo(info) => {
                    self.state.battery_info = Some(info);
                }
                HardwareUpdate::WifiInfo(info) => {
                    self.state.wifi_info = info;
                }
                HardwareUpdate::FanInfo(info) => {
                    self.state.fan_info = info;
                }
                HardwareUpdate::StorageInfo(info) => {
                    self.state.storage_info = info;
                }
                HardwareUpdate::AvailableThresholds(start, end) => {
                    self.state.available_start_thresholds = start;
                    self.state.available_end_thresholds = end;
                }
                HardwareUpdate::Error(err) => {
                    log::error!("Hardware update error: {}", err);
                }
            }
        }
        
        // Check pending battery update
        if let Some(mut rx) = self.state.pending_battery_update.take() {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    // Success
                }
                Ok(Err(e)) => {
                    self.state.show_message(format!("Battery update failed: {}", e), true);
                }
                Err(oneshot::error::TryRecvError::Empty) => {
                    // Still pending, put it back
                    self.state.pending_battery_update = Some(rx);
                }
                Err(oneshot::error::TryRecvError::Closed) => {
                    self.state.show_message("Battery update channel closed", true);
                }
            }
        }
    }
    
    fn draw_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                
                // Navigation tabs
                ui.selectable_value(&mut self.state.current_page, Page::Statistics, "📊 Statistics");
                ui.selectable_value(&mut self.state.current_page, Page::Profiles, "📋 Profiles");
                ui.selectable_value(&mut self.state.current_page, Page::Tuning, "🔧 Tuning");
                ui.selectable_value(&mut self.state.current_page, Page::Settings, "⚙️ Settings");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Save button (only show if config is dirty)
                    if self.state.config_dirty {
                        if ui.button("💾 Save").clicked() {
                            let _ = self.state.save_config();
                        }
                    }
                    
                    // Current profile indicator
                    ui.label(format!("Profile: {}", self.state.config.current_profile));
                });
            });
            ui.add_space(8.0);
        });
        
        // Status message bar (if any)
        if let Some(ref msg) = self.state.status_message.clone() {
            if msg.shown_at.elapsed() < Duration::from_secs(5) {
                TopBottomPanel::top("status_bar").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        let color = if msg.is_error {
                            egui::Color32::from_rgb(220, 80, 80)
                        } else {
                            egui::Color32::from_rgb(80, 200, 120)
                        };
                        ui.colored_label(color, &msg.text);
                    });
                });
            } else {
                self.state.status_message = None;
            }
        }
    }
}

impl eframe::App for TuxedoApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        self.shortcuts.handle_shortcuts(ctx, &mut self.state);
        
        // Handle background hardware updates
        self.handle_hardware_updates();
        
        // Draw top bar
        self.draw_top_bar(ctx);
        
        // Draw main content
        CentralPanel::default().show(ctx, |ui| {
            match self.state.current_page {
                Page::Statistics => {
                    statistics::draw(ui, &mut self.state);
                }
                Page::Profiles => {
                    profiles::draw(ui, &mut self.state, self.dbus_client.as_ref());
                }
                Page::Tuning => {
                    tuning::draw(ui, &mut self.state, self.dbus_client.as_ref());
                }
                Page::Settings => {
                    settings::draw(ui, &mut self.state, self.dbus_client.as_ref());
                }
            }
        });
        
        // Request continuous repaint for real-time updates
        ctx.request_repaint();
    }
}

fn start_background_polling(
    client: DbusClient,
    tx: mpsc::UnboundedSender<HardwareUpdate>,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(1000));
        let mut tick_count = 0u64;
        
        // Keep pending requests
        let mut pending_cpu: Option<oneshot::Receiver<Result<CpuInfo, anyhow::Error>>> = None;
        let mut pending_fans: Option<oneshot::Receiver<Result<Vec<FanInfo>, anyhow::Error>>> = None;
        let mut pending_system: Option<oneshot::Receiver<Result<SystemInfo, anyhow::Error>>> = None;
        let mut pending_gpu: Option<oneshot::Receiver<Result<Vec<GpuInfo>, anyhow::Error>>> = None;
        
        loop {
            interval.tick().await;
            tick_count += 1;
            
            // Check and send CPU info
            if let Some(mut rx) = pending_cpu.take() {
                match rx.try_recv() {
                    Ok(Ok(info)) => {
                        let _ = tx.send(HardwareUpdate::CpuInfo(info));
                    }
                    Ok(Err(e)) => {
                        log::warn!("CPU info error: {}", e);
                    }
                    Err(oneshot::error::TryRecvError::Empty) => {
                        pending_cpu = Some(rx);
                    }
                    Err(oneshot::error::TryRecvError::Closed) => {}
                }
            }
            if pending_cpu.is_none() {
                pending_cpu = Some(client.get_cpu_info());
            }
            
            // Check and send fan info
            if let Some(mut rx) = pending_fans.take() {
                match rx.try_recv() {
                    Ok(Ok(info)) => {
                        let _ = tx.send(HardwareUpdate::FanInfo(info));
                    }
                    Ok(Err(_)) => {}
                    Err(oneshot::error::TryRecvError::Empty) => {
                        pending_fans = Some(rx);
                    }
                    Err(oneshot::error::TryRecvError::Closed) => {}
                }
            }
            if pending_fans.is_none() {
                pending_fans = Some(client.get_fan_info());
            }
            
            // Battery every 5 seconds
            if tick_count % 5 == 0 {
                if let Ok(info) = read_battery_info() {
                    let _ = tx.send(HardwareUpdate::BatteryInfo(info));
                }
            }
            
            // System info every 60 seconds
            if tick_count % 60 == 0 {
                if let Some(mut rx) = pending_system.take() {
                    match rx.try_recv() {
                        Ok(Ok(info)) => {
                            let _ = tx.send(HardwareUpdate::SystemInfo(info));
                        }
                        Ok(Err(_)) => {}
                        Err(oneshot::error::TryRecvError::Empty) => {
                            pending_system = Some(rx);
                        }
                        Err(oneshot::error::TryRecvError::Closed) => {}
                    }
                }
                if pending_system.is_none() {
                    pending_system = Some(client.get_system_info());
                }
            }
            
            // GPU every 2 seconds
            if tick_count % 2 == 0 {
                if let Some(mut rx) = pending_gpu.take() {
                    match rx.try_recv() {
                        Ok(Ok(info)) => {
                            let _ = tx.send(HardwareUpdate::GpuInfo(info));
                        }
                        Ok(Err(_)) => {}
                        Err(oneshot::error::TryRecvError::Empty) => {
                            pending_gpu = Some(rx);
                        }
                        Err(oneshot::error::TryRecvError::Closed) => {}
                    }
                }
                if pending_gpu.is_none() {
                    pending_gpu = Some(client.get_gpu_info());
                }
            }
            
            // Storage every 30 seconds
            if tick_count % 30 == 0 {
                if let Ok(info) = read_storage_info() {
                    let _ = tx.send(HardwareUpdate::StorageInfo(info));
                }
            }
        }
    });
}

fn load_config_from_disk() -> anyhow::Result<AppConfig> {
    let config_dir = std::env::var("HOME")? + "/.config/tuxedo-control-center";
    let config_path = format!("{}/config.json", config_dir);
    let json = std::fs::read_to_string(config_path)?;
    Ok(serde_json::from_str(&json)?)
}

fn save_config_to_disk(config: &AppConfig) -> anyhow::Result<()> {
    let config_dir = std::env::var("HOME")? + "/.config/tuxedo-control-center";
    std::fs::create_dir_all(&config_dir)?;
    let config_path = format!("{}/config.json", config_dir);
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(config_path, json)?;
    Ok(())
}

fn read_battery_info() -> anyhow::Result<BatteryInfo> {

    let base = if std::path::Path::new("/sys/class/power_supply/BAT0").exists() {
        "/sys/class/power_supply/BAT0"
    } else {
        "/sys/class/power_supply/BAT1"
    };
    
    Ok(BatteryInfo {
        voltage_mv: read_sysfs_u64(&format!("{}/voltage_now", base))? / 1000,
        current_ma: read_sysfs_i64(&format!("{}/current_now", base))? / 1000,
        charge_percent: read_sysfs_u64(&format!("{}/capacity", base))?,
        capacity_mah: read_sysfs_u64(&format!("{}/charge_full", base))? / 1000,
        manufacturer: read_sysfs_string(&format!("{}/manufacturer", base))?,
        model: read_sysfs_string(&format!("{}/model_name", base))?,
        charge_start_threshold: read_sysfs_u64(&format!("{}/charge_control_start_threshold", base)).ok().map(|v| v as u8),
        charge_end_threshold: read_sysfs_u64(&format!("{}/charge_control_end_threshold", base)).ok().map(|v| v as u8),
    })
}

fn read_storage_info() -> anyhow::Result<Vec<StorageInfo>> {
    use std::fs;
    let mut storage_devices = Vec::new();
    
    for entry in fs::read_dir("/sys/block")? {
        let entry = entry?;
        let dev_name = entry.file_name().to_string_lossy().to_string();
        
        if dev_name.starts_with("loop") || dev_name.starts_with("ram") {
            continue;
        }
        
        let path = entry.path();
        let model = fs::read_to_string(path.join("device/model"))
            .unwrap_or_else(|_| dev_name.clone())
            .trim()
            .to_string();
        
        let size_gb = if let Ok(size_str) = fs::read_to_string(path.join("size")) {
            if let Ok(sectors) = size_str.trim().parse::<u64>() {
                (sectors * 512) / 1_000_000_000
            } else {
                0
            }
        } else {
            0
        };
        
        storage_devices.push(StorageInfo {
            device: format!("/dev/{}", dev_name),
            model,
            size_gb,
            temperature: None,
        });
    }
    
    Ok(storage_devices)
}

fn read_sysfs_u64(path: &str) -> anyhow::Result<u64> {
    Ok(std::fs::read_to_string(path)?.trim().parse()?)
}

fn read_sysfs_i64(path: &str) -> anyhow::Result<i64> {
    Ok(std::fs::read_to_string(path)?.trim().parse()?)
}

fn read_sysfs_string(path: &str) -> anyhow::Result<String> {
    Ok(std::fs::read_to_string(path)?.trim().to_string())
}
