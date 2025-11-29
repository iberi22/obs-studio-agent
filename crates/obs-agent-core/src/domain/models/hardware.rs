use serde::{Deserialize, Serialize};

/// Información de CPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUInfo {
    pub name: String,
    pub brand: String,
    pub cores_physical: usize,
    pub cores_logical: usize,
    pub frequency_mhz: f64,
    pub arch: String,
}

/// Vendor de GPU
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GPUVendor {
    NVIDIA,
    AMD,
    Intel,
    Unknown,
}

/// Información de GPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUInfo {
    pub name: String,
    pub vendor: GPUVendor,
    pub memory_total_mb: u64,
    pub memory_free_mb: u64,
    pub driver_version: String,
    pub supports_nvenc: bool,
    pub supports_nvenc_hevc: bool,
    pub supports_amf: bool,
    pub supports_qsv: bool,
}

/// Información de RAM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAMInfo {
    pub total_gb: f64,
    pub available_gb: f64,
    pub used_percent: f64,
}

/// Tipo de encoder recomendado
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncoderType {
    NVENC,     // NVIDIA
    AMF,       // AMD
    QSV,       // Intel Quick Sync
    X264,      // Software (CPU)
    X265,      // Software HEVC
}

/// Información completa del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub os: String,
    pub os_version: String,
    pub hostname: String,
    pub cpu: CPUInfo,
    pub gpu: Option<GPUInfo>,
    pub ram: RAMInfo,
    pub recommended_encoder: EncoderType,
    pub recommended_preset: String,
    pub recommended_resolution: (u32, u32),
    pub recommended_fps: u32,
    pub recommended_bitrate: u32,
}

impl HardwareInfo {
    pub fn has_hardware_encoder(&self) -> bool {
        self.gpu.as_ref().map_or(false, |gpu| {
            gpu.supports_nvenc || gpu.supports_amf || gpu.supports_qsv
        })
    }

    pub fn can_handle_1080p(&self) -> bool {
        self.has_hardware_encoder() && self.ram.total_gb >= 16.0
    }

    pub fn can_handle_60fps(&self) -> bool {
        self.has_hardware_encoder() || self.cpu.cores_physical >= 8
    }
}
