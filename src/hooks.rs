//! Hooks globales de teclado y ratón

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::constants::WM_USER_HIDE_SPOTLIGHT;
use crate::spotlight::GlobalState;

/// Hook de teclado: detecta doble Ctrl y otras teclas
pub unsafe extern "system" fn keyboard_hook_proc(
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
pub unsafe extern "system" fn mouse_hook_proc(
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
                crate::constants::WM_USER_SHOW_SPOTLIGHT
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
