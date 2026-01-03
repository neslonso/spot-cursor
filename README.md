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
- **System Tray:** Icono personalizado en la bandeja del sistema con menú de configuración
- **Interfaz Gráfica:** Ventana de opciones con controles deslizantes para ajustar todos los parámetros
- **Configuración Persistente:** Los ajustes se guardan automáticamente en un archivo `.json` con el mismo nombre del ejecutable

## Configuración

La aplicación incluye una **interfaz gráfica de configuración** accesible desde:
- Click derecho en el icono del system tray → "Opciones..."
- Doble click en el icono del system tray

La configuración se guarda automáticamente en un archivo `.json` **con el mismo nombre del ejecutable** en el mismo directorio.
Por ejemplo: `spot-cursor.exe` → `spot-cursor.json`

Parámetros configurables:

- **Tiempo de doble toque:** 100-1000 ms (predeterminado: 400 ms)
- **Opacidad del fondo:** 0-255 (predeterminado: 180)
- **Radio del spotlight:** 50-500 px (predeterminado: 100 px)
- **Retardo de auto-ocultado:** 100-5000 ms (predeterminado: 2000 ms)

Formato del archivo de configuración (ejemplo `spot-cursor.json`):
```json
{
  "double_tap_time_ms": 400,
  "backdrop_opacity": 180,
  "spotlight_radius": 100,
  "auto_hide_delay_ms": 2000
}
```

**Nota:** El archivo de configuración toma el nombre del ejecutable. Si renombras `spot-cursor.exe` a otro nombre, la configuración se guardará con ese nuevo nombre más la extensión `.json`.

Si el archivo no existe, se creará automáticamente con valores por defecto al iniciar la aplicación. También puedes editarlo manualmente si lo prefieres.

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
2. Ejecútalo - aparecerá un icono púrpura con un punto blanco en la bandeja del sistema
3. **Configurar (opcional):**
   - Click derecho en el icono → "Opciones..."
   - Ajusta los parámetros con los controles deslizantes
   - Click "OK" para guardar los cambios
4. Pulsa **Ctrl dos veces** rápidamente para activar el spotlight
5. Para cerrar la aplicación:
   - **Recomendado:** Click derecho en el icono del tray → "Salir"
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
