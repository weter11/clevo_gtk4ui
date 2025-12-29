use anyhow::Result;
use tuxedo_common::types::*;
use zbus::blocking::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DbusClient {
    connection: Arc<Mutex<Connection>>,
}

impl DbusClient {
    pub fn new() -> Result<Self> {
        let connection = Connection::system()?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
    
    pub fn get_system_info(&self) -> Result<SystemInfo> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        let json: String = proxy.call("GetSystemInfo", &())?;
        Ok(serde_json::from_str(&json)?)
    }
    
    pub fn get_cpu_info(&self) -> Result<CpuInfo> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        let json: String = proxy.call("GetCpuInfo", &())?;
        Ok(serde_json::from_str(&json)?)
    }
    
    pub fn get_gpu_info(&self) -> Result<Vec<GpuInfo>> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        let json: String = proxy.call("GetGpuInfo", &())?;
        Ok(serde_json::from_str(&json)?)
    }
    
    pub fn set_cpu_governor(&self, governor: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        proxy.call::<_, _, ()>("SetCpuGovernor", &(governor,))?;
        Ok(())
    }
    
    pub fn set_cpu_frequency_limits(&self, min_freq: u64, max_freq: u64) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        proxy.call::<_, _, ()>("SetCpuFrequencyLimits", &(min_freq, max_freq))?;
        Ok(())
    }
    
    pub fn set_cpu_boost(&self, enabled: bool) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        proxy.call::<_, _, ()>("SetCpuBoost", &(enabled,))?;
        Ok(())
    }
    
    pub fn set_smt(&self, enabled: bool) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        proxy.call::<_, _, ()>("SetSmt", &(enabled,))?;
        Ok(())
    }
    
    pub fn set_amd_pstate_status(&self, status: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        proxy.call::<_, _, ()>("SetAmdPstateStatus", &(status,))?;
        Ok(())
    }
    
    pub fn apply_profile(&self, profile: &Profile) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        let proxy = zbus::blocking::Proxy::new(
            &*conn,
            "com.tuxedo.Control",
            "/com/tuxedo/Control",
            "com.tuxedo.Control",
        )?;
        
        let json = serde_json::to_string(profile)?;
        proxy.call::<_, _, ()>("ApplyProfile", &(json.as_str(),))?;
        Ok(())
    }

    pub fn get_tdp_profiles(&self) -> Result<Vec<String>> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    let json: String = proxy.call("GetTdpProfiles", &())?;
    Ok(serde_json::from_str(&json)?)
}

pub fn get_current_tdp_profile(&self) -> Result<String> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    Ok(proxy.call("GetCurrentTdpProfile", &())?)
}

pub fn get_fan_speeds(&self) -> Result<Vec<(u32, u32)>> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    let json: String = proxy.call("GetFanSpeeds", &())?;
    Ok(serde_json::from_str(&json)?)
}

pub fn get_fan_temperature(&self, fan_id: u32) -> Result<u32> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    Ok(proxy.call("GetFanTemperature", &(fan_id,))?)
}

pub fn set_fan_speed(&self, fan_id: u32, speed: u32) -> Result<()> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    proxy.call::<_, _, ()>("SetFanSpeed", &(fan_id, speed))?;
    Ok(())
}

pub fn set_fan_auto(&self, fan_id: u32) -> Result<()> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    proxy.call::<_, _, ()>("SetFanAuto", &(fan_id,))?;
    Ok(())
}

pub fn get_webcam_state(&self) -> Result<bool> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    Ok(proxy.call("GetWebcamState", &())?)
}

pub fn set_webcam_state(&self, enabled: bool) -> Result<()> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    proxy.call::<_, _, ()>("SetWebcamState", &(enabled,))?;
    Ok(())
}

pub fn set_tdp_profile(&self, profile: &str) -> Result<()> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    proxy.call::<_, _, ()>("SetTdpProfile", &(profile,))?;
    Ok(())
}

pub fn set_energy_performance_preference(&self, epp: &str) -> Result<()> {
    let conn = self.connection.lock().unwrap();
    let proxy = zbus::blocking::Proxy::new(
        &*conn,
        "com.tuxedo.Control",
        "/com/tuxedo/Control",
        "com.tuxedo.Control",
    )?;
    
    proxy.call::<_, _, ()>("SetEnergyPerformancePreference", &(epp,))?;
    Ok(())
}
}
