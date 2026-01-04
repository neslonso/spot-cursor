//! Configuración de la aplicación y persistencia

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::OnceLock;

/// Valores por defecto de la configuración
pub struct ConfigDefaults;

impl ConfigDefaults {
    pub const DOUBLE_TAP_TIME_MS: u64 = 400;
    pub const DOUBLE_TAP_MIN_TIME_MS: u64 = 50;
    pub const BACKDROP_OPACITY: u8 = 180;
    pub const SPOTLIGHT_RADIUS: i32 = 100;
    pub const AUTO_HIDE_DELAY_MS: u64 = 2000;
    pub const UPDATE_INTERVAL_MS: u32 = 16; // ~60 FPS

    // Constantes de animación
    pub const ANIMATION_INTERVAL_MS: u32 = 16; // ~60 FPS
    pub const ANIMATION_ENABLED: bool = true;
    pub const ANIMATION_INITIAL_RADIUS: i32 = 600; // Radio inicial grande
    pub const ANIMATION_DURATION_MS: u64 = 300; // Duración total de la animación

    // Color por defecto del backdrop (negro)
    pub const BACKDROP_COLOR: u32 = 0x00000000; // Negro
}

/// Configuración serializable para persistencia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub double_tap_time_ms: u64,
    pub backdrop_opacity: u8,
    pub backdrop_color: u32,
    pub spotlight_radius: i32,
    pub auto_hide_delay_ms: u64,
    pub animation_enabled: bool,
    pub animation_initial_radius: i32,
    pub animation_duration_ms: u64,
}

impl Settings {
    /// Crea una configuración con valores por defecto
    pub fn default() -> Self {
        Self {
            double_tap_time_ms: ConfigDefaults::DOUBLE_TAP_TIME_MS,
            backdrop_opacity: ConfigDefaults::BACKDROP_OPACITY,
            backdrop_color: ConfigDefaults::BACKDROP_COLOR,
            spotlight_radius: ConfigDefaults::SPOTLIGHT_RADIUS,
            auto_hide_delay_ms: ConfigDefaults::AUTO_HIDE_DELAY_MS,
            animation_enabled: ConfigDefaults::ANIMATION_ENABLED,
            animation_initial_radius: ConfigDefaults::ANIMATION_INITIAL_RADIUS,
            animation_duration_ms: ConfigDefaults::ANIMATION_DURATION_MS,
        }
    }

    /// Valida que los valores estén en rangos válidos
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.double_tap_time_ms < 50 || self.double_tap_time_ms > 1000 {
            return Err("Double tap time debe estar entre 50-1000ms".to_string());
        }
        if self.spotlight_radius < 50 || self.spotlight_radius > 300 {
            return Err("Radio debe estar entre 50-300 píxeles".to_string());
        }
        if self.auto_hide_delay_ms < 500 || self.auto_hide_delay_ms > 10000 {
            return Err("Auto-hide delay debe estar entre 500-10000ms".to_string());
        }
        if self.animation_initial_radius < 100 || self.animation_initial_radius > 1000 {
            return Err("Radio inicial de animación debe estar entre 100-1000 píxeles".to_string());
        }
        if self.animation_duration_ms < 100 || self.animation_duration_ms > 2000 {
            return Err("Duración de animación debe estar entre 100-2000ms".to_string());
        }
        Ok(())
    }
}

/// Configuración runtime con valores atómicos para acceso thread-safe
pub struct RuntimeConfig {
    double_tap_time_ms: AtomicU64,
    backdrop_opacity: AtomicU8,
    backdrop_color: AtomicU32,
    spotlight_radius: AtomicI32,
    auto_hide_delay_ms: AtomicU64,
    animation_enabled: AtomicBool,
    animation_initial_radius: AtomicI32,
    animation_duration_ms: AtomicU64,
}

impl RuntimeConfig {
    /// Crea una configuración runtime con valores por defecto
    pub fn new() -> Self {
        Self {
            double_tap_time_ms: AtomicU64::new(ConfigDefaults::DOUBLE_TAP_TIME_MS),
            backdrop_opacity: AtomicU8::new(ConfigDefaults::BACKDROP_OPACITY),
            backdrop_color: AtomicU32::new(ConfigDefaults::BACKDROP_COLOR),
            spotlight_radius: AtomicI32::new(ConfigDefaults::SPOTLIGHT_RADIUS),
            auto_hide_delay_ms: AtomicU64::new(ConfigDefaults::AUTO_HIDE_DELAY_MS),
            animation_enabled: AtomicBool::new(ConfigDefaults::ANIMATION_ENABLED),
            animation_initial_radius: AtomicI32::new(ConfigDefaults::ANIMATION_INITIAL_RADIUS),
            animation_duration_ms: AtomicU64::new(ConfigDefaults::ANIMATION_DURATION_MS),
        }
    }

    /// Carga valores desde Settings
    pub fn load_from(&self, settings: &Settings) {
        self.double_tap_time_ms
            .store(settings.double_tap_time_ms, Ordering::Relaxed);
        self.backdrop_opacity
            .store(settings.backdrop_opacity, Ordering::Relaxed);
        self.backdrop_color
            .store(settings.backdrop_color, Ordering::Relaxed);
        self.spotlight_radius
            .store(settings.spotlight_radius, Ordering::Relaxed);
        self.auto_hide_delay_ms
            .store(settings.auto_hide_delay_ms, Ordering::Relaxed);
        self.animation_enabled
            .store(settings.animation_enabled, Ordering::Relaxed);
        self.animation_initial_radius
            .store(settings.animation_initial_radius, Ordering::Relaxed);
        self.animation_duration_ms
            .store(settings.animation_duration_ms, Ordering::Relaxed);
    }

    /// Exporta valores actuales a Settings
    #[allow(dead_code)]
    pub fn to_settings(&self) -> Settings {
        Settings {
            double_tap_time_ms: self.double_tap_time_ms.load(Ordering::Relaxed),
            backdrop_opacity: self.backdrop_opacity.load(Ordering::Relaxed),
            backdrop_color: self.backdrop_color.load(Ordering::Relaxed),
            spotlight_radius: self.spotlight_radius.load(Ordering::Relaxed),
            auto_hide_delay_ms: self.auto_hide_delay_ms.load(Ordering::Relaxed),
            animation_enabled: self.animation_enabled.load(Ordering::Relaxed),
            animation_initial_radius: self.animation_initial_radius.load(Ordering::Relaxed),
            animation_duration_ms: self.animation_duration_ms.load(Ordering::Relaxed),
        }
    }

    /// Obtiene el tiempo máximo entre pulsaciones de Ctrl
    #[inline]
    pub fn double_tap_time_ms(&self) -> u64 {
        self.double_tap_time_ms.load(Ordering::Relaxed)
    }

    /// Obtiene la opacidad del fondo
    #[inline]
    pub fn backdrop_opacity(&self) -> u8 {
        self.backdrop_opacity.load(Ordering::Relaxed)
    }

    /// Obtiene el radio del spotlight
    #[inline]
    pub fn spotlight_radius(&self) -> i32 {
        self.spotlight_radius.load(Ordering::Relaxed)
    }

    /// Obtiene el tiempo de auto-hide
    #[inline]
    pub fn auto_hide_delay_ms(&self) -> u64 {
        self.auto_hide_delay_ms.load(Ordering::Relaxed)
    }

    /// Obtiene el color del fondo (COLORREF)
    #[inline]
    pub fn backdrop_color(&self) -> u32 {
        self.backdrop_color.load(Ordering::Relaxed)
    }

    /// Obtiene si la animación está habilitada
    #[inline]
    pub fn animation_enabled(&self) -> bool {
        self.animation_enabled.load(Ordering::Relaxed)
    }

    /// Obtiene el radio inicial de la animación
    #[inline]
    pub fn animation_initial_radius(&self) -> i32 {
        self.animation_initial_radius.load(Ordering::Relaxed)
    }

    /// Obtiene la duración de la animación en milisegundos
    #[inline]
    pub fn animation_duration_ms(&self) -> u64 {
        self.animation_duration_ms.load(Ordering::Relaxed)
    }

    /// Establece el tiempo máximo entre pulsaciones de Ctrl
    #[inline]
    pub fn set_double_tap_time_ms(&self, value: u64) {
        self.double_tap_time_ms.store(value, Ordering::Relaxed);
    }

    /// Establece la opacidad del fondo
    #[inline]
    pub fn set_backdrop_opacity(&self, value: u8) {
        self.backdrop_opacity.store(value, Ordering::Relaxed);
    }

    /// Establece el radio del spotlight
    #[inline]
    pub fn set_spotlight_radius(&self, value: i32) {
        self.spotlight_radius.store(value, Ordering::Relaxed);
    }

    /// Establece el tiempo de auto-hide
    #[inline]
    pub fn set_auto_hide_delay_ms(&self, value: u64) {
        self.auto_hide_delay_ms.store(value, Ordering::Relaxed);
    }

    /// Establece el color del fondo (COLORREF)
    #[inline]
    pub fn set_backdrop_color(&self, value: u32) {
        self.backdrop_color.store(value, Ordering::Relaxed);
    }

    /// Establece si la animación está habilitada
    #[inline]
    pub fn set_animation_enabled(&self, value: bool) {
        self.animation_enabled.store(value, Ordering::Relaxed);
    }

    /// Establece el radio inicial de la animación
    #[inline]
    pub fn set_animation_initial_radius(&self, value: i32) {
        self.animation_initial_radius.store(value, Ordering::Relaxed);
    }

    /// Establece la duración de la animación
    #[inline]
    pub fn set_animation_duration_ms(&self, value: u64) {
        self.animation_duration_ms.store(value, Ordering::Relaxed);
    }
}

/// Instancia global de la configuración runtime
pub static RUNTIME_CONFIG: OnceLock<RuntimeConfig> = OnceLock::new();

// =============================================================================
// PERSISTENCIA
// =============================================================================

/// Obtiene la ruta del archivo de configuración
/// El archivo se llama igual que el ejecutable pero con extensión .json
/// Ejemplo: spot-cursor.exe -> spot-cursor.json
fn get_config_path() -> std::result::Result<PathBuf, String> {
    // Usar el mismo directorio que el ejecutable
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("No se pudo obtener la ruta del ejecutable: {}", e))?;

    let exe_dir = exe_path
        .parent()
        .ok_or("No se pudo obtener el directorio del ejecutable")?;

    // Obtener el nombre del ejecutable sin extensión y añadir .json
    let config_name = exe_path
        .file_stem()
        .ok_or("No se pudo obtener el nombre del ejecutable")?
        .to_string_lossy()
        .to_string()
        + ".json";

    let config_path = exe_dir.join(config_name);
    Ok(config_path)
}

/// Guarda la configuración en archivo
pub fn save_config(settings: &Settings) -> std::result::Result<(), String> {
    // Validar antes de guardar
    settings.validate()?;

    let path = get_config_path()?;
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Error al serializar config: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Error al guardar config: {}", e))?;

    Ok(())
}

/// Carga la configuración desde archivo
pub fn load_config() -> Settings {
    match get_config_path() {
        Ok(path) => {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(json) => match serde_json::from_str::<Settings>(&json) {
                        Ok(settings) => {
                            // Validar y retornar si es válido
                            if settings.validate().is_ok() {
                                return settings;
                            }
                        }
                        Err(_) => {}
                    },
                    Err(_) => {}
                }
            }
        }
        Err(_) => {}
    }

    // Si falla la carga por cualquier razón, usar valores por defecto
    Settings::default()
}
