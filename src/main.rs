#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
//! Quick Accent Local: la ventana es Rust en ambas plataformas.
//! En Linux reutiliza el motor IBus Python ya probado.
//! El backend de Windows aún no se ha implementado.

use eframe::egui;

#[cfg(target_os = "linux")]
mod linux;

#[derive(Default)]
struct QuickAccentApp {
    installed: bool,
    status: String,
    message: String,
    logs: String,
    showing_logs: bool,
    confirm_uninstall: bool,
}

impl QuickAccentApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut app = Self::default();
        app.refresh_status();
        app
    }

    fn refresh_status(&mut self) {
        #[cfg(target_os = "linux")]
        {
            self.installed = linux::is_installed();
            self.status = match linux::status() {
                Ok(s) => s,
                Err(err) => format!("No se pudo comprobar el estado: {err}"),
            };
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.installed = false;
            self.status = "Windows: motor todavía no disponible. La interfaz es multiplataforma, pero falta implementar y validar el adaptador nativo de teclado.".into();
        }
    }

    fn action(&mut self, label: &str) {
        #[cfg(target_os = "linux")]
        let result = match label {
            "Instalar" => linux::install(),
            "On" => linux::turn_on(),
            "Off" => linux::turn_off(),
            "Desinstalar" => linux::uninstall(),
            _ => Err("Acción desconocida".into()),
        };
        #[cfg(not(target_os = "linux"))]
        let result: Result<String, String> = Err("El adaptador nativo de Windows está pendiente.".into());

        self.message = match result {
            Ok(s) => s,
            Err(e) => format!("Error: {e}"),
        };
        self.refresh_status();
    }

    fn load_logs(&mut self) {
        #[cfg(target_os = "linux")]
        {
            self.logs = linux::logs().unwrap_or_else(|err| format!("Error: {err}"));
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.logs = "Los registros estarán disponibles cuando exista el motor de Windows.".into();
        }
        self.showing_logs = true;
    }
}

impl eframe::App for QuickAccentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ui.heading("Quick Accent Local");
            ui.label("Mantener una vocal o N + Espacio → á é í ó ú ñ");
            ui.label("Distribución US · sin sudo · sin acceso directo a /dev/input");
            ui.add_space(12.0);

            ui.group(|ui| {
                ui.label(egui::RichText::new("Estado").strong());
                ui.add_space(4.0);
                ui.label(&self.status);
            });
            ui.add_space(12.0);

            ui.horizontal_wrapped(|ui| {
                #[cfg(target_os = "linux")]
                let supported = true;
                #[cfg(not(target_os = "linux"))]
                let supported = false;

                let install_label = if self.installed { "Actualizar / Reinstalar" } else { "Instalar" };
                if ui.add_enabled(supported, egui::Button::new(install_label)).clicked() {
                    self.action("Instalar");
                }
                if ui.add_enabled(supported && self.installed, egui::Button::new("On")).clicked() {
                    self.action("On");
                }
                if ui.add_enabled(supported && self.installed, egui::Button::new("Off")).clicked() {
                    self.action("Off");
                }
                if ui.button("Actualizar estado").clicked() {
                    self.refresh_status();
                }
                if ui.add_enabled(supported && self.installed, egui::Button::new("Logs")).clicked() {
                    self.load_logs();
                }
                if ui.add_enabled(supported && self.installed, egui::Button::new("Desinstalar")).clicked() {
                    self.confirm_uninstall = true;
                }
            });
            if self.confirm_uninstall {
                ui.add_space(7.0);
                ui.group(|ui| {
                    ui.label("¿Seguro que querés desinstalar? Se quitarán el servicio, el lanzador y el motor de esta aplicación.");
                    ui.horizontal(|ui| {
                        if ui.button("Cancelar").clicked() {
                            self.confirm_uninstall = false;
                        }
                        if ui.button("Sí, desinstalar").clicked() {
                            self.confirm_uninstall = false;
                            self.action("Desinstalar");
                        }
                    });
                });
            }
            if !self.message.is_empty() {
                ui.add_space(8.0);
                ui.label(&self.message);
            }
            if self.showing_logs {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.heading("Registros del servicio");
                    if ui.small_button("Cerrar logs").clicked() {
                        self.showing_logs = false;
                    }
                    if ui.small_button("Actualizar logs").clicked() {
                        self.load_logs();
                    }
                });
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.logs)
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(11)
                            .interactive(false));
                    });
            }
            ui.add_space(12.0);
            ui.separator();
            ui.small("On activa el motor y su inicio automático. Off vuelve a US y desactiva el arranque. Cerrar esta ventana no detiene el motor.");
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([690.0, 425.0])
            .with_min_inner_size([600.0, 355.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Quick Accent Local",
        options,
        Box::new(|cc| Ok(Box::new(QuickAccentApp::new(cc)))),
    )
}
