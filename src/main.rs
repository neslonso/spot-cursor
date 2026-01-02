//! SpotCursor - Spotlight portable estilo PowerToys
//!
//! ## Funcionamiento
//!
//! - **Activación:** Doble pulsación de Ctrl (izquierdo o derecho)
//! - **Desactivación:** Clic de ratón, cualquier tecla, o auto-fade tras inactividad
//!
//! ## Arquitectura
//!
//! La aplicación funciona mediante:
//! 1. Una ventana transparente superpuesta que cubre todos los monitores
//! 2. Hooks globales de teclado y ratón para detectar eventos
//! 3. Una región GDI que crea el efecto "agujero" alrededor del cursor

#![windows_subsystem = "windows"]

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::OnceLock;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// =============================================================================
// CONFIGURACIÓN
// =============================================================================

/// Configuración del comportamiento del spotlight
struct Config;

impl Config {
    /// Tiempo máximo entre pulsaciones de Ctrl para activar (ms)
    const DOUBLE_TAP_TIME_MS: u64 = 400;

    /// Tiempo mínimo entre pulsaciones para evitar falsos positivos (ms)
    const DOUBLE_TAP_MIN_TIME_MS: u64 = 50;

    /// Opacidad del fondo oscuro (0-255)
    const BACKDROP_OPACITY: u8 = 180;

    /// Radio del círculo de luz alrededor del cursor (píxeles)
    const SPOTLIGHT_RADIUS: i32 = 100;

    /// Tiempo de inactividad antes de auto-ocultar (ms)
    const AUTO_HIDE_DELAY_MS: u64 = 2000;

    /// Intervalo de actualización del spotlight (ms)
    const UPDATE_INTERVAL_MS: u32 = 16; // ~60 FPS
}

// =============================================================================
// MENSAJES Y CONSTANTES WINDOWS
// =============================================================================

/// Mensaje personalizado para mostrar el spotlight
const WM_USER_SHOW_SPOTLIGHT: u32 = WM_USER + 1;

/// Mensaje personalizado para ocultar el spotlight
const WM_USER_HIDE_SPOTLIGHT: u32 = WM_USER + 2;

/// ID del timer de actualización
const TIMER_UPDATE: usize = 1;

// =============================================================================
// TIPOS Y WRAPPERS
// =============================================================================

/// Wrapper thread-safe para HWND
///
/// HWND es un handle opaco de Windows que puede compartirse entre threads
#[derive(Clone, Copy)]
struct SafeHwnd(HWND);

unsafe impl Send for SafeHwnd {}
unsafe impl Sync for SafeHwnd {}

impl SafeHwnd {
    /// Obtiene el HWND interno
    #[inline]
    fn get(&self) -> HWND {
        self.0
    }
}

/// Representa una posición en coordenadas de pantalla
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn from_point(point: POINT) -> Self {
        Self::new(point.x, point.y)
    }
}

/// Representa las dimensiones del área de pantalla virtual
#[derive(Debug, Clone, Copy)]
struct VirtualScreen {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl VirtualScreen {
    /// Obtiene las dimensiones actuales de la pantalla virtual
    unsafe fn get_current() -> Self {
        Self {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
            width: GetSystemMetrics(SM_CXVIRTUALSCREEN),
            height: GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    }
}

// =============================================================================
// ESTADO GLOBAL
// =============================================================================

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

/// Estado global de la aplicación
///
/// Se usa estado global con atomics porque los hooks de Windows requieren
/// funciones estáticas que no pueden capturar estado local.
struct GlobalState;

impl GlobalState {

    /// Verifica si el spotlight está activo
    #[inline]
    fn is_active() -> bool {
        SPOTLIGHT_ACTIVE.load(Ordering::Relaxed)
    }

    /// Activa o desactiva el spotlight
    #[inline]
    fn set_active(active: bool) {
        SPOTLIGHT_ACTIVE.store(active, Ordering::Relaxed);
    }

    /// Obtiene la última posición conocida del cursor
    fn get_last_position() -> Position {
        Position::new(
            LAST_MOUSE_X.load(Ordering::Relaxed),
            LAST_MOUSE_Y.load(Ordering::Relaxed),
        )
    }

    /// Actualiza la posición del cursor y el timestamp
    fn update_position(pos: Position) {
        LAST_MOUSE_X.store(pos.x, Ordering::Relaxed);
        LAST_MOUSE_Y.store(pos.y, Ordering::Relaxed);
        LAST_MOVE_TIME.store(get_current_time_ms(), Ordering::Relaxed);
    }

    /// Obtiene el tiempo transcurrido desde el último movimiento (ms)
    fn time_since_last_move() -> u64 {
        let now = get_current_time_ms();
        let last = LAST_MOVE_TIME.load(Ordering::Relaxed);
        now.saturating_sub(last)
    }

    /// Registra una pulsación de Ctrl y devuelve si fue doble tap
    fn register_ctrl_press() -> bool {
        let now = get_current_time_ms();
        let last = LAST_CTRL_TIME.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last);

        LAST_CTRL_TIME.store(now, Ordering::Relaxed);

        elapsed > Config::DOUBLE_TAP_MIN_TIME_MS
            && elapsed < Config::DOUBLE_TAP_TIME_MS
    }

    /// Obtiene el handle de la ventana del spotlight
    fn get_hwnd() -> Option<HWND> {
        SPOTLIGHT_HWND.get().map(|h| h.get())
    }

    /// Establece el handle de la ventana del spotlight
    fn set_hwnd(hwnd: HWND) {
        let _ = SPOTLIGHT_HWND.set(SafeHwnd(hwnd));
    }
}

// =============================================================================
// GESTIÓN DE VENTANA
// =============================================================================

/// Registra la clase de ventana para el spotlight
unsafe fn register_window_class(instance: HMODULE) -> Result<()> {
    let class_name = w!("SpotCursorSpotlight");

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance.into(),
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
        lpszClassName: class_name,
        ..Default::default()
    };

    if RegisterClassExW(&wc) == 0 {
        return Err(Error::from_win32());
    }

    Ok(())
}

/// Crea la ventana del spotlight
unsafe fn create_spotlight_window(instance: HMODULE) -> Result<HWND> {
    let screen = VirtualScreen::get_current();

    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
        w!("SpotCursorSpotlight"),
        w!("SpotCursor"),
        WS_POPUP,
        screen.x,
        screen.y,
        screen.width,
        screen.height,
        None,
        None,
        instance,
        None,
    )?;

    // Configurar opacidad de la capa
    SetLayeredWindowAttributes(
        hwnd,
        COLORREF(0),
        Config::BACKDROP_OPACITY,
        LWA_ALPHA,
    )?;

    Ok(hwnd)
}

/// Procedimiento de ventana (maneja mensajes de Windows)
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_USER_SHOW_SPOTLIGHT => {
            show_spotlight(hwnd);
            LRESULT(0)
        }
        WM_USER_HIDE_SPOTLIGHT => {
            hide_spotlight(hwnd);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_UPDATE {
                update_spotlight(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// =============================================================================
// HOOKS GLOBALES
// =============================================================================

/// Hook de teclado: detecta doble Ctrl y otras teclas
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let is_key_down = wparam.0 == WM_KEYDOWN as usize;

        if is_key_down {
            // Detectar doble pulsación de Ctrl
            if is_ctrl_key(kb.vkCode) {
                if GlobalState::register_ctrl_press() {
                    toggle_spotlight();
                }
            }
            // Cualquier otra tecla oculta el spotlight
            else if GlobalState::is_active() {
                send_hide_message();
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

/// Hook de ratón: detecta clics
unsafe extern "system" fn mouse_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 && GlobalState::is_active() {
        if is_mouse_button_down(wparam.0) {
            send_hide_message();
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

/// Verifica si una tecla virtual es Ctrl
#[inline]
fn is_ctrl_key(vk_code: u32) -> bool {
    vk_code == VK_LCONTROL.0 as u32 || vk_code == VK_RCONTROL.0 as u32
}

/// Verifica si un mensaje de ratón es un clic
#[inline]
fn is_mouse_button_down(msg: usize) -> bool {
    matches!(
        msg as u32,
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN
    )
}

/// Alterna el estado del spotlight (mostrar/ocultar)
fn toggle_spotlight() {
    if let Some(hwnd) = GlobalState::get_hwnd() {
        unsafe {
            let message = if GlobalState::is_active() {
                WM_USER_HIDE_SPOTLIGHT
            } else {
                WM_USER_SHOW_SPOTLIGHT
            };
            let _ = PostMessageW(hwnd, message, WPARAM(0), LPARAM(0));
        }
    }
}

/// Envía mensaje para ocultar el spotlight
fn send_hide_message() {
    if let Some(hwnd) = GlobalState::get_hwnd() {
        unsafe {
            let _ = PostMessageW(hwnd, WM_USER_HIDE_SPOTLIGHT, WPARAM(0), LPARAM(0));
        }
    }
}

// =============================================================================
// LÓGICA DEL SPOTLIGHT
// =============================================================================

/// Muestra el spotlight en la posición actual del cursor
unsafe fn show_spotlight(hwnd: HWND) {
    // Evitar mostrar si ya está activo
    if GlobalState::is_active() {
        return;
    }

    GlobalState::set_active(true);

    // Obtener posición del cursor
    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);
    let cursor_pos = Position::from_point(point);

    // Actualizar estado
    GlobalState::update_position(cursor_pos);

    // Actualizar geometría de la ventana
    let screen = VirtualScreen::get_current();
    let _ = SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        screen.x,
        screen.y,
        screen.width,
        screen.height,
        SWP_NOACTIVATE,
    );

    // Aplicar región con agujero en el cursor
    apply_spotlight_region(hwnd, cursor_pos, screen);

    // Mostrar ventana sin activarla
    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

    // Iniciar timer de actualización
    let _ = SetTimer(hwnd, TIMER_UPDATE, Config::UPDATE_INTERVAL_MS, None);
}

/// Oculta el spotlight
unsafe fn hide_spotlight(hwnd: HWND) {
    // Evitar ocultar si ya está inactivo
    if !GlobalState::is_active() {
        return;
    }

    GlobalState::set_active(false);

    // Detener timer
    let _ = KillTimer(hwnd, TIMER_UPDATE);

    // Ocultar ventana
    let _ = ShowWindow(hwnd, SW_HIDE);

    // Eliminar región
    let _ = SetWindowRgn(hwnd, None, true);
}

/// Actualiza el spotlight siguiendo el cursor
unsafe fn update_spotlight(hwnd: HWND) {
    if !GlobalState::is_active() {
        return;
    }

    // Obtener posición actual del cursor
    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);
    let current_pos = Position::from_point(point);
    let last_pos = GlobalState::get_last_position();

    // Verificar si el cursor se movió
    if current_pos != last_pos {
        // Cursor en movimiento: actualizar región
        GlobalState::update_position(current_pos);

        let screen = VirtualScreen::get_current();
        apply_spotlight_region(hwnd, current_pos, screen);
    } else {
        // Cursor quieto: verificar timeout de auto-hide
        if GlobalState::time_since_last_move() > Config::AUTO_HIDE_DELAY_MS {
            hide_spotlight(hwnd);
        }
    }
}

// =============================================================================
// GESTIÓN DE REGIÓN GDI
// =============================================================================

/// Aplica la región del spotlight (fondo con agujero circular)
unsafe fn apply_spotlight_region(
    hwnd: HWND,
    cursor_pos: Position,
    screen: VirtualScreen,
) {
    // Convertir a coordenadas relativas a la ventana
    let rel_x = cursor_pos.x - screen.x;
    let rel_y = cursor_pos.y - screen.y;

    // Crear región rectangular (todo el fondo)
    let backdrop_region = CreateRectRgn(0, 0, screen.width, screen.height);

    // Crear región elíptica (el agujero)
    let hole_region = CreateEllipticRgn(
        rel_x - Config::SPOTLIGHT_RADIUS,
        rel_y - Config::SPOTLIGHT_RADIUS,
        rel_x + Config::SPOTLIGHT_RADIUS,
        rel_y + Config::SPOTLIGHT_RADIUS,
    );

    // Restar el agujero del fondo
    let _ = CombineRgn(backdrop_region, backdrop_region, hole_region, RGN_DIFF);

    // Aplicar región a la ventana (toma ownership de backdrop_region)
    let _ = SetWindowRgn(hwnd, backdrop_region, true);

    // Limpiar región temporal
    let _ = DeleteObject(hole_region);
}

// =============================================================================
// UTILIDADES
// =============================================================================

/// Obtiene el tiempo actual del sistema en milisegundos
#[inline]
fn get_current_time_ms() -> u64 {
    unsafe { GetTickCount64() }
}

// =============================================================================
// PUNTO DE ENTRADA
// =============================================================================

fn main() -> Result<()> {
    unsafe {
        // Obtener handle de la instancia
        let instance = GetModuleHandleW(None)?;

        // Registrar clase de ventana
        register_window_class(instance)?;

        // Crear ventana del spotlight
        let hwnd = create_spotlight_window(instance)?;
        GlobalState::set_hwnd(hwnd);

        // Instalar hooks globales
        let keyboard_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            instance,
            0,
        )?;

        let mouse_hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(mouse_hook_proc),
            instance,
            0,
        )?;

        // Bucle de mensajes
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);

            // ret.0 == 0: WM_QUIT, ret.0 == -1: error
            if ret.0 == 0 || ret.0 == -1 {
                break;
            }

            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }

        // Limpiar hooks
        let _ = UnhookWindowsHookEx(keyboard_hook);
        let _ = UnhookWindowsHookEx(mouse_hook);

        Ok(())
    }
}
