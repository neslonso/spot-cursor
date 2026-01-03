//! Constantes Windows y IDs de mensajes

use windows::Win32::UI::WindowsAndMessaging::WM_USER;

/// Mensaje personalizado para mostrar el spotlight
pub const WM_USER_SHOW_SPOTLIGHT: u32 = WM_USER + 1;

/// Mensaje personalizado para ocultar el spotlight
pub const WM_USER_HIDE_SPOTLIGHT: u32 = WM_USER + 2;

/// Mensaje del system tray icon
pub const WM_TRAYICON: u32 = WM_USER + 100;

/// ID del icono en el system tray
pub const TRAY_ICON_ID: u32 = 1;

/// IDs de elementos del menú contextual
pub const IDM_EXIT: u32 = 1001;

/// ID del timer de actualización
pub const TIMER_UPDATE: usize = 1;
