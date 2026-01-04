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
pub const IDM_OPTIONS: u32 = 1000;
pub const IDM_EXIT: u32 = 1001;

/// ID del timer de actualización
pub const TIMER_UPDATE: usize = 1;

/// ID del timer de animación
pub const TIMER_ANIMATION: usize = 2;

/// IDs de controles del diálogo de configuración
pub const IDC_DOUBLE_TAP_LABEL: i32 = 2001;
pub const IDC_DOUBLE_TAP_SLIDER: i32 = 2002;
pub const IDC_DOUBLE_TAP_VALUE: i32 = 2003;
pub const IDC_OPACITY_LABEL: i32 = 2004;
pub const IDC_OPACITY_SLIDER: i32 = 2005;
pub const IDC_OPACITY_VALUE: i32 = 2006;
pub const IDC_RADIUS_LABEL: i32 = 2007;
pub const IDC_RADIUS_SLIDER: i32 = 2008;
pub const IDC_RADIUS_VALUE: i32 = 2009;
pub const IDC_AUTO_HIDE_LABEL: i32 = 2010;
pub const IDC_AUTO_HIDE_SLIDER: i32 = 2011;
pub const IDC_AUTO_HIDE_VALUE: i32 = 2012;
pub const IDC_COLOR_LABEL: i32 = 2013;
pub const IDC_COLOR_BUTTON: i32 = 2014;
pub const IDC_COLOR_PREVIEW: i32 = 2015;
pub const IDC_ANIMATION_ENABLE: i32 = 2016;
pub const IDC_ANIMATION_RADIUS_LABEL: i32 = 2017;
pub const IDC_ANIMATION_RADIUS_SLIDER: i32 = 2018;
pub const IDC_ANIMATION_RADIUS_VALUE: i32 = 2019;
pub const IDC_ANIMATION_DURATION_LABEL: i32 = 2020;
pub const IDC_ANIMATION_DURATION_SLIDER: i32 = 2021;
pub const IDC_ANIMATION_DURATION_VALUE: i32 = 2022;

/// Botones estándar del diálogo
pub const IDOK: i32 = 1;
pub const IDCANCEL: i32 = 2;
