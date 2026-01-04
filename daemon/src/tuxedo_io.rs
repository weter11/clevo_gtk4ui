use anyhow::{anyhow, Result};
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use nix::errno::Errno;
use nix::libc;
use std::mem;

const TUXEDO_IO_DEVICE: &str = "/dev/tuxedo_io";
const IOCTL_MAGIC: u8 = 0xEC;
const MAGIC_READ_CL: u8 = IOCTL_MAGIC + 1;
const MAGIC_WRITE_CL: u8 = IOCTL_MAGIC + 2;
const MAGIC_READ_UW: u8 = IOCTL_MAGIC + 3;
const MAGIC_WRITE_UW: u8 = IOCTL_MAGIC + 4;

// Hardware check ioctls
// nix::ioctl_read!(ioctl_cl_hw_interface_id, MAGIC_READ_CL, 0x00, [u8; 30]);
// nix::ioctl_read!(ioctl_hwcheck_cl, IOCTL_MAGIC, 0x05, i32);
// nix::ioctl_read!(ioctl_hwcheck_uw, IOCTL_MAGIC, 0x06, i32);

// Clevo read ioctls
// nix::ioctl_read!(ioctl_cl_faninfo1, MAGIC_READ_CL, 0x10, i32);
// nix::ioctl_read!(ioctl_cl_faninfo2, MAGIC_READ_CL, 0x11, i32);
// nix::ioctl_read!(ioctl_cl_faninfo3, MAGIC_READ_CL, 0x12, i32);
nix::ioctl_read!(ioctl_cl_webcam_sw, MAGIC_READ_CL, 0x13, i32);

// Clevo write ioctls
nix::ioctl_write_ptr!(ioctl_cl_fanspeed, MAGIC_WRITE_CL, 0x10, i32);
nix::ioctl_write_ptr!(ioctl_cl_fanauto, MAGIC_WRITE_CL, 0x11, i32);
nix::ioctl_write_ptr!(ioctl_cl_webcam_sw_w, MAGIC_WRITE_CL, 0x12, i32);
nix::ioctl_write_ptr!(ioctl_cl_perf_profile, MAGIC_WRITE_CL, 0x15, i32);

// Uniwill read ioctls
// nix::ioctl_read!(ioctl_uw_fanspeed, MAGIC_READ_UW, 0x10, i32);
// nix::ioctl_read!(ioctl_uw_fanspeed2, MAGIC_READ_UW, 0x11, i32);
// nix::ioctl_read!(ioctl_uw_fan_temp, MAGIC_READ_UW, 0x12, i32);
// nix::ioctl_read!(ioctl_uw_fan_temp2, MAGIC_READ_UW, 0x13, i32);
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
    fn request_code_read_i32(id: u8, seq: u8) -> libc::c_ulong {
        nix::request_code_read!(id, seq, mem::size_of::<*mut i32>()) as libc::c_ulong
    }

    fn ioctl_read_i32(fd: i32, request: libc::c_ulong) -> Result<i32> {
        let mut data: i32 = 0;
        let res = unsafe { libc::ioctl(fd, request, &mut data as *mut i32) };
        Errno::result(res)
            .map_err(|e| anyhow!("ioctl read failed (req=0x{:x}): {}", request, e))?;
        Ok(data)
    }
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

        let cl_res = Self::ioctl_read_i32(
            fd,
            Self::request_code_read_i32(IOCTL_MAGIC, 0x05),
        );
        let uw_res = Self::ioctl_read_i32(
            fd,
            Self::request_code_read_i32(IOCTL_MAGIC, 0x06),
        );

        if matches!(cl_res, Ok(1)) {
            return Ok(HardwareInterface::Clevo);
        }
        if matches!(uw_res, Ok(1)) {
            return Ok(HardwareInterface::Uniwill);
        }

        let probe_cl = Self::ioctl_read_i32(
            fd,
            Self::request_code_read_i32(MAGIC_READ_CL, 0x10),
        );
        if probe_cl.is_ok() {
            return Ok(HardwareInterface::Clevo);
        }

        let probe_uw = Self::ioctl_read_i32(
            fd,
            Self::request_code_read_i32(MAGIC_READ_UW, 0x10),
        );
        if probe_uw.is_ok() {
            return Ok(HardwareInterface::Uniwill);
        }

        Ok(HardwareInterface::None)
    }

    fn detect_fan_count(
        device: &std::fs::File,
        interface: HardwareInterface,
    ) -> Result<u32> {
        let fd = device.as_raw_fd();

        match interface {
            HardwareInterface::Clevo => {
                let mut count = 0;
                for fan_id in 0..3u32 {
                    let seq = match fan_id {
                        0 => 0x10,
                        1 => 0x11,
                        2 => 0x12,
                        _ => unreachable!(),
                    };
                    let raw = Self::ioctl_read_i32(
                        fd,
                        Self::request_code_read_i32(MAGIC_READ_CL, seq),
                    )?;
                    let temp2 = ((raw >> 16) & 0xFF) as u32;
                    if temp2 <= 1 {
                        break;
                    }
                    count += 1;
                }
                Ok(count)
            }

            HardwareInterface::Uniwill => {
                let r0 = Self::ioctl_read_i32(
                    fd,
                    Self::request_code_read_i32(MAGIC_READ_UW, 0x10),
                );
                if r0.is_err() {
                    return Ok(0);
                }
                let r1 = Self::ioctl_read_i32(
                    fd,
                    Self::request_code_read_i32(MAGIC_READ_UW, 0x11),
                );
                Ok(if r1.is_ok() { 2 } else { 1 })
            }

            HardwareInterface::None => Ok(0),
        }
    }

    // Fan control methods
pub fn get_fan_speed(&self, fan_id: u32) -> Result<u32> {
        let fd = self.device.as_raw_fd();

        match self.interface {
            HardwareInterface::Clevo => {
                let seq = match fan_id {
                    0 => 0x10,
                    1 => 0x11,
                    2 => 0x12,
                    _ => return Err(anyhow!("Invalid fan ID")),
                };

                let raw = Self::ioctl_read_i32(
                    fd,
                    Self::request_code_read_i32(MAGIC_READ_CL, seq),
                )?;

                Ok(Self::clevo_raw_to_percent((raw & 0xFF) as u8))
            }

            HardwareInterface::Uniwill => {
                let seq = match fan_id {
                    0 => 0x10,
                    1 => 0x11,
                    _ => return Err(anyhow!("Invalid fan ID")),
                };
                let val = Self::ioctl_read_i32(
                    fd,
                    Self::request_code_read_i32(MAGIC_READ_UW, seq),
                )?;
                Ok(val as u32)
            }

            HardwareInterface::None => Err(anyhow!("No hardware interface")),
        }
    }

    pub fn set_fan_speed(&self, fan_id: u32, speed_percent: u32) -> Result<()> {
        let fd = self.device.as_raw_fd();

        match self.interface {
            HardwareInterface::Clevo => {
                let mut current_raw = [0u8; 3];

                for i in 0..self.fan_count.min(3) {
                    let seq = match i {
                        0 => 0x10,
                        1 => 0x11,
                        2 => 0x12,
                        _ => unreachable!(),
                    };
                    if let Ok(raw) = Self::ioctl_read_i32(
                        fd,
                        Self::request_code_read_i32(MAGIC_READ_CL, seq),
                    ) {
                        current_raw[i as usize] = (raw & 0xFF) as u8;
                    }
                }

                current_raw[fan_id as usize] =
                    Self::clevo_percent_to_raw(speed_percent);

                let packed = (current_raw[0] as i32)
                    | ((current_raw[1] as i32) << 8)
                    | ((current_raw[2] as i32) << 16);
                
                    log::debug!(
        "Writing Clevo packed raw: {:02x} {:02x} {:02x} (packed=0x{:06x})",
        current_raw[0],
        current_raw[1],
        current_raw[2],
        packed
    );

                unsafe {
                    ioctl_cl_fanspeed(fd, &packed)?;
                }
                Ok(())
            }

            HardwareInterface::Uniwill => {
    let val: i32 = speed_percent.min(200) as i32;

    unsafe {
        match fan_id {
            0 => {
                ioctl_uw_fanspeed_w(fd, &val)?;
            }
            1 => {
                ioctl_uw_fanspeed2_w(fd, &val)?;
            }
            _ => return Err(anyhow!("Invalid fan ID")),
        }
    }

    Ok(())
}

            HardwareInterface::None => Err(anyhow!("No hardware interface")),
        }
    }

    pub fn set_fan_auto(&self) -> Result<()> {
        let fd = self.device.as_raw_fd();

        match self.interface {
            HardwareInterface::Clevo => {
                let auto_val: i32 = 0xF;
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
            HardwareInterface::None => Err(anyhow!("No hardware interface")),
        }
    }

    pub fn get_fan_temperature(&self, fan_id: u32) -> Result<u32> {
        let fd = self.device.as_raw_fd();

        match self.interface {
            HardwareInterface::Clevo => {
                let seq = match fan_id {
                    0 => 0x10,
                    1 => 0x11,
                    2 => 0x12,
                    _ => return Err(anyhow!("Invalid fan ID")),
                };

                let raw = Self::ioctl_read_i32(
                    fd,
                    Self::request_code_read_i32(MAGIC_READ_CL, seq),
                )?;

                let temp2 = ((raw >> 16) & 0xFF) as u32;
                if temp2 <= 1 {
                    return Err(anyhow!("Fan not available"));
                }
                Ok(temp2)
            }

            HardwareInterface::Uniwill => {
                let seq = match fan_id {
                    0 => 0x12,
                    1 => 0x13,
                    _ => return Err(anyhow!("Invalid fan ID")),
                };
                let val = Self::ioctl_read_i32(
                    fd,
                    Self::request_code_read_i32(MAGIC_READ_UW, seq),
                )?;
                Ok(val as u32)
            }

            HardwareInterface::None => Err(anyhow!("No hardware interface")),
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
