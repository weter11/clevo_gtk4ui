use anyhow::Result;
use tuxedo_common::types::*;
use zbus::{interface, Connection, ConnectionBuilder};

pub struct ControlInterface;

#[interface(name = "com.tuxedo.Control")]
impl ControlInterface {
    async fn get_system_info(&self) -> Result<String, zbus::fdo::Error> {
        match crate::hardware_detection::get_system_info() {
            Ok(info) => serde_json::to_string(&info)
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string())),
            Err(e) => Err(zbus::fdo::Error::Failed(e.to_string())),
        }
    }

    async fn get_cpu_info(&self) -> Result<String, zbus::fdo::Error> {
        match crate::hardware_detection::get_cpu_info() {
            Ok(info) => serde_json::to_string(&info)
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string())),
            Err(e) => Err(zbus::fdo::Error::Failed(e.to_string())),
        }
    }

    async fn get_gpu_info(&self) -> Result<String, zbus::fdo::Error> {
        match crate::hardware_detection::get_gpu_info() {
            Ok(info) => serde_json::to_string(&info)
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string())),
            Err(e) => Err(zbus::fdo::Error::Failed(e.to_string())),
        }
    }

    async fn set_cpu_governor(&self, governor: &str) -> Result<(), zbus::fdo::Error> {
        crate::hardware_control::set_cpu_governor(governor)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn set_cpu_frequency_limits(
        &self,
        min_freq: u64,
        max_freq: u64,
    ) -> Result<(), zbus::fdo::Error> {
        crate::hardware_control::set_cpu_frequency_limits(min_freq, max_freq)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn set_cpu_boost(&self, enabled: bool) -> Result<(), zbus::fdo::Error> {
        crate::hardware_control::set_cpu_boost(enabled)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn set_smt(&self, enabled: bool) -> Result<(), zbus::fdo::Error> {
        crate::hardware_control::set_smt(enabled)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn set_amd_pstate_status(&self, status: &str) -> Result<(), zbus::fdo::Error> {
        crate::hardware_control::set_amd_pstate_status(status)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn apply_profile(&self, profile_json: &str) -> Result<(), zbus::fdo::Error> {
        let profile: Profile = serde_json::from_str(profile_json)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        crate::hardware_control::apply_profile(&profile)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    async fn get_tdp_profiles(&self) -> Result<String, zbus::fdo::Error> {
    match crate::hardware_detection::get_tdp_profiles() {
        Ok(profiles) => serde_json::to_string(&profiles)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string())),
        Err(e) => Err(zbus::fdo::Error::Failed(e.to_string())),
    }
}

    async fn get_current_tdp_profile(&self) -> Result<String, zbus::fdo::Error> {
        match crate::hardware_detection::get_current_tdp_profile() {
            Ok(profile) => Ok(profile),
            Err(e) => Err(zbus::fdo::Error::Failed(e.to_string())),
        }
    }

async fn set_tdp_profile(&self, profile: &str) -> Result<(), zbus::fdo::Error> {
    crate::hardware_control::set_tdp_profile(profile)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
}

async fn set_energy_performance_preference(&self, epp: &str) -> Result<(), zbus::fdo::Error> {
    crate::hardware_control::set_energy_performance_preference(epp)
        .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
}
}

pub async fn start_service(_connection: Connection) -> Result<()> {
    let _conn = ConnectionBuilder::system()?
        .name("com.tuxedo.Control")?
        .serve_at("/com/tuxedo/Control", ControlInterface)?
        .build()
        .await?;
    
    // Keep connection alive
    std::future::pending::<()>().await;
    Ok(())
}
