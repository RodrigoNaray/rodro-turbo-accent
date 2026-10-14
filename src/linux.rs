//! Integración con GNOME/Wayland: usa IBus y systemd de *usuario*.
//! No usa sudo, no toca /usr, no modifica permisos de /dev/input.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

const SERVICE: &str = "quick-accent-local.service";
const ENGINE: &str = "quick-accent-local";
const OWNED_MARKER: &str = "Quick Accent Local — archivos de esta aplicación.\n";
const ENGINE_SOURCE: &str = include_str!("assets/quick_accent.py");

fn home() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "No encuentro la carpeta HOME del usuario".to_owned())
}

fn app_dir() -> Result<PathBuf, String> {
    Ok(home()?.join(".local/share/quick-accent-local"))
}
fn executable_path() -> Result<PathBuf, String> {
    Ok(home()?.join(".local/bin/quick-accent-local-ui"))
}
fn unit_path() -> Result<PathBuf, String> {
    Ok(home()?.join(".config/systemd/user").join(SERVICE))
}
fn desktop_path() -> Result<PathBuf, String> {
    Ok(home()?.join(".local/share/applications/quick-accent-local.desktop"))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}
fn ensure_project_file(path: &Path, marker: &str) -> Result<(), String> {
    if path.exists() && !read(path).contains(marker) {
        return Err(format!(
            "Ya existe un archivo que no parece ser de Quick Accent Local: {}. No lo sobrescribí.",
            path.display()
        ));
    }
    Ok(())
}
fn execute(program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("No pude ejecutar {program}: {e}"))
}
fn checked(program: &str, args: &[&str]) -> Result<String, String> {
    let output = execute(program, args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{program} {}: {}", args.join(" "), stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
fn systemctl(args: &[&str]) -> Result<String, String> {
    let mut full = vec!["--user"];
    full.extend_from_slice(args);
    checked("systemctl", &full)
}
fn output_ignoring_failure(program: &str, args: &[&str]) -> String {
    match execute(program, args) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => "no disponible".to_owned(),
    }
}
fn engine() -> String {
    output_ignoring_failure("ibus", &["engine"])
}

pub fn is_installed() -> bool {
    match (app_dir(), unit_path()) {
        (Ok(app), Ok(unit)) => app.join("quick_accent.py").exists() && unit.exists(),
        _ => false,
    }
}

pub fn status() -> Result<String, String> {
    if !cfg!(target_os = "linux") {
        return Ok("Plataforma no compatible".into());
    }
    let installed = is_installed();
    let active = output_ignoring_failure("systemctl", &["--user", "is-active", SERVICE]);
    let enabled = output_ignoring_failure("systemctl", &["--user", "is-enabled", SERVICE]);
    let current = engine();
    let mode = if current == ENGINE && active == "active" {
        "ON — acentos activos"
    } else if installed && enabled == "enabled" {
        "Instalado; aún no seleccionado en IBus"
    } else if installed {
        "OFF — teclado US normal"
    } else {
        "Sin instalar"
    };
    Ok(format!(
        "{mode}\nServicio: {active}  ·  Inicio automático: {enabled}\nMotor IBus: {current}"
    ))
}

fn check_requirements() -> Result<(), String> {
    for cmd in ["python3", "ibus", "systemctl"] {
        let output = Command::new(cmd).arg("--version").output();
        if output.is_err() {
            return Err(format!("Falta {cmd}. No se instaló nada."));
        }
    }
    checked("/usr/bin/python3", &["-c", "import gi; gi.require_version('IBus', '1.0'); from gi.repository import IBus, GLib"])?;
    Ok(())
}

fn desktop_exec_escape(path: &Path) -> String {
    // La especificación .desktop requiere comillas y escapes para las rutas con espacios.
    let escaped = path.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub fn install() -> Result<String, String> {
    check_requirements()?;
    let app = app_dir()?;
    let unit = unit_path()?;
    let desktop = desktop_path()?;
    let executable = executable_path()?;
    let engine_file = app.join("quick_accent.py");
    let marker_file = app.join(".owned-by-quick-accent-local");

    ensure_project_file(&engine_file, "Quick Accent Local:")?;
    ensure_project_file(&unit, "Description=Quick Accent Local")?;
    ensure_project_file(&desktop, "X-QuickAccentLocal-Managed=true")?;
    if executable.exists() && !marker_file.exists() {
        return Err(format!("Ya existe {} y no hay marca de propiedad del proyecto. No lo sobrescribí.", executable.display()));
    }

    fs::create_dir_all(&app).map_err(|e| e.to_string())?;
    fs::create_dir_all(unit.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::create_dir_all(desktop.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::create_dir_all(executable.parent().unwrap()).map_err(|e| e.to_string())?;

    // Al actualizar, detenemos la versión anterior antes de reemplazar el motor.
    if unit.exists() {
        let _ = systemctl(&["disable", "--now", SERVICE]);
    }

    fs::write(&engine_file, ENGINE_SOURCE).map_err(|e| e.to_string())?;
    fs::set_permissions(&engine_file, fs::Permissions::from_mode(0o700))
        .map_err(|e| e.to_string())?;

    let service_text = "# Quick Accent Local: unidad de usuario, no requiere sudo\n\
[Unit]\nDescription=Quick Accent Local - motor IBus de acentos\nAfter=graphical-session.target\n\
[Service]\nType=simple\nExecStart=/usr/bin/python3 %h/.local/share/quick-accent-local/quick_accent.py --auto-activate\nRestart=on-failure\nRestartSec=4\n\
[Install]\nWantedBy=default.target\n";
    fs::write(&unit, service_text).map_err(|e| e.to_string())?;

    let current = env::current_exe().map_err(|e| e.to_string())?;
    if current != executable {
        // Copia temporal + rename: reemplaza una versión anterior aunque esté abierta.
        let temporary = executable.with_extension("tmp-quick-accent-local");
        fs::copy(&current, &temporary).map_err(|e| format!("No pude copiar la aplicación: {e}"))?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
        fs::rename(&temporary, &executable).map_err(|e| format!("No pude instalar el ejecutable: {e}"))?;
    }
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
        .map_err(|e| e.to_string())?;

    let desktop_text = format!(
        "[Desktop Entry]\nType=Application\nName=Quick Accent Local\nComment=Gestionar acentos con vocal mantenida y espacio\nExec={}\nIcon=preferences-desktop-keyboard-shortcuts\nTerminal=false\nCategories=Utility;Accessibility;\nX-QuickAccentLocal-Managed=true\n",
        desktop_exec_escape(&executable)
    );
    fs::write(&desktop, desktop_text).map_err(|e| e.to_string())?;
    fs::write(marker_file, OWNED_MARKER).map_err(|e| e.to_string())?;
    systemctl(&["daemon-reload"])?;
    let on_message = turn_on()?;
    Ok(format!("Instalado solo para tu usuario. Aparecerá en el menú de aplicaciones. {on_message}"))
}

pub fn turn_on() -> Result<String, String> {
    if !is_installed() {
        return Err("Primero presioná Instalar".into());
    }
    systemctl(&["enable", "--now", SERVICE])?;
    for _ in 0..18 {
        if checked("ibus", &["engine", ENGINE]).is_ok() {
            return Ok("Acentos ON. El motor arrancará automáticamente al iniciar sesión.".into());
        }
        thread::sleep(Duration::from_millis(350));
    }
    Err("El servicio se inició, pero IBus aún no seleccionó nuestro motor. Revisá Logs y volvé a intentar On.".into())
}

pub fn turn_off() -> Result<String, String> {
    if !is_installed() {
        return Err("La aplicación no está instalada".into());
    }
    // No modificar el teclado si el usuario ya seleccionó otro distinto.
    if engine() == ENGINE {
        checked("ibus", &["engine", "xkb:us::eng"])?;
    }
    systemctl(&["disable", "--now", SERVICE])?;
    Ok("Acentos OFF. El servicio quedó detenido y no se iniciará al entrar en sesión.".into())
}

pub fn logs() -> Result<String, String> {
    checked("journalctl", &["--user", "--unit", SERVICE, "--no-pager", "-n", "80"])
}

pub fn uninstall() -> Result<String, String> {
    if !is_installed() {
        return Err("La aplicación no está instalada".into());
    }
    let app = app_dir()?;
    let unit = unit_path()?;
    let desktop = desktop_path()?;
    let executable = executable_path()?;
    let engine_file = app.join("quick_accent.py");
    let marker_file = app.join(".owned-by-quick-accent-local");

    ensure_project_file(&engine_file, "Quick Accent Local:")?;
    ensure_project_file(&unit, "Description=Quick Accent Local")?;
    ensure_project_file(&desktop, "X-QuickAccentLocal-Managed=true")?;
    if !marker_file.exists() {
        return Err("Falta la marca de propiedad: no eliminaré archivos que podrían ser de otro proyecto".into());
    }

    if engine() == ENGINE {
        checked("ibus", &["engine", "xkb:us::eng"])?;
    }
    systemctl(&["disable", "--now", SERVICE])?;
    fs::remove_file(unit).map_err(|e| e.to_string())?;
    systemctl(&["daemon-reload"])?;
    for path in [&engine_file, &desktop, &executable, &marker_file] {
        if path.exists() {
            fs::remove_file(path).map_err(|e| format!("No pude borrar {}: {e}", path.display()))?;
        }
    }
    // rmdir solo si quedó vacía: no borra datos de otros programas.
    let _ = fs::remove_dir(app);
    Ok("Desinstalado: archivos y arranque automático eliminados. El repositorio original sigue intacto.".into())
}
