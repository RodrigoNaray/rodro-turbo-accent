# Contribuir a Rodro Turbo Accent

Describí el problema y el comportamiento esperado antes de cambiar el motor de entrada. Incluí entorno gráfico, versión de GNOME, distribución y versión de la aplicación. Evitá adjuntar texto escrito, datos personales o logs sin revisar.

Usá un correo noreply de GitHub para los commits. Podés configurar el del repositorio con `git config --local user.email DIRECCION_NOREPLY`. El chequeo de publicación exige direcciones noreply en los commits de la revisión que se publique.

Mantené la separación entre GUI Rust y backend Linux. Los identificadores `quick-accent-local` se conservan por compatibilidad; no deben renombrarse sin una migración probada. Windows necesita un backend propio.

Antes de proponer cambios, ejecutá formato, pruebas y Clippy según README. Las pruebas automatizadas no deben gestionar los servicios reales de la sesión. Cualquier prueba de instalación, cambio de motor o ajustes de GNOME debe explicar sus efectos antes de ejecutarse.

No agregues credenciales, archivos `.env`, certificados privados, respaldos Git ni ejecutables locales al historial. Para distribuir binarios, usá los scripts de compilación y empaquetado que remapean rutas y normalizan el tar.
