use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tipos de anomalías detectables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Fuente desconectada o no disponible
    MissingSource,
    /// Temperatura de CPU alta
    HighCPUTemp,
    /// Temperatura de GPU alta
    HighGPUTemp,
    /// Fuga de memoria detectada
    MemoryLeak,
    /// Frames perdidos en encoder
    DroppedFrames,
    /// Encoder sobrecargado
    EncoderOverload,
    /// Audio saturado (clipping)
    AudioClipping,
    /// Espacio en disco bajo
    DiskSpaceLow,
    /// Plugin crasheado
    PluginCrash,
    /// Configuración inválida
    InvalidConfig,
    /// Red inestable
    NetworkUnstable,
    /// Bitrate inconsistente
    BitrateIssue,
}

/// Severidad de la anomalía
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// Anomalía detectada en el sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: Uuid,
    pub anomaly_type: AnomalyType,
    pub severity: Severity,
    pub timestamp: DateTime<Utc>,
    pub details: String,
    pub source: Option<String>,
    pub recommended_action: String,
    pub auto_fixable: bool,
}

impl Default for Anomaly {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            anomaly_type: AnomalyType::InvalidConfig,
            severity: Severity::Info,
            timestamp: Utc::now(),
            details: String::new(),
            source: None,
            recommended_action: String::new(),
            auto_fixable: false,
        }
    }
}

impl Anomaly {
    pub fn new(
        anomaly_type: AnomalyType,
        severity: Severity,
        details: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            anomaly_type,
            severity,
            timestamp: Utc::now(),
            details: details.into(),
            source: None,
            recommended_action: String::new(),
            auto_fixable: false,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.recommended_action = action.into();
        self
    }

    pub fn auto_fixable(mut self, fixable: bool) -> Self {
        self.auto_fixable = fixable;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_creation() {
        let anomaly = Anomaly::new(
            AnomalyType::MissingSource,
            Severity::Critical,
            "Test anomaly",
        )
        .with_source("webcam")
        .with_action("Connect the device")
        .auto_fixable(false);

        assert_eq!(anomaly.anomaly_type, AnomalyType::MissingSource);
        assert_eq!(anomaly.severity, Severity::Critical);
        assert_eq!(anomaly.source, Some("webcam".to_string()));
        assert!(!anomaly.auto_fixable);
    }
}
