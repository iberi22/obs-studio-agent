use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use obs_agent_core::application::ports::{
    AIPort, ConfigAnalysis, ImageAnalysis, OBSConfig, OverlayDesign,
};
use obs_agent_core::domain::models::{Anomaly, HardwareInfo, VideoSettings};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use tracing::{debug, info};

/// Adapter para interactuar con Google Gemini API
pub struct AIAdapter {
    api_key: String,
    model: String,
    client: Client,
    last_request: Arc<Mutex<Instant>>,
    rate_limit: Duration,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Debug, Serialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: String,
}

impl AIAdapter {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "gemini-pro".to_string(),
            client: Client::new(),
            last_request: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(60))), // Allow first request immediately
            rate_limit: Duration::from_millis(1500), // Example: 1.5 seconds between requests
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    async fn generate_content(&self, prompt: &str) -> Result<String> {
        self.enforce_rate_limit().await?;
        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models/{}:generateContent",
            self.model
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part::Text {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: None,
        };

        let response = self
            .client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Gemini")?;

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .context("Failed to parse Gemini response")?;

        let text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .context("No response from Gemini")?;

        Ok(text)
    }

    async fn analyze_image_with_prompt(&self, image: &[u8], prompt: &str) -> Result<String> {
        self.enforce_rate_limit().await?;
        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models/gemini-pro-vision:generateContent"
        );

        let base64_image = general_purpose::STANDARD.encode(image);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![
                    Part::Text {
                        text: prompt.to_string(),
                    },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: "image/png".to_string(),
                            data: base64_image,
                        },
                    },
                ],
            }],
            generation_config: Some(GenerationConfig {
                temperature: Some(0.4),
                max_output_tokens: Some(1024),
            }),
        };

        let response = self
            .client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send image analysis request")?;

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .context("Failed to parse image analysis response")?;

        let text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .context("No response from Gemini")?;

        Ok(text)
    }

    async fn enforce_rate_limit(&self) -> Result<()> {
        let mut last_request = self.last_request.lock().await;
        let elapsed = last_request.elapsed();

        if elapsed < self.rate_limit {
            let sleep_duration = self.rate_limit - elapsed;
            tokio::time::sleep(sleep_duration).await;
        }

        *last_request = Instant::now();
        Ok(())
    }
}

#[async_trait]
impl AIPort for AIAdapter {
    async fn analyze_config(&self, hardware: &HardwareInfo) -> Result<ConfigAnalysis> {
        debug!("Analyzing OBS config with AI");

        let prompt = format!(
            "Analyze this hardware configuration for OBS streaming:\n\
             CPU: {} ({} cores)\n\
             GPU: {:?}\n\
             RAM: {:.1} GB\n\
             \n\
             Recommended encoder: {:?}\n\
             Recommended preset: {}\n\
             Recommended resolution: {}x{}\n\
             Recommended FPS: {}\n\
             Recommended bitrate: {} kbps\n\
             \n\
             Provide:\n\
             1. Is this configuration optimal? (yes/no)\n\
             2. List any issues\n\
             3. List recommendations\n\
             4. List required plugins\n\
             \n\
             Format as JSON with keys: is_optimal, issues, recommendations, required_plugins",
            hardware.cpu.brand,
            hardware.cpu.cores_physical,
            hardware.gpu,
            hardware.ram.total_gb,
            hardware.recommended_encoder,
            hardware.recommended_preset,
            hardware.recommended_resolution.0,
            hardware.recommended_resolution.1,
            hardware.recommended_fps,
            hardware.recommended_bitrate
        );

        let response = self.generate_content(&prompt).await?;
        let analysis: ConfigAnalysis =
            serde_json::from_str(&response).context("Failed to parse AI response for config analysis")?;
        Ok(analysis)
    }

    async fn suggest_fix(&self, anomaly: &Anomaly) -> Result<String> {
        debug!("Generating fix suggestion for anomaly: {:?}", anomaly.anomaly_type);

        let prompt = format!(
            "An anomaly was detected in OBS Studio:\n\
             Type: {:?}\n\
             Severity: {:?}\n\
             Details: {}\n\
             Source: {:?}\n\
             \n\
             Provide a clear, step-by-step solution to fix this issue. \
             Be specific and actionable.",
            anomaly.anomaly_type,
            anomaly.severity,
            anomaly.details,
            anomaly.source
        );

        self.generate_content(&prompt).await
    }

    async fn analyze_image(&self, image: &[u8]) -> Result<ImageAnalysis> {
        debug!("Analyzing image with AI");

        let prompt = "Analyze this OBS Studio interface screenshot. \
                      Identify all UI elements, buttons, settings, and text. \
                      Describe what you see in detail.";

        let response = self.analyze_image_with_prompt(image, prompt).await?;
        let analysis: ImageAnalysis =
            serde_json::from_str(&response).context("Failed to parse AI response for image analysis")?;
        Ok(analysis)
    }

    async fn optimize_settings(&self, hardware: &HardwareInfo) -> Result<OBSConfig> {
        debug!("Optimizing OBS settings with AI");

        let prompt = format!(
            "Create optimal OBS settings for this hardware:\n\
             CPU: {} ({} cores)\n\
             GPU: {:?}\n\
             RAM: {:.1} GB\n\
             \n\
             Provide optimal encoder, preset, bitrate, and resolution. \
             Format as JSON.",
            hardware.cpu.brand,
            hardware.cpu.cores_physical,
            hardware.gpu,
            hardware.ram.total_gb
        );

        let response = self.generate_content(&prompt).await?;
        let config: OBSConfig =
            serde_json::from_str(&response).context("Failed to parse AI response for settings optimization")?;
        Ok(config)
    }

    async fn generate_overlay_design(&self, prompt: &str) -> Result<OverlayDesign> {
        info!("Generating overlay design: {}", prompt);

        let full_prompt = format!(
            "Design a streaming overlay based on this description:\n\
             {}\n\
             \n\
             Provide:\n\
             1. Color palette (hex codes)\n\
             2. Layout description\n\
             3. List of elements with positions and sizes\n\
             Format as JSON.",
            prompt
        );

        let response = self.generate_content(&full_prompt).await?;
        let design: OverlayDesign =
            serde_json::from_str(&response).context("Failed to parse AI response for overlay design")?;
        Ok(design)
    }

    async fn generate(&self, prompt: &str) -> Result<String> {
        self.generate_content(prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requiere API key
    async fn test_ai_generation() {
        let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
        let adapter = AIAdapter::new(api_key);

        let result = adapter.generate("Say hello").await;
        assert!(result.is_ok());
    }
}
