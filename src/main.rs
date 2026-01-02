//! FindMyCursor - Spotlight portable estilo PowerToys
//!
//! Activación: Doble pulsación de Ctrl
//! Desactivación: Clic, tecla, o auto-fade tras dejar de mover

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

// === CONFIGURACIÓN ===
const DOUBLE_TAP_TIME_MS: u64 = 400;
const BACKDROP_OPACITY: u8 = 180;
const SPOTLIGHT_RADIUS: i32 = 100;
const FADE_AFTER_STOP_MS: u64 = 2000;
const UPDATE_INTERVAL_MS: u32 = 16;

// === WRAPPER PARA HWND ===
#[derive(Clone, Copy)]
struct SendHwnd(HWND);

// SAFETY: HWND es un handle opaco que Windows permite usar entre threads
unsafe impl Send for SendHwnd {}
unsafe impl Sync for SendHwnd {}

impl SendHwnd {
    fn get(&self) -> HWND {
        self.0
    }
}

// === ESTADO GLOBAL ===
static SPOTLIGHT_ACTIVE: AtomicBool = AtomicBool::new(false);
static LAST_CTRL_TIME: AtomicU64 = AtomicU64::new(0);
static LAST_MOUSE_X: AtomicI32 = AtomicI32::new(0);
static LAST_MOUSE_Y: AtomicI32 = AtomicI32::new(0);
static LAST_MOVE_TIME: AtomicU64 = AtomicU64::new(0);
static SPOTLIGHT_HWND: OnceLock<SendHwnd> = OnceLock::new();

const WM_USER_SHOW_SPOTLIGHT: u32 = WM_USER + 1;
const WM_USER_HIDE_SPOTLIGHT: u32 = WM_USER + 2;
const TIMER_UPDATE: usize = 1;

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        // Registrar clase de ventana
        let class_name = w!("FindMyCursorSpotlight");
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

        // Obtener dimensiones de pantalla virtual (todos los monitores)
        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        // Crear ventana layered (inicialmente oculta)
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
            class_name,
            w!("FindMyCursor"),
            WS_POPUP,
            vx,
            vy,
            vw,
            vh,
            None,
            None,
            instance,
            None,
        )?;

        let _ = SPOTLIGHT_HWND.set(SendHwnd(hwnd));

        // Configurar ventana layered
        SetLayeredWindowAttributes(hwnd, COLORREF(0), BACKDROP_OPACITY, LWA_ALPHA)?;

        // Instalar hooks globales
        let kb_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), instance, 0)?;
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), instance, 0)?;

        // Bucle de mensajes
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            if ret.0 == 0 || ret.0 == -1 {
                break; // WM_QUIT o error
            }
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }

        // Limpiar hooks
        let _ = UnhookWindowsHookEx(kb_hook);
        let _ = UnhookWindowsHookEx(mouse_hook);

        Ok(())
    }
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb_struct = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

        // Detectar Ctrl (izquierdo o derecho)
        if kb_struct.vkCode == VK_LCONTROL.0 as u32 || kb_struct.vkCode == VK_RCONTROL.0 as u32 {
            if wparam.0 == WM_KEYDOWN as usize {
                let now = get_tick_count();
                let last = LAST_CTRL_TIME.load(Ordering::Relaxed);
                let diff = now.saturating_sub(last);

                LAST_CTRL_TIME.store(now, Ordering::Relaxed);

                // Doble pulsación detectada
                if diff > 50 && diff < DOUBLE_TAP_TIME_MS {
                    if let Some(hwnd) = SPOTLIGHT_HWND.get() {
                        let h = hwnd.get();
                        if SPOTLIGHT_ACTIVE.load(Ordering::Relaxed) {
                            let _ = PostMessageW(h, WM_USER_HIDE_SPOTLIGHT, WPARAM(0), LPARAM(0));
                        } else {
                            let _ = PostMessageW(h, WM_USER_SHOW_SPOTLIGHT, WPARAM(0), LPARAM(0));
                        }
                    }
                }
            }
        }
        // Cualquier otra tecla cierra el spotlight
        else if SPOTLIGHT_ACTIVE.load(Ordering::Relaxed) && wparam.0 == WM_KEYDOWN as usize {
            if let Some(hwnd) = SPOTLIGHT_HWND.get() {
                let _ = PostMessageW(hwnd.get(), WM_USER_HIDE_SPOTLIGHT, WPARAM(0), LPARAM(0));
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && SPOTLIGHT_ACTIVE.load(Ordering::Relaxed) {
        // Clic cierra el spotlight
        if wparam.0 == WM_LBUTTONDOWN as usize
            || wparam.0 == WM_RBUTTONDOWN as usize
            || wparam.0 == WM_MBUTTONDOWN as usize
        {
            if let Some(hwnd) = SPOTLIGHT_HWND.get() {
                let _ = PostMessageW(hwnd.get(), WM_USER_HIDE_SPOTLIGHT, WPARAM(0), LPARAM(0));
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

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

unsafe fn show_spotlight(hwnd: HWND) {
    if SPOTLIGHT_ACTIVE.load(Ordering::Relaxed) {
        return;
    }

    SPOTLIGHT_ACTIVE.store(true, Ordering::Relaxed);

    // Obtener posición del ratón
    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);

    LAST_MOUSE_X.store(point.x, Ordering::Relaxed);
    LAST_MOUSE_Y.store(point.y, Ordering::Relaxed);
    LAST_MOVE_TIME.store(get_tick_count(), Ordering::Relaxed);

    // Actualizar posición y tamaño de la ventana
    let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
    let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
    let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

    let _ = SetWindowPos(hwnd, HWND_TOPMOST, vx, vy, vw, vh, SWP_NOACTIVATE);

    // Aplicar región con agujero
    update_region(hwnd, point.x, point.y, vx, vy, vw, vh);

    // Mostrar ventana
    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

    // Iniciar timer de actualización
    let _ = SetTimer(hwnd, TIMER_UPDATE, UPDATE_INTERVAL_MS, None);
}

unsafe fn hide_spotlight(hwnd: HWND) {
    if !SPOTLIGHT_ACTIVE.load(Ordering::Relaxed) {
        return;
    }

    SPOTLIGHT_ACTIVE.store(false, Ordering::Relaxed);

    let _ = KillTimer(hwnd, TIMER_UPDATE);
    let _ = ShowWindow(hwnd, SW_HIDE);

    // Eliminar región
    let _ = SetWindowRgn(hwnd, None, true);
}

unsafe fn update_spotlight(hwnd: HWND) {
    if !SPOTLIGHT_ACTIVE.load(Ordering::Relaxed) {
        return;
    }

    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);

    let last_x = LAST_MOUSE_X.load(Ordering::Relaxed);
    let last_y = LAST_MOUSE_Y.load(Ordering::Relaxed);

    // Si el ratón se movió
    if point.x != last_x || point.y != last_y {
        LAST_MOUSE_X.store(point.x, Ordering::Relaxed);
        LAST_MOUSE_Y.store(point.y, Ordering::Relaxed);
        LAST_MOVE_TIME.store(get_tick_count(), Ordering::Relaxed);

        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        update_region(hwnd, point.x, point.y, vx, vy, vw, vh);
    } else {
        // Verificar timeout
        let now = get_tick_count();
        let last_move = LAST_MOVE_TIME.load(Ordering::Relaxed);

        if now.saturating_sub(last_move) > FADE_AFTER_STOP_MS {
            hide_spotlight(hwnd);
        }
    }
}

unsafe fn update_region(
    hwnd: HWND,
    mouse_x: i32,
    mouse_y: i32,
    vx: i32,
    vy: i32,
    vw: i32,
    vh: i32,
) {
    // Coordenadas del ratón relativas a la ventana
    let rel_x = mouse_x - vx;
    let rel_y = mouse_y - vy;

    // Región rectangular (toda la ventana)
    let rect_region = CreateRectRgn(0, 0, vw, vh);

    // Región elíptica (el agujero)
    let ellipse_region = CreateEllipticRgn(
        rel_x - SPOTLIGHT_RADIUS,
        rel_y - SPOTLIGHT_RADIUS,
        rel_x + SPOTLIGHT_RADIUS,
        rel_y + SPOTLIGHT_RADIUS,
    );

    // Restar la elipse del rectángulo
    let _ = CombineRgn(rect_region, rect_region, ellipse_region, RGN_DIFF);

    // Aplicar región a la ventana
    let _ = SetWindowRgn(hwnd, rect_region, true);

    // Limpiar región temporal (SetWindowRgn toma ownership de rect_region)
    let _ = DeleteObject(ellipse_region);
}

fn get_tick_count() -> u64 {
    unsafe { GetTickCount64() }
}
