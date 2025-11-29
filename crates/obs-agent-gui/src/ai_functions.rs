// Funciones adicionales para la pestaña AI
// Este archivo contiene las implementaciones que se agregarán a ui.rs

fn render_ai(&mut self, ui: &mut egui::Ui) {
    ui.heading("🤖 IA & Servicios de Suscripción");
    ui.add_space(10.0);

    // Panel de usuario y suscripción
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("👤 Tu Cuenta");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.config.user_email.is_some() {
                    if ui.button("🚪 Cerrar Sesión").clicked() {
                        self.logout();
                    }
                }
            });
        });
        ui.separator();

        if let Some(email) = &self.config.user_email {
            // Usuario autenticado
            ui.label(format!("📧 Email: {}", email));
            ui.label(format!("💎 Tier: {}", self.config.subscription_tier.display_name()));
            ui.label(format!("⭐ Créditos disponibles: {}/{}", 
                self.config.credits_available,
                self.config.subscription_tier.credits_per_month()));

            // Barra de progreso de créditos
            let progress = self.config.credits_available as f32 
                / self.config.subscription_tier.credits_per_month() as f32;
            ui.add(egui::ProgressBar::new(progress).text("Créditos"));

            ui.add_space(10.0);

            // Mostrar features del tier actual
            ui.label("✨ Funcionalidades de tu plan:");
            for feature in self.config.subscription_tier.features() {
                ui.label(format!("  • {}", feature));
            }

        } else {
            // Usuario no autenticado
            ui.label("🔓 No has iniciado sesión");
            ui.add_space(5.0);

            if !self.show_register_form {
                // Formulario de login
                ui.label("📧 Email:");
                ui.text_edit_singleline(&mut self.login_email);
                
                ui.label("🔑 Contraseña:");
                ui.add(egui::TextEdit::singleline(&mut self.login_password).password(true));

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("🔐 Iniciar Sesión").clicked() {
                        self.handle_login();
                    }
                    if ui.button("📝 Registrarse").clicked() {
                        self.show_register_form = true;
                    }
                });

            } else {
                // Formulario de registro
                ui.heading("📝 Crear Nueva Cuenta");
                ui.separator();

                ui.label("📧 Email:");
                ui.text_edit_singleline(&mut self.register_email);
                
                ui.label("🔑 Contraseña:");
                ui.add(egui::TextEdit::singleline(&mut self.register_password).password(true));
                
                ui.label("🔑 Confirmar Contraseña:");
                ui.add(egui::TextEdit::singleline(&mut self.register_confirm_password).password(true));

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("✅ Crear Cuenta").clicked() {
                        self.handle_register();
                    }
                    if ui.button("❌ Cancelar").clicked() {
                        self.show_register_form = false;
                    }
                });
            }

            if let Some(msg) = &self.auth_message {
                ui.add_space(5.0);
                ui.colored_label(egui::Color32::YELLOW, msg);
            }
        }
    });

    ui.add_space(15.0);

    // Panel de Planes de Suscripción
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.heading("💰 Planes de Suscripción");
        ui.separator();

        ui.horizontal(|ui| {
            // Plan Free
            self.render_tier_card(ui, crate::config::SubscriptionTier::Free);
            ui.add_space(10.0);
            // Plan Basic
            self.render_tier_card(ui, crate::config::SubscriptionTier::Basic);
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            // Plan Pro
            self.render_tier_card(ui, crate::config::SubscriptionTier::Pro);
            ui.add_space(10.0);
            // Plan Enterprise
            self.render_tier_card(ui, crate::config::SubscriptionTier::Enterprise);
        });
    });

    ui.add_space(15.0);

    // Panel de Chat con IA
    if self.config.user_email.is_some() {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("💬 Chat con Asistente IA");
            ui.separator();

            // Área de mensajes
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if self.chat_messages.is_empty() {
                        ui.label("🤖 ¡Hola! Soy tu asistente de streaming con IA.");
                        ui.label("Puedo ayudarte con:");
                        ui.label("  • Optimizar tu configuración de OBS");
                        ui.label("  • Detectar y resolver problemas");
                        ui.label("  • Sugerencias para mejorar tu stream");
                        ui.label("  • Generar overlays y animaciones");
                    } else {
                        for msg in &self.chat_messages {
                            self.render_chat_message(ui, msg);
                        }
                    }

                    if self.ai_thinking {
                        ui.label("🤔 IA pensando...");
                        ui.spinner();
                    }
                });

            ui.separator();

            // Input de chat
            ui.horizontal(|ui| {
                let response = ui.add_sized(
                    [ui.available_width() - 70.0, 30.0],
                    egui::TextEdit::singleline(&mut self.chat_input)
                        .hint_text("Escribe tu pregunta...")
                );

                if ui.button("📤 Enviar").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    if !self.chat_input.is_empty() && !self.ai_thinking {
                        self.send_chat_message();
                    }
                }
            });

            ui.label(format!("💰 Costo: 1 crédito por mensaje • Disponibles: {}", 
                self.config.credits_available));
        });
    } else {
        ui.colored_label(egui::Color32::GRAY, "💬 Inicia sesión para usar el chat con IA");
    }
}

fn render_tier_card(&mut self, ui: &mut egui::Ui, tier: crate::config::SubscriptionTier) {
    let is_current = self.config.subscription_tier == tier;
    
    egui::Frame::none()
        .fill(if is_current { 
            egui::Color32::from_rgb(40, 60, 80) 
        } else { 
            egui::Color32::from_rgb(30, 30, 30) 
        })
        .stroke(if is_current {
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 180, 255))
        } else {
            egui::Stroke::new(1.0, egui::Color32::GRAY)
        })
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_width(180.0);
            
            ui.heading(tier.display_name());
            if is_current {
                ui.label("✅ Plan Actual");
            }
            ui.separator();

            ui.label(format!("{} créditos/mes", tier.credits_per_month()));
            ui.add_space(5.0);

            for feature in tier.features() {
                ui.label(format!("• {}", feature));
            }

            ui.add_space(5.0);

            if !is_current {
                if ui.button("⬆️ Actualizar").clicked() {
                    self.upgrade_tier(tier);
                }
            }
        });
}

fn render_chat_message(&self, ui: &mut egui::Ui, msg: &ChatMessage) {
    let (bg_color, align, prefix) = match msg.role {
        MessageRole::User => (egui::Color32::from_rgb(50, 80, 120), egui::Align::RIGHT, "👤"),
        MessageRole::Assistant => (egui::Color32::from_rgb(60, 60, 60), egui::Align::LEFT, "🤖"),
        MessageRole::System => (egui::Color32::from_rgb(80, 60, 40), egui::Align::Center, "ℹ️"),
    };

    ui.with_layout(egui::Layout::top_down(align), |ui| {
        egui::Frame::none()
            .fill(bg_color)
            .inner_margin(8.0)
            .rounding(5.0)
            .show(ui, |ui| {
                ui.set_max_width(400.0);
                ui.label(format!("{} {}", prefix, msg.content));
                ui.label(egui::RichText::new(&msg.timestamp).small().weak());
            });
    });
    ui.add_space(5.0);
}

fn send_chat_message(&mut self) {
    let user_message = self.chat_input.clone();
    self.chat_input.clear();

    // Agregar mensaje del usuario
    self.chat_messages.push(ChatMessage {
        role: MessageRole::User,
        content: user_message.clone(),
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
    });

    // Verificar créditos
    if self.config.credits_available == 0 {
        self.chat_messages.push(ChatMessage {
            role: MessageRole::System,
            content: "⚠️ No tienes créditos disponibles. Actualiza tu plan para continuar.".to_string(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        return;
    }

    self.ai_thinking = true;
    self.config.credits_available = self.config.credits_available.saturating_sub(1);

    // Simular respuesta de IA (aquí iría la integración real con Gemini)
    let response = self.get_ai_response(&user_message);
    
    self.chat_messages.push(ChatMessage {
        role: MessageRole::Assistant,
        content: response,
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
    });

    self.ai_thinking = false;
    
    // Guardar config actualizada
    let _ = self.config.save();
}

fn get_ai_response(&self, _user_message: &str) -> String {
    // Mock response - en producción, llamaría a Gemini API
    "🤖 Gracias por tu pregunta. Actualmente estoy analizando tu configuración de OBS y tus stats de hardware. ¿Te gustaría que te ayude a optimizar tu configuración de streaming?".to_string()
}

fn handle_login(&mut self) {
    if self.login_email.is_empty() || self.login_password.is_empty() {
        self.auth_message = Some("⚠️ Por favor completa todos los campos".to_string());
        return;
    }

    // Mock login - en producción llamaría a tu API
    self.config.user_email = Some(self.login_email.clone());
    self.config.user_token = Some(format!("mock_token_{}", chrono::Local::now().timestamp()));
    self.config.subscription_tier = crate::config::SubscriptionTier::Free;
    self.config.credits_available = 100;
    
    self.auth_message = Some("✅ Sesión iniciada correctamente".to_string());
    self.login_password.clear();
    
    let _ = self.config.save();
}

fn handle_register(&mut self) {
    if self.register_email.is_empty() || self.register_password.is_empty() {
        self.auth_message = Some("⚠️ Por favor completa todos los campos".to_string());
        return;
    }

    if self.register_password != self.register_confirm_password {
        self.auth_message = Some("⚠️ Las contraseñas no coinciden".to_string());
        return;
    }

    // Mock register - en producción llamaría a tu API
    self.config.user_email = Some(self.register_email.clone());
    self.config.user_token = Some(format!("mock_token_{}", chrono::Local::now().timestamp()));
    self.config.subscription_tier = crate::config::SubscriptionTier::Free;
    self.config.credits_available = 100;
    
    self.auth_message = Some("✅ Cuenta creada exitosamente".to_string());
    self.show_register_form = false;
    self.register_password.clear();
    self.register_confirm_password.clear();
    
    let _ = self.config.save();
}

fn logout(&mut self) {
    self.config.user_email = None;
    self.config.user_token = None;
    self.config.subscription_tier = crate::config::SubscriptionTier::Free;
    self.config.credits_available = 0;
    self.chat_messages.clear();
    
    let _ = self.config.save();
}

fn upgrade_tier(&mut self, tier: crate::config::SubscriptionTier) {
    // Mock upgrade - en producción abriría página de pago
    self.config.subscription_tier = tier.clone();
    self.config.credits_available = tier.credits_per_month();
    
    self.chat_messages.push(ChatMessage {
        role: MessageRole::System,
        content: format!("✅ Plan actualizado a {}. Se han agregado {} créditos.", 
            tier.display_name(), tier.credits_per_month()),
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
    });
    
    let _ = self.config.save();
}
