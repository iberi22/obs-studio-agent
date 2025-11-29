# 🚀 OBS Agent GUI - Modo Portable

## ✨ Características

- 🎨 **Interfaz Gráfica Moderna** - Construida con egui
- 💾 **Modo Portable** - No requiere instalación, configuración local
- 🔌 **Detección Automática** - Encuentra tu instalación de OBS automáticamente
- 🔑 **Gestión de API Keys** - Interfaz para configurar Gemini API
- 📊 **Monitoreo en Tiempo Real** - Hardware, health checks, anomalías
- ⚙️ **Configuración Persistente** - Guarda configuración en archivo local

---

## 🎯 Instalación Portable

### Opción 1: Usar Binario Compilado

```powershell
# Después de compilar, el ejecutable está en:
target\release\obs-agent-gui.exe

# Copiar a donde quieras usarlo
Copy-Item target\release\obs-agent-gui.exe D:\PortableApps\obs-agent-gui.exe

# Ejecutar
D:\PortableApps\obs-agent-gui.exe
```

### Opción 2: Compilar desde Fuente

```powershell
# Compilar en modo release
cargo build --release --bin obs-agent-gui

# El ejecutable está en target\release\obs-agent-gui.exe
```

---

## 📁 Estructura de Archivos Portable

```
📂 DirectorioPortable/
├── obs-agent-gui.exe           # Ejecutable principal
└── obs-agent-config.toml       # Configuración (se crea automáticamente)
```

La configuración se guarda automáticamente en el mismo directorio que el ejecutable.

---

## ⚙️ Configuración

### Primera Ejecución

1. **Ejecutar la aplicación**: Doble clic en `obs-agent-gui.exe`
2. **Ir a pestaña "Config"**
3. **Configurar OBS WebSocket**:
   - Host: `localhost` (por defecto)
   - Puerto: `4455` (por defecto)
   - Password: (si configuraste uno en OBS)
4. **Agregar Gemini API Key** (opcional):
   - Obtener en: https://ai.google.dev
   - Pegar en el campo correspondiente
5. **Auto-detectar OBS**: Click en "Auto-detectar" para encontrar tu instalación
6. **Guardar**: Click en "Guardar Configuración"

### Configuración Manual

También puedes editar `obs-agent-config.toml` directamente:

```toml
obs_config_dir = "C:\\Users\\TuUsuario\\AppData\\Roaming\\obs-studio"
obs_host = "localhost"
obs_port = 4455
obs_password = "tu_password_opcional"
gemini_api_key = "tu_api_key_aqui"
portable_mode = true
```

---

## 🎮 Uso de la Interfaz

### Pestaña 🏠 Inicio
- **Probar Conexión OBS**: Verifica conectividad con OBS
- **Detectar Hardware**: Analiza tu sistema
- **Health Check**: Validación completa pre-stream
- **Escanear Anomalías**: Detección de problemas

### Pestaña ⚙️ Config
- Configurar credenciales OBS WebSocket
- Agregar Gemini API Key
- Auto-detectar instalación de OBS
- Activar/desactivar modo portable

### Pestaña 🖥️ Hardware
- Información detallada de CPU, GPU, RAM
- Recomendaciones de encoder y configuración
- Detección de aceleración por hardware (NVENC, AMF, QSV)

### Pestaña 🏥 Health
- Pre-flight check completo del sistema
- Validación de escenas y fuentes
- Verificación de disponibilidad de recursos
- Estado de preparación para streaming

### Pestaña 🔍 Anomalías
- Detección en tiempo real de problemas
- Temperatura de CPU/GPU
- Frames perdidos
- Espacio en disco
- Audio saturado
- Sugerencias de corrección

---

## 🔌 Configurar OBS WebSocket

Para que OBS Agent pueda conectarse a OBS:

1. **Abrir OBS Studio**
2. **Ir a**: `Herramientas` → `WebSocket Server Settings`
3. **Activar**: `Enable WebSocket server`
4. **Configurar**:
   - Puerto: `4455` (recomendado)
   - Password: (opcional, pero recomendado)
5. **Aplicar y cerrar**

---

## 🚀 Acceso a Configuración de OBS

OBS Agent puede leer tu configuración actual de OBS para:

- Detectar escenas existentes
- Validar fuentes y dispositivos
- Analizar configuración de video/audio
- Proponer optimizaciones

### Ubicaciones de Config de OBS

**Windows:**
```
C:\Users\TuUsuario\AppData\Roaming\obs-studio
```

**Linux:**
```
~/.config/obs-studio
```

**macOS:**
```
~/Library/Application Support/obs-studio
```

La aplicación detecta automáticamente la ubicación con el botón "Auto-detectar".

---

## 🔑 Obtener Gemini API Key

1. Ir a: https://ai.google.dev
2. Click en "Get API Key"
3. Iniciar sesión con Google
4. Crear proyecto (o usar existente)
5. Generar API Key
6. Copiar y pegar en OBS Agent

**Nota**: La API key se guarda localmente en `obs-agent-config.toml` (modo portable).

---

## 🛠️ Solución de Problemas

### No se puede conectar a OBS

- ✅ Verificar que OBS está ejecutándose
- ✅ Confirmar que WebSocket está habilitado en OBS
- ✅ Revisar puerto (4455 por defecto)
- ✅ Verificar password (si está configurado)

### No detecta instalación de OBS

- ✅ OBS debe estar instalado (no portable de OBS)
- ✅ Buscar manualmente en `AppData\Roaming\obs-studio`
- ✅ Configurar ruta manualmente si es necesario

### Gemini API no funciona

- ✅ Verificar que la API key es válida
- ✅ Confirmar que hay créditos disponibles
- ✅ Revisar conexión a internet

---

## 📊 Diferencias con OBS Studio

| Aspecto | OBS Studio | OBS Agent |
|---------|------------|-----------|
| **Propósito** | Streaming/Recording | Monitoreo y optimización |
| **Config** | `obs-studio/` | `obs-agent-config.toml` |
| **Perfiles** | Múltiples perfiles | Config única compartida |
| **Puerto** | 4455 (WebSocket) | Se conecta al 4455 de OBS |
| **Datos** | Escenas, fuentes, etc. | Lee config de OBS (read-only) |

**Importante**: OBS Agent NO modifica los archivos de configuración de OBS Studio. Solo lee información para análisis.

---

## 🔄 Actualización

Para actualizar a una nueva versión:

1. Descargar nuevo `obs-agent-gui.exe`
2. Reemplazar el ejecutable anterior
3. Tu configuración (`obs-agent-config.toml`) se mantiene intacta

---

## 🐛 Reportar Problemas

Si encuentras algún problema:

1. Revisar logs en la terminal (si se ejecuta desde consola)
2. Verificar archivo de configuración
3. Reportar en: https://github.com/iberi22/obs-studio-agent/issues

---

## 📄 Licencia

MIT License

---

**¡Listo para usar! 🎉**

Ejecuta `obs-agent-gui.exe` y empieza a optimizar tus streams con IA.
