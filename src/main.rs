#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
//! Quick Accent Local: la ventana es Rust en ambas plataformas.
//! En Linux reutiliza el motor IBus Python ya probado.
//! El backend de Windows aún no se ha implementado.

use eframe::egui;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

struct WorkResult {
    installed: bool,
    status: String,
    message: Option<String>,
    logs: Option<String>,
}

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
    pending: Option<Receiver<WorkResult>>,
}

impl QuickAccentApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut app = Self::default();
        app.refresh_status();
        app
    }

    fn refresh_status(&mut self) {
        self.start_work("Estado");
    }

    fn action(&mut self, label: &str) {
        self.start_work(label);
    }

    fn load_logs(&mut self) {
        self.start_work("Logs");
    }

    fn start_work(&mut self, label: &str) {
        if self.pending.is_some() {
            return;
        }
        let label = label.to_owned();
        let (sender, receiver) = mpsc::channel();
        self.pending = Some(receiver);
        std::thread::spawn(move || {
            #[cfg(target_os = "linux")]
            let result = {
                let action = match label.as_str() {
                    "Instalar" => Some(linux::install()),
                    "On" => Some(linux::turn_on()),
                    "Off" => Some(linux::turn_off()),
                    "Desinstalar" => Some(linux::uninstall()),
                    _ => None,
                };
                WorkResult {
                    installed: linux::is_installed(),
                    status: linux::status().unwrap_or_else(|e| format!("Error de estado: {e}")),
                    message: action.map(|r| r.unwrap_or_else(|e| format!("Error: {e}"))),
                    logs: if label == "Logs" {
                        Some(linux::logs().unwrap_or_else(|e| format!("Error: {e}")))
                    } else {
                        None
                    },
                }
            };
            #[cfg(not(target_os = "linux"))]
            let result = WorkResult {
                installed: false,
                status: "Windows: adaptador nativo de teclado todavía pendiente.".into(),
                message: None,
                logs: None,
            };
            let _ = sender.send(result);
        });
    }

    fn poll_work(&mut self, ctx: &egui::Context) {
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(result) => {
                    self.installed = result.installed;
                    self.status = result.status;
                    if let Some(message) = result.message {
                        self.message = message;
                    }
                    if let Some(logs) = result.logs {
                        self.logs = logs;
                        self.showing_logs = true;
                    }
                    self.pending = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.message = "La operación terminó inesperadamente; consultá Estado antes de reintentar.".into();
                    self.pending = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(100))
                }
            }
        }
    }
}

impl eframe::App for QuickAccentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_work(ctx);
        if self.pending.is_some() && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ui.heading("Quick Accent Local");
            ui.label("Mantener una vocal o N durante 250 ms + Espacio → á é í ó ú ñ");
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
                let supported = self.pending.is_none();
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
                if ui.add_enabled(self.pending.is_none(), egui::Button::new("Actualizar estado")).clicked() {
                    self.refresh_status();
                }
                if ui.add_enabled(supported && self.installed, egui::Button::new("Logs")).clicked() {
                    self.load_logs();
                }
                if ui.add_enabled(supported && self.installed, egui::Button::new("Desinstalar")).clicked() {
                    self.confirm_uninstall = true;
                }
            });
            if self.pending.is_some() {
                ui.label("Operación en curso… Esperando confirmación de IBus/systemd.");
            }
            if self.confirm_uninstall {
                ui.add_space(7.0);
                ui.group(|ui| {
                    ui.label("¿Seguro que querés desinstalar? Se quitarán el servicio, el lanzador y el motor de esta aplicación.");
                    ui.horizontal(|ui| {
                        if ui.button("Cancelar").clicked() {
                            self.confirm_uninstall = false;
                        }
                        if ui.add_enabled(self.pending.is_none(), egui::Button::new("Sí, desinstalar")).clicked() {
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
