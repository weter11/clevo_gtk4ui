use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time;
use tuxedo_common::types::FanSettings;

pub struct FanDaemon {
    settings: Option<FanSettings>,
    running: bool,
}

impl FanDaemon {
    pub fn new() -> Self {
        Self {
            settings: None,
            running: false,
        }
    }
    
    pub fn update_settings(&mut self, settings: FanSettings) {
        self.settings = Some(settings);
    }
    
    pub async fn run(&mut self) {
        self.running = true;
        let mut interval = time::interval(Duration::from_secs(2));
        
        while self.running {
            interval.tick().await;
            
            if let Some(ref settings) = self.settings {
                if settings.control_enabled {
                    if let Err(e) = self.apply_fan_curves(settings) {
                        log::error!("Failed to apply fan curves: {}", e);
                    }
                }
            }
        }
    }
    
    pub fn stop(&mut self) {
        self.running = false;
    }
    
    fn apply_fan_curves(&self, settings: &FanSettings) -> Result<()> {
        let tuxedo_io_path = "/sys/devices/platform/tuxedo_io";
        
        if !Path::new(tuxedo_io_path).exists() {
            return Ok(());
        }
        
        for curve in &settings.curves {
            // Read current temperature for this fan
            let temp = self.read_fan_temperature(curve.fan_id)?;
            
            // Find appropriate speed based on curve
            let speed = self.calculate_fan_speed(temp, &curve.points);
            
            // Apply speed
            let speed_path = format!("{}/fan{}_speed", tuxedo_io_path, curve.fan_id);
            if Path::new(&speed_path).exists() {
                fs::write(&speed_path, speed.to_string())?;
            }
        }
        
        Ok(())
    }
    
    fn read_fan_temperature(&self, fan_id: u32) -> Result<u8> {
        // TODO: Map fan to appropriate temperature sensor
        // For now, use package temperature
        for entry in fs::read_dir("/sys/class/hwmon")? {
            let entry = entry?;
            let name_path = entry.path().join("name");
            if let Ok(name) = fs::read_to_string(&name_path) {
                if name.trim() == "coretemp" || name.trim() == "k10temp" {
                    let temp_path = entry.path().join("temp1_input");
                    if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                        if let Ok(temp) = temp_str.trim().parse::<f32>() {
                            return Ok((temp / 1000.0) as u8);
                        }
                    }
                }
            }
        }
        
        Ok(50) // Default fallback
    }
    
    fn calculate_fan_speed(&self, temp: u8, points: &[(u8, u8)]) -> u8 {
        if points.is_empty() {
            return 50; // Default
        }
        
        // Find the two points to interpolate between
        for i in 0..points.len() - 1 {
            let (temp1, speed1) = points[i];
            let (temp2, speed2) = points[i + 1];
            
            if temp >= temp1 && temp <= temp2 {
                // Linear interpolation
                let temp_range = temp2 - temp1;
                if temp_range == 0 {
                    return speed1;
                }
                
                let temp_offset = temp - temp1;
                let speed_range = speed2 as i16 - speed1 as i16;
                let speed = speed1 as i16 + (speed_range * temp_offset as i16) / temp_range as i16;
                
                return speed.clamp(0, 100) as u8;
            }
        }
        
        // If temp is below first point
        if temp < points[0].0 {
            return points[0].1;
        }
        
        // If temp is above last point
        points.last().unwrap().1
    }
}
