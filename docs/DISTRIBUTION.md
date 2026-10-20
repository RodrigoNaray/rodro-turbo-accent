# Distribución de Rodro Turbo Accent

Revisión de opciones: 20 de octubre de 2026. Estado: preparación local; no se publicó un repositorio, paquete de tienda ni extensión.

## Primera distribución: aplicación Linux

La arquitectura actual encaja con una descarga nativa para Linux x86_64: GUI Rust, motor Python/IBus y servicio por usuario. El entorno confirmado es Linux Mint con GNOME/Wayland y teclado US. La propuesta inicial es publicar una release experimental con `.tar.gz`, checksum, licencia, documentación y metadatos del commit. Se instala desde la GUI sin sudo; las dependencias del sistema deben estar disponibles.

El siguiente paso de empaquetado puede ser un `.deb` para Mint/Ubuntu, después de definir cómo convivirá la instalación por paquete con la instalación por usuario. No se preparó un `.deb` que ejecute automáticamente cambios de sesión o instale paquetes del sistema. No se anuncia soporte para otras distribuciones sin comprobarlo.

## GNOME Shell Extensions

El catálogo de GNOME distribuye extensiones con `extension.js` y `metadata.json`. Su revisión exige código JavaScript legible; prohíbe incluir ejecutables binarios y limita el uso de scripts externos. Por eso la aplicación Rust/Python actual no es directamente un ZIP publicable en extensions.gnome.org.

Una extensión complementaria podría ofrecer estado y controles de panel, comunicándose de forma asíncrona con el backend instalado aparte. Requiere diseñar la interfaz de control, probar versiones reales de GNOME y limpiar señales/temporizadores en `disable()`. No debe prometer compatibilidad con versiones futuras. La aceptación depende de la revisión de GNOME, no de esta planificación.

Fuentes oficiales: [estructura de una extensión](https://gjs.guide/extensions/overview/anatomy.html), [reglas de revisión](https://gjs.guide/extensions/review-guidelines/review-guidelines.html).

## Flatpak y Flathub

Flatpak limita por defecto el acceso al sistema de archivos y al D-Bus del host. Esta aplicación copia un motor y una unidad systemd a la cuenta del usuario, de modo que no basta con envolver el instalador actual en un Flatpak.

La evaluación propuesta es separar backend del host y GUI antes de diseñar un manifiesto con permisos mínimos. No hay manifiesto Flatpak ni aprobación de Flathub. Evitar dar acceso global al host para forzar el instalador actual.

Fuentes oficiales: [permisos de sandbox](https://docs.flatpak.org/en/latest/sandbox-permissions.html), [envío de aplicaciones a Flathub](https://docs.flathub.org/docs/for-app-authors/submission).

## Donaciones (posterior)

La decisión actual es mantener la aplicación disponible bajo MIT y añadir donaciones voluntarias más adelante. No hay enlace activo ni cuenta de cobro configurada.

Cuando se elija una plataforma y exista una URL real, se puede añadir una sección de apoyo al README y `.github/FUNDING.yml` para mostrar el botón Sponsor. GitHub documenta proveedores admitidos y enlaces personalizados. Una futura extensión también admite metadatos de donaciones, sujetos a sus reglas. No se crean perfiles, cuentas ni promesas de beneficios de pago en esta preparación.

Fuentes oficiales: [botón Sponsor en GitHub](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/displaying-a-sponsor-button-in-your-repository), [metadatos de extensiones](https://gjs.guide/extensions/overview/anatomy.html).
