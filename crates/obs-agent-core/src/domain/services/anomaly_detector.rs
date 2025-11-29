use crate::domain::models::{Anomaly, AnomalyType, Severity};
use rayon::prelude::*;
use std::sync::Arc;
use tracing::{debug, warn};

/// Contexto del sistema para detección de anomalías
#[derive(Debug, Clone)]
pub struct SystemContext {
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub cpu_usage: f32,
    pub memory_used_percent: f64,
    pub disk_free_gb: f64,
    pub obs_dropped_frames_percent: f64,
    pub obs_cpu_usage: f64,
    pub missing_sources: Vec<String>,
    pub audio_peak_db: Option<f32>,
    pub network_bitrate: Option<u32>,
}

/// Regla de detección de anomalías
pub trait AnomalyRule: Send + Sync {
    fn check(&self, context: &SystemContext) -> Option<Anomaly>;
    fn name(&self) -> &str;
}

/// Regla: Fuente faltante
pub struct MissingSourceRule;

impl AnomalyRule for MissingSourceRule {
    fn check(&self, context: &SystemContext) -> Option<Anomaly> {
        if !context.missing_sources.is_empty() {
            Some(
                Anomaly::new(
                    AnomalyType::MissingSource,
                    Severity::Critical,
                    format!("Missing {} source(s)", context.missing_sources.len()),
                )
                .with_source(context.missing_sources.join(", "))
                .with_action("Connect or remove missing sources before streaming")
                .auto_fixable(false),
            )
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "MissingSource"
    }
}

/// Regla: CPU temperatura alta
pub struct HighCPUTempRule {
    pub warning_threshold: f32,
    pub critical_threshold: f32,
}

impl AnomalyRule for HighCPUTempRule {
    fn check(&self, context: &SystemContext) -> Option<Anomaly> {
        if context.cpu_temp >= self.critical_threshold {
            Some(
                Anomaly::new(
                    AnomalyType::HighCPUTemp,
                    Severity::Critical,
                    format!("CPU temperature is {}°C (critical)", context.cpu_temp),
                )
                .with_action("Stop streaming immediately and check cooling")
                .auto_fixable(false),
            )
        } else if context.cpu_temp >= self.warning_threshold {
            Some(
                Anomaly::new(
                    AnomalyType::HighCPUTemp,
                    Severity::Warning,
                    format!("CPU temperature is {}°C (high)", context.cpu_temp),
                )
                .with_action("Consider reducing encoder preset or enabling hardware encoding")
                .auto_fixable(true),
            )
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "HighCPUTemp"
    }
}

/// Regla: GPU temperatura alta
pub struct HighGPUTempRule {
    pub warning_threshold: f32,
    pub critical_threshold: f32,
}

impl AnomalyRule for HighGPUTempRule {
    fn check(&self, context: &SystemContext) -> Option<Anomaly> {
        if context.gpu_temp >= self.critical_threshold {
            Some(
                Anomaly::new(
                    AnomalyType::HighGPUTemp,
                    Severity::Critical,
                    format!("GPU temperature is {}°C (critical)", context.gpu_temp),
                )
                .with_action("Stop streaming and check GPU cooling")
                .auto_fixable(false),
            )
        } else if context.gpu_temp >= self.warning_threshold {
            Some(
                Anomaly::new(
                    AnomalyType::HighGPUTemp,
                    Severity::Warning,
                    format!("GPU temperature is {}°C (high)", context.gpu_temp),
                )
                .with_action("Consider reducing resolution or frame rate")
                .auto_fixable(true),
            )
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "HighGPUTemp"
    }
}

/// Regla: Frames perdidos
pub struct DroppedFramesRule {
    pub threshold_percent: f64,
}

impl AnomalyRule for DroppedFramesRule {
    fn check(&self, context: &SystemContext) -> Option<Anomaly> {
        if context.obs_dropped_frames_percent >= self.threshold_percent {
            Some(
                Anomaly::new(
                    AnomalyType::DroppedFrames,
                    Severity::Warning,
                    format!(
                        "Dropping {:.1}% of frames",
                        context.obs_dropped_frames_percent
                    ),
                )
                .with_action("Reduce encoder preset, resolution, or frame rate")
                .auto_fixable(true),
            )
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "DroppedFrames"
    }
}

/// Regla: Memoria baja
pub struct LowMemoryRule {
    pub threshold_percent: f64,
}

impl AnomalyRule for LowMemoryRule {
    fn check(&self, context: &SystemContext) -> Option<Anomaly> {
        if context.memory_used_percent >= self.threshold_percent {
            Some(
                Anomaly::new(
                    AnomalyType::MemoryLeak,
                    Severity::Warning,
                    format!("Memory usage at {:.1}%", context.memory_used_percent),
                )
                .with_action("Close unnecessary applications or restart OBS")
                .auto_fixable(false),
            )
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "LowMemory"
    }
}

/// Regla: Espacio en disco bajo
pub struct LowDiskSpaceRule {
    pub threshold_gb: f64,
}

impl AnomalyRule for LowDiskSpaceRule {
    fn check(&self, context: &SystemContext) -> Option<Anomaly> {
        if context.disk_free_gb <= self.threshold_gb {
            Some(
                Anomaly::new(
                    AnomalyType::DiskSpaceLow,
                    Severity::Critical,
                    format!("Only {:.1} GB free disk space", context.disk_free_gb),
                )
                .with_action("Free up disk space before recording")
                .auto_fixable(false),
            )
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "LowDiskSpace"
    }
}

/// Regla: Audio saturado
pub struct AudioClippingRule;

impl AnomalyRule for AudioClippingRule {
    fn check(&self, context: &SystemContext) -> Option<Anomaly> {
        if let Some(peak_db) = context.audio_peak_db {
            if peak_db >= 0.0 {
                return Some(
                    Anomaly::new(
                        AnomalyType::AudioClipping,
                        Severity::Warning,
                        format!("Audio peaking at {:.1} dB (clipping)", peak_db),
                    )
                    .with_action("Reduce microphone gain or enable compressor")
                    .auto_fixable(true),
                );
            }
        }
        None
    }

    fn name(&self) -> &str {
        "AudioClipping"
    }
}

/// Motor de detección de anomalías con Rayon
pub struct AnomalyDetector {
    rules: Vec<Arc<dyn AnomalyRule>>,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_rule(mut self, rule: Arc<dyn AnomalyRule>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_default_rules() -> Self {
        Self::new()
            .with_rule(Arc::new(MissingSourceRule))
            .with_rule(Arc::new(HighCPUTempRule {
                warning_threshold: 75.0,
                critical_threshold: 85.0,
            }))
            .with_rule(Arc::new(HighGPUTempRule {
                warning_threshold: 80.0,
                critical_threshold: 90.0,
            }))
            .with_rule(Arc::new(DroppedFramesRule {
                threshold_percent: 1.0,
            }))
            .with_rule(Arc::new(LowMemoryRule {
                threshold_percent: 90.0,
            }))
            .with_rule(Arc::new(LowDiskSpaceRule { threshold_gb: 10.0 }))
            .with_rule(Arc::new(AudioClippingRule))
    }

    /// Escanea el sistema en busca de anomalías (paralelo con Rayon)
    pub fn scan(&self, context: &SystemContext) -> Vec<Anomaly> {
        debug!("Scanning for anomalies with {} rules", self.rules.len());

        // Ejecutar todas las reglas en paralelo
        let anomalies: Vec<Anomaly> = self
            .rules
            .par_iter()
            .filter_map(|rule| {
                let result = rule.check(context);
                if let Some(ref anomaly) = result {
                    warn!(
                        "Rule '{}' detected {:?} anomaly: {}",
                        rule.name(),
                        anomaly.severity,
                        anomaly.details
                    );
                }
                result
            })
            .collect();

        debug!("Found {} anomalies", anomalies.len());
        anomalies
    }

    /// Escanea y filtra por severidad mínima
    pub fn scan_filtered(&self, context: &SystemContext, min_severity: Severity) -> Vec<Anomaly> {
        self.scan(context)
            .into_iter()
            .filter(|a| a.severity >= min_severity)
            .collect()
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::with_default_rules()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_source_detection() {
        let detector = AnomalyDetector::default();
        let context = SystemContext {
            cpu_temp: 50.0,
            gpu_temp: 60.0,
            cpu_usage: 30.0,
            memory_used_percent: 50.0,
            disk_free_gb: 100.0,
            obs_dropped_frames_percent: 0.0,
            obs_cpu_usage: 10.0,
            missing_sources: vec!["webcam".to_string()],
            audio_peak_db: None,
            network_bitrate: None,
        };

        let anomalies = detector.scan(&context);
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyType::MissingSource));
    }

    #[test]
    fn test_high_temperature_detection() {
        let detector = AnomalyDetector::default();
        let context = SystemContext {
            cpu_temp: 90.0, // Critical!
            gpu_temp: 60.0,
            cpu_usage: 30.0,
            memory_used_percent: 50.0,
            disk_free_gb: 100.0,
            obs_dropped_frames_percent: 0.0,
            obs_cpu_usage: 10.0,
            missing_sources: vec![],
            audio_peak_db: None,
            network_bitrate: None,
        };

        let anomalies = detector.scan(&context);
        assert!(anomalies.iter().any(|a| {
            a.anomaly_type == AnomalyType::HighCPUTemp && a.severity == Severity::Critical
        }));
    }

    #[test]
    fn test_parallel_execution() {
        use std::time::Instant;

        let detector = AnomalyDetector::default();
        let context = SystemContext {
            cpu_temp: 50.0,
            gpu_temp: 60.0,
            cpu_usage: 30.0,
            memory_used_percent: 50.0,
            disk_free_gb: 100.0,
            obs_dropped_frames_percent: 0.0,
            obs_cpu_usage: 10.0,
            missing_sources: vec![],
            audio_peak_db: None,
            network_bitrate: None,
        };

        let start = Instant::now();
        let anomalies = detector.scan(&context);
        let duration = start.elapsed();

        println!("Scan completed in {:?} with {} anomalies", duration, anomalies.len());
        assert!(duration.as_millis() < 100); // Debe ser rápido
    }
}
