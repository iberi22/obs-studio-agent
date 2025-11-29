mod config;
mod ui;

use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter("obs_agent_gui=debug,obs_agent_core=info,obs_agent_infra=info")
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "OBS Agent - AI Streaming Assistant",
        options,
        Box::new(|cc| Ok(Box::new(ui::OBSAgentApp::new(cc)))),
    ).map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}
