use anyhow::{Context, Result};
use obs_agent_core::application::ports::MonitorPort;
use obs_agent_core::domain::models::{
    CPUInfo, EncoderType, GPUInfo, GPUVendor, HardwareInfo, RAMInfo,
};
use sysinfo::System;
use tracing::{debug, info};

/// Adapter para monitorear hardware del sistema
pub struct MonitorAdapter {
    system: System,
}

impl MonitorAdapter {
    pub fn new() -> Self {
        let system = System::new_all();
        Self { system }
    }

    #[allow(dead_code)]
    fn refresh(&mut self) {
        self.system.refresh_all();
    }

    fn detect_gpu_vendor(name: &str) -> GPUVendor {
        let name_lower = name.to_lowercase();
        if name_lower.contains("nvidia") || name_lower.contains("geforce") || name_lower.contains("rtx") {
            GPUVendor::NVIDIA
        } else if name_lower.contains("amd") || name_lower.contains("radeon") {
            GPUVendor::AMD
        } else if name_lower.contains("intel") {
            GPUVendor::Intel
        } else {
            GPUVendor::Unknown
        }
    }

    fn detect_encoder(gpu_vendor: GPUVendor, cpu_cores: usize) -> EncoderType {
        match gpu_vendor {
            GPUVendor::NVIDIA => EncoderType::NVENC,
            GPUVendor::AMD => EncoderType::AMF,
            GPUVendor::Intel if cpu_cores >= 4 => EncoderType::QSV,
            _ => EncoderType::X264,
        }
    }

    fn recommend_preset(gpu_vendor: GPUVendor, cpu_cores: usize) -> String {
        match gpu_vendor {
            GPUVendor::NVIDIA => "p5".to_string(), // NVENC preset
            GPUVendor::AMD => "balanced".to_string(),
            _ if cpu_cores >= 8 => "medium".to_string(),
            _ => "veryfast".to_string(),
        }
    }

    fn recommend_resolution(ram_gb: f64, has_hw_encoder: bool) -> (u32, u32) {
        if has_hw_encoder && ram_gb >= 16.0 {
            (1920, 1080) // 1080p
        } else if ram_gb >= 8.0 {
            (1280, 720) // 720p
        } else {
            (854, 480) // 480p
        }
    }

    fn recommend_fps(cpu_cores: usize, has_hw_encoder: bool) -> u32 {
        if has_hw_encoder || cpu_cores >= 8 {
            60
        } else {
            30
        }
    }

    fn recommend_bitrate(resolution: (u32, u32), fps: u32) -> u32 {
        match (resolution, fps) {
            ((1920, 1080), 60) => 6000,
            ((1920, 1080), 30) => 4500,
            ((1280, 720), 60) => 4500,
            ((1280, 720), 30) => 3000,
            _ => 2500,
        }
    }
}

impl Default for MonitorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitorPort for MonitorAdapter {
    fn get_cpu_temp(&self) -> Result<f32> {
        // sysinfo no proporciona temperaturas directamente
        // Necesitaríamos usar una librería específica de plataforma
        debug!("CPU temperature monitoring not yet implemented");
        Ok(0.0)
    }

    fn get_gpu_temp(&self) -> Result<f32> {
        #[cfg(feature = "nvidia")]
        {
            use nvml_wrapper::Nvml;
            let nvml = Nvml::init().context("Failed to initialize NVML")?;
            let device = nvml.device_by_index(0).context("Failed to get GPU device")?;
            let temp = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .context("Failed to get GPU temperature")?;
            return Ok(temp as f32);
        }

        #[cfg(not(feature = "nvidia"))]
        {
            debug!("GPU temperature monitoring requires 'nvidia' feature");
            Ok(0.0)
        }
    }

    fn get_cpu_usage(&self) -> Result<f32> {
        let usage = self.system.global_cpu_info().cpu_usage();
        Ok(usage)
    }

    fn get_memory_info(&self) -> Result<RAMInfo> {
        let total_gb = self.system.total_memory() as f64 / 1_073_741_824.0;
        let available_gb = self.system.available_memory() as f64 / 1_073_741_824.0;
        let used_percent = ((total_gb - available_gb) / total_gb) * 100.0;

        Ok(RAMInfo {
            total_gb,
            available_gb,
            used_percent,
        })
    }

    fn get_disk_space(&self) -> Result<super::DiskInfo> {
        // Obtener info del primer disco disponible
        let disks = sysinfo::Disks::new_with_refreshed_list();

        if let Some(disk) = disks.iter().next() {
            let total_gb = disk.total_space() as f64 / 1_073_741_824.0;
            let free_gb = disk.available_space() as f64 / 1_073_741_824.0;
            let used_percent = ((total_gb - free_gb) / total_gb) * 100.0;

            Ok(super::DiskInfo {
                total_gb,
                free_gb,
                used_percent,
            })
        } else {
            anyhow::bail!("No disks found");
        }
    }

    fn detect_hardware(&self) -> Result<HardwareInfo> {
        info!("Detecting hardware...");

        // CPU Info
        let cpu = self.system.cpus().first().context("No CPU found")?;
        let cpu_info = CPUInfo {
            name: cpu.name().to_string(),
            brand: cpu.brand().to_string(),
            cores_physical: self.system.physical_core_count().unwrap_or(1),
            cores_logical: self.system.cpus().len(),
            frequency_mhz: cpu.frequency() as f64,
            arch: std::env::consts::ARCH.to_string(),
        };

        // GPU Info (básico - necesitaría vulkano o wgpu para detección completa)
        let gpu_name = if cfg!(target_os = "windows") {
            // En Windows podríamos usar WMI
            "Unknown GPU".to_string()
        } else {
            "Unknown GPU".to_string()
        };

        let gpu_vendor = Self::detect_gpu_vendor(&gpu_name);
        let gpu_info = Some(GPUInfo {
            name: gpu_name,
            vendor: gpu_vendor,
            memory_total_mb: 0,
            memory_free_mb: 0,
            driver_version: "Unknown".to_string(),
            supports_nvenc: gpu_vendor == GPUVendor::NVIDIA,
            supports_nvenc_hevc: gpu_vendor == GPUVendor::NVIDIA,
            supports_amf: gpu_vendor == GPUVendor::AMD,
            supports_qsv: gpu_vendor == GPUVendor::Intel,
        });

        // RAM Info
        let total_gb = self.system.total_memory() as f64 / 1_073_741_824.0;
        let available_gb = self.system.available_memory() as f64 / 1_073_741_824.0;
        let ram_info = RAMInfo {
            total_gb,
            available_gb,
            used_percent: ((total_gb - available_gb) / total_gb) * 100.0,
        };

        // Recomendaciones
        let has_hw_encoder = gpu_vendor != GPUVendor::Unknown;
        let recommended_encoder = Self::detect_encoder(gpu_vendor, cpu_info.cores_physical);
        let recommended_preset = Self::recommend_preset(gpu_vendor, cpu_info.cores_physical);
        let recommended_resolution = Self::recommend_resolution(total_gb, has_hw_encoder);
        let recommended_fps = Self::recommend_fps(cpu_info.cores_physical, has_hw_encoder);
        let recommended_bitrate = Self::recommend_bitrate(recommended_resolution, recommended_fps);

        let hardware = HardwareInfo {
            os: std::env::consts::OS.to_string(),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            cpu: cpu_info,
            gpu: gpu_info,
            ram: ram_info,
            recommended_encoder,
            recommended_preset,
            recommended_resolution,
            recommended_fps,
            recommended_bitrate,
        };

        info!("Hardware detection complete: {} cores, {:.1} GB RAM",
            hardware.cpu.cores_physical, hardware.ram.total_gb);

        Ok(hardware)
    }
}

// Re-export DiskInfo desde monitor_port
pub use obs_agent_core::application::ports::DiskInfo;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let adapter = MonitorAdapter::new();
        let hardware = adapter.detect_hardware().unwrap();

        assert!(hardware.cpu.cores_physical > 0);
        assert!(hardware.ram.total_gb > 0.0);
    }
}
