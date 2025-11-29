use std::process::Command;

/// Detecta si OBS Studio está corriendo
pub fn is_obs_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist")
            .args(&["/FI", "IMAGENAME eq obs64.exe"])
            .output();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("obs64.exe")
        } else {
            false
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("pgrep")
            .arg("obs")
            .output();
        
        output.is_ok() && output.unwrap().status.success()
    }
    
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pgrep")
            .arg("obs")
            .output();
        
        output.is_ok() && output.unwrap().status.success()
    }
}

/// Intenta lanzar OBS Studio
pub fn launch_obs() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Rutas comunes de OBS en Windows
        let paths = vec![
            r"C:\Program Files\obs-studio\bin\64bit\obs64.exe",
            r"C:\Program Files (x86)\obs-studio\bin\64bit\obs64.exe",
            r"C:\Users\Public\Desktop\OBS Studio.lnk",
        ];
        
        for path in paths {
            if std::path::Path::new(path).exists() {
                match Command::new(path).spawn() {
                    Ok(_) => {
                        // Esperar un poco para que OBS inicie
                        std::thread::sleep(std::time::Duration::from_secs(3));
                        return Ok(());
                    }
                    Err(e) => continue,
                }
            }
        }
        
        // Intentar lanzar via PATH
        match Command::new("obs64").spawn() {
            Ok(_) => {
                std::thread::sleep(std::time::Duration::from_secs(3));
                Ok(())
            }
            Err(_) => Err("No se pudo encontrar OBS Studio. Asegúrate de que esté instalado.".to_string())
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        match Command::new("obs").spawn() {
            Ok(_) => {
                std::thread::sleep(std::time::Duration::from_secs(3));
                Ok(())
            }
            Err(_) => Err("No se pudo lanzar OBS Studio".to_string())
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        match Command::new("open").arg("-a").arg("OBS").spawn() {
            Ok(_) => {
                std::thread::sleep(std::time::Duration::from_secs(3));
                Ok(())
            }
            Err(_) => Err("No se pudo lanzar OBS Studio".to_string())
        }
    }
}

/// Auto-launch: detecta si OBS está cerrado y lo abre automáticamente
pub fn ensure_obs_running() -> Result<bool, String> {
    if is_obs_running() {
        Ok(true)
    } else {
        println!("⚠️  OBS no está corriendo. Lanzando automáticamente...");
        launch_obs()?;
        
        // Verificar que se haya lanzado
        std::thread::sleep(std::time::Duration::from_secs(2));
        if is_obs_running() {
            println!("✅ OBS lanzado exitosamente");
            Ok(true)
        } else {
            Err("OBS se lanzó pero no está respondiendo".to_string())
        }
    }
}
