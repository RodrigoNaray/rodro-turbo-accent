#!/usr/bin/env python3
"""Rodro Turbo Accent: motor IBus para acentos con letra mantenida + espacio.

Instalable por usuario: sin red, sin permisos elevados, sin acceso a /dev/input.
"""

from __future__ import annotations

import signal
import sys
import tempfile
import time
from pathlib import Path
from xml.sax.saxutils import escape

ENGINE_NAME = "quick-accent-local"
DBUS_NAME = "org.freedesktop.IBus.QuickAccentLocal"
MIN_HOLD_SECONDS = 0.250
ACCENTS = {
    "a": "á", "e": "é", "i": "í", "o": "ó", "u": "ú", "n": "ñ",
    "A": "Á", "E": "É", "I": "Í", "O": "Ó", "U": "Ú", "N": "Ñ",
}


class AccentState:
    """Lógica sin IBus: facilita pruebas y evita acceder a otras teclas.

    Las acciones son ('preedit', str), ('commit', str) y ('clear', '').
    """

    RELEASE_MASK = 1 << 30
    CTRL_MASK = 1 << 2
    ALT_MASK = 1 << 3
    SUPER_MASK = 1 << 6

    def __init__(self, min_hold_seconds=MIN_HOLD_SECONDS, clock=time.monotonic):
        self.min_hold_seconds = min_hold_seconds
        self.clock = clock
        self.pending: tuple[str, int, float] | None = None

    def flush(self):
        if self.pending is None:
            return []
        char, _, _ = self.pending
        self.pending = None
        return [("clear", ""), ("commit", char)]

    def cancel(self):
        self.pending = None
        return [("clear", "")]

    def event(self, keyval: int, keycode: int, state: int):
        released = bool(state & self.RELEASE_MASK)
        actions = []
        if released:
            if self.pending is not None and self.pending[1] == keycode:
                return True, self.flush()
            return False, actions

        special_modifiers = self.CTRL_MASK | self.ALT_MASK | self.SUPER_MASK
        is_modified = bool(state & special_modifiers)

        if self.pending is not None:
            char, pending_keycode, pressed_at = self.pending
            if (keyval == 0x20 and not is_modified
                    and self.clock() - pressed_at >= self.min_hold_seconds):
                self.pending = None
                return True, [("clear", ""), ("commit", ACCENTS[char])]
            if keycode == pending_keycode:  # repetición automática de la letra mantenida
                return True, []
            actions.extend(self.flush())

        if not is_modified and keyval < 128 and chr(keyval) in ACCENTS:
            char = chr(keyval)
            self.pending = (char, keycode, self.clock())
            actions.append(("preedit", char))
            return True, actions

        return False, actions


class Lifecycle:
    """Una sola operación IBus en vuelo; nunca bloquear el bucle de la fábrica.

    Al parar se espera la respuesta pendiente antes de restaurar US: cancelar
    un cliente D-Bus no cancela necesariamente la operación en el servidor.
    """

    def __init__(self, bus, scheduler, quit_loop, clock=time.monotonic):
        self.bus, self.scheduler = bus, scheduler
        self.quit_loop, self.clock = quit_loop, clock
        self.deadline = clock() + 30
        self.stopping = False
        self.busy = False
        self.finished = False
        self.failed = False
        self.succeeded = False

    def start(self):
        self.scheduler.timeout_add_seconds(1, self.tick)

    def stop(self):
        if not self.stopping:
            self.stopping = True
            self.finished = False
            self.scheduler.timeout_add_seconds(10, self.stop_timeout)
            self.tick()
        return False

    def stop_timeout(self):
        if self.stopping and not self.finished:
            self.fail_stop("Tiempo agotado restaurando US")
        return False

    def fail_stop(self, message):
        print(message, file=sys.stderr, flush=True)
        self.failed = True
        self.finished = True
        self.quit_loop()

    def tick(self):
        if self.finished:
            return False
        if self.busy:
            return True
        if not self.stopping and self.clock() >= self.deadline:
            self.finished = True
            print("Fin de activación automática; selección confirmada: "
                  + str(self.succeeded) + ". Consultá Estado para el motor actual.", flush=True)
            return False
        self.busy = True
        self.bus.get_global_engine_async(2000, None, self.queried, None)
        return True

    def queried(self, bus, result, _data=None):
        try:
            desc = bus.get_global_engine_async_finish(result)
            current = desc.get_name() if desc else ""
        except Exception as exc:
            self.busy = False
            if self.stopping:
                self.fail_stop(f"No se pudo comprobar el motor al parar: {exc}")
            else:
                print(f"Esperando IBus: {exc}", flush=True)
            return
        if not self.stopping and self.clock() >= self.deadline:
            self.finished = True
            self.busy = False
            return
        if self.stopping:
            if current != ENGINE_NAME:
                self.finished = True
                self.busy = False
                self.quit_loop()
                return
            target = "xkb:us::eng"
        elif current == ENGINE_NAME:
            if not self.succeeded:
                print("Activación automática: selección verificada", flush=True)
            self.succeeded = True
            self.busy = False
            return
        elif current not in ("", "xkb:us::eng"):
            # No reemplazar otro teclado elegido en la sesión.
            print("Activación suspendida: hay otro motor seleccionado", flush=True)
            self.finished = True
            self.busy = False
            return
        else:
            target = ENGINE_NAME
        self.bus.set_global_engine_async(target, 2000, None, self.selected, None)

    def selected(self, bus, result, _data=None):
        try:
            if not bus.set_global_engine_async_finish(result):
                raise RuntimeError("IBus rechazó la selección")
        except Exception as exc:
            print(f"No se pudo cambiar el motor: {exc}", file=sys.stderr, flush=True)
            if self.stopping:
                self.fail_stop("No se confirmó la restauración de US")
        finally:
            self.busy = False
        # Incluso si Stop llegó durante SetGlobalEngine, verificar y restaurar
        # después de su respuesta, nunca en paralelo con esa selección.
        if self.stopping and not self.finished:
            self.tick()


def run_ibus():
    import gi

    gi.require_version("IBus", "1.0")
    from gi.repository import GLib, IBus

    class AccentEngine(IBus.Engine):
        __gtype_name__ = "QuickAccentLocalIBusEngine"

        def __init__(self, **kwargs):
            super().__init__(**kwargs)
            self.logic = AccentState()
            self.sensitive = False

        def _actions(self, actions):
            for kind, value in actions:
                if kind == "preedit":
                    self.update_preedit_text(IBus.Text.new_from_string(value), len(value), True)
                elif kind == "clear":
                    self.update_preedit_text(IBus.Text.new_from_string(""), 0, False)
                elif kind == "commit":
                    self.commit_text(IBus.Text.new_from_string(value))

        def do_process_key_event(self, keyval, keycode, state):
            if self.sensitive:
                return False
            handled, actions = self.logic.event(keyval, keycode, state)
            self._actions(actions)
            return handled

        def do_set_content_type(self, purpose, hints):
            # Protección de conveniencia, NO garantía: algunas apps no identifican
            # los campos sensibles ante IBus.
            self.sensitive = purpose in (IBus.InputPurpose.PASSWORD, IBus.InputPurpose.PIN)
            if self.sensitive:
                self._actions(self.logic.cancel())

        def do_focus_out(self):
            self._actions(self.logic.flush())

        def do_disable(self):
            self._actions(self.logic.flush())

        def do_reset(self):
            self._actions(self.logic.cancel())

    IBus.init()
    bus = IBus.Bus()
    if not bus.is_connected():
        raise RuntimeError("No se pudo conectar a IBus")

    factory = IBus.Factory.new(bus.get_connection())
    factory.add_engine(ENGINE_NAME, AccentEngine.__gtype__)

    # Registro temporal: descriptor XML en directorio temporal, NO en /usr.
    xml = f"""<?xml version="1.0" encoding="UTF-8"?>
<component>
  <name>{DBUS_NAME}</name>
  <description>Rodro Turbo Accent experimental</description>
  <exec>{escape(sys.executable)} {escape(str(Path(__file__).resolve()))}</exec>
  <version>0.5.0</version>
  <author>Usuario local</author>
  <homepage></homepage>
  <textdomain>quick-accent-local</textdomain>
  <engines>
    <engine>
      <name>{ENGINE_NAME}</name>
      <language>es</language>
      <author>Usuario local</author>
      <icon>ibus-keyboard</icon>
      <layout>us</layout>
      <longname>Rodro Turbo Accent</longname>
      <description>Vocal/N + espacio, manteniendo pulsada la letra</description>
      <rank>1</rank>
    </engine>
  </engines>
</component>"""

    with tempfile.TemporaryDirectory(prefix="quick-accent-ibus-") as folder:
        xml_path = Path(folder) / "component.xml"
        xml_path.write_text(xml, encoding="utf-8")
        component = IBus.Component.new_from_file(str(xml_path))
        if component is None or not bus.register_component(component):
            raise RuntimeError("IBus no pudo registrar el motor local")

    print("Motor registrado en IBus: quick-accent-local", flush=True)

    loop = GLib.MainLoop()
    lifecycle = Lifecycle(bus, GLib, loop.quit)
    if "--auto-activate" in sys.argv[1:]:
        lifecycle.start()
    else:
        lifecycle.finished = True
        print("Para activar: ibus engine quick-accent-local", flush=True)

    GLib.unix_signal_add(GLib.PRIORITY_DEFAULT, signal.SIGINT, lifecycle.stop)
    GLib.unix_signal_add(GLib.PRIORITY_DEFAULT, signal.SIGTERM, lifecycle.stop)
    # Tras perder la conexión, salir con error permite que systemd registre
    # de nuevo el componente cuando IBus esté disponible.
    disconnected = []
    def on_disconnect(_bus):
        disconnected.append(True)
        loop.quit()
    bus.connect("disconnected", on_disconnect)
    loop.run()
    if lifecycle.failed or disconnected:
        raise RuntimeError("El motor terminó sin una transición IBus confirmada")


def diagnose():
    import gi
    gi.require_version("IBus", "1.0")
    from gi.repository import IBus
    IBus.init()
    bus = IBus.Bus()
    if not bus.is_connected():
        raise RuntimeError("IBus no está conectado")
    dynamic = any(e.get_name() == ENGINE_NAME for e in bus.list_active_engines())
    catalog = any(e.get_name() == ENGINE_NAME for e in bus.list_engines())
    print(f"Registro activo IBus: {dynamic} · Catálogo persistente: {catalog}")


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        now = [0.0]
        a = AccentState(clock=lambda: now[0])
        assert a.event(ord("a"), 38, 0) == (True, [("preedit", "a")])
        now[0] = MIN_HOLD_SECONDS
        assert a.event(ord(" "), 65, 0) == (True, [("clear", ""), ("commit", "á")])
        assert a.event(ord("a"), 38, a.RELEASE_MASK) == (False, [])
        b = AccentState()
        b.event(ord("n"), 57, 0)
        assert b.event(ord("n"), 57, b.RELEASE_MASK) == (True, [("clear", ""), ("commit", "n")])
        c = AccentState(clock=lambda: now[0])
        c.event(ord("E"), 26, 1)
        now[0] += MIN_HOLD_SECONDS
        assert c.event(32, 65, 0)[1][-1] == ("commit", "É")
        d = AccentState()
        d.event(ord("a"), 38, 0)
        assert d.event(ord("x"), 53, 0) == (False, [("clear", ""), ("commit", "a")])
        print("Pruebas de lógica: OK")
    elif sys.argv[1:] == ["--diagnose"]:
        diagnose()
    elif sys.argv[1:] in ([], ["--auto-activate"]):
        run_ibus()
    else:
        print("Uso: quick_accent.py [--self-test | --auto-activate | --diagnose]", file=sys.stderr)
        sys.exit(2)
