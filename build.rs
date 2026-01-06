//! Build script para compilar recursos de Windows

use std::process::Command;
use std::env;

fn main() {
    // Solo compilar recursos si el target es Windows
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        println!("cargo:rerun-if-changed=resources.rc");
        println!("cargo:rerun-if-changed=icon.ico");

        let target = env::var("TARGET").unwrap();
        let out_dir = env::var("OUT_DIR").unwrap();

        // Determinar el compilador de recursos según el target
        let windres = if target.contains("gnu") {
            "x86_64-w64-mingw32-windres"
        } else {
            "windres"
        };

        // Compilar el archivo .rc a .o
        // -c 65001 = UTF-8 codepage para que interprete correctamente tildes y símbolos
        let status = Command::new(windres)
            .args(&[
                "-c", "65001",              // UTF-8 input
                "resources.rc",
                "-O", "coff",
                "-o", &format!("{}/resources.o", out_dir),
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:rustc-link-arg={}/resources.o", out_dir);
                println!("✓ Recursos de Windows compilados correctamente");
            }
            Ok(s) => {
                eprintln!("⚠ windres falló con código: {:?}", s.code());
                eprintln!("  Los metadatos de Windows no se incluirán");
            }
            Err(e) => {
                eprintln!("⚠ No se pudo ejecutar {}: {}", windres, e);
                eprintln!("  Los metadatos de Windows no se incluirán");
                eprintln!("  Instala mingw-w64: sudo apt-get install mingw-w64");
            }
        }
    }
}
