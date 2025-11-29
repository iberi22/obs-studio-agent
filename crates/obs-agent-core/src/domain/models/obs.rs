use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Escena de OBS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub uuid: Option<String>,
    pub sources: Vec<Source>,
}

/// Fuente/Input de OBS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    pub uuid: Option<String>,
    pub kind: String,
    pub is_available: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Estadísticas de OBS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OBSStats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_fps: f64,
    pub render_skipped_frames: u64,
    pub render_total_frames: u64,
    pub output_skipped_frames: u64,
    pub output_total_frames: u64,
}

impl OBSStats {
    pub fn dropped_frames_percent(&self) -> f64 {
        if self.output_total_frames == 0 {
            return 0.0;
        }
        (self.output_skipped_frames as f64 / self.output_total_frames as f64) * 100.0
    }

    pub fn is_dropping_frames(&self) -> bool {
        self.dropped_frames_percent() > 1.0 // Más del 1% es problemático
    }
}

/// Configuración de video de OBS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub base_width: u32,
    pub base_height: u32,
    pub output_width: u32,
    pub output_height: u32,
    pub fps_numerator: u32,
    pub fps_denominator: u32,
}

impl VideoSettings {
    pub fn fps(&self) -> f64 {
        self.fps_numerator as f64 / self.fps_denominator as f64
    }
}
