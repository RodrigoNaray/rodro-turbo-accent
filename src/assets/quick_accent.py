#!/usr/bin/env python3
"""Quick Accent Local: motor IBus para acentos con letra mantenida + espacio.

Instalable por usuario: sin red, sin permisos elevados, sin acceso a /dev/input.
"""

from __future__ import annotations

import signal
import sys
import tempfile
from pathlib import Path
from xml.sax.saxutils import escape

ENGINE_NAME = "quick-accent-local"
DBUS_NAME = "org.freedesktop.IBus.QuickAccentLocal"
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

    def __init__(self):
        self.pending: tuple[str, int] | None = None

    def flush(self):
        if self.pending is None:
            return []
        char, _ = self.pending
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
            char, pending_keycode = self.pending
            if keyval == 0x20 and not is_modified:  # Espacio mientras la letra sigue pulsada
                self.pending = None
                return True, [("clear", ""), ("commit", ACCENTS[char])]
            if keycode == pending_keycode:  # repetición automática de la letra mantenida
                return True, []
            actions.extend(self.flush())

        if not is_modified and keyval < 128 and chr(keyval) in ACCENTS:
            char = chr(keyval)
            self.pending = (char, keycode)
            actions.append(("preedit", char))
            return True, actions

        return False, actions


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
  <description>Quick Accent Local experimental</description>
  <exec>{escape(sys.executable)} {escape(str(Path(__file__).resolve()))}</exec>
  <version>0.1</version>
  <author>Usuario local</author>
  <license>MIT</license>
  <homepage></homepage>
  <textdomain>quick-accent-local</textdomain>
  <engines>
    <engine>
      <name>{ENGINE_NAME}</name>
      <language>es</language>
      <license>MIT</license>
      <author>Usuario local</author>
      <icon>ibus-keyboard</icon>
      <layout>us</layout>
      <longname>Quick Accent Local (prueba)</longname>
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

    # El modo servicio intenta activar el motor una vez, después de que
    # GNOME haya tenido tiempo de inicializar su sistema de entrada.
    # No vuelve a forzar el motor si luego el usuario elige otro teclado.
    if "--auto-activate" in sys.argv[1:]:
        def activate_once():
            try:
                ok = bus.set_global_engine(ENGINE_NAME)
                print("Activación automática: " + ("OK" if ok else "falló"), flush=True)
            except Exception as exc:
                print(f"No se pudo activar automáticamente: {exc}", file=sys.stderr, flush=True)
            return GLib.SOURCE_REMOVE

        GLib.timeout_add_seconds(4, activate_once)
    else:
        print("Para activar: ibus engine quick-accent-local", flush=True)

    loop = GLib.MainLoop()
    GLib.unix_signal_add(GLib.PRIORITY_DEFAULT, signal.SIGINT, loop.quit)
    GLib.unix_signal_add(GLib.PRIORITY_DEFAULT, signal.SIGTERM, loop.quit)
    loop.run()


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        a = AccentState()
        assert a.event(ord("a"), 38, 0) == (True, [("preedit", "a")])
        assert a.event(ord(" "), 65, 0) == (True, [("clear", ""), ("commit", "á")])
        assert a.event(ord("a"), 38, a.RELEASE_MASK) == (False, [])
        b = AccentState()
        b.event(ord("n"), 57, 0)
        assert b.event(ord("n"), 57, b.RELEASE_MASK) == (True, [("clear", ""), ("commit", "n")])
        c = AccentState()
        c.event(ord("E"), 26, 1)
        assert c.event(32, 65, 0)[1][-1] == ("commit", "É")
        d = AccentState()
        d.event(ord("a"), 38, 0)
        assert d.event(ord("x"), 53, 0) == (False, [("clear", ""), ("commit", "a")])
        print("Pruebas de lógica: OK")
    elif sys.argv[1:] in ([], ["--auto-activate"]):
        run_ibus()
    else:
        print("Uso: quick_accent.py [--self-test | --auto-activate]", file=sys.stderr)
        sys.exit(2)
