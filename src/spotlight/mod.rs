//! Módulo de spotlight - gestión del efecto de iluminación del cursor

mod region;
mod state;
mod window;

// Re-exports públicos
pub use state::GlobalState;
pub use window::{create_spotlight_window, register_window_class};
