//! Estado global del spotlight

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::OnceLock;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::SystemInformation::GetTickCount64;

use crate::config::{ConfigDefaults, RUNTIME_CONFIG};
use crate::types::{Position, SafeHwnd};

/// Indica si el spotlight está actualmente visible
static SPOTLIGHT_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Timestamp de la última pulsación de Ctrl (para detectar doble tap)
static LAST_CTRL_TIME: AtomicU64 = AtomicU64::new(0);

/// Última posición X conocida del cursor
static LAST_MOUSE_X: AtomicI32 = AtomicI32::new(0);

/// Última posición Y conocida del cursor
static LAST_MOUSE_Y: AtomicI32 = AtomicI32::new(0);

/// Timestamp del último movimiento del cursor
static LAST_MOVE_TIME: AtomicU64 = AtomicU64::new(0);

/// Handle de la ventana del spotlight
static SPOTLIGHT_HWND: OnceLock<SafeHwnd> = OnceLock::new();

/// Indica si hay una animación en progreso
static ANIMATING: AtomicBool = AtomicBool::new(false);

/// Radio actual durante la animación
static ANIMATION_CURRENT_RADIUS: AtomicI32 = AtomicI32::new(0);

/// Timestamp de inicio de la animación
static ANIMATION_START_TIME: AtomicU64 = AtomicU64::new(0);

/// Estado global de la aplicación
///
/// Se usa estado global con atomics porque los hooks de Windows requieren
/// funciones estáticas que no pueden capturar estado local.
pub struct GlobalState;

impl GlobalState {
    /// Verifica si el spotlight está activo
    #[inline]
    pub fn is_active() -> bool {
        SPOTLIGHT_ACTIVE.load(Ordering::Relaxed)
    }

    /// Activa o desactiva el spotlight
    #[inline]
    pub fn set_active(active: bool) {
        SPOTLIGHT_ACTIVE.store(active, Ordering::Relaxed);
    }

    /// Obtiene la última posición conocida del cursor
    pub fn get_last_position() -> Position {
        Position::new(
            LAST_MOUSE_X.load(Ordering::Relaxed),
            LAST_MOUSE_Y.load(Ordering::Relaxed),
        )
    }

    /// Actualiza la posición del cursor y el timestamp
    pub fn update_position(pos: Position) {
        LAST_MOUSE_X.store(pos.x, Ordering::Relaxed);
        LAST_MOUSE_Y.store(pos.y, Ordering::Relaxed);
        LAST_MOVE_TIME.store(get_current_time_ms(), Ordering::Relaxed);
    }

    /// Obtiene el tiempo transcurrido desde el último movimiento (ms)
    pub fn time_since_last_move() -> u64 {
        let now = get_current_time_ms();
        let last = LAST_MOVE_TIME.load(Ordering::Relaxed);
        now.saturating_sub(last)
    }

    /// Registra una pulsación de Ctrl y devuelve si fue doble tap
    pub fn register_ctrl_press() -> bool {
        let now = get_current_time_ms();
        let last = LAST_CTRL_TIME.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last);

        LAST_CTRL_TIME.store(now, Ordering::Relaxed);

        let config = RUNTIME_CONFIG.get().unwrap();
        elapsed > ConfigDefaults::DOUBLE_TAP_MIN_TIME_MS && elapsed < config.double_tap_time_ms()
    }

    /// Obtiene el handle de la ventana del spotlight
    pub fn get_hwnd() -> Option<HWND> {
        SPOTLIGHT_HWND.get().map(|h| h.get())
    }

    /// Establece el handle de la ventana del spotlight
    pub fn set_hwnd(hwnd: HWND) {
        let _ = SPOTLIGHT_HWND.set(SafeHwnd(hwnd));
    }

    /// Inicia la animación del spotlight con un radio inicial
    pub fn start_animation(initial_radius: i32) {
        ANIMATION_CURRENT_RADIUS.store(initial_radius, Ordering::Relaxed);
        ANIMATION_START_TIME.store(get_current_time_ms(), Ordering::Relaxed);
        ANIMATING.store(true, Ordering::Relaxed);
    }

    /// Verifica si hay una animación en progreso
    #[inline]
    pub fn is_animating() -> bool {
        ANIMATING.load(Ordering::Relaxed)
    }

    /// Obtiene el radio actual de la animación
    #[inline]
    pub fn get_animation_radius() -> i32 {
        ANIMATION_CURRENT_RADIUS.load(Ordering::Relaxed)
    }

    /// Actualiza el radio de la animación y retorna el nuevo valor
    pub fn update_animation_radius(new_radius: i32) -> i32 {
        ANIMATION_CURRENT_RADIUS.store(new_radius, Ordering::Relaxed);
        new_radius
    }

    /// Detiene la animación
    pub fn stop_animation() {
        ANIMATING.store(false, Ordering::Relaxed);
    }

    /// Obtiene el tiempo transcurrido desde el inicio de la animación (ms)
    pub fn animation_elapsed_time() -> u64 {
        let now = get_current_time_ms();
        let start = ANIMATION_START_TIME.load(Ordering::Relaxed);
        now.saturating_sub(start)
    }
}

/// Obtiene el tiempo actual del sistema en milisegundos
#[inline]
fn get_current_time_ms() -> u64 {
    unsafe { GetTickCount64() }
}
