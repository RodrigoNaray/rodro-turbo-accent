# Rodro Turbo Accent

Escribí **á, é, í, ó, ú y ñ** con un teclado US: mantené una vocal o N durante **250 ms** y pulsá Espacio sin soltarla. También funciona con mayúsculas. Si Espacio llega antes, se escriben la letra normal y el espacio.

Aplicación local para **Linux con GNOME/Wayland e IBus**, inspirada en Quick Accent de PowerToys. Interfaz Rust/egui, motor Python/IBus y servicio `systemd --user`. Proyecto independiente y experimental.

## Estado

**Versión 0.5.0 experimental.** El usuario confirmó instalación, acentos, inicio automático y On/Off en Linux Mint con GNOME/Wayland y teclado US. La interfaz se adapta al ancho, usa texto de mayor tamaño, limita el ancho de sus botones y permite desplazamiento vertical.

Windows, una extensión de GNOME Shell y los paquetes para gestores de software todavía no están implementados. No se anuncia compatibilidad con otros entornos sin probarlos.

## Instalar sin Rust ni sudo

El paquete `rodro-turbo-accent-0.5.0-linux-x86_64.tar.gz` contiene el ejecutable, README y documentación. El equipo debe tener **IBus, Python 3 con GI y el typelib IBus 1.0, systemd de usuario y una sesión gráfica compatible**.

1. Extraé el paquete.
2. Abrí el ejecutable `rodro-turbo-accent`. Si hace falta, ejecutá `chmod +x rodro-turbo-accent` y después `./rodro-turbo-accent`.
3. Pulsá **Instalar** o **Actualizar / Reinstalar**.
4. Usá **On** para habilitar los acentos y su inicio automático, y **Off** para volver al teclado normal y deshabilitar el inicio automático.

Cerrar la ventana mantiene el servicio activo. Durante una operación la ventana espera a que termine antes de permitir el cierre. **Desinstalar** detiene el servicio y retira los archivos administrados. El instalador no instala paquetes del sistema: comprueba los requisitos antes de copiar archivos.

Para descargar desde CI: **Actions → Compilar Linux → Run workflow**. El artefacto `rodro-turbo-accent-linux-x86_64` contiene el paquete y su checksum SHA-256. Verificá que corresponda al commit que querés distribuir. No es un lanzamiento auditado ni una versión estable para todas las distribuciones.

## Compatibilidad con instalaciones anteriores

Rodro Turbo Accent es el nuevo nombre de Quick Accent Local. Se conservan los identificadores de IBus, del servicio y de los archivos instalados para actualizar la versión 0.4 sin crear un segundo motor:

- Motor IBus: `quick-accent-local`.
- Servicio: `quick-accent-local.service`.
- Ejecutable instalado: `~/.local/bin/quick-accent-local-ui`.
- Motor y marca de propiedad: `~/.local/share/quick-accent-local/`.
- Lanzador: `~/.local/share/applications/quick-accent-local.desktop`, visible como **Rodro Turbo Accent**.
- Unidad: `~/.config/systemd/user/quick-accent-local.service`.

El nombre del ejecutable distribuido y del paquete Rust es `rodro-turbo-accent`. El archivo fuente del motor sigue siendo `src/assets/quick_accent.py`. El repositorio y el remoto de GitHub se renombran por separado; cambiar estos archivos no cambia su visibilidad ni publica una release.

## Privacidad y funcionamiento

- No usa sudo, grupos de acceso a teclado ni captura directa de `/dev/input`.
- El código propio no transmite pulsaciones por red ni las guarda en logs. IBus procesa eventos de escritura: la detección de campos de contraseña es una protección de conveniencia, no una garantía para todas las aplicaciones.
- Mantiene la distribución US y no modifica `sources` ni `mru-sources` de GNOME.
- El registro IBus es dinámico. `ibus list-engine` puede no listar el motor aunque funcione; la ventana muestra por separado servicio, selección y registro activo.
- Durante los primeros 30 segundos puede restaurar Quick Accent si GNOME selecciona US; una selección manual de US en esa ventana también puede ser reemplazada. Usá Off para apagarlo. Otro motor suspende los reintentos; después de esa ventana no fuerza cambios.
- La actualización requiere que los archivos existentes pasen las comprobaciones de propiedad. La desinstalación conserva archivos ajenos. Los archivos de la carpeta donde descargaste el paquete quedan fuera de ella.

El bloqueo de operaciones utiliza `$XDG_RUNTIME_DIR/quick-accent-local.lock`, sin datos de escritura. El sistema limpia ese directorio al finalizar la sesión.

### Recuperación manual

```bash
ibus engine xkb:us::eng
systemctl --user disable --now quick-accent-local.service
```

## Desarrollo y comprobaciones

Rust estable, Python 3 y las bibliotecas necesarias de ventanas. La toolchain puede instalarse por usuario con [rustup](https://rust-lang.org/tools/install/), sin sudo. Si no se modificó el PATH: `source "$HOME/.cargo/env"`.

```bash
python3 scripts/check-publication.py
python3 src/assets/quick_accent.py --self-test
python3 -m unittest discover -s tests -v
/usr/bin/python3 tests/ibus_smoke.py
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
python3 scripts/build-release.py
python3 scripts/package-release.py
```

Las pruebas Rust usan HOME/runtime temporales y comandos de gestión simulados. La prueba IBus crea un D-Bus y un IBus privados; no cambia servicios o ajustes reales. El sandbox puede pedir permiso para crear sockets. Abrir la GUI con `cargo run --release --locked` sí permite gestionar la instalación real mediante sus botones.

Para distribuir, compilá con `scripts/build-release.py`: remapea las rutas personales del compilador. El empaquetado verifica que esas rutas no estén en el binario y elimina nombres de usuario del tar. `dist/` contiene el paquete, su checksum y metadatos públicos del commit; queda fuera de Git. Se necesita Python 3.11 o posterior para el script de empaquetado.

## Publicación, GNOME y apoyo al proyecto

La vía inicial propuesta es una descarga Linux desde GitHub Releases. Una extensión GNOME sería un desarrollo complementario; no se puede subir el ejecutable Rust actual como si fuera una extensión. El análisis y los pasos pendientes están en [distribución](docs/DISTRIBUTION.md).

Las donaciones quedan para una etapa posterior. No hay cuentas, enlaces de cobro, pagos obligatorios ni mecanismos de activación configurados.

- [Preparación del repositorio público](docs/PUBLICATION.md)
- [Guía para contribuir](CONTRIBUTING.md)
- [Seguridad y privacidad](SECURITY.md)
- [Cambios de versión](CHANGELOG.md)
- [Revisión histórica de la versión 0.4](docs/v0.4-review.md)
