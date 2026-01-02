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
- **System Tray:** Icono en la bandeja del sistema con menú de salida
- **Configuración Persistente:** Los ajustes se guardan automáticamente en `%APPDATA%/SpotCursor/config.json`

## Configuración

La configuración se guarda automáticamente en formato JSON. Puedes editarla manualmente en:

```
%APPDATA%\SpotCursor\config.json
```

Parámetros configurables:

```json
{
  "double_tap_time_ms": 400,    // Tiempo máximo entre pulsaciones de Ctrl (50-1000ms)
  "backdrop_opacity": 180,       // Opacidad del fondo oscuro (0-255)
  "spotlight_radius": 100,       // Radio del círculo de luz (50-300 píxeles)
  "auto_hide_delay_ms": 2000    // Tiempo de inactividad antes de auto-ocultar (500-10000ms)
}
```

Si el archivo no existe, se creará automáticamente con valores por defecto al iniciar la aplicación.

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
2. Ejecútalo - aparecerá un icono en la bandeja del sistema
3. Pulsa Ctrl dos veces rápidamente para activar el spotlight
4. Para cerrar la aplicación:
   - **Recomendado:** Click derecho en el icono del tray → Salir
   - Alternativa: `taskkill /IM spot-cursor.exe` o Administrador de tareas

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
