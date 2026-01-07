use anyhow::Result;
use tuxedo_common::types::*;
use zbus::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct DbusClient {
    connection: Arc<Mutex<Connection>>,
}

impl DbusClient {
    pub fn new() -> Result<Self> {
        // Create runtime for blocking call
        let rt = tokio::runtime::Runtime::new()?;
        let connection = rt.block_on(async {
            Connection::system().await
        })?;
        
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
    
    // Blocking methods that can be called from egui's update()
    
    pub fn get_system_info(&self) -> Result<SystemInfo> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let conn = self.connection.lock().await;
            let proxy = zbus::Proxy::new(
                &*conn,
                "com.tuxedo.Control",
                "/com/tuxedo/Control",
                "com.tuxedo.Control",
            ).await?;
            
            let json: String = proxy.call("GetSystemInfo", &()).await?;
            Ok(serde_json::from_str(&json)?)
        })
    }
    
    pub fn get_cpu_info(&self) -> Result<CpuInfo> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let conn = self.connection.lock().await;
            let proxy = zbus::Proxy::new(
                &*conn,
                "com.tuxedo.Control",
                "/com/tuxedo/Control",
                "com.tuxedo.Control",
            ).await?;
            
            let json: String = proxy.call("GetCpuInfo", &()).await?;
            Ok(serde_json::from_str(&json)?)
        })
    }
    
    pub fn get_gpu_info(&self) -> Result<Vec<GpuInfo>> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let conn = self.connection.lock().await;
            let proxy = zbus::Proxy::new(
                &*conn,
                "com.tuxedo.Control",
                "/com/tuxedo/Control",
                "com.tuxedo.Control",
            ).await?;
            
            let json: String = proxy.call("GetGpuInfo", &()).await?;
            Ok(serde_json::from_str(&json)?)
        })
    }
    
    pub fn get_fan_info(&self) -> Result<Vec<FanInfo>> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let conn = self.connection.lock().await;
            let proxy = zbus::Proxy::new(
                &*conn,
                "com.tuxedo.Control",
                "/com/tuxedo/Control",
                "com.tuxedo.Control",
            ).await?;
            
            let json: String = proxy.call("GetFanInfo", &()).await?;
            Ok(serde_json::from_str(&json)?)
        })
    }
    
    pub fn apply_profile(&self, profile: &Profile) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let conn = self.connection.lock().await;
            let proxy = zbus::Proxy::new(
                &*conn,
                "com.tuxedo.Control",
                "/com/tuxedo/Control",
                "com.tuxedo.Control",
            ).await?;
            
            let json = serde_json::to_string(profile)?;
            proxy.call::<_, _, ()>("ApplyProfile", &(json.as_str(),)).await?;
            Ok(())
        })
    }
    
    pub fn set_cpu_governor(&self, governor: &str) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let conn = self.connection.lock().await;
            let proxy = zbus::Proxy::new(
                &*conn,
                "com.tuxedo.Control",
                "/com/tuxedo/Control",
                "com.tuxedo.Control",
            ).await?;
            
            proxy.call::<_, _, ()>("SetCpuGovernor", &(governor,)).await?;
            Ok(())
        })
    }
    
    pub fn set_cpu_boost(&self, enabled: bool) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let conn = self.connection.lock().await;
            let proxy = zbus::Proxy::new(
                &*conn,
                "com.tuxedo.Control",
                "/com/tuxedo/Control",
                "com.tuxedo.Control",
            ).await?;
            
            proxy.call::<_, _, ()>("SetCpuBoost", &(enabled,)).await?;
            Ok(())
        })
    }
    
    pub fn preview_keyboard_settings(&self, settings: &KeyboardSettings) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let conn = self.connection.lock().await;
            let proxy = zbus::Proxy::new(
                &*conn,
                "com.tuxedo.Control",
                "/com/tuxedo/Control",
                "com.tuxedo.Control",
            ).await?;
            
            let json = serde_json::to_string(settings)?;
            proxy.call::<_, _, ()>("PreviewKeyboardSettings", &(json.as_str(),)).await?;
            Ok(())
        })
    }
}
