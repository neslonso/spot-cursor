//! Gestión de región GDI para el efecto spotlight

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::*;

use crate::config::RUNTIME_CONFIG;
use crate::types::{Position, VirtualScreen};

/// Aplica la región del spotlight (fondo con agujero circular)
pub unsafe fn apply_spotlight_region(
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
    let config = RUNTIME_CONFIG.get().unwrap();
    let radius = config.spotlight_radius();
    let hole_region = CreateEllipticRgn(
        rel_x - radius,
        rel_y - radius,
        rel_x + radius,
        rel_y + radius,
    );

    // Restar el agujero del fondo
    let _ = CombineRgn(backdrop_region, backdrop_region, hole_region, RGN_DIFF);

    // Aplicar región a la ventana (toma ownership de backdrop_region)
    let _ = SetWindowRgn(hwnd, backdrop_region, true);

    // Limpiar región temporal
    let _ = DeleteObject(hole_region);
}
