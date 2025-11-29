use crate::config::{PortableConfig, SubscriptionTier};
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
    AI,
}

#[derive(Debug, Clone)]
struct ChatMessage {
    role: MessageRole,
    content: String,
    timestamp: String,
}

#[derive(Debug, Clone)]
enum MessageRole {
    User,
    Assistant,
    System,
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

    // AI Chat State
    chat_messages: Vec<ChatMessage>,
    chat_input: String,
    ai_thinking: bool,

    // Auth State
    is_authenticated: bool,
    user_email: String,
    user_tier: SubscriptionTier,
    ai_credits: u32,
    login_email: String,
    login_password: String,
    register_email: String,
    register_password: String,
    register_confirm_password: String,
    show_register_form: bool,
    auth_message: Option<String>,
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
            chat_messages: Vec::new(),
            chat_input: String::new(),
            ai_thinking: false,
            is_authenticated: false,
            user_email: String::new(),
            user_tier: SubscriptionTier::Free,
            ai_credits: 0,
            login_email: String::new(),
            login_password: String::new(),
            register_email: String::new(),
            register_password: String::new(),
            register_confirm_password: String::new(),
            show_register_form: false,
            auth_message: None,
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

    // === AI & Subscription Functions ===
    
    fn render_ai(&mut self, ui: &mut egui::Ui) {
        ui.heading("🤖 Asistente IA & Servicios Premium");
        ui.add_space(10.0);

        // Panel de estado de autenticación
        ui.horizontal(|ui| {
            if self.is_authenticated {
                ui.label(format!("👤 Usuario: {}", self.user_email));
                ui.label(format!("⭐ Plan: {:?}", self.user_tier));
                ui.label(format!("💳 Créditos: {}", self.ai_credits));
                
                if ui.button("🚪 Cerrar Sesión").clicked() {
                    self.logout();
                }
            } else {
                ui.label("⚠️ No autenticado");
                if ui.button("🔑 Iniciar Sesión").clicked() {
                    self.show_register_form = false;
                }
            }
        });

        ui.add_space(10.0);
        ui.separator();

        // Si no está autenticado, mostrar formulario de login/registro
        if !self.is_authenticated {
            ui.add_space(10.0);
            
            if self.show_register_form {
                // Formulario de Registro
                ui.heading("📝 Crear Cuenta");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.register_email);
                });

                ui.horizontal(|ui| {
                    ui.label("Contraseña:");
                    ui.add(egui::TextEdit::singleline(&mut self.register_password).password(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Confirmar:");
                    ui.add(egui::TextEdit::singleline(&mut self.register_confirm_password).password(true));
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("✅ Registrarse").clicked() {
                        self.handle_register();
                    }

                    if ui.button("← Volver al Login").clicked() {
                        self.show_register_form = false;
                    }
                });

            } else {
                // Formulario de Login
                ui.heading("🔐 Iniciar Sesión");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut self.login_email);
                });

                ui.horizontal(|ui| {
                    ui.label("Contraseña:");
                    ui.add(egui::TextEdit::singleline(&mut self.login_password).password(true));
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("🚀 Iniciar Sesión").clicked() {
                        self.handle_login();
                    }

                    if ui.button("📝 Crear Cuenta").clicked() {
                        self.show_register_form = true;
                    }
                });
            }

            // Mostrar mensaje de autenticación si existe
            if let Some(msg) = &self.auth_message {
                ui.add_space(10.0);
                ui.colored_label(
                    if msg.contains("✅") { egui::Color32::GREEN } else { egui::Color32::RED },
                    msg
                );
            }

            return; // No mostrar el resto si no está autenticado
        }

        // Panel de suscripción
        ui.add_space(10.0);
        ui.heading("💎 Planes de Suscripción");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            self.render_tier_card(ui, SubscriptionTier::Free);
            self.render_tier_card(ui, SubscriptionTier::Basic);
            self.render_tier_card(ui, SubscriptionTier::Pro);
            self.render_tier_card(ui, SubscriptionTier::Enterprise);
        });

        ui.add_space(20.0);
        ui.separator();

        // Chat con IA
        ui.add_space(10.0);
        ui.heading("💬 Chat con IA");
        ui.add_space(10.0);

        // Área de mensajes
        egui::ScrollArea::vertical()
            .id_source("ai_chat_scroll")
            .max_height(300.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &self.chat_messages {
                    self.render_chat_message(ui, msg);
                }

                if self.ai_thinking {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("IA pensando...");
                    });
                }
            });

        ui.add_space(10.0);

        // Input de chat
        ui.horizontal(|ui| {
            let response = ui.add_sized(
                [ui.available_width() - 80.0, 25.0],
                egui::TextEdit::singleline(&mut self.chat_input)
                    .hint_text("Escribe un mensaje para la IA...")
            );

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.send_chat_message();
            }

            if ui.button("🚀 Enviar").clicked() {
                self.send_chat_message();
            }
        });

        ui.add_space(5.0);
        ui.label(format!("💳 {} créditos restantes", self.ai_credits));
    }

    fn render_tier_card(&mut self, ui: &mut egui::Ui, tier: SubscriptionTier) {
        let (name, credits, price, color) = match tier {
            SubscriptionTier::Free => ("Free", "100", "Gratis", egui::Color32::GRAY),
            SubscriptionTier::Basic => ("Basic", "1,000", "$9.99/mes", egui::Color32::LIGHT_BLUE),
            SubscriptionTier::Pro => ("Pro", "10,000", "$29.99/mes", egui::Color32::GOLD),
            SubscriptionTier::Enterprise => ("Enterprise", "100,000", "$99.99/mes", egui::Color32::from_rgb(147, 51, 234)),
        };

        let is_current = self.user_tier == tier;

        ui.group(|ui| {
            ui.set_min_width(150.0);
            ui.vertical(|ui| {
                ui.colored_label(color, format!("⭐ {}", name));
                ui.label(format!("💳 {} créditos", credits));
                ui.label(price);
                ui.add_space(5.0);

                if is_current {
                    ui.colored_label(egui::Color32::GREEN, "✅ Plan Actual");
                } else if tier as u8 > self.user_tier.clone() as u8 {
                    if ui.button("⬆️ Actualizar").clicked() {
                        self.upgrade_tier(tier);
                    }
                }
            });
        });
    }

    fn render_chat_message(&self, ui: &mut egui::Ui, msg: &ChatMessage) {
        ui.horizontal(|ui| {
            let color = match msg.role {
                MessageRole::User => egui::Color32::LIGHT_BLUE,
                MessageRole::Assistant => egui::Color32::LIGHT_GREEN,
                MessageRole::System => egui::Color32::YELLOW,
            };

            let icon = match msg.role {
                MessageRole::User => "👤",
                MessageRole::Assistant => "🤖",
                MessageRole::System => "⚙️",
            };

            ui.colored_label(color, icon);
            ui.vertical(|ui| {
                ui.label(&msg.content);
                ui.label(
                    egui::RichText::new(&msg.timestamp)
                        .size(10.0)
                        .color(egui::Color32::GRAY)
                );
            });
        });
        ui.add_space(8.0);
    }

    fn send_chat_message(&mut self) {
        if self.chat_input.trim().is_empty() {
            return;
        }

        if self.ai_credits < 1 {
            self.chat_messages.push(ChatMessage {
                role: MessageRole::System,
                content: "⚠️ Sin créditos. Actualiza tu plan para continuar.".to_string(),
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            });
            return;
        }

        // Mensaje del usuario
        let user_message = self.chat_input.clone();
        self.chat_messages.push(ChatMessage {
            role: MessageRole::User,
            content: user_message.clone(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });

        self.chat_input.clear();
        self.ai_thinking = true;

        // Simular respuesta de IA (en producción, llamar API real)
        let ai_response = self.get_ai_response(&user_message);

        self.chat_messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: ai_response,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });

        self.ai_credits -= 1;
        self.ai_thinking = false;
    }

    fn get_ai_response(&self, _user_message: &str) -> String {
        "🤖 Respuesta de IA (mock). En producción, esto llamará a Gemini API.".to_string()
    }

    fn handle_login(&mut self) {
        // Mock login (en producción, llamar API)
        if !self.login_email.is_empty() && !self.login_password.is_empty() {
            self.is_authenticated = true;
            self.user_email = self.login_email.clone();
            self.user_tier = SubscriptionTier::Free;
            self.ai_credits = 100;
            self.auth_message = Some("✅ Inicio de sesión exitoso".to_string());
            self.login_password.clear();
        } else {
            self.auth_message = Some("❌ Email y contraseña requeridos".to_string());
        }
    }

    fn handle_register(&mut self) {
        // Mock register (en producción, llamar API)
        if self.register_password != self.register_confirm_password {
            self.auth_message = Some("❌ Las contraseñas no coinciden".to_string());
            return;
        }

        if !self.register_email.is_empty() && self.register_password.len() >= 6 {
            self.is_authenticated = true;
            self.user_email = self.register_email.clone();
            self.user_tier = SubscriptionTier::Free;
            self.ai_credits = 100;
            self.auth_message = Some("✅ Cuenta creada exitosamente".to_string());
            self.register_password.clear();
            self.register_confirm_password.clear();
        } else {
            self.auth_message = Some("❌ Email válido y contraseña de al menos 6 caracteres".to_string());
        }
    }

    fn logout(&mut self) {
        self.is_authenticated = false;
        self.user_email = String::new();
        self.user_tier = SubscriptionTier::Free;
        self.ai_credits = 0;
        self.chat_messages.clear();
        self.login_email.clear();
        self.login_password.clear();
    }

    fn upgrade_tier(&mut self, tier: SubscriptionTier) {
        // Mock upgrade (en producción, integrar Stripe)
        self.user_tier = tier.clone();
        self.ai_credits = match tier {
            SubscriptionTier::Free => 100,
            SubscriptionTier::Basic => 1000,
            SubscriptionTier::Pro => 10000,
            SubscriptionTier::Enterprise => 100000,
        };

        self.chat_messages.push(ChatMessage {
            role: MessageRole::System,
            content: format!("✅ Plan actualizado a {:?}. {} créditos añadidos.", tier, self.ai_credits),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
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
                ui.selectable_value(&mut self.current_tab, Tab::AI, "🤖 IA & Servicios");
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
                    Tab::AI => self.render_ai(ui),
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
