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
use tracing::{debug, info};

/// Adapter para interactuar con Google Gemini API
pub struct AIAdapter {
    api_key: String,
    model: String,
    client: Client,
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
            model: "gemini-1.5-pro".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    async fn generate_content(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part::Text {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: Some(GenerationConfig {
                temperature: Some(0.7),
                max_output_tokens: Some(2048),
            }),
        };

        let response = self
            .client
            .post(&url)
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
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
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
            .post(&url)
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

        // Parse JSON response (simplificado - en producción usar serde)
        Ok(ConfigAnalysis {
            is_optimal: response.contains("\"is_optimal\": true"),
            issues: vec![],
            recommendations: vec![],
            required_plugins: vec![],
        })
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

        Ok(ImageAnalysis {
            description: response,
            elements: vec![],
            text_detected: None,
        })
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

        let _response = self.generate_content(&prompt).await?;

        // Simplificado - en producción parsear el JSON
        Ok(OBSConfig {
            video: VideoSettings {
                base_width: hardware.recommended_resolution.0,
                base_height: hardware.recommended_resolution.1,
                output_width: hardware.recommended_resolution.0,
                output_height: hardware.recommended_resolution.1,
                fps_numerator: hardware.recommended_fps,
                fps_denominator: 1,
            },
            encoder: format!("{:?}", hardware.recommended_encoder),
            preset: hardware.recommended_preset.clone(),
            bitrate: hardware.recommended_bitrate,
            audio_settings: json!({}),
        })
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

        let _response = self.generate_content(&full_prompt).await?;

        // Simplificado - en producción parsear el JSON
        Ok(OverlayDesign {
            colors: vec!["#FF0000".to_string(), "#00FF00".to_string()],
            layout: "modern".to_string(),
            elements: vec![],
        })
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
