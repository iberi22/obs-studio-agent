// Test básico de configuración portable
// Los tests de UI completos requieren un contexto gráfico

use std::path::PathBuf;

#[test]
fn test_config_toml_serialization() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestConfig {
        obs_host: String,
        obs_port: u16,
        portable_mode: bool,
    }

    let config = TestConfig {
        obs_host: "localhost".to_string(),
        obs_port: 4455,
        portable_mode: true,
    };

    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("localhost"));
    assert!(toml_str.contains("4455"));

    let loaded: TestConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(loaded.obs_host, "localhost");
    assert_eq!(loaded.obs_port, 4455);
}

#[test]
fn test_path_operations() {
    let path = PathBuf::from("obs-agent-config.toml");
    assert_eq!(path.file_name().unwrap(), "obs-agent-config.toml");

    let exe_dir = std::env::current_exe().ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    assert!(exe_dir.is_some());
}
