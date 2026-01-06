//! System tray icon y menú contextual

use windows::core::*;
use windows::Win32::Foundation::{HWND, LPARAM, POINT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::constants::{IDM_EXIT, IDM_OPTIONS, TRAY_ICON_ID, WM_TRAYICON};
use crate::settings_dialog::show_settings_dialog;

/// Carga el icono desde los recursos embebidos en el ejecutable
unsafe fn load_icon_from_resource() -> Result<HICON> {
    // Obtener handle del módulo actual
    let hinstance = GetModuleHandleW(None)?;

    // Cargar el icono con ID 100 desde los recursos
    // MAKEINTRESOURCEW convierte un ID numérico a PCWSTR
    let icon = LoadIconW(hinstance, PCWSTR(100 as *const u16))?;

    Ok(icon)
}

/// Añade el icono al system tray
pub unsafe fn add_tray_icon(hwnd: HWND) -> Result<()> {
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        hIcon: load_icon_from_resource()?,
        ..Default::default()
    };

    // Tooltip
    let tooltip = w!("SpotCursor - Doble Ctrl para activar");
    let tooltip_bytes = tooltip.as_wide();
    let copy_len = tooltip_bytes.len().min(nid.szTip.len() - 1);
    nid.szTip[..copy_len].copy_from_slice(&tooltip_bytes[..copy_len]);

    if Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
        Ok(())
    } else {
        Err(Error::from_win32())
    }
}

/// Elimina el icono del system tray
pub unsafe fn remove_tray_icon(hwnd: HWND) {
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        ..Default::default()
    };

    let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
}

/// Muestra el menú contextual del system tray
unsafe fn show_tray_menu(hwnd: HWND) {
    let hmenu = CreatePopupMenu().unwrap();

    // Añadir elementos del menú
    let _ = AppendMenuW(hmenu, MF_STRING, IDM_OPTIONS as usize, w!("Opciones..."));
    let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(hmenu, MF_STRING, IDM_EXIT as usize, w!("Salir"));

    // Obtener posición del cursor para el menú
    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);

    // Hacer que la ventana sea foreground para que el menú se cierre correctamente
    let _ = SetForegroundWindow(hwnd);

    // Mostrar menú
    let _ = TrackPopupMenu(hmenu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);

    // Limpiar
    let _ = DestroyMenu(hmenu);
}

/// Maneja los mensajes del system tray
pub unsafe fn handle_tray_message(hwnd: HWND, lparam: LPARAM) {
    match lparam.0 as u32 {
        WM_RBUTTONUP => {
            show_tray_menu(hwnd);
        }
        WM_LBUTTONDBLCLK => {
            // Doble click - abrir opciones
            let _ = show_settings_dialog(hwnd);
        }
        _ => {}
    }
}

/// Maneja los comandos del menú del system tray
pub unsafe fn handle_tray_command(hwnd: HWND, command: u32) {
    match command {
        IDM_OPTIONS => {
            let _ = show_settings_dialog(hwnd);
        }
        _ => {}
    }
}
