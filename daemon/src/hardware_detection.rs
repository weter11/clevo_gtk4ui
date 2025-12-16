use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use tuxedo_common::types::*;

pub fn get_system_info() -> Result<SystemInfo> {
    Ok(SystemInfo {
        product_name: read_dmi("product_name")?,
        manufacturer: read_dmi("sys_vendor")?,
        bios_version: read_dmi("bios_version")?,
    })
}

fn read_dmi(file: &str) -> Result<String> {
    let path = format!("/sys/class/dmi/id/{}", file);
    fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|e| anyhow!("Failed to read {}: {}", path, e))
}

pub fn get_cpu_info() -> Result<CpuInfo> {
    let name = get_cpu_name()?;
    let core_count = get_cpu_count()?;
    
    let mut cores = Vec::new();
    let mut frequencies = Vec::new();
    
    for i in 0..core_count {
        let freq = read_cpu_frequency(i)?;
        frequencies.push(freq);
        cores.push(CoreInfo {
            id: i,
            frequency: freq,
            load: 0.0, // TODO: Implement load calculation
            temperature: get_core_temp(i).unwrap_or(0.0),
        });
    }
    
    let median_frequency = calculate_median(&frequencies);
    let package_temp = get_package_temp().unwrap_or(0.0);
    let package_power = get_cpu_power();
    
    let governor = read_governor()?;
    let available_governors = read_available_governors()?;
    
    let boost_enabled = is_boost_enabled()?;
    let smt_enabled = is_smt_enabled()?;
    
    let scaling_driver = read_scaling_driver()?;
    let amd_pstate_status = read_amd_pstate_status().ok();
    
    let (min_freq, max_freq) = read_frequency_limits();
    let (hw_min_freq, hw_max_freq) = read_hw_frequency_limits()?;

    // Get all power sources
    let all_power_sources = get_all_power_sources();
    
    // Choose the primary power source (you can adjust this based on your logic)
    let power_source = all_power_sources.iter().find(|s| s.name == "RAPL")
    .cloned()  // .cloned() to clone the `PowerSource` from the reference
    .unwrap_or_else(|| PowerSource {
        name: "Unknown".to_string(),
        value: 0.0,
        description: "No valid power source found".to_string(),
    });

let power_source_name = Some(power_source.name);  // Extract just the `name` field as an Option<String>

Ok(CpuInfo {
    name,
    median_frequency,
    median_load: 0.0,
    package_temp,
    package_power,
    cores,
    governor,
    available_governors,
    boost_enabled,
    smt_enabled,
    scaling_driver,
    amd_pstate_status,
    min_freq,
    max_freq,
    hw_min_freq,
    hw_max_freq,
    all_power_sources,
    power_source: power_source_name,  // Pass the Option<String> here
})

}

fn get_cpu_name() -> Result<String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;
    for line in cpuinfo.lines() {
        if line.starts_with("model name") {
            if let Some(name) = line.split(':').nth(1) {
                return Ok(name.trim().to_string());
            }
        }
    }
    Err(anyhow!("CPU name not found"))
}

fn get_cpu_count() -> Result<u32> {
    let mut count = 0;
    for i in 0..1024 {
        let path = format!("/sys/devices/system/cpu/cpu{}", i);
        if !Path::new(&path).exists() {
            break;
        }
        count += 1;
    }
    Ok(count)
}

fn read_cpu_frequency(cpu: u32) -> Result<u64> {
    let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", cpu);
    fs::read_to_string(&path)?
        .trim()
        .parse()
        .map_err(|e| anyhow!("Failed to parse frequency: {}", e))
}

fn calculate_median(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

fn get_core_temp(cpu: u32) -> Result<f32> {
    // Try to find temperature for specific core
    for entry in fs::read_dir("/sys/class/hwmon")? {
        let entry = entry?;
        let name_path = entry.path().join("name");
        if let Ok(name) = fs::read_to_string(&name_path) {
            if name.trim() == "coretemp" || name.trim() == "zenpower" {
                let temp_path = entry.path().join(format!("temp{}_input", cpu + 2));
                if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                    if let Ok(temp) = temp_str.trim().parse::<f32>() {
                        return Ok(temp / 1000.0);
                    }
                }
            }
        }
    }
    Err(anyhow!("Core temperature not found"))
}

fn get_package_temp() -> Result<f32> {
    for entry in fs::read_dir("/sys/class/hwmon")? {
        let entry = entry?;
        let name_path = entry.path().join("name");
        if let Ok(name) = fs::read_to_string(&name_path) {
            if name.trim() == "coretemp" || name.trim() == "zenpower" {
                let temp_path = entry.path().join("temp1_input");
                if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                    if let Ok(temp) = temp_str.trim().parse::<f32>() {
                        return Ok(temp / 1000.0);
                    }
                }
            }
        }
    }
    Err(anyhow!("Package temperature not found"))
}

fn get_cpu_power() -> Option<f32> {
    let all_sources = get_all_power_sources();
    
    // Priority 1: Try RAPL (Intel/AMD modern CPUs)
    if let Some(rapl) = all_sources.iter().find(|s| s.name == "RAPL") {
        return Some(rapl.value);
    }
    
    // Priority 2: Try AMD-specific hwmon power
    // Note: For AMD APUs (CPU+iGPU), this shows total power consumption
    // Only use if no discrete GPU present to avoid confusion
    if is_amd_cpu() && get_amd_dgpu_count() == 0 {
        if let Some(amdgpu) = all_sources.iter().find(|s| s.name == "amdgpu") {
            return Some(amdgpu.value);
        }
        if let Some(zenpower) = all_sources.iter().find(|s| s.name == "zenpower") {
            return Some(zenpower.value);
        }
    }
    
    None
}

fn get_all_power_sources() -> Vec<PowerSource> {
    let mut sources = Vec::new();
    
    // Try RAPL
    if let Ok(power) = try_rapl() {
        sources.push(PowerSource {
            name: "RAPL".to_string(),
            value: power,
            description: "Intel/AMD RAPL (Running Average Power Limit)".to_string(),
        });
    }
    
    // Try AMD hwmon sources
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let name_path = entry.path().join("name");
            if let Ok(name) = fs::read_to_string(&name_path) {
                let name = name.trim();
                
                match name {
                    "amdgpu" => {
                        if let Ok(power) = read_hwmon_power(&entry.path()) {
                            sources.push(PowerSource {
                                name: "amdgpu".to_string(),
                                value: power,
                                description: "AMD APU Total Power (CPU+iGPU)".to_string(),
                            });
                        }
                    },
                    "zenpower" => {
                        if let Ok(power) = read_hwmon_power(&entry.path()) {
                            sources.push(PowerSource {
                                name: "zenpower".to_string(),
                                value: power,
                                description: "Zenpower Driver (AMD Ryzen)".to_string(),
                            });
                        }
                    },
                    "amd_energy" => {
                        if let Ok(power) = read_hwmon_power(&entry.path()) {
                            sources.push(PowerSource {
                                name: "amd_energy".to_string(),
                                value: power,
                                description: "AMD Energy Driver".to_string(),
                            });
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    
    sources
}

fn read_hwmon_power(hwmon_path: &std::path::Path) -> Result<f32> {
    let power_path = hwmon_path.join("power1_average");
    if let Ok(power_str) = fs::read_to_string(&power_path) {
        if let Ok(microwatts) = power_str.trim().parse::<f32>() {
            return Ok(microwatts / 1_000_000.0);
        }
    }
    Err(anyhow!("Power reading not available"))
}

fn is_amd_cpu() -> bool {
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("vendor_id") && line.contains("AuthenticAMD") {
                return true;
            }
        }
    }
    false
}

fn get_amd_dgpu_count() -> u32 {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Check for discrete GPU (renderD129+, card1+)
                if name.starts_with("renderD") {
                    if let Ok(num) = name.trim_start_matches("renderD").parse::<u32>() {
                        if num > 128 {
                            // Check if it's AMD
                            let vendor_path = path.join("device/vendor");
                            if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                                if vendor.trim() == "0x1002" {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    count
}

fn try_rapl() -> Result<f32> {
    let path = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";
    if !Path::new(path).exists() {
        return Err(anyhow!("RAPL not available"));
    }
    
    let energy1: u64 = fs::read_to_string(path)?.trim().parse()?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let energy2: u64 = fs::read_to_string(path)?.trim().parse()?;
    
    let energy_diff = energy2.saturating_sub(energy1) as f32;
    let watts = (energy_diff / 100000.0) * 10.0; // Convert to watts
    
    Ok(watts)
}

fn try_amd_hwmon_power() -> Result<f32> {
    // For AMD APUs, look for amdgpu power reading
    // This represents CPU+iGPU total power on APU systems
    for entry in fs::read_dir("/sys/class/hwmon")? {
        let entry = entry?;
        let name_path = entry.path().join("name");
        if let Ok(name) = fs::read_to_string(&name_path) {
            let name = name.trim();
            // Only use amdgpu power if it's the integrated GPU
            if name == "amdgpu" {
                let power_path = entry.path().join("power1_average");
                if let Ok(power_str) = fs::read_to_string(&power_path) {
                    if let Ok(microwatts) = power_str.trim().parse::<f32>() {
                        return Ok(microwatts / 1_000_000.0);
                    }
                }
            }
        }
    }
    Err(anyhow!("AMD hwmon power not available"))
}

fn read_governor() -> Result<String> {
    fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .map(|s| s.trim().to_string())
        .map_err(|e| anyhow!("Failed to read governor: {}", e))
}

fn read_available_governors() -> Result<Vec<String>> {
    let governors = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors")?;
    Ok(governors.split_whitespace().map(String::from).collect())
}

fn is_boost_enabled() -> Result<bool> {
    // Try AMD boost
    if let Ok(boost) = fs::read_to_string("/sys/devices/system/cpu/cpufreq/boost") {
        return Ok(boost.trim() == "1");
    }
    
    // Try Intel turbo (inverted logic)
    if let Ok(no_turbo) = fs::read_to_string("/sys/devices/system/cpu/intel_pstate/no_turbo") {
        return Ok(no_turbo.trim() == "0");
    }
    
    Err(anyhow!("Boost status not available"))
}

fn is_smt_enabled() -> Result<bool> {
    let status = fs::read_to_string("/sys/devices/system/cpu/smt/control")?;
    Ok(status.trim() == "on")
}

fn read_scaling_driver() -> Result<String> {
    fs::read_to_string("/sys/devices/system/cpu/cpufreq/policy0/scaling_driver")
        .map(|s| s.trim().to_string())
        .map_err(|e| anyhow!("Failed to read scaling driver: {}", e))
}

fn read_amd_pstate_status() -> Result<String> {
    fs::read_to_string("/sys/devices/system/cpu/amd_pstate/status")
        .map(|s| s.trim().to_string())
        .map_err(|e| anyhow!("AMD pstate not available: {}", e))
}

fn read_frequency_limits() -> (Option<u64>, Option<u64>) {
    let min_freq = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq")
        .ok()
        .and_then(|s| s.trim().parse().ok());
    
    let max_freq = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq")
        .ok()
        .and_then(|s| s.trim().parse().ok());
    
    (min_freq, max_freq)
}

fn read_hw_frequency_limits() -> Result<(u64, u64)> {
    let min_freq: u64 = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq")?
        .trim()
        .parse()?;
    
    let max_freq: u64 = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")?
        .trim()
        .parse()?;
    
    Ok((min_freq, max_freq))
}

pub fn get_gpu_info() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();
    
    // Try renderD devices first
    for i in 128..132 {
        let render_path = format!("/sys/class/drm/renderD{}", i);
        if Path::new(&render_path).exists() {
            if let Ok(gpu) = detect_gpu(&render_path, i - 128) {
                gpus.push(gpu);
            }
        }
    }
    
    // Fallback to card devices
    if gpus.is_empty() {
        for i in 0..4 {
            let card_path = format!("/sys/class/drm/card{}", i);
            if Path::new(&card_path).exists() {
                if let Ok(gpu) = detect_gpu(&card_path, i) {
                    gpus.push(gpu);
                }
            }
        }
    }
    
    Ok(gpus)
}

fn detect_gpu(path: &str, id: u32) -> Result<GpuInfo> {
    let device_path = format!("{}/device", path);
    
    let _vendor = fs::read_to_string(format!("{}/vendor", device_path))
        .unwrap_or_default()
        .trim()
        .to_string();
    
    let gpu_type = if id == 0 {
        GpuType::Integrated
    } else {
        GpuType::Discrete
    };
    
    let status = fs::read_to_string(format!("{}/power/runtime_status", device_path))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    Ok(GpuInfo {
        name: format!("GPU {}", id),
        gpu_type,
        status,
        frequency: None,
        temperature: None,
        load: None,
        power: None,
        voltage: None,
    })
}
