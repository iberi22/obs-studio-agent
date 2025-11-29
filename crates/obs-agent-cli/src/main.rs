use anyhow::Result;
use clap::{Parser, Subcommand};
use obs_agent_core::application::ports::*;
use obs_agent_core::domain::services::*;
use obs_agent_infra::*;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "obs-agent")]
#[command(about = "OBS Studio Agent - AI-powered streaming assistant", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// OBS WebSocket host
    #[arg(long, default_value = "localhost", env = "OBS_WEBSOCKET_HOST")]
    obs_host: String,

    /// OBS WebSocket port
    #[arg(long, default_value = "4455", env = "OBS_WEBSOCKET_PORT")]
    obs_port: u16,

    /// OBS WebSocket password
    #[arg(long, env = "OBS_WEBSOCKET_PASSWORD")]
    obs_password: Option<String>,

    /// Gemini API key
    #[arg(long, env = "GEMINI_API_KEY")]
    gemini_api_key: Option<String>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check system health
    Health {
        /// Quick check (only critical issues)
        #[arg(short, long)]
        quick: bool,
    },

    /// Detect hardware
    Hardware,

    /// List OBS scenes
    Scenes,

    /// Get OBS statistics
    Stats,

    /// Scan for anomalies
    Scan {
        /// Minimum severity (info, warning, critical)
        #[arg(short, long, default_value = "info")]
        severity: String,
    },

    /// Optimize OBS configuration
    Optimize,

    /// Test OBS connection
    Connect,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("obs_agent={},obs_agent_core={},obs_agent_infra={}", log_level, log_level, log_level))
        .init();

    // Load .env if exists
    let _ = dotenv::dotenv();

    // Run command
    match &cli.command {
        Commands::Connect => cmd_connect(&cli).await,
        Commands::Hardware => cmd_hardware(&cli).await,
        Commands::Scenes => cmd_scenes(&cli).await,
        Commands::Stats => cmd_stats(&cli).await,
        Commands::Health { quick } => cmd_health(&cli, *quick).await,
        Commands::Scan { severity } => cmd_scan(&cli, severity).await,
        Commands::Optimize => cmd_optimize(&cli).await,
    }
}

async fn cmd_connect(cli: &Cli) -> Result<()> {
    info!("Testing OBS connection...");

    let adapter = OBSAdapter::new(&cli.obs_host, cli.obs_port, cli.obs_password.clone());
    adapter.connect().await?;

    println!("✅ Successfully connected to OBS at {}:{}", cli.obs_host, cli.obs_port);

    adapter.disconnect().await?;
    Ok(())
}

async fn cmd_hardware(_cli: &Cli) -> Result<()> {
    info!("Detecting hardware...");

    let monitor = MonitorAdapter::new();
    let hardware = monitor.detect_hardware()?;

    println!("\n🖥️  HARDWARE INFORMATION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("OS: {} {}", hardware.os, hardware.os_version);
    println!("Hostname: {}", hardware.hostname);
    println!("\n💻 CPU:");
    println!("  Name: {}", hardware.cpu.brand);
    println!("  Cores: {} physical / {} logical", hardware.cpu.cores_physical, hardware.cpu.cores_logical);
    println!("  Frequency: {:.0} MHz", hardware.cpu.frequency_mhz);
    println!("\n🎮 GPU:");
    if let Some(gpu) = &hardware.gpu {
        println!("  Name: {}", gpu.name);
        println!("  Vendor: {:?}", gpu.vendor);
        println!("  NVENC: {}", if gpu.supports_nvenc { "Yes" } else { "No" });
        println!("  AMF: {}", if gpu.supports_amf { "Yes" } else { "No" });
        println!("  QSV: {}", if gpu.supports_qsv { "Yes" } else { "No" });
    } else {
        println!("  Not detected");
    }
    println!("\n💾 RAM:");
    println!("  Total: {:.1} GB", hardware.ram.total_gb);
    println!("  Available: {:.1} GB", hardware.ram.available_gb);
    println!("  Used: {:.1}%", hardware.ram.used_percent);
    println!("\n📊 RECOMMENDATIONS:");
    println!("  Encoder: {:?}", hardware.recommended_encoder);
    println!("  Preset: {}", hardware.recommended_preset);
    println!("  Resolution: {}x{}", hardware.recommended_resolution.0, hardware.recommended_resolution.1);
    println!("  FPS: {}", hardware.recommended_fps);
    println!("  Bitrate: {} kbps", hardware.recommended_bitrate);

    Ok(())
}

async fn cmd_scenes(cli: &Cli) -> Result<()> {
    info!("Fetching OBS scenes...");

    let obs = Arc::new(OBSAdapter::new(&cli.obs_host, cli.obs_port, cli.obs_password.clone())) as Arc<dyn OBSPort>;
    obs.connect().await?;

    let scenes = obs.get_scenes().await?;
    let current = obs.get_current_scene().await?;

    println!("\n🎬 OBS SCENES ({} total)", scenes.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for scene in scenes {
        let is_current = scene.name == current;
        let marker = if is_current { "▶ " } else { "  " };
        println!("{}{} ({} sources)", marker, scene.name, scene.sources.len());

        for source in &scene.sources {
            let status = if source.is_available { "✓" } else { "✗" };
            println!("    {} {} ({})", status, source.name, source.kind);
        }
    }

    obs.disconnect().await?;
    Ok(())
}

async fn cmd_stats(cli: &Cli) -> Result<()> {
    info!("Getting OBS stats...");

    let obs = Arc::new(OBSAdapter::new(&cli.obs_host, cli.obs_port, cli.obs_password.clone())) as Arc<dyn OBSPort>;
    obs.connect().await?;

    let stats = obs.get_stats().await?;

    println!("\n📈 OBS STATISTICS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("CPU Usage: {:.1}%", stats.cpu_usage);
    println!("Memory Usage: {:.1} MB", stats.memory_usage);
    println!("Active FPS: {:.1}", stats.active_fps);
    println!("\nFrames:");
    println!("  Render Total: {}", stats.render_total_frames);
    println!("  Render Skipped: {}", stats.render_skipped_frames);
    println!("  Output Total: {}", stats.output_total_frames);
    println!("  Output Skipped: {} ({:.2}%)",
        stats.output_skipped_frames,
        stats.dropped_frames_percent()
    );

    if stats.is_dropping_frames() {
        println!("\n⚠️  WARNING: Dropping frames! Consider reducing quality.");
    }

    obs.disconnect().await?;
    Ok(())
}

async fn cmd_health(cli: &Cli, quick: bool) -> Result<()> {
    if quick {
        info!("Running quick health check...");
    } else {
        info!("Running full health check...");
    }

    let obs = Arc::new(OBSAdapter::new(&cli.obs_host, cli.obs_port, cli.obs_password.clone())) as Arc<dyn OBSPort>;
    let monitor = Arc::new(MonitorAdapter::new()) as Arc<dyn MonitorPort>;

    let service = HealthCheckService::new(obs, monitor);

    if quick {
        let is_healthy = service.quick_check().await?;
        if is_healthy {
            println!("✅ System is healthy");
        } else {
            println!("❌ Critical issues detected");
        }
        return Ok(());
    }

    let report = service.check().await?;

    println!("\n🏥 HEALTH CHECK REPORT");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Timestamp: {}", report.timestamp);
    println!("Overall Status: {}", if report.is_healthy { "✅ Healthy" } else { "⚠️  Issues Found" });
    println!("Can Stream: {}", if report.can_stream { "Yes" } else { "No" });
    println!("Can Record: {}", if report.can_record { "Yes" } else { "No" });

    if !report.critical_issues.is_empty() {
        println!("\n🔴 CRITICAL ISSUES:");
        for issue in &report.critical_issues {
            println!("  • {}", issue);
        }
    }

    if !report.warnings.is_empty() {
        println!("\n⚠️  WARNINGS:");
        for warning in &report.warnings {
            println!("  • {}", warning);
        }
    }

    if !report.anomalies.is_empty() {
        println!("\n📋 DETAILED ANOMALIES:");
        for anomaly in &report.anomalies {
            println!("  [{:?}] {:?}: {}", anomaly.severity, anomaly.anomaly_type, anomaly.details);
            if !anomaly.recommended_action.is_empty() {
                println!("    → {}", anomaly.recommended_action);
            }
        }
    }

    println!("\n{}", report.summary());

    Ok(())
}

async fn cmd_scan(cli: &Cli, severity: &str) -> Result<()> {
    use obs_agent_core::domain::models::Severity;

    let min_severity = match severity.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    };

    info!("Scanning for anomalies (min severity: {:?})...", min_severity);

    let monitor = MonitorAdapter::new();
    let detector = AnomalyDetector::with_default_rules();

    let context = SystemContext {
        cpu_temp: monitor.get_cpu_temp().unwrap_or(0.0),
        gpu_temp: monitor.get_gpu_temp().unwrap_or(0.0),
        cpu_usage: monitor.get_cpu_usage().unwrap_or(0.0),
        memory_used_percent: monitor.get_memory_info()?.used_percent,
        disk_free_gb: monitor.get_disk_space()?.free_gb,
        obs_dropped_frames_percent: 0.0,
        obs_cpu_usage: 0.0,
        missing_sources: vec![],
        audio_peak_db: None,
        network_bitrate: None,
    };

    let anomalies = detector.scan_filtered(&context, min_severity);

    println!("\n🔍 ANOMALY SCAN RESULTS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Found {} anomalies", anomalies.len());

    if anomalies.is_empty() {
        println!("✅ No anomalies detected");
    } else {
        for anomaly in &anomalies {
            println!("\n[{:?}] {:?}", anomaly.severity, anomaly.anomaly_type);
            println!("  Details: {}", anomaly.details);
            if let Some(source) = &anomaly.source {
                println!("  Source: {}", source);
            }
            if !anomaly.recommended_action.is_empty() {
                println!("  Action: {}", anomaly.recommended_action);
            }
            println!("  Auto-fixable: {}", if anomaly.auto_fixable { "Yes" } else { "No" });
        }
    }

    Ok(())
}

async fn cmd_optimize(cli: &Cli) -> Result<()> {
    let api_key = cli.gemini_api_key.as_ref()
        .ok_or_else(|| anyhow::anyhow!("GEMINI_API_KEY not set"))?;

    info!("Optimizing OBS configuration with AI...");

    let monitor = MonitorAdapter::new();
    let hardware = monitor.detect_hardware()?;

    let ai = Arc::new(AIAdapter::new(api_key)) as Arc<dyn AIPort>;
    let optimizer = ConfigOptimizer::new(ai);

    let config = optimizer.optimize(&hardware).await?;

    println!("\n⚙️  OPTIMIZED CONFIGURATION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Video:");
    println!("  Resolution: {}x{}", config.video.output_width, config.video.output_height);
    println!("  FPS: {:.0}", config.video.fps());
    println!("Encoder: {}", config.encoder);
    println!("Preset: {}", config.preset);
    println!("Bitrate: {} kbps", config.bitrate);

    let is_valid = optimizer.validate_config(&config, &hardware);
    println!("\nValidation: {}", if is_valid { "✅ Valid" } else { "❌ Invalid" });

    Ok(())
}
