//! Integración con GNOME/Wayland: usa IBus y systemd de *usuario*.
//! No usa sudo, no toca /usr, no modifica permisos de /dev/input.

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("No pude ejecutar {program}: {e}"))?;
    // Drenar ambos pipes durante la espera, incluso si journalctl produce mucho texto.
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = thread::spawn(move || {
        let mut b = Vec::new();
        stdout.read_to_end(&mut b).map(|_| b)
    });
    let err = thread::spawn(move || {
        let mut b = Vec::new();
        stderr.read_to_end(&mut b).map(|_| b)
    });
    let timeout = Duration::from_secs(if program == "systemctl" { 20 } else { 5 });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(30)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{program} {} no terminó a tiempo: {result:?}",
                    args.join(" ")
                ));
            }
        }
    };
    Ok(Output {
        status,
        stdout: out
            .join()
            .map_err(|_| "Error leyendo stdout")?
            .map_err(|e| e.to_string())?,
        stderr: err
            .join()
            .map_err(|_| "Error leyendo stderr")?
            .map_err(|e| e.to_string())?,
    })
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

fn operation_lock() -> Result<File, String> {
    let runtime = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .ok_or("Falta XDG_RUNTIME_DIR; abrí la aplicación en tu sesión gráfica")?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(runtime.join("quick-accent-local.lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock()
        .map_err(|e| format!("Hay otra operación de Quick Accent en curso: {e}"))?;
    Ok(file)
}

fn selected_engine() -> Result<String, String> {
    checked("ibus", &["engine"])
}

fn restore_us() -> Result<(), String> {
    if selected_engine()? == ENGINE {
        checked("ibus", &["engine", "xkb:us::eng"])?;
        if selected_engine()? != "xkb:us::eng" {
            return Err("IBus no confirmó el retorno a US; el servicio sigue disponible".into());
        }
    }
    Ok(())
}

// Usado por Off, actualización y desinstalación. No ignorar una parada fallida.
fn stop_service() -> Result<(), String> {
    restore_us()?;
    systemctl(&["disable", "--now", SERVICE])?;
    // IBus puede invalidar la selección al retirar el componente dinámico.
    // Esperar confirmaciones después de salir, recuperando US si quedó vacío.
    for _ in 0..3 {
        thread::sleep(Duration::from_millis(150));
        let current = selected_engine().unwrap_or_default();
        if current.is_empty() || current == ENGINE {
            checked("ibus", &["engine", "xkb:us::eng"])?;
            if selected_engine()? != "xkb:us::eng" {
                return Err("No se confirmó US después de detener el servicio".into());
            }
        }
    }
    let active = output_ignoring_failure("systemctl", &["--user", "is-active", SERVICE]);
    if active != "inactive" && active != "failed" {
        return Err(format!("El servicio no confirmó la parada: {active}"));
    }
    Ok(())
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
    let registration = if installed {
        let script = app_dir()?.join("quick_accent.py");
        output_ignoring_failure(
            "/usr/bin/python3",
            &[script.to_str().ok_or("Ruta no UTF-8")?, "--diagnose"],
        )
    } else {
        "Sin registro instalado".into()
    };
    let mode = if current == ENGINE && active == "active" {
        "ON — acentos activos"
    } else if installed && active == "active" {
        "Servicio activo; motor no seleccionado en IBus"
    } else if installed && enabled == "enabled" {
        "Instalado; aún no seleccionado en IBus"
    } else if installed {
        "OFF — acentos desactivados"
    } else {
        "Sin instalar"
    };
    Ok(format!(
        "{mode}\nServicio: {active}  ·  Inicio automático: {enabled}\nMotor IBus: {current}\n{registration}"
    ))
}

fn check_requirements() -> Result<(), String> {
    for (cmd, argument) in [
        ("python3", "--version"),
        ("ibus", "version"),
        ("systemctl", "--version"),
    ] {
        checked(cmd, &[argument])?;
    }
    checked(
        "/usr/bin/python3",
        &[
            "-c",
            "import gi; gi.require_version('IBus', '1.0'); from gi.repository import IBus, GLib",
        ],
    )?;
    Ok(())
}

fn desktop_exec_escape(path: &Path) -> String {
    // La especificación .desktop requiere comillas y escapes para las rutas con espacios.
    let escaped = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub fn install() -> Result<String, String> {
    let _lock = operation_lock()?;
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
    if executable.exists() && read(&marker_file) != OWNED_MARKER {
        return Err(format!(
            "Ya existe {} y no hay marca de propiedad del proyecto. No lo sobrescribí.",
            executable.display()
        ));
    }

    fs::create_dir_all(&app).map_err(|e| e.to_string())?;
    fs::create_dir_all(unit.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::create_dir_all(desktop.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::create_dir_all(executable.parent().unwrap()).map_err(|e| e.to_string())?;

    // Al actualizar, detenemos la versión anterior antes de reemplazar el motor.
    if unit.exists() {
        stop_service()?;
    }

    fs::write(&engine_file, ENGINE_SOURCE).map_err(|e| e.to_string())?;
    fs::set_permissions(&engine_file, fs::Permissions::from_mode(0o700))
        .map_err(|e| e.to_string())?;

    let service_text = include_str!("assets/quick-accent-local.service");
    fs::write(&unit, service_text).map_err(|e| e.to_string())?;

    let current = env::current_exe().map_err(|e| e.to_string())?;
    if current != executable {
        // Copia temporal + rename: reemplaza una versión anterior aunque esté abierta.
        let temporary = executable.with_extension("tmp-quick-accent-local");
        fs::copy(&current, &temporary).map_err(|e| format!("No pude copiar la aplicación: {e}"))?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
        fs::rename(&temporary, &executable)
            .map_err(|e| format!("No pude instalar el ejecutable: {e}"))?;
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
    let on_message = turn_on_locked()?;
    Ok(format!(
        "Instalado solo para tu usuario. Aparecerá en el menú de aplicaciones. {on_message}"
    ))
}

pub fn turn_on() -> Result<String, String> {
    let _lock = operation_lock()?;
    turn_on_locked()
}

fn turn_on_locked() -> Result<String, String> {
    if !is_installed() {
        return Err("Primero presioná Instalar".into());
    }
    let was_active =
        output_ignoring_failure("systemctl", &["--user", "is-active", SERVICE]) == "active";
    systemctl(&["enable", "--now", SERVICE])?;
    // El servicio es el único que activa. La GUI solo comprueba la selección.
    // Si ya estaba activo pero se eligió US, un reinicio inicia una nueva activación.
    if was_active && selected_engine().unwrap_or_default() != ENGINE {
        systemctl(&["restart", SERVICE])?;
    }
    let deadline = Instant::now() + Duration::from_secs(35);
    let mut confirmations = 0;
    while Instant::now() < deadline {
        if selected_engine().unwrap_or_default() == ENGINE {
            confirmations += 1;
            if confirmations >= 3 {
                return Ok(
                    "Acentos ON: selección IBus verificada. Inicio automático habilitado.".into(),
                );
            }
        } else {
            confirmations = 0;
        }
        thread::sleep(Duration::from_millis(350));
    }
    Err("El servicio arrancó pero no se confirmó el motor en IBus. Revisá Estado y Logs; no se modificó GNOME.".into())
}

pub fn turn_off() -> Result<String, String> {
    let _lock = operation_lock()?;
    if !is_installed() {
        return Err("La aplicación no está instalada".into());
    }
    stop_service()?;
    Ok("Acentos OFF. Parada verificada; inicio automático deshabilitado.".into())
}

pub fn logs() -> Result<String, String> {
    checked(
        "journalctl",
        &["--user", "--unit", SERVICE, "--no-pager", "-n", "80"],
    )
}

pub fn uninstall() -> Result<String, String> {
    let _lock = operation_lock()?;
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
    if read(&marker_file) != OWNED_MARKER {
        return Err(
            "Falta la marca de propiedad: no eliminaré archivos que podrían ser de otro proyecto"
                .into(),
        );
    }

    stop_service()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    // Un único test modifica el entorno del proceso. Todos los comandos de
    // gestión son dobles locales y HOME/runtime apuntan a un directorio temporal.
    #[test]
    fn isolated_install_transitions_and_uninstall() {
        let root = env::temp_dir().join(format!("quick-accent-rust-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let bin = root.join("bin");
        fs::create_dir(&bin).unwrap();
        let previous_home = env::var_os("HOME");
        let previous_runtime = env::var_os("XDG_RUNTIME_DIR");
        let previous_path = env::var_os("PATH").unwrap();
        let systemctl_mock = r#"#!/bin/sh
if [ "$1" = --version ]; then exit 0; fi
[ "$1" = --user ] || exit 99
shift
printf '%s\n' "$*" >> "$HOME/calls"
case "$1" in
 is-active) cat "$HOME/active" ;;
 is-enabled) cat "$HOME/enabled" ;;
 enable|restart)
  printf active > "$HOME/active"
  printf enabled > "$HOME/enabled"
  printf quick-accent-local > "$HOME/engine"
  ;;
 disable)
  [ ! -e "$HOME/fail-stop" ] || exit 1
  printf inactive > "$HOME/active"
  printf disabled > "$HOME/enabled"
  if [ -e "$HOME/clear-after-stop" ]; then printf "" > "$HOME/engine"; fi
  ;;
 daemon-reload) exit 0 ;;
 *) exit 98 ;;
esac
"#;
        let ibus_mock = r#"#!/bin/sh
case "$1" in
 version) exit 0 ;;
 engine)
  if [ "$#" = 1 ]; then cat "$HOME/engine"; else
   [ ! -e "$HOME/fail-select" ] || exit 1
   printf '%s' "$2" > "$HOME/engine"
  fi ;;
 *) exit 97 ;;
esac
"#;
        for (name, source) in [("systemctl", systemctl_mock), ("ibus", ibus_mock)] {
            let path = bin.join(name);
            fs::write(&path, source).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        env::set_var("HOME", &root);
        env::set_var("XDG_RUNTIME_DIR", &root);
        let mut paths = vec![bin];
        paths.extend(env::split_paths(&previous_path));
        env::set_var("PATH", env::join_paths(paths).unwrap());
        fs::write(root.join("active"), "inactive").unwrap();
        fs::write(root.join("enabled"), "disabled").unwrap();
        fs::write(root.join("engine"), "xkb:us::eng").unwrap();

        install().unwrap();
        assert!(is_installed());
        assert_eq!(engine(), ENGINE);
        let unit = read(&unit_path().unwrap());
        assert!(unit.contains("WantedBy=graphical-session.target"));
        assert!(unit.contains("PartOf=graphical-session.target"));
        assert!(!unit.contains("WantedBy=default.target"));

        let lock = operation_lock().unwrap();
        assert!(turn_off().unwrap_err().contains("otra operación"));
        drop(lock);

        fs::write(root.join("fail-select"), "").unwrap();
        assert!(turn_off().is_err());
        assert_eq!(read(&root.join("active")), "active");
        fs::remove_file(root.join("fail-select")).unwrap();
        fs::write(root.join("fail-stop"), "").unwrap();
        assert!(uninstall().is_err());
        assert!(is_installed());
        fs::remove_file(root.join("fail-stop")).unwrap();

        fs::write(root.join("clear-after-stop"), "").unwrap();
        turn_off().unwrap();
        fs::remove_file(root.join("clear-after-stop")).unwrap();
        assert_eq!(engine(), "xkb:us::eng");
        assert_eq!(read(&root.join("active")), "inactive");
        turn_on().unwrap();
        assert_eq!(engine(), ENGINE);
        // Si el usuario eligió otro motor, Off debe conservarlo.
        fs::write(root.join("engine"), "xkb:latam::spa").unwrap();
        turn_off().unwrap();
        assert_eq!(engine(), "xkb:latam::spa");

        let extra = app_dir().unwrap().join("archivo-del-usuario");
        fs::write(&extra, "conservar").unwrap();
        uninstall().unwrap();
        assert!(!is_installed());
        assert!(!executable_path().unwrap().exists());
        assert!(!desktop_path().unwrap().exists());
        assert_eq!(read(&extra), "conservar");
        for (key, value) in [
            ("HOME", previous_home),
            ("XDG_RUNTIME_DIR", previous_runtime),
        ] {
            if let Some(value) = value {
                env::set_var(key, value);
            } else {
                env::remove_var(key);
            }
        }
        env::set_var("PATH", previous_path);
        fs::remove_dir_all(root).unwrap();
    }
}
