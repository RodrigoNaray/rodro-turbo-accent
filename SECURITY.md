# Seguridad y privacidad

Rodro Turbo Accent procesa eventos mediante IBus para transformar vocal/N + Espacio. El código propio no envía esos eventos por red ni escribe texto pulsado en logs. Algunas aplicaciones pueden no identificar correctamente los campos sensibles: no se garantiza que el motor deje de recibir todos esos eventos.

Si encontrás una vulnerabilidad o una credencial expuesta, no publiques la clave ni datos sensibles en una issue. Utilizá el reporte privado de vulnerabilidades de GitHub si está habilitado. Si no lo está, abrí una issue solicitando un canal privado, sin incluir el contenido sensible. Un correo noreply de GitHub es una identidad de commits, no un buzón de soporte.

Para preparar una publicación:

- Revisá el código, el historial de la rama que se publicará, los metadatos de autoría y el paquete descargable.
- Ejecutá `python3 scripts/check-publication.py`; es una comprobación básica de patrones, no una auditoría completa ni una garantía de ausencia de secretos.
- No adjuntes respaldos privados ni el directorio `.git` a una release.
- Generá binarios con `scripts/build-release.py` y paquetes con `scripts/package-release.py`.
- Revisá permisos, logs y adjuntos del repositorio alojado en GitHub antes de cambiar su visibilidad. La revisión local no inspecciona esos datos.

Si una clave ya estuvo expuesta, eliminarla del código no la invalida: debe revocarse en su proveedor. No se necesita una clave ni un token para utilizar esta aplicación.
