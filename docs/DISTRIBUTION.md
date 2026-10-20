# Distribución de Rodro Turbo Accent

Copyright © 2026 Rodrigo Naray. SPDX-License-Identifier: GPL-3.0-only.

## Formato disponible

El repositorio contiene un MVP funcional con GUI Rust, motor Python/IBus y servicio systemd de usuario. El entorno de uso confirmado es Linux Mint con GNOME/Wayland y teclado US. El empaquetado existente produce un archivo Linux x86_64; no hay instaladores .deb, Flatpak ni extensión GNOME Shell.

La instalación se realiza desde la GUI por usuario, sin sudo. Las dependencias del sistema deben estar disponibles previamente. La interfaz permite actualizar o desinstalar los archivos administrados; conserva identificadores del antiguo Quick Accent Local por compatibilidad.

## Compilación y paquete local

```bash
python3 scripts/build-release.py
python3 scripts/package-release.py
```

El primer script compila con Cargo.lock y remapea las rutas personales del compilador. El segundo comprueba el formato ELF x86_64 y la ausencia de las rutas actuales del usuario/repositorio, normaliza los propietarios/permisos del tar y genera un checksum SHA-256. No instala ni publica el resultado.

El archivo en `dist/` contiene el ejecutable, ambos README, GPL v3 íntegra, copyright, avisos de terceros, documentación y `BUILD-INFO.json`. Los metadatos identifican el commit y si había cambios sin confirmar. Un paquete generado durante esta preparación se marca como no confirmado; no debe presentarse como una release del commit anterior.

## GitHub Actions

Se conserva **Compilar Linux**, que usa Ubuntu 24.04, ejecuta privacidad, pruebas Python/IBus, formato, pruebas Rust y Clippy, y genera un artefacto con el paquete y su checksum. El workflow tiene permisos de lectura y no persiste credenciales de checkout. Rustfmt y Clippy se solicitan explícitamente en la toolchain.

La compilación de CI no reemplaza probar la instalación en una sesión gráfica compatible. Un artefacto de compilación no significa que exista una versión estable ni una release publicada. Antes de cambiar la visibilidad, revisá también artefactos antiguos del servidor.

## Antes de compartir binarios

Se debe ofrecer el código fuente correspondiente mediante un método permitido por GPLv3, incluyendo las dependencias requeridas y los scripts de compilación/instalación. El paquete actual no genera un archivo con dependencias vendorizadas. Tampoco instala por separado una carpeta de avisos legales. Consultá [LICENSING.md](LICENSING.md) y conservá todos los avisos del paquete.

Las fuentes integradas conservan sus licencias propias: no deben presentarse como relicenciadas bajo GPL. Verificá estas obligaciones para el artefacto concreto antes de distribuirlo públicamente. Para publicar solo el repositorio, seguí [PUBLICATION.md](PUBLICATION.md).
