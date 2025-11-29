use crate::domain::models::*;
use anyhow::Result;
use async_trait::async_trait;

/// Puerto para interactuar con OBS Studio
#[async_trait]
pub trait OBSPort: Send + Sync {
    /// Conecta al servidor WebSocket de OBS
    async fn connect(&self) -> Result<()>;

    /// Desconecta del servidor
    async fn disconnect(&self) -> Result<()>;

    /// Obtiene todas las escenas
    async fn get_scenes(&self) -> Result<Vec<Scene>>;

    /// Obtiene la escena actual
    async fn get_current_scene(&self) -> Result<String>;

    /// Valida una escena (verifica sources, etc)
    async fn validate_scene(&self, scene_name: &str) -> Result<ValidationReport>;

    /// Obtiene estadísticas de OBS
    async fn get_stats(&self) -> Result<OBSStats>;

    /// Obtiene configuración de video
    async fn get_video_settings(&self) -> Result<VideoSettings>;

    /// Configura ajustes de video
    async fn set_video_settings(&self, settings: &VideoSettings) -> Result<()>;

    /// Toma screenshot de una fuente
    async fn take_screenshot(&self, source: &str) -> Result<Vec<u8>>;
}

/// Reporte de validación de escena
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub scene_name: String,
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub source_name: String,
    pub issue_type: String,
    pub description: String,
    pub severity: crate::domain::models::Severity,
}
