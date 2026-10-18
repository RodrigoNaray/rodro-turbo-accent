# Quick Accent Local — GUI Rust + motor IBus (0.4 experimental)

App de escritorio con una sola ventana para **Instalar · On · Off · Estado · Logs · Desinstalar**.

**Linux (GNOME/Wayland):** interfaz escrita en Rust usando egui. Conserva el motor IBus/Python probado en Linux Mint. Al instalarlo, copia **solo archivos de tu usuario**, crea un servicio `systemd --user` y añade el lanzador del menú de aplicaciones. No usa `sudo`, no añade grupos y no solicita acceso directo a `/dev/input`.

**Windows:** **todavía no implementado**. La interfaz Rust puede compilarse en Windows, pero los controles no instalan un motor allí. Se necesita un adaptador nativo Win32 que procese de forma segura la combinación letra mantenida + espacio. **No desinstales PowerToys basándote en esta versión**.

## Función

- Mantener `a` al menos **250 ms** y pulsar espacio, sin soltar `a` → `á`.
- Si Espacio llega antes de los 250 ms, se escriben la letra normal y el espacio. Así se evitan acentos por el solapamiento de teclas al separar palabras.
- `e/i/o/u/n` → `é/í/ó/ú/ñ`; incluye mayúsculas.
- Pulsar y soltar la letra sin espacio → la letra normal.
- Mantiene la disposición US; no instala ni cambia la distribución física.

El motor genera un carácter *preedit* en IBus y lo confirma o sustituye. Dependemos de que cada aplicación entregue los eventos y de que el motor esté activo. El motor se conserva de la versión probada. La versión 0.4 tiene pruebas con un IBus privado, pero **el inicio de sesión y el comportamiento en aplicaciones reales todavía requieren validación en tu sesión de GNOME**.

## Descarga sin instalar Rust en el equipo

Al subir este proyecto a un repositorio privado de GitHub:

1. Abre **Actions** → **Compilar Linux** → **Run workflow**.
2. Al terminar, descarga el artifact `quick-accent-local-linux-x86_64` (GitHub lo entrega comprimido).
3. Extrae el archivo `.tar.gz` incluido; contiene `quick-accent-local-ui` y este README.
4. Desde la carpeta donde lo extrajiste, ejecuta:

   ```bash
   chmod +x quick-accent-local-ui
   ./quick-accent-local-ui
   ```

5. Presiona **Instalar**. La app quedará en **Aplicaciones → Quick Accent Local**.

El instalador está **dentro de la ventana**; no tienes que ejecutar scripts descargados de Internet ni usar `sudo`. Para usarlo necesitas IBus y Python GI instalados (en tu Mint ya están).

> Los ejecutables compilados por GitHub Actions son artefactos de CI, no lanzamientos auditados. Verifica que el workflow haya compilado el código de tu repositorio y no compartas el artefacto como software validado antes de probarlo.

## Para desarrolladores

Con la toolchain oficial de Rust y las bibliotecas de compilación requeridas por eframe/egui en Linux:

```bash
python3 src/assets/quick_accent.py --self-test
python3 -m unittest discover -s tests -v
/usr/bin/python3 tests/ibus_smoke.py
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
# Abrir la GUI solo cuando se quiera gestionar la instalación real:
cargo run --release --locked
```

Para instalar Rust localmente usá [rustup oficial](https://rust-lang.org/tools/install/), con las opciones `--profile minimal --component rustfmt,clippy --no-modify-path`. Instala en `~/.cargo` y `~/.rustup`, sin sudo. Para habilitarlo solo en la terminal actual: `source "$HOME/.cargo/env"`. No hace falta editar `.zshrc`. Se puede retirar con `~/.cargo/bin/rustup self uninstall`.

`cargo test` usa comandos simulados con HOME/runtime temporales: no controla servicios reales. `tests/ibus_smoke.py` levanta un D-Bus y un IBus privados y configura US solo en carpetas temporales; requiere `ibus-daemon`, `dbus-run-session`, Python GI y el typelib IBus. El sandbox puede pedir autorización para crear sockets. No toca la sesión de GNOME ni el servicio instalado.

**No se ejecuta con sudo.** En esta versión el motor se guarda embebido dentro del ejecutable Rust con `include_str!` y se extrae durante la instalación; basta distribuir el binario una vez compilado. Durante el build se requieren librerías nativas de ventanas, pero no hacen falta para usar el binario en una instalación compatible.

## Qué hacen los botones

| Botón | Comportamiento |
| --- | --- |
| Instalar | Comprueba IBus/Python, copia el ejecutable y motor, crea lanzador y servicio, y lo activa |
| On | Habilita/inicia el servicio; espera la selección IBus y la verifica. Reinicia el motor si ya corría sin estar seleccionado |
| Off | Restaura US si corresponde, coordina la parada, desactiva el autoinicio y verifica la selección después de salir |
| Actualizar estado | Muestra servicio, autoinicio, motor seleccionado y registro dinámico IBus |
| Logs | Muestra las últimas líneas de `journalctl --user -u quick-accent-local.service` |
| Desinstalar | Restaura US (si está activo), deshabilita el servicio, borra únicamente archivos propiedad del proyecto |

**Cerrar la ventana no detiene el motor.** Mientras una operación está pendiente, los controles quedan bloqueados y la ventana espera para cerrarse. Al terminar, cerrar la ventana no afecta al servicio. Dos ventanas no pueden modificar la instalación simultáneamente.

Si lo dejaste en On, se inicia al entrar a tu sesión (no antes de iniciar sesión o al encender físicamente la computadora). Si lo dejaste en Off, permanecerá apagado al volver a entrar.

### Archivos instalados en Linux

- `~/.local/bin/quick-accent-local-ui`
- `~/.local/share/quick-accent-local/quick_accent.py`
- `~/.local/share/quick-accent-local/.owned-by-quick-accent-local`
- `~/.local/share/applications/quick-accent-local.desktop`
- `~/.config/systemd/user/quick-accent-local.service`

El bloqueo de operaciones usa `$XDG_RUNTIME_DIR/quick-accent-local.lock`: es transitorio, se libera al salir del proceso y el sistema limpia el archivo al finalizar la sesión. No contiene datos de escritura.

No se modifica `~/.zshrc`, GNOME Tweaks, los repositorios APT, los controladores del teclado ni los grupos del usuario. El ZIP/carpeta de proyecto original queda fuera de la desinstalación.

## Seguridad y limitaciones

- El motor IBus procesa eventos de escritura de aplicaciones que usen ese método de entrada. Aunque **no tenga acceso a `/dev/input`**, **no significa que sea incapaz de recibir texto sensible**. El código no escribe contraseñas ni pulsaciones en disco, pero no garantiza detección perfecta de todos los campos de contraseña.
- No se realizan conexiones de red en el código propio. `cargo` sí descarga dependencias al compilar, y GitHub Actions necesita conexión para el build.
- Está orientado a **GNOME + Wayland + IBus**. El motor IBus y GNOME pueden competir al restaurar el método de entrada al iniciar sesión. Es importante probar cerrar sesión y entrar de nuevo.
- No está probado en una gran variedad de programas. Probá en Editor, Firefox y un IDE antes de confiar en él para documentos importantes.
- El botón **Instalar** reemplaza una instalación previa del mismo proyecto, pero verifica firmas/marcadores de los archivos de texto conocidos y evita sobrescribir rutas inesperadas. No es un sistema de actualizaciones firmado.
- `--auto-activate` consulta y selecciona IBus de forma asíncrona, con una operación en vuelo y tiempos de espera de dos segundos. Durante los primeros 30 segundos puede volver a seleccionar Quick Accent si GNOME restaura US. Una elección manual de US durante esa ventana también puede ser reemplazada: usá Off para desactivar. Otro motor suspende los reintentos. Después de esa ventana no fuerza cambios; si GNOME lo reemplaza más tarde, consultá Estado y usá On.
- El servicio pertenece a `graphical-session.target`, en lugar de `default.target`. Espera la sesión gráfica, se detiene con ella y se reinicia si pierde IBus. `active` significa proceso iniciado, no selección confirmada.
- Se conserva el registro dinámico mediante `register_component`: no se instala un XML en el catálogo permanente. Por eso `ibus list-engine --name-only` puede no listar Quick Accent aun cuando funcione. Estado o `python3 ~/.local/share/quick-accent-local/quick_accent.py --diagnose` comprueban el registro activo por separado. No se reinicia el daemon IBus ni se modifican `sources` o `mru-sources`.
- Instalar/actualizar sí detiene la versión anterior, reemplaza sus archivos y habilita/inicia el servicio. Off deshabilita su autoinicio. Desinstalar retira los archivos administrados tras comprobar la parada; conserva archivos ajenos. Estas acciones solo ocurren al pulsar los botones, no durante las pruebas automatizadas.
- Si el binario se borra mientras esta ventana está abierta (Desinstalar), la ventana actual seguirá existiendo hasta cerrarla; no habrá un lanzador en el menú.

### Recuperación manual

Si por cualquier motivo no pudieras usar la ventana:

```bash
ibus engine xkb:us::eng
systemctl --user disable --now quick-accent-local.service
```

Estos comandos no modifican distribuciones de GNOME ni instalan paquetes.

La revisión técnica, límites y pruebas manuales pendientes están en [docs/v0.4-review.md](docs/v0.4-review.md).

## Próximo paso: Windows

Reutilizar la **misma GUI Rust** con un adaptador Win32 para teclado (no IBus). Requiere pruebas de auto-repetición, teclas modificadoras, aplicaciones elevadas, campos de contraseña y teclados distintos. Una captura global de teclado en Windows implica acceso sensible; no debe publicarse como herramienta segura hasta auditarlo y probarlo. La lógica de acentos puede extraerse a un módulo Rust común, pero el backend de cada sistema operativo necesariamente será diferente.

## Licencia

MIT. Ver `LICENSE`.
