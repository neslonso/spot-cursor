//! Tipos personalizados y wrappers

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics;
use windows::Win32::UI::WindowsAndMessaging::{
    SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

/// Wrapper thread-safe para HWND
///
/// HWND es un handle opaco de Windows que puede compartirse entre threads
#[derive(Clone, Copy)]
pub struct SafeHwnd(pub HWND);

unsafe impl Send for SafeHwnd {}
unsafe impl Sync for SafeHwnd {}

impl SafeHwnd {
    /// Obtiene el HWND interno
    #[inline]
    pub fn get(&self) -> HWND {
        self.0
    }
}

/// Representa una posición en coordenadas de pantalla
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn from_point(point: POINT) -> Self {
        Self::new(point.x, point.y)
    }
}

/// Representa las dimensiones del área de pantalla virtual
#[derive(Debug, Clone, Copy)]
pub struct VirtualScreen {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl VirtualScreen {
    /// Obtiene las dimensiones actuales de la pantalla virtual
    pub unsafe fn get_current() -> Self {
        Self {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
            width: GetSystemMetrics(SM_CXVIRTUALSCREEN),
            height: GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    }
}
