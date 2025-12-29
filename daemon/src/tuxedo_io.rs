use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use anyhow::{anyhow, Result};

const TUXEDO_IO_DEVICE: &str = "/dev/tuxedo_io";

const MAGIC_NUMBER: u8 = 0xEC;

nix::ioctl_read!(ioctl_get_fan_speed, MAGIC_NUMBER, 0x10, u32);
nix::ioctl_write_ptr!(ioctl_set_fan_speed, MAGIC_NUMBER, 0x11, u32);
nix::ioctl_read!(ioctl_get_fan_temp, MAGIC_NUMBER, 0x12, u32);
nix::ioctl_read!(ioctl_get_webcam_sw, MAGIC_NUMBER, 0x13, u32);
nix::ioctl_write_ptr!(ioctl_set_webcam_sw, MAGIC_NUMBER, 0x14, u32);
nix::ioctl_read!(ioctl_get_tdp, MAGIC_NUMBER, 0x20, u32);
nix::ioctl_write_ptr!(ioctl_set_tdp, MAGIC_NUMBER, 0x21, u32);
nix::ioctl_read!(ioctl_get_tdp_min, MAGIC_NUMBER, 0x22, u32);
nix::ioctl_read!(ioctl_get_tdp_max, MAGIC_NUMBER, 0x23, u32);
nix::ioctl_read_buf!(ioctl_get_perf_profiles, MAGIC_NUMBER, 0x30, u8);
nix::ioctl_read!(ioctl_get_perf_profile, MAGIC_NUMBER, 0x31, u32);
nix::ioctl_write_ptr!(ioctl_set_perf_profile, MAGIC_NUMBER, 0x32, u32);

pub struct TuxedoIo {
    device: File,
}

impl TuxedoIo {
    pub fn new() -> Result<Self> {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(TUXEDO_IO_DEVICE)?;
        
        Ok(TuxedoIo { device })
    }
    
    pub fn is_available() -> bool {
        std::path::Path::new(TUXEDO_IO_DEVICE).exists()
    }
    
    pub fn get_fan_speed(&self, fan_id: u32) -> Result<u32> {
        let mut speed: u32 = fan_id;
        unsafe {
            ioctl_get_fan_speed(self.device.as_raw_fd(), &mut speed)
                .map_err(|e| anyhow!("Failed to get fan speed: {}", e))?;
        }
        Ok(speed)
    }
    
    pub fn set_fan_speed(&self, fan_id: u32, speed: u32) -> Result<()> {
        let data = (fan_id << 16) | (speed & 0xFFFF);
        unsafe {
            ioctl_set_fan_speed(self.device.as_raw_fd(), &data)
                .map_err(|e| anyhow!("Failed to set fan speed: {}", e))?;
        }
        Ok(())
    }
    
    pub fn get_fan_temperature(&self, fan_id: u32) -> Result<u32> {
        let mut temp: u32 = fan_id;
        unsafe {
            ioctl_get_fan_temp(self.device.as_raw_fd(), &mut temp)
                .map_err(|e| anyhow!("Failed to get fan temperature: {}", e))?;
        }
        Ok(temp)
    }
    
    pub fn get_tdp(&self) -> Result<u32> {
        let mut tdp: u32 = 0;
        unsafe {
            ioctl_get_tdp(self.device.as_raw_fd(), &mut tdp)
                .map_err(|e| anyhow!("Failed to get TDP: {}", e))?;
        }
        Ok(tdp)
    }
    
    pub fn set_tdp(&self, tdp: u32) -> Result<()> {
        unsafe {
            ioctl_set_tdp(self.device.as_raw_fd(), &tdp)
                .map_err(|e| anyhow!("Failed to set TDP: {}", e))?;
        }
        Ok(())
    }
    
    pub fn get_tdp_min(&self) -> Result<u32> {
        let mut tdp_min: u32 = 0;
        unsafe {
            ioctl_get_tdp_min(self.device.as_raw_fd(), &mut tdp_min)
                .map_err(|e| anyhow!("Failed to get TDP min: {}", e))?;
        }
        Ok(tdp_min)
    }
    
    pub fn get_tdp_max(&self) -> Result<u32> {
        let mut tdp_max: u32 = 0;
        unsafe {
            ioctl_get_tdp_max(self.device.as_raw_fd(), &mut tdp_max)
                .map_err(|e| anyhow!("Failed to get TDP max: {}", e))?;
        }
        Ok(tdp_max)
    }
    
    pub fn get_performance_profiles(&self) -> Result<Vec<String>> {
        let mut buffer = [0u8; 256];
        unsafe {
            ioctl_get_perf_profiles(self.device.as_raw_fd(), &mut buffer)
                .map_err(|e| anyhow!("Failed to get performance profiles: {}", e))?;
        }
        
        let profiles_str = String::from_utf8_lossy(&buffer);
        let profiles: Vec<String> = profiles_str
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        
        Ok(profiles)
    }
    
    pub fn get_performance_profile(&self) -> Result<u32> {
        let mut profile: u32 = 0;
        unsafe {
            ioctl_get_perf_profile(self.device.as_raw_fd(), &mut profile)
                .map_err(|e| anyhow!("Failed to get performance profile: {}", e))?;
        }
        Ok(profile)
    }
    
    pub fn set_performance_profile(&self, profile: u32) -> Result<()> {
        unsafe {
            ioctl_set_perf_profile(self.device.as_raw_fd(), &profile)
                .map_err(|e| anyhow!("Failed to set performance profile: {}", e))?;
        }
        Ok(())
    }
    
    pub fn get_webcam_state(&self) -> Result<bool> {
        let mut state: u32 = 0;
        unsafe {
            ioctl_get_webcam_sw(self.device.as_raw_fd(), &mut state)
                .map_err(|e| anyhow!("Failed to get webcam state: {}", e))?;
        }
        Ok(state != 0)
    }
    
    pub fn set_webcam_state(&self, enabled: bool) -> Result<()> {
        let state: u32 = if enabled { 1 } else { 0 };
        unsafe {
            ioctl_set_webcam_sw(self.device.as_raw_fd(), &state)
                .map_err(|e| anyhow!("Failed to set webcam state: {}", e))?;
        }
        Ok(())
    }
}
