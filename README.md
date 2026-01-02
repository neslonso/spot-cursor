# SpotCursor

Spotlight portable para localizar el cursor en configuraciones multi-monitor, estilo PowerToys.

## Características

- **Activación:** Doble pulsación de Ctrl (izquierdo o derecho)
- **Efecto:** Oscurece toda la pantalla excepto un círculo alrededor del cursor
- **Desactivación:** 
  - Clic de ratón
  - Cualquier tecla
  - Automático tras 2 segundos sin mover el ratón
  - Doble Ctrl de nuevo

## Configuración (editar en `src/main.rs`)

```rust
const DOUBLE_TAP_TIME_MS: u64 = 400;      // Tiempo máximo entre pulsaciones
const BACKDROP_OPACITY: u8 = 180;          // Opacidad del fondo (0-255)
const SPOTLIGHT_RADIUS: i32 = 100;         // Radio del círculo en píxeles
const FADE_AFTER_STOP_MS: u64 = 2000;      // Auto-cerrar tras X ms sin mover
```

## Compilación

### Requisitos en el contenedor Docker

```bash
# Instalar target de Windows
rustup target add x86_64-pc-windows-gnu

# Instalar linker MinGW
sudo apt-get update
sudo apt-get install -y gcc-mingw-w64-x86-64
```

### Compilar

```bash
# Debug (rápido, para probar)
cargo build --target x86_64-pc-windows-gnu

# Release (optimizado, pequeño)
cargo build --release --target x86_64-pc-windows-gnu
```

El ejecutable estará en:
- Debug: `target/x86_64-pc-windows-gnu/debug/spot-cursor.exe`
- Release: `target/x86_64-pc-windows-gnu/release/spot-cursor.exe`

## Uso

1. Copia `spot-cursor.exe` a tu Windows
2. Ejecútalo (se queda en segundo plano, sin ventana)
3. Pulsa Ctrl dos veces rápidamente para activar el spotlight
4. Para cerrar la aplicación: Administrador de tareas o `taskkill /IM spot-cursor.exe`

### Auto-inicio con Windows (opcional)

Crea un acceso directo en:
```
%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
```

## Tamaño esperado

- Debug: ~1-2 MB
- Release: ~200-400 KB

## Licencia

MIT
