use crate::domain::models::Anomaly;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Eventos del dominio
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    AnomalyDetected(AnomalyDetectedEvent),
    StreamStarted(StreamStartedEvent),
    StreamStopped(StreamStoppedEvent),
    HealthCheckCompleted(HealthCheckCompletedEvent),
    ConfigurationChanged(ConfigurationChangedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub anomaly: Anomaly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStartedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub scene_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStoppedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckCompletedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub is_healthy: bool,
    pub anomalies_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChangedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub changes: Vec<String>,
}

impl DomainEvent {
    pub fn anomaly_detected(anomaly: Anomaly) -> Self {
        Self::AnomalyDetected(AnomalyDetectedEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            anomaly,
        })
    }

    pub fn stream_started(scene_name: impl Into<String>) -> Self {
        Self::StreamStarted(StreamStartedEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            scene_name: scene_name.into(),
        })
    }

    pub fn stream_stopped(duration_seconds: u64) -> Self {
        Self::StreamStopped(StreamStoppedEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            duration_seconds,
        })
    }

    pub fn health_check_completed(is_healthy: bool, anomalies_count: usize) -> Self {
        Self::HealthCheckCompleted(HealthCheckCompletedEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            is_healthy,
            anomalies_count,
        })
    }

    pub fn config_changed(changes: Vec<String>) -> Self {
        Self::ConfigurationChanged(ConfigurationChangedEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            changes,
        })
    }
}
