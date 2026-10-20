# Rodro Turbo Accent

[English](README.md) | **Español**

Herramienta para Linux que permite escribir **á, é, í, ó, ú y ñ** manteniendo una distribución de teclado estadounidense. Desarrollada con una interfaz de escritorio en Rust y un motor IBus en Python.

Mantené una vocal o **N** durante al menos **250 ms** y pulsá **Espacio** sin soltarla. También funciona con mayúsculas. Por ejemplo, mantener `a` y pulsar Espacio produce `á`. Si Espacio llega antes, se conservan la letra normal y el espacio.

## Motivación y estado

El teclado US resulta práctico para programar, pero escribir en español requiere caracteres que no ofrece directamente. Este proyecto personal explora una interacción inspirada en Quick Accent de Microsoft PowerToys, usando IBus en Linux con GNOME/Wayland, sin cambiar la distribución ni leer dispositivos de entrada directamente. Es independiente, sin afiliación con Microsoft ni PowerToys.

**Versión 0.5.0 — MVP funcional, proyecto personal experimental en desarrollo.** El autor confirmó instalación, escritura de acentos, activación al iniciar sesión y On/Off en Linux Mint con GNOME/Wayland y teclado US. Esto no garantiza estabilidad ni compatibilidad generales.

## Funcionalidades actuales

- Vocales acentuadas y `ñ` / `Ñ`, en minúsculas y mayúsculas.
- Umbral fijo de 250 ms para reducir acentos accidentales al separar palabras.
- Interfaz redimensionable y desplazable: instalación, On, Off, estado, logs y desinstalación con confirmación.
- Instalación por usuario e inicio automático mediante `systemd --user`. Cerrar la ventana mantiene el motor activo.
- Activación IBus asíncrona, reintentos de arranque acotados y parada coordinada.
- Comprobaciones de propiedad al actualizar o eliminar archivos instalados.

La interfaz está actualmente en español. Solo el README es bilingüe.

## Tecnologías y requisitos

| Componente | Tecnología / requisito |
| --- | --- |
| Interfaz de escritorio | Rust, eframe/egui 0.31.1, renderizado OpenGL |
| Motor de entrada | Python 3, PyGObject (`gi`), introspección IBus 1.0 |
| Sesión | IBus, D-Bus, `systemd --user`, `journalctl`, `XDG_RUNTIME_DIR` |
| Escritorio confirmado | Linux Mint, GNOME sobre Wayland, distribución US |
| Compilación | Rust 1.89+, Cargo, herramientas C, pkg-config y bibliotecas de desarrollo de ventanas/OpenGL |
| Scripts de distribución | Python 3.11+; empaquetado Linux x86_64 |

La aplicación comprueba los requisitos, pero no instala paquetes del sistema. No requiere sudo, pertenecer al grupo `input` ni acceso directo a `/dev/input`. La sesión gráfica debe disponer de una conexión IBus funcional y un gestor systemd de usuario.

CI compila en Ubuntu 24.04 x86_64; compilar no demuestra el funcionamiento del escritorio en todas las instalaciones de Ubuntu. No se validaron Windows, macOS, otros escritorios ni otras arquitecturas. Este repositorio no contiene una extensión GNOME Shell, Flatpak ni paquete `.deb`.

## Compilar y ejecutar

Instalá Rust mediante [rustup](https://rust-lang.org/tools/install/) si hace falta. En Mint/Ubuntu, la lista de dependencias de CI sirve como referencia: `build-essential`, `pkg-config`, `libx11-dev`, `libxi-dev`, `libxcursor-dev`, `libxrandr-dev`, `libxkbcommon-dev`, `libwayland-dev`, `libgl1-mesa-dev`, `ibus`, `dbus-x11`, `python3-gi` y `gir1.2-ibus-1.0`. La instalación de paquetes del sistema corresponde por separado a su administración.

```bash
git clone https://github.com/RodrigoNaray/rodro-turbo-accent.git
cd rodro-turbo-accent
python3 scripts/build-release.py
./target/release/rodro-turbo-accent
```

Usá `source "$HOME/.cargo/env"` si Cargo no está en el PATH. El script también busca `~/.cargo/bin/cargo`. Acepta `--offline` si las dependencias fijadas ya están descargadas. Para una compilación de desarrollo, usá `cargo run --locked`.

Abrir la interfaz muestra el estado. Sus botones de gestión modifican la instalación y el servicio reales del usuario; ejecutar la interfaz no es una prueba aislada.

## Instalar y utilizar

1. Compilá y abrí la aplicación. También podés extraer un artefacto de CI desde **Actions → Compilar Linux**, verificar su checksum SHA-256 y ejecutar `./rodro-turbo-accent`. Comprobá que corresponda a la revisión de código prevista. No se necesita Rust para usar un binario compilado.
2. Pulsá **Instalar**. Copia el ejecutable, motor, lanzador y servicio de usuario, y después habilita e inicia el motor. **Actualizar / Reinstalar** actualiza una instalación existente.
3. **On** activa los acentos y su inicio automático. **Off** los detiene y deshabilita el inicio automático. Off restaura US si está seleccionado este motor; conserva otro motor seleccionado.
4. Mantené una letra admitida al menos 250 ms y pulsá Espacio. Soltá la letra antes de escribir la siguiente. Espacio se consume al producir el acento; pulsalo otra vez si necesitás separar palabras.
5. **Actualizar estado** consulta el servicio e IBus. **Logs** muestra las últimas 80 entradas del journal del servicio. **Desinstalar** pide confirmación, detiene el servicio y elimina los archivos administrados por la aplicación.

Cerrar la ventana mantiene el servicio activo. La interfaz espera a que termine una operación de gestión pendiente antes de cerrarse. La desinstalación no elimina el repositorio, el archivo descargado, el paquete extraído ni archivos ajenos.

El identificador interno del motor y el servicio siguen siendo `quick-accent-local` por compatibilidad con el nombre anterior. La instalación utiliza `~/.local/share/quick-accent-local/`, `~/.local/bin/quick-accent-local-ui`, `~/.config/systemd/user/quick-accent-local.service` y `~/.local/share/applications/quick-accent-local.desktop`.

## Limitaciones conocidas y recuperación

- El umbral es fijo y no hay pantalla de ajustes. Mantener una letra admitida suprime su repetición automática normal.
- La entrada depende de la integración IBus de la aplicación de destino. La detección de contraseñas/PIN depende de que las aplicaciones comuniquen el tipo de contenido; no garantiza protección para todos los campos sensibles.
- Durante 30 segundos desde el arranque del motor, la activación puede reemplazar US si GNOME lo selecciona. Elegir US manualmente en esa ventana también puede ser revertido; usá **Off** para desactivar el motor. Otro motor seleccionado suspende los reintentos.
- El registro es dinámico: `ibus list-engine` puede omitir `quick-accent-local` aunque funcione. Estado distingue el registro activo del catálogo persistente.
- On puede esperar hasta 35 segundos para confirmar la selección. Un servicio activo por sí solo no demuestra que los acentos estén funcionando.

El código del proyecto no transmite pulsaciones por red ni guarda el texto escrito en logs. Mantiene la distribución US y no modifica `sources` ni `mru-sources` de GNOME. Consultá [seguridad y privacidad](SECURITY.md).

Si hace falta, estos comandos de recuperación seleccionan US y deshabilitan el servicio:

```bash
ibus engine xkb:us::eng
systemctl --user disable --now quick-accent-local.service
```

## Pruebas y estructura

```bash
python3 scripts/check-publication.py
python3 src/assets/quick_accent.py --self-test
python3 -m unittest discover -s tests -v
/usr/bin/python3 tests/ibus_smoke.py
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Las pruebas Rust usan HOME/runtime temporales y comandos de gestión simulados. La prueba IBus utiliza un D-Bus e IBus privados con directorios temporales; no cambia la configuración real de GNOME ni el servicio systemd. Requiere el intérprete Python del sistema con GI y permiso para crear sockets locales.

| Ruta | Función |
| --- | --- |
| `src/main.rs` | Interfaz y tareas de gestión en segundo plano |
| `src/linux.rs` | Instalación y controles de servicio e IBus en Linux |
| `src/assets/` | Motor Python integrado y unidad systemd de usuario |
| `tests/` | Pruebas unitarias Python e integración IBus aislada |
| `scripts/` | Privacidad, compilación de distribución y empaquetado |
| `.github/workflows/` | Workflow de compilación y pruebas Linux |
| `docs/` | Publicación, licencias, donaciones y revisión histórica |

Después de compilar la versión de distribución, `python3 scripts/package-release.py` escribe un archivo y checksum en `dist/`, que queda ignorado. Compilá primero con `scripts/build-release.py`: remapea las rutas personales del compilador; el empaquetado verifica el binario y normaliza la propiedad del archivo. Los metadatos registran el commit y los cambios sin confirmar. Revisá las [obligaciones de distribución](docs/LICENSING.md) antes de compartir binarios.

## Errores y contribuciones

Usá [GitHub Issues](https://github.com/RodrigoNaray/rodro-turbo-accent/issues) para errores reproducibles o sugerencias. Incluí versión, distribución, detalles de GNOME/sesión, teclado, pasos, comportamiento esperado y real. Revisá los logs antes de adjuntarlos. Para problemas de seguridad, seguí [SECURITY.md](SECURITY.md).

Las contribuciones son bienvenidas: conversá los cambios en una issue, mantené las pull requests enfocadas y ejecutá las comprobaciones anteriores. Leé [CONTRIBUTING.md](CONTRIBUTING.md); preservá el motor funcional y los identificadores de instalación existentes.

## Licencia, autor y apoyo opcional

Copyright © 2026 Rodrigo Naray.

El código original se distribuye bajo **GNU GPL versión 3 únicamente (`GPL-3.0-only`)**, sin la opción «o cualquier versión posterior». [LICENSE](LICENSE) contiene el texto completo y sin modificaciones. Las dependencias y fuentes de terceros conservan sus licencias y copyrights; consultá [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) y la [revisión de licencias](docs/LICENSING.md).

Documentación adicional: [publicación](docs/PUBLICATION.md), [distribución](docs/DISTRIBUTION.md), [cambios](CHANGELOG.md), [revisión histórica de 0.4](docs/v0.4-review.md). Los documentos de apoyo están en inglés o español según su contenido.
