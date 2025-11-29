use crate::domain::models::*;
use anyhow::Result;

/// Puerto para monitorear hardware del sistema
pub trait MonitorPort: Send + Sync {
    /// Obtiene temperatura de CPU
    fn get_cpu_temp(&self) -> Result<f32>;

    /// Obtiene temperatura de GPU
    fn get_gpu_temp(&self) -> Result<f32>;

    /// Obtiene uso de CPU
    fn get_cpu_usage(&self) -> Result<f32>;

    /// Obtiene información de memoria
    fn get_memory_info(&self) -> Result<RAMInfo>;

    /// Obtiene información de disco
    fn get_disk_space(&self) -> Result<DiskInfo>;

    /// Detecta todo el hardware
    fn detect_hardware(&self) -> Result<HardwareInfo>;
}

/// Información de disco
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_percent: f64,
}

impl DiskInfo {
    pub fn is_low_space(&self) -> bool {
        self.free_gb < 10.0 || self.used_percent > 90.0
    }
}
