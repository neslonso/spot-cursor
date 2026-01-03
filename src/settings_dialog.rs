//! Diálogo de configuración para SpotCursor
//!
//! Proporciona una interfaz gráfica para ajustar los parámetros del spotlight:
//! - Tiempo de doble toque (double tap)
//! - Opacidad del fondo (backdrop)
//! - Radio del spotlight
//! - Retardo de auto-ocultado

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::config::{save_config, Settings, RUNTIME_CONFIG};
use crate::constants::{
    IDC_AUTO_HIDE_LABEL, IDC_AUTO_HIDE_SLIDER, IDC_AUTO_HIDE_VALUE, IDC_DOUBLE_TAP_LABEL,
    IDC_DOUBLE_TAP_SLIDER, IDC_DOUBLE_TAP_VALUE, IDC_OPACITY_LABEL, IDC_OPACITY_SLIDER,
    IDC_OPACITY_VALUE, IDC_RADIUS_LABEL, IDC_RADIUS_SLIDER, IDC_RADIUS_VALUE,
};
use crate::spotlight::GlobalState;

// IDs de botones estándar (evitar ambigüedad)
const IDOK: i32 = 1;
const IDCANCEL: i32 = 2;

// Mensajes de trackbar que no están en windows-rs
const TBM_GETPOS: u32 = 0x0400;
const TBM_SETPOS: u32 = 0x0405;
const TBM_SETRANGE: u32 = 0x0406;
const TBM_SETTICFREQ: u32 = 0x0414;

const DIALOG_WIDTH: i32 = 450;
const DIALOG_HEIGHT: i32 = 350;
const MARGIN: i32 = 20;
const CONTROL_HEIGHT: i32 = 25;
const LABEL_HEIGHT: i32 = 20;
const SPACING: i32 = 15;
const SLIDER_WIDTH: i32 = 280;
const VALUE_WIDTH: i32 = 80;
const BUTTON_WIDTH: i32 = 100;
const BUTTON_HEIGHT: i32 = 30;

/// Clase de ventana para el diálogo
const SETTINGS_DIALOG_CLASS: PCWSTR = w!("SpotCursorSettingsDialog");

/// Muestra el diálogo de configuración
pub unsafe fn show_settings_dialog(parent_hwnd: HWND) -> Result<()> {
    // Verificar si ya existe una ventana de configuración
    if let Ok(existing) = FindWindowW(SETTINGS_DIALOG_CLASS, None) {
        if !existing.is_invalid() {
            // Si ya existe, traerla al frente
            let _ = SetForegroundWindow(existing);
            return Ok(());
        }
    }

    // Registrar clase de ventana si no está registrada
    register_dialog_class()?;

    // Obtener tamaño de pantalla para centrar el diálogo
    let screen_width = GetSystemMetrics(SM_CXSCREEN);
    let screen_height = GetSystemMetrics(SM_CYSCREEN);
    let x = (screen_width - DIALOG_WIDTH) / 2;
    let y = (screen_height - DIALOG_HEIGHT) / 2;

    // Crear ventana del diálogo
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        SETTINGS_DIALOG_CLASS,
        w!("SpotCursor - Configuración"),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
        x,
        y,
        DIALOG_WIDTH,
        DIALOG_HEIGHT,
        parent_hwnd,
        None,
        GetModuleHandleW(None)?,
        None,
    )?;

    // Mostrar la ventana
    let _ = ShowWindow(hwnd, SW_SHOW);

    Ok(())
}

/// Registra la clase de ventana para el diálogo
unsafe fn register_dialog_class() -> Result<()> {
    let instance = GetModuleHandleW(None)?.into();

    let wc = WNDCLASSW {
        lpfnWndProc: Some(dialog_proc),
        hInstance: instance,
        lpszClassName: SETTINGS_DIALOG_CLASS,
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        hbrBackground: HBRUSH((COLOR_BTNFACE.0 as i32 + 1) as isize as *mut _),
        style: CS_HREDRAW | CS_VREDRAW,
        ..Default::default()
    };

    if RegisterClassW(&wc) == 0 {
        let error = GetLastError();
        // Si el error es que la clase ya está registrada, no es un error
        if error.0 != ERROR_CLASS_ALREADY_EXISTS.0 {
            return Err(Error::from(error));
        }
    }

    Ok(())
}

/// Procedimiento de ventana para el diálogo
unsafe extern "system" fn dialog_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            create_controls(hwnd);
            load_current_settings(hwnd);
            LRESULT(0)
        }
        WM_HSCROLL => {
            handle_slider_change(hwnd, lparam);
            LRESULT(0)
        }
        WM_COMMAND => {
            let command = (wparam.0 as u16) as i32;
            match command {
                IDOK => {
                    save_current_settings(hwnd);
                    let _ = DestroyWindow(hwnd);
                    LRESULT(0)
                }
                IDCANCEL => {
                    let _ = DestroyWindow(hwnd);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Crea todos los controles del diálogo
unsafe fn create_controls(hwnd: HWND) {
    let instance = GetModuleHandleW(None).unwrap().into();
    let mut y_pos = MARGIN;

    // --- Double Tap Time ---
    create_label(
        hwnd,
        instance,
        "Tiempo de doble toque (ms):",
        MARGIN,
        y_pos,
        SLIDER_WIDTH,
        LABEL_HEIGHT,
        IDC_DOUBLE_TAP_LABEL,
    );
    y_pos += LABEL_HEIGHT + 5;

    create_slider(
        hwnd,
        instance,
        MARGIN,
        y_pos,
        SLIDER_WIDTH,
        CONTROL_HEIGHT,
        IDC_DOUBLE_TAP_SLIDER,
        100,
        1000,
    );

    create_label(
        hwnd,
        instance,
        "400",
        MARGIN + SLIDER_WIDTH + 10,
        y_pos,
        VALUE_WIDTH,
        CONTROL_HEIGHT,
        IDC_DOUBLE_TAP_VALUE,
    );

    y_pos += CONTROL_HEIGHT + SPACING;

    // --- Backdrop Opacity ---
    create_label(
        hwnd,
        instance,
        "Opacidad del fondo (0-255):",
        MARGIN,
        y_pos,
        SLIDER_WIDTH,
        LABEL_HEIGHT,
        IDC_OPACITY_LABEL,
    );
    y_pos += LABEL_HEIGHT + 5;

    create_slider(
        hwnd,
        instance,
        MARGIN,
        y_pos,
        SLIDER_WIDTH,
        CONTROL_HEIGHT,
        IDC_OPACITY_SLIDER,
        0,
        255,
    );

    create_label(
        hwnd,
        instance,
        "180",
        MARGIN + SLIDER_WIDTH + 10,
        y_pos,
        VALUE_WIDTH,
        CONTROL_HEIGHT,
        IDC_OPACITY_VALUE,
    );

    y_pos += CONTROL_HEIGHT + SPACING;

    // --- Spotlight Radius ---
    create_label(
        hwnd,
        instance,
        "Radio del spotlight (px):",
        MARGIN,
        y_pos,
        SLIDER_WIDTH,
        LABEL_HEIGHT,
        IDC_RADIUS_LABEL,
    );
    y_pos += LABEL_HEIGHT + 5;

    create_slider(
        hwnd,
        instance,
        MARGIN,
        y_pos,
        SLIDER_WIDTH,
        CONTROL_HEIGHT,
        IDC_RADIUS_SLIDER,
        50,
        500,
    );

    create_label(
        hwnd,
        instance,
        "200",
        MARGIN + SLIDER_WIDTH + 10,
        y_pos,
        VALUE_WIDTH,
        CONTROL_HEIGHT,
        IDC_RADIUS_VALUE,
    );

    y_pos += CONTROL_HEIGHT + SPACING;

    // --- Auto Hide Delay ---
    create_label(
        hwnd,
        instance,
        "Retardo de auto-ocultado (ms):",
        MARGIN,
        y_pos,
        SLIDER_WIDTH,
        LABEL_HEIGHT,
        IDC_AUTO_HIDE_LABEL,
    );
    y_pos += LABEL_HEIGHT + 5;

    create_slider(
        hwnd,
        instance,
        MARGIN,
        y_pos,
        SLIDER_WIDTH,
        CONTROL_HEIGHT,
        IDC_AUTO_HIDE_SLIDER,
        100,
        5000,
    );

    create_label(
        hwnd,
        instance,
        "2000",
        MARGIN + SLIDER_WIDTH + 10,
        y_pos,
        VALUE_WIDTH,
        CONTROL_HEIGHT,
        IDC_AUTO_HIDE_VALUE,
    );

    y_pos += CONTROL_HEIGHT + SPACING + 10;

    // --- Botones OK y Cancel ---
    let button_y = DIALOG_HEIGHT - MARGIN - BUTTON_HEIGHT - 40;
    let button_x_ok = DIALOG_WIDTH - MARGIN - BUTTON_WIDTH * 2 - 10;
    let button_x_cancel = DIALOG_WIDTH - MARGIN - BUTTON_WIDTH;

    create_button(
        hwnd,
        instance,
        "OK",
        button_x_ok,
        button_y,
        BUTTON_WIDTH,
        BUTTON_HEIGHT,
        IDOK,
    );

    create_button(
        hwnd,
        instance,
        "Cancelar",
        button_x_cancel,
        button_y,
        BUTTON_WIDTH,
        BUTTON_HEIGHT,
        IDCANCEL,
    );
}

/// Crea un label (texto estático)
unsafe fn create_label(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) {
    let text_wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();

    let _ = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("STATIC"),
        PCWSTR(text_wide.as_ptr()),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        width,
        height,
        parent,
        HMENU(id as *mut _),
        instance,
        None,
    );
}

/// Crea un slider (trackbar)
unsafe fn create_slider(
    parent: HWND,
    instance: HINSTANCE,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
    min: i32,
    max: i32,
) {
    let slider = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("msctls_trackbar32"),
        w!(""),
        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(TBS_HORZ | TBS_AUTOTICKS),
        x,
        y,
        width,
        height,
        parent,
        HMENU(id as *mut _),
        instance,
        None,
    )
    .unwrap();

    // Configurar rango del slider
    let _ = SendMessageW(
        slider,
        TBM_SETRANGE,
        WPARAM(1),
        LPARAM((min as u32 | ((max as u32) << 16)) as isize),
    );
    let _ = SendMessageW(
        slider,
        TBM_SETTICFREQ,
        WPARAM((max - min) as usize / 10),
        LPARAM(0),
    );
}

/// Crea un botón
unsafe fn create_button(
    parent: HWND,
    instance: HINSTANCE,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) {
    let text_wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();

    let _ = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        w!("BUTTON"),
        PCWSTR(text_wide.as_ptr()),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_PUSHBUTTON as u32),
        x,
        y,
        width,
        height,
        parent,
        HMENU(id as *mut _),
        instance,
        None,
    );
}

/// Carga la configuración actual en los controles
unsafe fn load_current_settings(hwnd: HWND) {
    if let Some(config) = RUNTIME_CONFIG.get() {
        // Double tap time
        let double_tap = config.double_tap_time_ms();
        set_slider_value(hwnd, IDC_DOUBLE_TAP_SLIDER, double_tap as i32);
        update_value_label(hwnd, IDC_DOUBLE_TAP_VALUE, double_tap as i32, "");

        // Backdrop opacity
        let opacity = config.backdrop_opacity();
        set_slider_value(hwnd, IDC_OPACITY_SLIDER, opacity as i32);
        update_value_label(hwnd, IDC_OPACITY_VALUE, opacity as i32, "");

        // Spotlight radius
        let radius = config.spotlight_radius();
        set_slider_value(hwnd, IDC_RADIUS_SLIDER, radius);
        update_value_label(hwnd, IDC_RADIUS_VALUE, radius, "");

        // Auto hide delay
        let auto_hide = config.auto_hide_delay_ms();
        set_slider_value(hwnd, IDC_AUTO_HIDE_SLIDER, auto_hide as i32);
        update_value_label(hwnd, IDC_AUTO_HIDE_VALUE, auto_hide as i32, "");
    }
}

/// Establece el valor de un slider
unsafe fn set_slider_value(hwnd: HWND, slider_id: i32, value: i32) {
    if let Ok(slider) = GetDlgItem(hwnd, slider_id) {
        let _ = SendMessageW(slider, TBM_SETPOS, WPARAM(1), LPARAM(value as isize));
    }
}

/// Obtiene el valor actual de un slider
unsafe fn get_slider_value(hwnd: HWND, slider_id: i32) -> i32 {
    if let Ok(slider) = GetDlgItem(hwnd, slider_id) {
        return SendMessageW(slider, TBM_GETPOS, WPARAM(0), LPARAM(0)).0 as i32;
    }
    0
}

/// Actualiza el label que muestra el valor actual
unsafe fn update_value_label(hwnd: HWND, label_id: i32, value: i32, suffix: &str) {
    let text = format!("{}{}", value, suffix);
    let text_wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    if let Ok(label) = GetDlgItem(hwnd, label_id) {
        let _ = SetWindowTextW(label, PCWSTR(text_wide.as_ptr()));
    }
}

/// Maneja cambios en los sliders
unsafe fn handle_slider_change(hwnd: HWND, lparam: LPARAM) {
    let slider_hwnd = HWND(lparam.0 as *mut _);

    // Obtener el ID del slider
    let slider_id = GetDlgCtrlID(slider_hwnd);

    // Obtener el valor actual
    let value = SendMessageW(slider_hwnd, TBM_GETPOS, WPARAM(0), LPARAM(0)).0 as i32;

    // Actualizar el label correspondiente
    match slider_id {
        IDC_DOUBLE_TAP_SLIDER => {
            update_value_label(hwnd, IDC_DOUBLE_TAP_VALUE, value, "");
        }
        IDC_OPACITY_SLIDER => {
            update_value_label(hwnd, IDC_OPACITY_VALUE, value, "");
        }
        IDC_RADIUS_SLIDER => {
            update_value_label(hwnd, IDC_RADIUS_VALUE, value, "");
        }
        IDC_AUTO_HIDE_SLIDER => {
            update_value_label(hwnd, IDC_AUTO_HIDE_VALUE, value, "");
        }
        _ => {}
    }
}

/// Guarda la configuración actual desde los controles
unsafe fn save_current_settings(hwnd: HWND) {
    // Obtener valores de los sliders
    let double_tap = get_slider_value(hwnd, IDC_DOUBLE_TAP_SLIDER) as u64;
    let opacity = get_slider_value(hwnd, IDC_OPACITY_SLIDER) as u8;
    let radius = get_slider_value(hwnd, IDC_RADIUS_SLIDER);
    let auto_hide = get_slider_value(hwnd, IDC_AUTO_HIDE_SLIDER) as u64;

    // Actualizar RuntimeConfig
    if let Some(config) = RUNTIME_CONFIG.get() {
        config.set_double_tap_time_ms(double_tap);
        config.set_backdrop_opacity(opacity);
        config.set_spotlight_radius(radius);
        config.set_auto_hide_delay_ms(auto_hide);

        // Actualizar la opacidad de la ventana del spotlight inmediatamente
        if let Some(spotlight_hwnd) = GlobalState::get_hwnd() {
            let _ = SetLayeredWindowAttributes(
                spotlight_hwnd,
                COLORREF(0),
                opacity,
                LWA_ALPHA,
            );
        }

        // Crear Settings y guardar a JSON
        let settings = Settings {
            double_tap_time_ms: double_tap,
            backdrop_opacity: opacity,
            spotlight_radius: radius,
            auto_hide_delay_ms: auto_hide,
        };

        let _ = save_config(&settings);
    }
}
