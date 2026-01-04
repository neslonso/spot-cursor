//! System tray icon y menú contextual

use windows::core::*;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::constants::{IDM_EXIT, IDM_OPTIONS, TRAY_ICON_ID, WM_TRAYICON};
use crate::settings_dialog::show_settings_dialog;

/// Crea un icono personalizado para el system tray
/// Dibuja un círculo púrpura con un punto blanco (representando el spotlight)
unsafe fn create_embedded_icon() -> Result<HICON> {
    const ICON_SIZE: i32 = 16;

    // Obtener DC de pantalla
    let screen_dc = GetDC(None);
    if screen_dc.is_invalid() {
        return Err(Error::from_win32());
    }

    // Crear DCs compatibles para el icono y la máscara
    let icon_dc = CreateCompatibleDC(screen_dc);
    let mask_dc = CreateCompatibleDC(screen_dc);

    if icon_dc.is_invalid() || mask_dc.is_invalid() {
        let _ = ReleaseDC(None, screen_dc);
        return Err(Error::from_win32());
    }

    // Crear bitmaps
    let icon_bitmap = CreateCompatibleBitmap(screen_dc, ICON_SIZE, ICON_SIZE);
    let mask_bitmap = CreateCompatibleBitmap(screen_dc, ICON_SIZE, ICON_SIZE);

    if icon_bitmap.is_invalid() || mask_bitmap.is_invalid() {
        let _ = DeleteDC(icon_dc);
        let _ = DeleteDC(mask_dc);
        let _ = ReleaseDC(None, screen_dc);
        return Err(Error::from_win32());
    }

    // Seleccionar bitmaps en los DCs
    let old_icon_bmp = SelectObject(icon_dc, icon_bitmap);
    let old_mask_bmp = SelectObject(mask_dc, mask_bitmap);

    // Dibujar máscara (todo negro = opaco, todo blanco = transparente)
    let white_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let rect = RECT {
        left: 0,
        top: 0,
        right: ICON_SIZE,
        bottom: ICON_SIZE,
    };
    let _ = FillRect(mask_dc, &rect, white_brush);
    let _ = DeleteObject(white_brush);

    // Dibujar círculo negro en la máscara (zona opaca)
    let black_brush = CreateSolidBrush(COLORREF(0x00000000));
    let old_brush = SelectObject(mask_dc, black_brush);
    let _ = Ellipse(mask_dc, 1, 1, ICON_SIZE - 1, ICON_SIZE - 1);
    let _ = SelectObject(mask_dc, old_brush);
    let _ = DeleteObject(black_brush);

    // Dibujar icono en color
    // Fondo blanco
    let bg_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let _ = FillRect(icon_dc, &rect, bg_brush);
    let _ = DeleteObject(bg_brush);

    // Círculo púrpura/azul
    let purple_brush = CreateSolidBrush(COLORREF(0x00AA4488)); // Púrpura
    let old_brush = SelectObject(icon_dc, purple_brush);
    let _ = Ellipse(icon_dc, 1, 1, ICON_SIZE - 1, ICON_SIZE - 1);
    let _ = SelectObject(icon_dc, old_brush);
    let _ = DeleteObject(purple_brush);

    // Punto blanco en el centro (spotlight)
    let white_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
    let old_brush = SelectObject(icon_dc, white_brush);
    let center = ICON_SIZE / 2;
    let spot_size = 3;
    let _ = Ellipse(
        icon_dc,
        center - spot_size / 2,
        center - spot_size / 2,
        center + spot_size / 2 + 1,
        center + spot_size / 2 + 1,
    );
    let _ = SelectObject(icon_dc, old_brush);
    let _ = DeleteObject(white_brush);

    // Crear el icono
    let icon_info = ICONINFO {
        fIcon: true.into(),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: mask_bitmap,
        hbmColor: icon_bitmap,
    };

    let icon = CreateIconIndirect(&icon_info)?;

    // Limpiar recursos
    let _ = SelectObject(icon_dc, old_icon_bmp);
    let _ = SelectObject(mask_dc, old_mask_bmp);
    let _ = DeleteObject(icon_bitmap);
    let _ = DeleteObject(mask_bitmap);
    let _ = DeleteDC(icon_dc);
    let _ = DeleteDC(mask_dc);
    let _ = ReleaseDC(None, screen_dc);

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
        hIcon: create_embedded_icon()?,
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
