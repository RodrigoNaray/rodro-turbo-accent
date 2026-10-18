#!/usr/bin/env python3
"""IBus real en D-Bus privado; HOME/runtime temporales. No usa systemd/GNOME.

Requiere ibus-daemon, dbus-run-session, /usr/bin/python3 y GI IBus.
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
ENGINE = ROOT / "src/assets/quick_accent.py"


def wait_for(check, timeout=8):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if check():
            return
        time.sleep(0.05)
    raise AssertionError("Tiempo agotado esperando IBus aislado")


def inside():
    def ibus(*args):
        return subprocess.run(["ibus", *args], capture_output=True, text=True, timeout=4)

    def current():
        return ibus("engine").stdout.strip()

    daemon = subprocess.Popen([
        "ibus-daemon", "--panel=disable",
        "--emoji-extension=disable", "--desktop=test", "--cache=none",
        "--address=" + os.environ["IBUS_ADDRESS"],
    ], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    process = None
    try:
        wait_for(lambda: ibus("list-engine", "--name-only").returncode == 0)
        import gi
        gi.require_version("IBus", "1.0")
        from gi.repository import IBus, GLib
        IBus.init()
        control = IBus.Bus()
        config = IBus.Config.new(control.get_connection())
        def configured():
            context = GLib.MainContext.default()
            while context.pending():
                context.iteration(False)
            config.set_value("general", "preload-engines", GLib.Variant("as", ["xkb:us::eng"]))
            value = config.get_value("general", "preload-engines")
            return value is not None and value.unpack() == ["xkb:us::eng"]
        wait_for(configured)
        # La CLI intenta ejecutar setxkbmap; este bus no tiene display.
        assert control.set_global_engine("xkb:us::eng")
        for cycle in range(5):
            process = subprocess.Popen(["/usr/bin/python3", str(ENGINE), "--auto-activate"],
                                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            wait_for(lambda: current() == "quick-accent-local")
            if cycle == 0:
                result = subprocess.run(["/usr/bin/python3", str(ENGINE), "--diagnose"],
                                        capture_output=True, text=True, timeout=4, check=True)
                assert "Registro activo IBus: True" in result.stdout, result.stdout
                assert "quick-accent-local" not in ibus("list-engine", "--name-only").stdout.splitlines()
                # Simular la restauración de US por GNOME durante el arranque.
                assert control.set_global_engine("xkb:us::eng")
                wait_for(lambda: current() == "quick-accent-local")
            process.terminate()
            output, _ = process.communicate(timeout=12)
            assert process.returncode == 0, output
            # Igual que el backend Rust: tras retirar el componente dinámico,
            # IBus puede invalidar el global. Confirmar/restaurar después de salir.
            for _ in range(3):
                time.sleep(0.15)
                selected = current()
                if selected in ("", "quick-accent-local"):
                    assert control.set_global_engine("xkb:us::eng")
                assert current() == "xkb:us::eng", (current(), output)
            print(f"Ciclo real On/Off {cycle + 1}: OK", flush=True)
            process = None
        print("IBus aislado: registro dinámico, activación, recuperación y parada OK", flush=True)
    finally:
        if process and process.poll() is None:
            process.kill()
            process.communicate()
        daemon.terminate()
        try:
            _, errors = daemon.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            daemon.kill()
            _, errors = daemon.communicate()
        if daemon.returncode not in (0, -15):
            print(errors, file=sys.stderr)


if __name__ == "__main__":
    if sys.argv[1:] == ["--inside"]:
        inside()
    else:
        with tempfile.TemporaryDirectory(prefix="quick-accent-test-") as folder:
            base = Path(folder)
            env = {key: value for key, value in os.environ.items()
                   if key not in ("DBUS_SESSION_BUS_ADDRESS", "DISPLAY", "WAYLAND_DISPLAY", "IBUS_ADDRESS")}
            for key, name in [("HOME", "home"), ("XDG_RUNTIME_DIR", "runtime"),
                              ("XDG_CONFIG_HOME", "config"), ("XDG_CACHE_HOME", "cache"),
                              ("XDG_DATA_HOME", "data")]:
                path = base / name
                path.mkdir(mode=0o700)
                env[key] = str(path)
            env["GIO_USE_VFS"] = "local"
            env["IBUS_ADDRESS"] = "unix:path=" + str(base / "ibus.sock")
            subprocess.run(["dbus-run-session", "--", "/usr/bin/python3", __file__, "--inside"],
                           env=env, check=True, timeout=65)
