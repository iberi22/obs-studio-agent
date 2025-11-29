use crate::config::PortableConfig;
use eframe::egui;
use obs_agent_core::application::ports::{MonitorPort, OBSPort};
use obs_agent_core::domain::services::{AnomalyDetector, HealthCheckService, SystemContext};
use obs_agent_infra::{MonitorAdapter, OBSAdapter};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Home,
    Config,
    Hardware,
    Health,
    Anomalies,
}

pub struct OBSAgentApp {
    config: PortableConfig,
    runtime: Arc<Runtime>,
    current_tab: Tab,

    // Estado
    connection_status: String,
    hardware_info: Option<String>,
    health_report: Option<String>,
    anomalies: Vec<String>,

    // UI state
    show_config_saved: bool,
    error_message: Option<String>,
}

impl OBSAgentApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = PortableConfig::load().unwrap_or_default();
        let runtime = Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));

        Self {
            config,
            runtime,
            current_tab: Tab::Home,
            connection_status: "Not connected".to_string(),
            hardware_info: None,
            health_report: None,
            anomalies: Vec::new(),
            show_config_saved: false,
            error_message: None,
        }
    }

    fn render_home(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎬 OBS Agent - AI Streaming Assistant");
        ui.add_space(20.0);

        ui.label("Estado de la conexión:");
        ui.label(&self.connection_status);
        ui.add_space(10.0);

        if ui.button("🔌 Probar Conexión OBS").clicked() {
            self.test_obs_connection();
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label("Acciones rápidas:");
        ui.add_space(10.0);

        if ui.button("🖥️  Detectar Hardware").clicked() {
            self.detect_hardware();
        }

        if ui.button("🏥 Health Check").clicked() {
            self.run_health_check();
        }

        if ui.button("🔍 Escanear Anomalías").clicked() {
            self.scan_anomalies();
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Configuración");
        ui.add_space(20.0);

        egui::Grid::new("config_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                // OBS Configuration
                ui.label("OBS WebSocket Host:");
                ui.text_edit_singleline(&mut self.config.obs_host);
                ui.end_row();

                ui.label("OBS WebSocket Port:");
                ui.add(egui::DragValue::new(&mut self.config.obs_port).speed(1.0));
                ui.end_row();

                ui.label("OBS WebSocket Password:");
                let mut password = self.config.obs_password.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut password).changed() {
                    self.config.obs_password = if password.is_empty() {
                        None
                    } else {
                        Some(password)
                    };
                }
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();

                // Gemini API Key
                ui.label("Gemini API Key:");
                let mut api_key = self.config.gemini_api_key.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut api_key).changed() {
                    self.config.gemini_api_key = if api_key.is_empty() {
                        None
                    } else {
                        Some(api_key)
                    };
                }
                ui.end_row();

                if ui.small_button("?").clicked() {
                    // Abrir enlace a documentación
                }
                ui.label("Obtén tu API key en: https://ai.google.dev");
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();

                // OBS Config Directory
                ui.label("Directorio de OBS:");
                if let Some(dir) = &self.config.obs_config_dir {
                    ui.label(dir.display().to_string());
                } else {
                    ui.label("No detectado");
                }
                ui.end_row();

                ui.label("");
                if ui.button("🔍 Auto-detectar").clicked() {
                    if let Some(dir) = PortableConfig::detect_obs_config_dir() {
                        self.config.obs_config_dir = Some(dir);
                    } else {
                        self.error_message = Some("No se pudo detectar OBS instalado".to_string());
                    }
                }
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();

                // Portable Mode
                ui.label("Modo Portable:");
                ui.checkbox(&mut self.config.portable_mode, "Usar configuración local");
                ui.end_row();
            });

        ui.add_space(20.0);

        if ui.button("💾 Guardar Configuración").clicked() {
            match self.config.save() {
                Ok(_) => {
                    self.show_config_saved = true;
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Error al guardar: {}", e));
                }
            }
        }

        if self.show_config_saved {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::GREEN, "✅ Configuración guardada exitosamente");
        }

        if let Some(error) = &self.error_message {
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
        }

        ui.add_space(10.0);
        ui.label(format!("📁 Archivo de config: {}", PortableConfig::config_path().display()));
    }

    fn render_hardware(&mut self, ui: &mut egui::Ui) {
        ui.heading("🖥️  Información de Hardware");
        ui.add_space(20.0);

        if ui.button("🔄 Detectar Hardware").clicked() {
            self.detect_hardware();
        }

        ui.add_space(10.0);

        if let Some(info) = &self.hardware_info {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(info);
            });
        } else {
            ui.label("Presiona 'Detectar Hardware' para ver información del sistema");
        }
    }

    fn render_health(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏥 Health Check");
        ui.add_space(20.0);

        if ui.button("🔄 Ejecutar Health Check").clicked() {
            self.run_health_check();
        }

        ui.add_space(10.0);

        if let Some(report) = &self.health_report {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(report);
            });
        } else {
            ui.label("Presiona 'Ejecutar Health Check' para validar el sistema");
        }
    }

    fn render_anomalies(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔍 Detección de Anomalías");
        ui.add_space(20.0);

        if ui.button("🔄 Escanear Anomalías").clicked() {
            self.scan_anomalies();
        }

        ui.add_space(10.0);

        if self.anomalies.is_empty() {
            ui.label("No se han detectado anomalías");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for anomaly in &self.anomalies {
                    ui.label(anomaly);
                    ui.separator();
                }
            });
        }
    }

    fn test_obs_connection(&mut self) {
        let config = self.config.clone();
        let runtime = Arc::clone(&self.runtime);

        self.connection_status = "Conectando...".to_string();

        // Spawn async task
        let adapter = OBSAdapter::new(&config.obs_host, config.obs_port, config.obs_password);

        match runtime.block_on(adapter.connect()) {
            Ok(_) => {
                self.connection_status = "✅ Conectado a OBS".to_string();
                let _ = runtime.block_on(adapter.disconnect());
            }
            Err(e) => {
                self.connection_status = format!("❌ Error: {}", e);
            }
        }
    }

    fn detect_hardware(&mut self) {
        let monitor = MonitorAdapter::new();

        match monitor.detect_hardware() {
            Ok(hw) => {
                let info = format!(
                    "OS: {} {}\nHostname: {}\n\n\
                    💻 CPU:\n  Nombre: {}\n  Cores: {} físicos / {} lógicos\n  Frecuencia: {:.0} MHz\n\n\
                    🎮 GPU:\n  {}\n\n\
                    💾 RAM:\n  Total: {:.1} GB\n  Disponible: {:.1} GB\n  Uso: {:.1}%\n\n\
                    📊 RECOMENDACIONES:\n  Encoder: {:?}\n  Preset: {}\n  Resolución: {}x{}\n  FPS: {}\n  Bitrate: {} kbps",
                    hw.os, hw.os_version, hw.hostname,
                    hw.cpu.brand, hw.cpu.cores_physical, hw.cpu.cores_logical, hw.cpu.frequency_mhz,
                    if let Some(gpu) = &hw.gpu {
                        format!("Nombre: {}\n  Vendor: {:?}\n  NVENC: {}\n  AMF: {}\n  QSV: {}",
                            gpu.name, gpu.vendor, gpu.supports_nvenc, gpu.supports_amf, gpu.supports_qsv)
                    } else {
                        "No detectado".to_string()
                    },
                    hw.ram.total_gb, hw.ram.available_gb, hw.ram.used_percent,
                    hw.recommended_encoder, hw.recommended_preset,
                    hw.recommended_resolution.0, hw.recommended_resolution.1,
                    hw.recommended_fps, hw.recommended_bitrate
                );

                self.hardware_info = Some(info);
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Error al detectar hardware: {}", e));
            }
        }
    }

    fn run_health_check(&mut self) {
        let config = self.config.clone();
        let runtime = Arc::clone(&self.runtime);

        let obs = Arc::new(OBSAdapter::new(&config.obs_host, config.obs_port, config.obs_password)) as Arc<dyn OBSPort>;
        let monitor = Arc::new(MonitorAdapter::new()) as Arc<dyn MonitorPort>;

        let service = HealthCheckService::new(obs, monitor);

        match runtime.block_on(service.check()) {
            Ok(report) => {
                let info = format!(
                    "🏥 HEALTH CHECK REPORT\n\
                    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                    Timestamp: {}\n\
                    Estado General: {}\n\
                    Puede Hacer Stream: {}\n\
                    Puede Grabar: {}\n\n\
                    🔴 PROBLEMAS CRÍTICOS: {}\n\
                    {}\n\n\
                    ⚠️  ADVERTENCIAS: {}\n\
                    {}\n\n\
                    Resumen: {}",
                    report.timestamp,
                    if report.is_healthy { "✅ Saludable" } else { "⚠️  Con Problemas" },
                    if report.can_stream { "Sí" } else { "No" },
                    if report.can_record { "Sí" } else { "No" },
                    report.critical_issues.len(),
                    report.critical_issues.join("\n"),
                    report.warnings.len(),
                    report.warnings.join("\n"),
                    report.summary()
                );

                self.health_report = Some(info);
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Error en health check: {}", e));
            }
        }
    }

    fn scan_anomalies(&mut self) {
        let monitor = MonitorAdapter::new();
        let detector = AnomalyDetector::with_default_rules();

        let context = SystemContext {
            cpu_temp: monitor.get_cpu_temp().unwrap_or(0.0),
            gpu_temp: monitor.get_gpu_temp().unwrap_or(0.0),
            cpu_usage: monitor.get_cpu_usage().unwrap_or(0.0),
            memory_used_percent: monitor.get_memory_info().map(|m| m.used_percent).unwrap_or(0.0),
            disk_free_gb: monitor.get_disk_space().map(|d| d.free_gb).unwrap_or(0.0),
            obs_dropped_frames_percent: 0.0,
            obs_cpu_usage: 0.0,
            missing_sources: vec![],
            audio_peak_db: None,
            network_bitrate: None,
        };

        let anomalies = detector.scan(&context);

        if anomalies.is_empty() {
            self.anomalies = vec!["✅ No se detectaron anomalías".to_string()];
        } else {
            self.anomalies = anomalies.iter().map(|a| {
                format!(
                    "[{:?}] {:?}\n  Detalles: {}\n  Acción: {}{}",
                    a.severity,
                    a.anomaly_type,
                    a.details,
                    a.recommended_action,
                    if a.auto_fixable { "\n  ✅ Auto-reparable" } else { "" }
                )
            }).collect();
        }

        self.error_message = None;
    }
}

impl eframe::App for OBSAgentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Home, "🏠 Inicio");
                ui.selectable_value(&mut self.current_tab, Tab::Config, "⚙️ Config");
                ui.selectable_value(&mut self.current_tab, Tab::Hardware, "🖥️  Hardware");
                ui.selectable_value(&mut self.current_tab, Tab::Health, "🏥 Health");
                ui.selectable_value(&mut self.current_tab, Tab::Anomalies, "🔍 Anomalías");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);

                match self.current_tab {
                    Tab::Home => self.render_home(ui),
                    Tab::Config => self.render_config(ui),
                    Tab::Hardware => self.render_hardware(ui),
                    Tab::Health => self.render_health(ui),
                    Tab::Anomalies => self.render_anomalies(ui),
                }

                ui.add_space(20.0);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("OBS Agent v0.1.0 - Portable Mode");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("Documentación", "https://github.com/iberi22/obs-studio-agent");
                });
            });
        });
    }
}
