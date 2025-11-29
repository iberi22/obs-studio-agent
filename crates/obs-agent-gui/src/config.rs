use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

/// Configuración portable del OBS Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableConfig {
    /// Ruta al directorio de configuración de OBS
    pub obs_config_dir: Option<PathBuf>,

    /// Host de OBS WebSocket
    pub obs_host: String,

    /// Puerto de OBS WebSocket
    pub obs_port: u16,

    /// Password de OBS WebSocket (opcional)
    pub obs_password: Option<String>,

    /// API Key de Gemini
    pub gemini_api_key: Option<String>,

    /// Modo portable (si es true, usa directorio local)
    pub portable_mode: bool,

    // --- Sistema de Usuario y Suscripción ---
    /// Email del usuario registrado
    pub user_email: Option<String>,

    /// Token de autenticación del usuario
    pub user_token: Option<String>,

    /// Tier de suscripción (Free, Basic, Pro, Enterprise)
    pub subscription_tier: SubscriptionTier,

    /// Créditos disponibles
    pub credits_available: u32,

    /// API endpoint del servidor de suscripciones
    pub subscription_api_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Basic,
    Pro,
    Enterprise,
}

impl SubscriptionTier {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Free => "Free",
            Self::Basic => "Basic ($9.99/mes)",
            Self::Pro => "Pro ($29.99/mes)",
            Self::Enterprise => "Enterprise ($99.99/mes)",
        }
    }

    pub fn credits_per_month(&self) -> u32 {
        match self {
            Self::Free => 100,
            Self::Basic => 1000,
            Self::Pro => 10000,
            Self::Enterprise => 100000,
        }
    }

    pub fn features(&self) -> Vec<&str> {
        match self {
            Self::Free => vec![
                "100 créditos/mes",
                "Detección de anomalías básica",
                "Health checks",
            ],
            Self::Basic => vec![
                "1,000 créditos/mes",
                "Detección avanzada de anomalías",
                "Optimización con IA",
                "Soporte por email",
            ],
            Self::Pro => vec![
                "10,000 créditos/mes",
                "Análisis en tiempo real",
                "Generación de overlays",
                "Chat con IA ilimitado",
                "Soporte prioritario",
            ],
            Self::Enterprise => vec![
                "100,000 créditos/mes",
                "Todas las funciones Pro",
                "API dedicada",
                "Soporte 24/7",
                "Integración custom",
            ],
        }
    }
}

impl Default for PortableConfig {
    fn default() -> Self {
        Self {
            obs_config_dir: None,
            obs_host: "localhost".to_string(),
            obs_port: 4455,
            obs_password: None,
            gemini_api_key: None,
            portable_mode: true,
            user_email: None,
            user_token: None,
            subscription_tier: SubscriptionTier::Free,
            credits_available: 100,
            subscription_api_url: "https://api.obsagent.io".to_string(),
        }
    }
}

impl PortableConfig {
    /// Obtiene la ruta del archivo de configuración
    pub fn config_path() -> PathBuf {
        // En modo portable, usa el directorio actual
        let mut path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();

        path.push("obs-agent-config.toml");
        path
    }

    /// Carga la configuración desde archivo
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: PortableConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Guarda la configuración en archivo
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Detecta automáticamente el directorio de configuración de OBS
    pub fn detect_obs_config_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = dirs::config_dir() {
                let obs_path = appdata.join("obs-studio");
                if obs_path.exists() {
                    return Some(obs_path);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(config) = dirs::config_dir() {
                let obs_path = config.join("obs-studio");
                if obs_path.exists() {
                    return Some(obs_path);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                let obs_path = home.join("Library/Application Support/obs-studio");
                if obs_path.exists() {
                    return Some(obs_path);
                }
            }
        }

        None
    }

    /// Valida si la configuración es válida
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        !self.obs_host.is_empty() && self.obs_port > 0
    }
}
