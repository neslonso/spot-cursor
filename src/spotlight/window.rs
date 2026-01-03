//! Gestión de ventana del spotlight

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use super::region::apply_spotlight_region;
use super::state::GlobalState;
use crate::config::{ConfigDefaults, RUNTIME_CONFIG};
use crate::constants::*;
use crate::tray::remove_tray_icon;
use crate::types::{Position, VirtualScreen};

/// Registra la clase de ventana para el spotlight
pub unsafe fn register_window_class(instance: HINSTANCE) -> Result<()> {
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
pub unsafe fn create_spotlight_window(instance: HINSTANCE) -> Result<HWND> {
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
    let config = RUNTIME_CONFIG.get().unwrap();
    SetLayeredWindowAttributes(hwnd, COLORREF(0), config.backdrop_opacity(), LWA_ALPHA)?;

    Ok(hwnd)
}

/// Procedimiento de ventana (maneja mensajes de Windows)
pub unsafe extern "system" fn window_proc(
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
        WM_TRAYICON => {
            crate::tray::handle_tray_message(hwnd, lparam);
            LRESULT(0)
        }
        WM_COMMAND => {
            match wparam.0 as u32 {
                IDM_EXIT => {
                    remove_tray_icon(hwnd);
                    PostQuitMessage(0);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            remove_tray_icon(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Muestra el spotlight en la posición actual del cursor
pub unsafe fn show_spotlight(hwnd: HWND) {
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
    let _ = SetTimer(hwnd, TIMER_UPDATE, ConfigDefaults::UPDATE_INTERVAL_MS, None);
}

/// Oculta el spotlight
pub unsafe fn hide_spotlight(hwnd: HWND) {
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
pub unsafe fn update_spotlight(hwnd: HWND) {
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
        let config = RUNTIME_CONFIG.get().unwrap();
        if GlobalState::time_since_last_move() > config.auto_hide_delay_ms() {
            hide_spotlight(hwnd);
        }
    }
}
