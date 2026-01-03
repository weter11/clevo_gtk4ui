use anyhow::{anyhow, Result};
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

const TUXEDO_IO_DEVICE: &str = "/dev/tuxedo_io";
const IOCTL_MAGIC: u8 = 0xEC;
const MAGIC_READ_CL: u8 = IOCTL_MAGIC + 1;
const MAGIC_WRITE_CL: u8 = IOCTL_MAGIC + 2;
const MAGIC_READ_UW: u8 = IOCTL_MAGIC + 3;
const MAGIC_WRITE_UW: u8 = IOCTL_MAGIC + 4;

// Hardware check ioctls
nix::ioctl_read!(ioctl_hwcheck_cl, IOCTL_MAGIC, 0x05, i32);
nix::ioctl_read!(ioctl_hwcheck_uw, IOCTL_MAGIC, 0x06, i32);

// Clevo read ioctls
nix::ioctl_read!(ioctl_cl_faninfo1, MAGIC_READ_CL, 0x10, i32);
nix::ioctl_read!(ioctl_cl_faninfo2, MAGIC_READ_CL, 0x11, i32);
nix::ioctl_read!(ioctl_cl_faninfo3, MAGIC_READ_CL, 0x12, i32);
nix::ioctl_read!(ioctl_cl_webcam_sw, MAGIC_READ_CL, 0x13, i32);

// Clevo write ioctls
nix::ioctl_write_ptr!(ioctl_cl_fanspeed, MAGIC_WRITE_CL, 0x10, i32);
nix::ioctl_write_ptr!(ioctl_cl_fanauto, MAGIC_WRITE_CL, 0x11, i32);
nix::ioctl_write_ptr!(ioctl_cl_webcam_sw_w, MAGIC_WRITE_CL, 0x12, i32);
nix::ioctl_write_ptr!(ioctl_cl_perf_profile, MAGIC_WRITE_CL, 0x15, i32);

// Uniwill read ioctls
nix::ioctl_read!(ioctl_uw_fanspeed, MAGIC_READ_UW, 0x10, i32);
nix::ioctl_read!(ioctl_uw_fanspeed2, MAGIC_READ_UW, 0x11, i32);
nix::ioctl_read!(ioctl_uw_fan_temp, MAGIC_READ_UW, 0x12, i32);
nix::ioctl_read!(ioctl_uw_fan_temp2, MAGIC_READ_UW, 0x13, i32);
nix::ioctl_read!(ioctl_uw_tdp0, MAGIC_READ_UW, 0x18, i32);
nix::ioctl_read!(ioctl_uw_tdp1, MAGIC_READ_UW, 0x19, i32);
nix::ioctl_read!(ioctl_uw_tdp2, MAGIC_READ_UW, 0x1a, i32);
nix::ioctl_read!(ioctl_uw_tdp0_min, MAGIC_READ_UW, 0x1b, i32);
nix::ioctl_read!(ioctl_uw_tdp1_min, MAGIC_READ_UW, 0x1c, i32);
nix::ioctl_read!(ioctl_uw_tdp2_min, MAGIC_READ_UW, 0x1d, i32);
nix::ioctl_read!(ioctl_uw_tdp0_max, MAGIC_READ_UW, 0x1e, i32);
nix::ioctl_read!(ioctl_uw_tdp1_max, MAGIC_READ_UW, 0x1f, i32);
nix::ioctl_read!(ioctl_uw_tdp2_max, MAGIC_READ_UW, 0x20, i32);
nix::ioctl_read!(ioctl_uw_profs_available, MAGIC_READ_UW, 0x21, i32);

// Uniwill write ioctls
nix::ioctl_write_ptr!(ioctl_uw_fanspeed_w, MAGIC_WRITE_UW, 0x10, i32);
nix::ioctl_write_ptr!(ioctl_uw_fanspeed2_w, MAGIC_WRITE_UW, 0x11, i32);
nix::ioctl_write_int!(ioctl_uw_fanauto, MAGIC_WRITE_UW, 0x14);
nix::ioctl_write_ptr!(ioctl_uw_tdp0_w, MAGIC_WRITE_UW, 0x15, i32);
nix::ioctl_write_ptr!(ioctl_uw_tdp1_w, MAGIC_WRITE_UW, 0x16, i32);
nix::ioctl_write_ptr!(ioctl_uw_tdp2_w, MAGIC_WRITE_UW, 0x17, i32);
nix::ioctl_write_ptr!(ioctl_uw_perf_prof, MAGIC_WRITE_UW, 0x18, i32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HardwareInterface {
    Clevo,
    Uniwill,
    None,
}

pub struct TuxedoIo {
    device: std::fs::File,
    interface: HardwareInterface,
    fan_count: u32,
}

impl TuxedoIo {
    pub fn new() -> Result<Self> {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(TUXEDO_IO_DEVICE)?;

        let interface = Self::detect_interface(&device)?;
        let fan_count = Self::detect_fan_count(&device, interface)?;

        Ok(TuxedoIo {
            device,
            interface,
            fan_count,
        })
    }

    pub fn is_available() -> bool {
        std::path::Path::new(TUXEDO_IO_DEVICE).exists()
    }

    pub fn get_interface(&self) -> HardwareInterface {
        self.interface
    }

    pub fn get_fan_count(&self) -> u32 {
        self.fan_count
    }

    fn clevo_raw_to_percent(raw: u8) -> u32 {
        // Clevo returns raw 0..255
        ((raw as u32 * 100) + 127) / 255
    }

    fn clevo_percent_to_raw(percent: u32) -> u8 {
        let p = percent.min(100);
        (((p * 255) + 50) / 100) as u8
    }

    fn detect_interface(device: &std::fs::File) -> Result<HardwareInterface> {
        let fd = device.as_raw_fd();

        let mut clevo_check: i32 = 0;
        let mut uniwill_check: i32 = 0;

        let cl_res = unsafe { ioctl_hwcheck_cl(fd, &mut clevo_check) };
        let uw_res = unsafe { ioctl_hwcheck_uw(fd, &mut uniwill_check) };

        log::debug!(
            "tuxedo_io hwcheck: clevo={:?} val={}, uniwill={:?} val={}",
            cl_res,
            clevo_check,
            uw_res,
            uniwill_check
        );

        if clevo_check == 1 {
            return Ok(HardwareInterface::Clevo);
        }
        if uniwill_check == 1 {
            return Ok(HardwareInterface::Uniwill);
        }

        // Fallback probing: some systems don't report hwcheck==1 reliably.
        log::warn!("tuxedo_io hwcheck did not identify interface; probing fan ioctls...");

        // Probe clevo faninfo ioctl
        let mut probe: i32 = 0;
        if unsafe { ioctl_cl_faninfo1(fd, &mut probe) }.is_ok() {
            log::info!("tuxedo_io probe: Clevo faninfo ioctl succeeded; assuming Clevo interface");
            return Ok(HardwareInterface::Clevo);
        } else {
            log::debug!("tuxedo_io probe: Clevo faninfo ioctl failed");
        }

        // Probe uniwill fanspeed ioctl
        probe = 0;
        if unsafe { ioctl_uw_fanspeed(fd, &mut probe) }.is_ok() {
            log::info!("tuxedo_io probe: Uniwill fanspeed ioctl succeeded; assuming Uniwill interface");
            return Ok(HardwareInterface::Uniwill);
        } else {
            log::debug!("tuxedo_io probe: Uniwill fanspeed ioctl failed");
        }

        Ok(HardwareInterface::None)
    }

    fn detect_fan_count(device: &std::fs::File, interface: HardwareInterface) -> Result<u32> {
        let fd = device.as_raw_fd();

        match interface {
            HardwareInterface::Clevo => {
                // Try reading all 3 fan infos; treat ioctl success as "fan exists"
                let mut count = 0;
                let mut result: i32 = 0;

                unsafe {
                    if ioctl_cl_faninfo1(fd, &mut result).is_ok() {
                        count += 1;
                    }
                    if ioctl_cl_faninfo2(fd, &mut result).is_ok() {
                        count += 1;
                    }
                    if ioctl_cl_faninfo3(fd, &mut result).is_ok() {
                        count += 1;
                    }
                }

                Ok(count)
            }
            HardwareInterface::Uniwill => {
                let mut result: i32 = 0;
                unsafe {
                    if ioctl_uw_fanspeed(fd, &mut result).is_ok() {
                        let mut count = 1;
                        if ioctl_uw_fanspeed2(fd, &mut result).is_ok() {
                            count = 2;
                        }
                        return Ok(count);
                    }
                }
                Ok(0)
            }
            HardwareInterface::None => Ok(0),
        }
    }

    // Fan control methods
    pub fn get_fan_speed(&self, fan_id: u32) -> Result<u32> {
        let fd = self.device.as_raw_fd();

        match self.interface {
            HardwareInterface::Clevo => {
                let mut result: i32 = 0;
                unsafe {
                    match fan_id {
                        0 => {
                            let _ = ioctl_cl_faninfo1(fd, &mut result)?;
                        }
                        1 => {
                            let _ = ioctl_cl_faninfo2(fd, &mut result)?;
                        }
                        2 => {
                            let _ = ioctl_cl_faninfo3(fd, &mut result)?;
                        }
                        _ => return Err(anyhow!("Invalid fan ID")),
                    }
                }
                // Clevo: lower byte is raw 0..255 => convert to percent 0..100
                let raw = (result & 0xFF) as u8;
                Ok(Self::clevo_raw_to_percent(raw))
            }
            HardwareInterface::Uniwill => {
                let mut result: i32 = 0;
                unsafe {
                    match fan_id {
                        0 => {
                            let _ = ioctl_uw_fanspeed(fd, &mut result)?;
                        }
                        1 => {
                            let _ = ioctl_uw_fanspeed2(fd, &mut result)?;
                        }
                        _ => return Err(anyhow!("Invalid fan ID")),
                    }
                }
                // NOTE: uniwill is raw-ish; existing code uses 0..200 elsewhere.
                Ok(result as u32)
            }
            HardwareInterface::None => Err(anyhow!("No hardware interface available")),
        }
    }

    pub fn set_fan_speed(&self, fan_id: u32, speed_percent: u32) -> Result<()> {
        let fd = self.device.as_raw_fd();

        match self.interface {
            HardwareInterface::Clevo => {
                // Clevo sets all fans at once with packed RAW values (0..255)
                let mut current_raw = [0u8; 3];

                // Read current raw speeds from faninfo so we only update one fan
                for i in 0..self.fan_count.min(3) {
                    let mut result: i32 = 0;
                    let ok = unsafe {
                        match i {
                            0 => ioctl_cl_faninfo1(fd, &mut result).is_ok(),
                            1 => ioctl_cl_faninfo2(fd, &mut result).is_ok(),
                            2 => ioctl_cl_faninfo3(fd, &mut result).is_ok(),
                            _ => false,
                        }
                    };
                    if ok {
                        current_raw[i as usize] = (result & 0xFF) as u8;
                    }
                }

                if fan_id >= 3 {
                    return Err(anyhow!("Invalid fan ID"));
                }

                current_raw[fan_id as usize] = Self::clevo_percent_to_raw(speed_percent);

                let packed: i32 = (current_raw[0] as i32)
                    | ((current_raw[1] as i32) << 8)
                    | ((current_raw[2] as i32) << 16);

                unsafe {
                    ioctl_cl_fanspeed(fd, &packed)?;
                }
                Ok(())
            }
            HardwareInterface::Uniwill => {
                let speed_val = speed_percent.min(200) as i32; // keep existing behavior for now
                unsafe {
                    match fan_id {
                        0 => {
                            let _ = ioctl_uw_fanspeed_w(fd, &speed_val)?;
                        }
                        1 => {
                            let _ = ioctl_uw_fanspeed2_w(fd, &speed_val)?;
                        }
                        _ => return Err(anyhow!("Invalid fan ID")),
                    }
                }
                Ok(())
            }
            HardwareInterface::None => Err(anyhow!("No hardware interface available")),
        }
    }

    pub fn set_fan_auto(&self) -> Result<()> {
        let fd = self.device.as_raw_fd();

        match self.interface {
            HardwareInterface::Clevo => {
                let auto_val: i32 = 1;
                unsafe {
                    ioctl_cl_fanauto(fd, &auto_val)?;
                }
                Ok(())
            }
            HardwareInterface::Uniwill => {
                unsafe {
                    ioctl_uw_fanauto(fd, 1)?;
                }
                Ok(())
            }
            HardwareInterface::None => Err(anyhow!("No hardware interface available")),
        }
    }

    pub fn get_fan_temperature(&self, fan_id: u32) -> Result<u32> {
        let fd = self.device.as_raw_fd();

        match self.interface {
            HardwareInterface::Clevo => {
                // Clevo returns temperature in upper byte of faninfo
                let mut result: i32 = 0;
                unsafe {
                    match fan_id {
                        0 => {
                            let _ = ioctl_cl_faninfo1(fd, &mut result)?;
                        }
                        1 => {
                            let _ = ioctl_cl_faninfo2(fd, &mut result)?;
                        }
                        2 => {
                            let _ = ioctl_cl_faninfo3(fd, &mut result)?;
                        }
                        _ => return Err(anyhow!("Invalid fan ID")),
                    }
                }
                Ok(((result >> 8) & 0xFF) as u32)
            }
            HardwareInterface::Uniwill => {
                let mut result: i32 = 0;
                unsafe {
                    match fan_id {
                        0 => {
                            let _ = ioctl_uw_fan_temp(fd, &mut result)?;
                        }
                        1 => {
                            let _ = ioctl_uw_fan_temp2(fd, &mut result)?;
                        }
                        _ => return Err(anyhow!("Invalid fan ID")),
                    }
                }
                Ok(result as u32)
            }
            HardwareInterface::None => Err(anyhow!("No hardware interface available")),
        }
    }
    
    // Performance profile methods
    pub fn get_available_profiles(&self) -> Result<Vec<String>> {
        match self.interface {
            HardwareInterface::Clevo => {
                // Clevo has fixed profiles
                Ok(vec![
                    "Power Saving".to_string(),
                    "Balanced".to_string(),
                    "Performance".to_string(),
                ])
            }
            HardwareInterface::Uniwill => {
                let fd = self.device.as_raw_fd();
                let mut result: i32 = 0;
                unsafe {
                    ioctl_uw_profs_available(fd, &mut result)?;
                }
                
                let mut profiles = vec![];
                if result >= 2 {
                    profiles.push("Power Saving".to_string());
                    profiles.push("Enthusiast".to_string());
                }
                if result >= 3 {
                    profiles.push("Overboost".to_string());
                }
                Ok(profiles)
            }
            HardwareInterface::None => Ok(vec![]),
        }
    }
    
    pub fn set_performance_profile(&self, profile_id: u32) -> Result<()> {
        let fd = self.device.as_raw_fd();
        
        match self.interface {
            HardwareInterface::Clevo => {
                let profile_val = (profile_id + 1) as i32; // Clevo uses 1-3
                unsafe {
                    ioctl_cl_perf_profile(fd, &profile_val)?;
                }
                Ok(())
            }
            HardwareInterface::Uniwill => {
                let profile_val = (profile_id + 1) as i32; // Uniwill also uses 1-3
                unsafe {
                    ioctl_uw_perf_prof(fd, &profile_val)?;
                }
                Ok(())
            }
            HardwareInterface::None => Err(anyhow!("No hardware interface available")),
        }
    }
    
    // TDP methods (Uniwill only)
    pub fn get_tdp(&self, tdp_index: u8) -> Result<i32> {
        if self.interface != HardwareInterface::Uniwill {
            return Err(anyhow!("TDP control only available on Uniwill interface"));
        }
        
        let fd = self.device.as_raw_fd();
        let mut result: i32 = 0;
        
        unsafe {
            match tdp_index {
                0 => { let _ = ioctl_uw_tdp0(fd, &mut result)?; }
                1 => { let _ = ioctl_uw_tdp1(fd, &mut result)?; }
                2 => { let _ = ioctl_uw_tdp2(fd, &mut result)?; }
                _ => return Err(anyhow!("Invalid TDP index")),
            }
        }
        
        Ok(result)
    }
    
    pub fn get_tdp_min(&self, tdp_index: u8) -> Result<i32> {
        if self.interface != HardwareInterface::Uniwill {
            return Err(anyhow!("TDP control only available on Uniwill interface"));
        }
        
        let fd = self.device.as_raw_fd();
        let mut result: i32 = 0;
        
        unsafe {
            match tdp_index {
                0 => { let _ = ioctl_uw_tdp0_min(fd, &mut result)?; }
                1 => { let _ = ioctl_uw_tdp1_min(fd, &mut result)?; }
                2 => { let _ = ioctl_uw_tdp2_min(fd, &mut result)?; }
                _ => return Err(anyhow!("Invalid TDP index")),
            }
        }
        
        Ok(result)
    }
    
    pub fn get_tdp_max(&self, tdp_index: u8) -> Result<i32> {
        if self.interface != HardwareInterface::Uniwill {
            return Err(anyhow!("TDP control only available on Uniwill interface"));
        }
        
        let fd = self.device.as_raw_fd();
        let mut result: i32 = 0;
        
        unsafe {
            match tdp_index {
                0 => { let _ = ioctl_uw_tdp0_max(fd, &mut result)?; }
                1 => { let _ = ioctl_uw_tdp1_max(fd, &mut result)?; }
                2 => { let _ = ioctl_uw_tdp2_max(fd, &mut result)?; }
                _ => return Err(anyhow!("Invalid TDP index")),
            }
        }
        
        Ok(result)
    }
    
    pub fn set_tdp(&self, tdp_index: u8, value: i32) -> Result<()> {
        if self.interface != HardwareInterface::Uniwill {
            return Err(anyhow!("TDP control only available on Uniwill interface"));
        }
        
        let fd = self.device.as_raw_fd();
        
        unsafe {
            match tdp_index {
                0 => { let _ = ioctl_uw_tdp0_w(fd, &value)?; }
                1 => { let _ = ioctl_uw_tdp1_w(fd, &value)?; }
                2 => { let _ = ioctl_uw_tdp2_w(fd, &value)?; }
                _ => return Err(anyhow!("Invalid TDP index")),
            }
        }
        
        Ok(())
    }
    
    // Webcam control (Clevo only)
    pub fn get_webcam_state(&self) -> Result<bool> {
        if self.interface != HardwareInterface::Clevo {
            return Err(anyhow!("Webcam control only available on Clevo interface"));
        }
        
        let fd = self.device.as_raw_fd();
        let mut result: i32 = 0;
        
        unsafe {
            ioctl_cl_webcam_sw(fd, &mut result)?;
        }
        
        Ok(result != 0)
    }
    
    pub fn set_webcam_state(&self, enabled: bool) -> Result<()> {
        if self.interface != HardwareInterface::Clevo {
            return Err(anyhow!("Webcam control only available on Clevo interface"));
        }
        
        let fd = self.device.as_raw_fd();
        let value: i32 = if enabled { 1 } else { 0 };
        
        unsafe {
            ioctl_cl_webcam_sw_w(fd, &value)?;
        }
        
        Ok(())
    }
}
