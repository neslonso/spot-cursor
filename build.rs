//! Build script para generar recursos de Windows

fn main() {
    // Solo generar recursos en Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();

        // Información del producto
        res.set("ProductName", "SpotCursor")
            .set("FileDescription", "Spotlight para localizar el cursor")
            .set("CompanyName", "Néstor")
            .set("LegalCopyright", "Copyright © 2024-2025 Néstor")
            .set("OriginalFilename", "spot-cursor.exe");

        // Versión del archivo y del producto (leer de Cargo.toml)
        let version = env!("CARGO_PKG_VERSION");
        res.set("ProductVersion", version)
            .set("FileVersion", version);

        // Compilar recursos
        if let Err(e) = res.compile() {
            eprintln!("Error compilando recursos de Windows: {}", e);
        }
    }
}
