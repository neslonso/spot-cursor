//! SpotCursor - Spotlight portable estilo PowerToys
//!
//! ## Funcionamiento
//!
//! - **Activación:** Doble pulsación de Ctrl (izquierdo o derecho)
//! - **Desactivación:** Clic de ratón, cualquier tecla, o auto-fade tras inactividad
//!
//! ## Arquitectura
//!
//! La aplicación funciona mediante:
//! 1. Una ventana transparente superpuesta que cubre todos los monitores
//! 2. Hooks globales de teclado y ratón para detectar eventos
//! 3. Una región GDI que crea el efecto "agujero" alrededor del cursor
//! 4. System tray icon para control de la aplicación
//! 5. Configuración persistente en JSON

#![windows_subsystem = "windows"]

// Módulos
mod config;
mod constants;
mod hooks;
mod settings_dialog;
mod spotlight;
mod tray;
mod types;

// Imports
use windows::core::Result;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

use config::{load_config, RuntimeConfig, RUNTIME_CONFIG};
use spotlight::GlobalState;
use tray::add_tray_icon;

fn main() -> Result<()> {
    unsafe {
        // Inicializar configuración runtime
        let runtime_config = RuntimeConfig::new();

        // Cargar configuración desde archivo
        let settings = load_config();
        runtime_config.load_from(&settings);

        // Almacenar configuración global
        let _ = RUNTIME_CONFIG.set(runtime_config);

        // Obtener handle de la instancia
        let instance = GetModuleHandleW(None)?;

        // Registrar clase de ventana
        spotlight::register_window_class(instance.into())?;

        // Crear ventana del spotlight
        let hwnd = spotlight::create_spotlight_window(instance.into())?;
        GlobalState::set_hwnd(hwnd);

        // Crear icono en system tray
        add_tray_icon(hwnd)?;

        // Instalar hooks globales
        let keyboard_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(hooks::keyboard_hook_proc),
            instance,
            0,
        )?;

        let mouse_hook =
            SetWindowsHookExW(WH_MOUSE_LL, Some(hooks::mouse_hook_proc), instance, 0)?;

        // Bucle de mensajes
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);

            // ret.0 == 0: WM_QUIT, ret.0 == -1: error
            if ret.0 == 0 || ret.0 == -1 {
                break;
            }

            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }

        // Limpiar hooks
        let _ = UnhookWindowsHookEx(keyboard_hook);
        let _ = UnhookWindowsHookEx(mouse_hook);

        Ok(())
    }
}
