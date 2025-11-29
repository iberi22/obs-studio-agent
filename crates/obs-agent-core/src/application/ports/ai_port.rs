use crate::domain::models::*;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Puerto para interactuar con sistemas de IA/LLM
#[async_trait]
pub trait AIPort: Send + Sync {
    /// Analiza una configuración de OBS
    async fn analyze_config(&self, hardware: &HardwareInfo) -> Result<ConfigAnalysis>;

    /// Sugiere una solución para una anomalía
    async fn suggest_fix(&self, anomaly: &Anomaly) -> Result<String>;

    /// Analiza una imagen
    async fn analyze_image(&self, image: &[u8]) -> Result<ImageAnalysis>;

    /// Optimiza configuración basada en hardware
    async fn optimize_settings(&self, hardware: &HardwareInfo) -> Result<OBSConfig>;

    /// Genera diseño de overlay
    async fn generate_overlay_design(&self, prompt: &str) -> Result<OverlayDesign>;

    /// Genera contenido (genérico)
    async fn generate(&self, prompt: &str) -> Result<String>;
}

/// Análisis de configuración
#[derive(Debug, Clone)]
pub struct ConfigAnalysis {
    pub is_optimal: bool,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub required_plugins: Vec<PluginInfo>,
}

/// Análisis de imagen
#[derive(Debug, Clone)]
pub struct ImageAnalysis {
    pub description: String,
    pub elements: Vec<UIElement>,
    pub text_detected: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UIElement {
    pub element_type: String,
    pub label: String,
    pub bounds: (u32, u32, u32, u32), // x, y, width, height
}

/// Configuración completa de OBS
#[derive(Debug, Clone)]
pub struct OBSConfig {
    pub video: VideoSettings,
    pub encoder: String,
    pub preset: String,
    pub bitrate: u32,
    pub audio_settings: Value,
}

/// Diseño de overlay
#[derive(Debug, Clone)]
pub struct OverlayDesign {
    pub colors: Vec<String>,
    pub layout: String,
    pub elements: Vec<DesignElement>,
}

#[derive(Debug, Clone)]
pub struct DesignElement {
    pub element_type: String,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub style: Value,
}

/// Información de plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub description: String,
}
