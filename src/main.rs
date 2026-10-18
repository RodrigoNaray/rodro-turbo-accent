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

fn configure_style(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    ctx.style_mut(|style| {
        for (text_style, size) in [
            (egui::TextStyle::Heading, 28.0),
            (egui::TextStyle::Body, 18.0),
            (egui::TextStyle::Button, 18.0),
            (egui::TextStyle::Small, 15.0),
            (egui::TextStyle::Monospace, 16.0),
        ] {
            let family = if text_style == egui::TextStyle::Monospace {
                egui::FontFamily::Monospace
            } else {
                egui::FontFamily::Proportional
            };
            style
                .text_styles
                .insert(text_style, egui::FontId::new(size, family));
        }
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 9.0);
        style.spacing.interact_size.y = 42.0;
    });
}

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
        configure_style(&cc.egui_ctx);
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

    fn show_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.add_space(10.0);
        ui.heading("Quick Accent Local");
        ui.horizontal_wrapped(|ui| {
            ui.label("Mantené una vocal o N durante 250 ms y pulsá Espacio");
            // Dibujar la flecha evita depender de un glifo que puede faltar.
            let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 24.0), egui::Sense::hover());
            ui.painter().arrow(
                egui::pos2(rect.left() + 3.0, rect.center().y),
                egui::vec2(24.0, 0.0),
                egui::Stroke::new(2.0_f32, ui.visuals().text_color()),
            );
            ui.label("á é í ó ú ñ");
        });
        ui.label("Distribución US · sin sudo · sin acceso directo a /dev/input");
        ui.add_space(12.0);

        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("Estado").strong());
            ui.add_space(4.0);
            ui.label(&self.status);
        });
        ui.add_space(12.0);

        let supported = cfg!(target_os = "linux") && self.pending.is_none();
        let install_label = if self.installed {
            "Actualizar / Reinstalar"
        } else {
            "Instalar"
        };
        let gap = 12.0;
        let columns = ((ui.available_width() + gap) / (220.0 + gap))
            .floor()
            .clamp(1.0, 3.0) as usize;
        let button_width = ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32)
            .min(240.0)
            .floor();
        let actions = [
            (install_label, "Instalar", supported),
            ("On", "On", supported && self.installed),
            ("Off", "Off", supported && self.installed),
            ("Actualizar estado", "Estado", self.pending.is_none()),
            ("Logs", "Logs", supported && self.installed),
            ("Desinstalar", "Desinstalar", supported && self.installed),
        ];
        egui::Grid::new(("action_buttons", columns))
            .num_columns(columns)
            .min_col_width(button_width)
            .max_col_width(button_width)
            .spacing([gap, gap])
            .show(ui, |ui| {
                for (index, (label, action, enabled)) in actions.into_iter().enumerate() {
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::new(label).min_size(egui::vec2(button_width, 46.0)),
                        )
                        .clicked()
                    {
                        match action {
                            "Estado" => self.refresh_status(),
                            "Logs" => self.load_logs(),
                            "Desinstalar" => self.confirm_uninstall = true,
                            _ => self.action(action),
                        }
                    }
                    if (index + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
        if self.pending.is_some() {
            ui.label("Operación en curso… Esperando confirmación de IBus/systemd.");
        }
        if self.confirm_uninstall {
            ui.add_space(7.0);
            ui.group(|ui| {
                ui.label("¿Seguro que querés desinstalar? Se quitarán el servicio, el lanzador y el motor de esta aplicación.");
                ui.horizontal_wrapped(|ui| {
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
            ui.horizontal_wrapped(|ui| {
                ui.heading("Registros del servicio");
                if ui.button("Cerrar logs").clicked() {
                    self.showing_logs = false;
                }
                if ui
                    .add_enabled(self.pending.is_none(), egui::Button::new("Actualizar logs"))
                    .clicked()
                {
                    self.load_logs();
                }
            });
            let log_height = (ctx.screen_rect().height() * 0.4).max(120.0);
            let line_height = ui.text_style_height(&egui::TextStyle::Monospace);
            egui::ScrollArea::vertical()
                .id_salt("service_logs")
                .max_height(log_height)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.logs)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows((log_height / line_height).floor() as usize)
                            .interactive(false),
                    );
                });
        }
        ui.add_space(12.0);
        ui.separator();
        ui.label("On activa el motor y su inicio automático. Off vuelve a US y desactiva el arranque. Cerrar esta ventana no detiene el motor.");
    }
}

impl eframe::App for QuickAccentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_work(ctx);
        if self.pending.is_some() && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.show_content(ui, ctx);
                });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 520.0])
            .with_min_inner_size([320.0, 240.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Quick Accent Local",
        options,
        Box::new(|cc| Ok(Box::new(QuickAccentApp::new(cc)))),
    )
}
