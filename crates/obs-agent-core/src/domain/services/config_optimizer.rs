use crate::application::ports::{AIPort, OBSConfig};
use crate::domain::models::HardwareInfo;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

/// Servicio para optimizar configuración de OBS
pub struct ConfigOptimizer {
    ai_port: Arc<dyn AIPort>,
}

impl ConfigOptimizer {
    pub fn new(ai_port: Arc<dyn AIPort>) -> Self {
        Self { ai_port }
    }

    /// Optimiza configuración basada en hardware
    pub async fn optimize(&self, hardware: &HardwareInfo) -> Result<OBSConfig> {
        info!("Optimizing OBS config for hardware");
        self.ai_port.optimize_settings(hardware).await
    }

    /// Valida si una configuración es apropiada para el hardware
    pub fn validate_config(&self, config: &OBSConfig, hardware: &HardwareInfo) -> bool {
        // Validar resolución
        let max_res = if hardware.can_handle_1080p() {
            (1920, 1080)
        } else {
            (1280, 720)
        };

        if config.video.output_width > max_res.0 || config.video.output_height > max_res.1 {
            return false;
        }

        // Validar FPS
        let max_fps = if hardware.can_handle_60fps() { 60 } else { 30 };
        if config.video.fps() > max_fps as f64 {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests requieren implementación de mock AIPort
}
