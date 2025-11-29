use crate::application::ports::{MonitorPort, OBSPort};
use crate::domain::models::{Anomaly, Severity};
use crate::domain::services::anomaly_detector::{AnomalyDetector, SystemContext};
use anyhow::Result;
use rayon::prelude::*;
use std::sync::Arc;
use tracing::{info, warn};

/// Reporte de salud del sistema
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub is_healthy: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub anomalies: Vec<Anomaly>,
    pub can_stream: bool,
    pub can_record: bool,
    pub warnings: Vec<String>,
    pub critical_issues: Vec<String>,
}

impl HealthReport {
    pub fn summary(&self) -> String {
        if self.is_healthy {
            "System is healthy and ready to stream".to_string()
        } else if self.can_stream {
            format!("System can stream but has {} warning(s)", self.warnings.len())
        } else {
            format!("System NOT ready: {} critical issue(s)", self.critical_issues.len())
        }
    }
}

/// Servicio de pre-flight check antes de streaming
pub struct HealthCheckService {
    obs_port: Arc<dyn OBSPort>,
    monitor_port: Arc<dyn MonitorPort>,
    detector: AnomalyDetector,
}

impl HealthCheckService {
    pub fn new(
        obs_port: Arc<dyn OBSPort>,
        monitor_port: Arc<dyn MonitorPort>,
    ) -> Self {
        Self {
            obs_port,
            monitor_port,
            detector: AnomalyDetector::with_default_rules(),
        }
    }

    pub fn with_detector(mut self, detector: AnomalyDetector) -> Self {
        self.detector = detector;
        self
    }

    /// Ejecuta health check completo
    pub async fn check(&self) -> Result<HealthReport> {
        info!("Starting health check...");

        // Recolectar datos del sistema (paralelo)
        let (hardware_result, obs_stats_result, scenes_result) = tokio::join!(
            async { self.monitor_port.detect_hardware() },
            async { self.obs_port.get_stats().await },
            async { self.obs_port.get_scenes().await },
        );

        let hardware = hardware_result?;
        let obs_stats = obs_stats_result?;
        let scenes = scenes_result?;

        // Validar escenas en paralelo con Rayon
        let missing_sources: Vec<String> = scenes
            .par_iter()
            .flat_map(|scene| {
                scene
                    .sources
                    .iter()
                    .filter(|s| !s.is_available)
                    .map(|s| format!("{}:{}", scene.name, s.name))
                    .collect::<Vec<_>>()
            })
            .collect();

        // Construir contexto del sistema
        let context = SystemContext {
            cpu_temp: self.monitor_port.get_cpu_temp().unwrap_or(0.0),
            gpu_temp: self.monitor_port.get_gpu_temp().unwrap_or(0.0),
            cpu_usage: self.monitor_port.get_cpu_usage().unwrap_or(0.0),
            memory_used_percent: hardware.ram.used_percent,
            disk_free_gb: self.monitor_port.get_disk_space()?.free_gb,
            obs_dropped_frames_percent: obs_stats.dropped_frames_percent(),
            obs_cpu_usage: obs_stats.cpu_usage,
            missing_sources,
            audio_peak_db: None, // TODO: Implementar
            network_bitrate: None, // TODO: Implementar
        };

        // Detectar anomalías (paralelo con Rayon)
        let anomalies = self.detector.scan(&context);

        // Categorizar anomalías
        let critical_issues: Vec<String> = anomalies
            .iter()
            .filter(|a| a.severity == Severity::Critical)
            .map(|a| a.details.clone())
            .collect();

        let warnings: Vec<String> = anomalies
            .iter()
            .filter(|a| a.severity == Severity::Warning)
            .map(|a| a.details.clone())
            .collect();

        let can_stream = critical_issues.is_empty();
        let can_record = critical_issues.is_empty() && hardware.ram.available_gb > 2.0;
        let is_healthy = critical_issues.is_empty() && warnings.is_empty();

        if !is_healthy {
            warn!(
                "Health check found {} critical issue(s) and {} warning(s)",
                critical_issues.len(),
                warnings.len()
            );
        } else {
            info!("Health check passed - system is healthy");
        }

        Ok(HealthReport {
            is_healthy,
            timestamp: chrono::Utc::now(),
            anomalies,
            can_stream,
            can_record,
            warnings,
            critical_issues,
        })
    }

    /// Check rápido (solo crítico)
    pub async fn quick_check(&self) -> Result<bool> {
        let context = SystemContext {
            cpu_temp: self.monitor_port.get_cpu_temp().unwrap_or(0.0),
            gpu_temp: self.monitor_port.get_gpu_temp().unwrap_or(0.0),
            cpu_usage: self.monitor_port.get_cpu_usage().unwrap_or(0.0),
            memory_used_percent: self.monitor_port.get_memory_info()?.used_percent,
            disk_free_gb: self.monitor_port.get_disk_space()?.free_gb,
            obs_dropped_frames_percent: 0.0,
            obs_cpu_usage: 0.0,
            missing_sources: vec![],
            audio_peak_db: None,
            network_bitrate: None,
        };

        let anomalies = self.detector.scan_filtered(&context, Severity::Critical);
        Ok(anomalies.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::*;
    use async_trait::async_trait;

    struct MockOBSPort;

    #[async_trait]
    impl OBSPort for MockOBSPort {
        async fn connect(&self) -> Result<()> { Ok(()) }
        async fn disconnect(&self) -> Result<()> { Ok(()) }
        async fn get_scenes(&self) -> Result<Vec<crate::domain::models::Scene>> { Ok(vec![]) }
        async fn get_current_scene(&self) -> Result<String> { Ok("Scene 1".to_string()) }
        async fn validate_scene(&self, _: &str) -> Result<ValidationReport> {
            Ok(ValidationReport {
                scene_name: "Scene 1".to_string(),
                is_valid: true,
                issues: vec![],
            })
        }
        async fn get_stats(&self) -> Result<crate::domain::models::OBSStats> {
            Ok(crate::domain::models::OBSStats {
                cpu_usage: 10.0,
                memory_usage: 500.0,
                active_fps: 60.0,
                render_skipped_frames: 0,
                render_total_frames: 1000,
                output_skipped_frames: 0,
                output_total_frames: 1000,
            })
        }
        async fn get_video_settings(&self) -> Result<crate::domain::models::VideoSettings> {
            Ok(crate::domain::models::VideoSettings {
                base_width: 1920,
                base_height: 1080,
                output_width: 1920,
                output_height: 1080,
                fps_numerator: 60,
                fps_denominator: 1,
            })
        }
        async fn set_video_settings(&self, _: &crate::domain::models::VideoSettings) -> Result<()> { Ok(()) }
        async fn take_screenshot(&self, _: &str) -> Result<Vec<u8>> { Ok(vec![]) }
    }

    struct MockMonitorPort;

    impl MonitorPort for MockMonitorPort {
        fn get_cpu_temp(&self) -> Result<f32> { Ok(50.0) }
        fn get_gpu_temp(&self) -> Result<f32> { Ok(60.0) }
        fn get_cpu_usage(&self) -> Result<f32> { Ok(30.0) }
        fn get_memory_info(&self) -> Result<crate::domain::models::RAMInfo> {
            Ok(crate::domain::models::RAMInfo {
                total_gb: 16.0,
                available_gb: 8.0,
                used_percent: 50.0,
            })
        }
        fn get_disk_space(&self) -> Result<DiskInfo> {
            Ok(DiskInfo {
                total_gb: 500.0,
                free_gb: 250.0,
                used_percent: 50.0,
            })
        }
        fn detect_hardware(&self) -> Result<crate::domain::models::HardwareInfo> {
            Ok(crate::domain::models::HardwareInfo {
                os: "Windows".to_string(),
                os_version: "11".to_string(),
                hostname: "test".to_string(),
                cpu: crate::domain::models::CPUInfo {
                    name: "Test CPU".to_string(),
                    brand: "Test".to_string(),
                    cores_physical: 8,
                    cores_logical: 16,
                    frequency_mhz: 3600.0,
                    arch: "x86_64".to_string(),
                },
                gpu: None,
                ram: crate::domain::models::RAMInfo {
                    total_gb: 16.0,
                    available_gb: 8.0,
                    used_percent: 50.0,
                },
                recommended_encoder: crate::domain::models::EncoderType::X264,
                recommended_preset: "medium".to_string(),
                recommended_resolution: (1920, 1080),
                recommended_fps: 60,
                recommended_bitrate: 6000,
            })
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let obs_port = Arc::new(MockOBSPort) as Arc<dyn OBSPort>;
        let monitor_port = Arc::new(MockMonitorPort) as Arc<dyn MonitorPort>;

        let service = HealthCheckService::new(obs_port, monitor_port);
        let report = service.check().await.unwrap();

        assert!(report.can_stream);
        println!("{}", report.summary());
    }
}
