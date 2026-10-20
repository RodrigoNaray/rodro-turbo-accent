# Preparación del repositorio público

## Identidad y nombre

Nombre público: **Rodro Turbo Accent**. Nombre sugerido del repositorio: `rodro-turbo-accent`. La carpeta de trabajo y el remoto existente no se renombran automáticamente.

La autoría y confirmación de los cinco commits de `main` se reescribieron con la dirección noreply de GitHub proporcionada por el propietario. Se conservaron árboles, mensajes, fechas y relaciones entre commits; cambiaron sus identificadores. Los futuros commits usan ese correo mediante configuración local de Git.

El respaldo previo está fuera del repositorio, en una ubicación privada comunicada al propietario. No debe añadirse al repositorio ni a las descargas.

**El remoto existente y su referencia local `origin/main` conservan el historial anterior.** No se hizo push ni se cambió la visibilidad. Por eso un escaneo de todas las referencias locales todavía puede encontrar el correo previo en `origin/main` o en objetos/reflogs de respaldo. La rama preparada para publicar es `main`; no se deben publicar respaldos, refs internas ni usar `push --mirror`.

## Publicación pendiente en GitHub

1. Revisar y confirmar los cambios de marca y preparación usando el correo noreply.
2. Ejecutar `python3 scripts/check-publication.py` sobre el commit que se publicará. El chequeo revisa la autoría de `HEAD`, sus ancestros y el contenido disponible localmente.
3. Decidir el destino: un repositorio público nuevo evita publicar el historial previo del remoto privado. Si se reutiliza el repositorio existente, hay que planificar explícitamente la sustitución del historial remoto y revisar sus otras ramas, logs, adjuntos y referencias; no alcanza con cambiar la visibilidad.
4. Renombrar o crear el repositorio remoto con autorización del propietario y configurar la URL correspondiente. No se incluyeron enlaces ficticios a un repositorio que todavía no existe.
5. Publicar únicamente la rama revisada y crear la release de un commit limpio. No mezclar de nuevo el historial antiguo ni subir la carpeta `.git`.
6. Ejecutar el workflow del nuevo commit y probar su paquete en una sesión de destino antes de compartirlo ampliamente.
7. Revisar los controles de GitHub para privacidad del email, reporte privado de vulnerabilidades y protección contra secretos.

El cambio de identidad no elimina copias ya existentes fuera del repositorio local. Esta preparación no revisa ni modifica datos alojados en GitHub.

## Artefactos y datos personales

`dist/`, `.env`, claves privadas y respaldos comunes están ignorados. El workflow usa permisos de lectura y no persiste credenciales de checkout. Esto reduce incorporaciones accidentales; no sustituye revisar cada cambio.

Usar los scripts de release remapea las rutas personales del compilador. El empaquetado también elimina nombres de usuario y propietarios de los metadatos tar. `BUILD-INFO.json` contiene versión, plataforma, commit y si había cambios sin confirmar. No es una firma criptográfica ni una garantía de auditoría.
