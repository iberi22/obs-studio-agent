use anyhow::{Context, Result};
use async_trait::async_trait;
use obs_agent_core::application::ports::{OBSPort, ValidationIssue, ValidationReport};
use obs_agent_core::domain::models::{OBSStats, Scene, Severity, VideoSettings};
use obws::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Adapter para conectar con OBS Studio vía WebSocket
pub struct OBSAdapter {
    host: String,
    port: u16,
    password: Option<String>,
    client: Arc<RwLock<Option<Client>>>,
}

impl OBSAdapter {
    pub fn new(host: impl Into<String>, port: u16, password: Option<String>) -> Self {
        Self {
            host: host.into(),
            port,
            password,
            client: Arc::new(RwLock::new(None)),
        }
    }

    async fn ensure_connected(&self) -> Result<()> {
        let client = self.client.read().await;
        if client.is_none() {
            drop(client);
            self.connect().await?;
        }
        Ok(())
    }

    async fn get_client(&self) -> Result<Arc<RwLock<Option<Client>>>> {
        self.ensure_connected().await?;
        Ok(Arc::clone(&self.client))
    }
}

#[async_trait]
impl OBSPort for OBSAdapter {
    async fn connect(&self) -> Result<()> {
        info!("Connecting to OBS at {}:{}", self.host, self.port);

        let client = Client::connect(&self.host, self.port, self.password.clone())
            .await
            .context("Failed to connect to OBS WebSocket")?;

        *self.client.write().await = Some(client);
        info!("Successfully connected to OBS");
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        info!("Disconnecting from OBS");
        *self.client.write().await = None;
        Ok(())
    }

    async fn get_scenes(&self) -> Result<Vec<Scene>> {
        debug!("Fetching all scenes");
        let client_arc = self.get_client().await?;
        let client = client_arc.read().await;
        let client = client.as_ref().context("Not connected to OBS")?;

        let scenes_response = client.scenes().list()
            .await
            .context("Failed to list scenes")?;

        let mut scenes = Vec::new();
        for scene in scenes_response.scenes {
            scenes.push(Scene {
                name: scene.name.clone(),
                uuid: None, // obws 0.11 no tiene UUID
                sources: vec![], // obws 0.11 no incluye sources en list()
            });
        }

        debug!("Found {} scenes", scenes.len());
        Ok(scenes)
    }

    async fn get_current_scene(&self) -> Result<String> {
        debug!("Getting current scene");
        let client_arc = self.get_client().await?;
        let client = client_arc.read().await;
        let client = client.as_ref().context("Not connected to OBS")?;

        let scene = client.scenes().current_program_scene()
            .await
            .context("Failed to get current scene")?;

        debug!("Current scene: {}", scene);
        Ok(scene)
    }

    async fn validate_scene(&self, scene_name: &str) -> Result<ValidationReport> {
        debug!("Validating scene: {}", scene_name);
        let scenes = self.get_scenes().await?;

        let scene = scenes
            .iter()
            .find(|s| s.name == scene_name)
            .context("Scene not found")?;

        let mut issues = Vec::new();

        // Validar si hay fuentes
        if scene.sources.is_empty() {
            issues.push(ValidationIssue {
                source_name: scene_name.to_string(),
                issue_type: "EmptyScene".to_string(),
                description: "Scene has no sources".to_string(),
                severity: Severity::Warning,
            });
        }

        // TODO: Validar disponibilidad de cada source
        // Esto requeriría queries adicionales a OBS

        let is_valid = issues.iter().all(|i| i.severity != Severity::Critical);

        Ok(ValidationReport {
            scene_name: scene_name.to_string(),
            is_valid,
            issues,
        })
    }

    async fn get_stats(&self) -> Result<OBSStats> {
        debug!("Getting OBS stats");
        let client_arc = self.get_client().await?;
        let client = client_arc.read().await;
        let client = client.as_ref().context("Not connected to OBS")?;

        let stats = client.general().stats()
            .await
            .context("Failed to get stats")?;

        Ok(OBSStats {
            cpu_usage: stats.cpu_usage,
            memory_usage: stats.memory_usage,
            active_fps: stats.active_fps,
            render_skipped_frames: stats.render_skipped_frames as u64,
            render_total_frames: stats.render_total_frames as u64,
            output_skipped_frames: stats.output_skipped_frames as u64,
            output_total_frames: stats.output_total_frames as u64,
        })
    }

    async fn get_video_settings(&self) -> Result<VideoSettings> {
        debug!("Getting video settings");
        let client_arc = self.get_client().await?;
        let client = client_arc.read().await;
        let client = client.as_ref().context("Not connected to OBS")?;

        let settings = client.config().video_settings()
            .await
            .context("Failed to get video settings")?;

        Ok(VideoSettings {
            base_width: settings.base_width,
            base_height: settings.base_height,
            output_width: settings.output_width,
            output_height: settings.output_height,
            fps_numerator: settings.fps_numerator,
            fps_denominator: settings.fps_denominator,
        })
    }

    async fn set_video_settings(&self, settings: &VideoSettings) -> Result<()> {
        info!("Setting video settings: {}x{} @ {} fps",
            settings.output_width, settings.output_height, settings.fps());

        let client_arc = self.get_client().await?;
        let client = client_arc.read().await;
        let client = client.as_ref().context("Not connected to OBS")?;

        client.config().set_video_settings(obws::requests::config::SetVideoSettings {
            base_width: Some(settings.base_width),
            base_height: Some(settings.base_height),
            output_width: Some(settings.output_width),
            output_height: Some(settings.output_height),
            fps_numerator: Some(settings.fps_numerator),
            fps_denominator: Some(settings.fps_denominator),
        })
        .await
        .context("Failed to set video settings")?;

        Ok(())
    }

    async fn take_screenshot(&self, _source: &str) -> Result<Vec<u8>> {
        // Screenshot API cambió en obws 0.11 - temporalmente deshabilitado
        anyhow::bail!("Screenshot functionality not available in obws 0.11 - needs update")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requiere OBS corriendo
    async fn test_obs_connection() {
        let adapter = OBSAdapter::new("localhost", 4455, None);
        let result = adapter.connect().await;
        assert!(result.is_ok());
    }
}
