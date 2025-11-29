use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

/// Puerto para interactuar con plugins Python
#[async_trait]
pub trait PluginPort: Send + Sync {
    /// Llama a un plugin Python genérico
    async fn call_python_plugin(&self, name: &str, args: Value) -> Result<Value>;

    /// OCR - Extrae texto de imagen
    async fn ocr_analyze(&self, image: &[u8]) -> Result<String>;

    /// Análisis de audio
    async fn audio_analyze(&self, audio: &[u8]) -> Result<AudioAnalysis>;

    /// Genera video
    async fn generate_video(&self, spec: &VideoSpec) -> Result<PathBuf>;

    /// Genera overlay
    async fn generate_overlay(&self, design: &OverlayDesign) -> Result<PathBuf>;

    /// Text-to-Speech
    async fn generate_tts(&self, text: &str, voice: &str) -> Result<PathBuf>;
}

/// Análisis de audio
#[derive(Debug, Clone)]
pub struct AudioAnalysis {
    pub peak_db: f32,
    pub rms_db: f32,
    pub is_clipping: bool,
    pub silence_percent: f32,
    pub frequency_spectrum: Vec<f32>,
}

/// Especificación de video
#[derive(Debug, Clone)]
pub struct VideoSpec {
    pub template: String,
    pub duration_secs: f32,
    pub text: String,
    pub style: String,
    pub resolution: (u32, u32),
    pub fps: u32,
}

/// Diseño de overlay (re-export para conveniencia)
pub use crate::application::ports::ai_port::OverlayDesign;
