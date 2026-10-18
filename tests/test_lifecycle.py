"""No importa GI ni accede a una sesión real."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("engine", Path(__file__).parents[1] / "src/assets/quick_accent.py")
engine = importlib.util.module_from_spec(spec)
spec.loader.exec_module(engine)


class Scheduler:
    def timeout_add_seconds(self, seconds, callback):
        return 1


class Bus:
    def __init__(self):
        self.current = "xkb:us::eng"
        self.queue = []
        self.selections = []
        self.reject = False

    def get_global_engine_async(self, timeout, cancellable, callback, data):
        self.queue.append(lambda: callback(self, None, data))

    def get_global_engine_async_finish(self, result):
        value = self.current
        return type("Desc", (), {"get_name": lambda self: value})()

    def set_global_engine_async(self, target, timeout, cancellable, callback, data):
        self.selections.append(target)
        def complete():
            if not self.reject:
                self.current = target
            callback(self, None, data)
        self.queue.append(complete)

    def set_global_engine_async_finish(self, result):
        return not self.reject

    def step(self):
        self.queue.pop(0)()

    def drain(self):
        for _ in range(20):
            if not self.queue:
                return
            self.step()
        raise AssertionError("Bucle inesperado")


class LifecycleTests(unittest.TestCase):
    def setUp(self):
        self.bus = Bus()
        self.now = 0
        self.quits = []
        self.life = engine.Lifecycle(self.bus, Scheduler(), lambda: self.quits.append(True), lambda: self.now)

    def activate(self):
        self.life.tick()
        self.bus.drain()
        self.life.tick()
        self.bus.drain()
        self.assertTrue(self.life.succeeded)

    def test_activation_does_not_block_factory(self):
        self.life.tick()
        self.assertTrue(self.life.busy)
        self.assertEqual(self.bus.current, "xkb:us::eng")
        self.bus.drain()
        self.assertEqual(self.bus.current, engine.ENGINE_NAME)

    def test_retries_after_gnome_restores_us_within_window(self):
        self.activate()
        self.bus.current = "xkb:us::eng"
        self.now = 20
        self.life.tick()
        self.bus.drain()
        self.assertEqual(self.bus.current, engine.ENGINE_NAME)

    def test_no_changes_after_startup_window(self):
        self.activate()
        self.now = 31
        self.bus.current = "xkb:us::eng"
        self.assertFalse(self.life.tick())
        self.assertEqual(self.bus.current, "xkb:us::eng")

    def test_deadline_during_query_prevents_late_activation(self):
        self.life.tick()
        self.now = 31
        self.bus.drain()
        self.assertEqual(self.bus.selections, [])

    def test_preserves_another_engine(self):
        self.bus.current = "xkb:latam::spa"
        self.life.tick()
        self.bus.drain()
        self.assertEqual(self.bus.selections, [])

    def test_stop_during_query_prevents_activation(self):
        self.life.tick()
        self.life.stop()
        self.bus.drain()
        self.assertEqual(self.bus.selections, [])
        self.assertEqual(len(self.quits), 1)

    def test_stop_during_selection_drains_then_restores(self):
        self.life.tick()
        self.bus.step()  # SetGlobalEngine en vuelo
        self.life.stop()
        self.assertEqual(self.quits, [])
        self.bus.drain()
        self.assertEqual(self.bus.selections, [engine.ENGINE_NAME, "xkb:us::eng"])
        self.assertEqual(self.bus.current, "xkb:us::eng")
        self.assertEqual(len(self.quits), 1)
        self.assertFalse(self.life.tick())

    def test_stop_after_watch_has_finished(self):
        self.activate()
        self.now = 31
        self.life.tick()
        self.life.stop()
        self.bus.drain()
        self.assertEqual(self.bus.current, "xkb:us::eng")
        self.assertTrue(self.quits)

    def test_rejected_activation_retries(self):
        self.bus.reject = True
        self.life.tick()
        self.bus.drain()
        self.assertFalse(self.life.failed)
        self.bus.reject = False
        self.activate()

    def test_rejected_stop_is_failure(self):
        self.bus.current = engine.ENGINE_NAME
        self.bus.reject = True
        self.life.stop()
        self.bus.drain()
        self.assertTrue(self.life.failed)
        self.assertTrue(self.quits)

    def test_repeated_stop_is_idempotent(self):
        self.activate()
        self.life.stop()
        self.life.stop()
        self.bus.drain()
        self.assertEqual(self.bus.selections.count("xkb:us::eng"), 1)

    def test_timeout_does_not_report_success(self):
        self.life.stop()
        self.life.stop_timeout()
        self.assertTrue(self.life.failed)


class AccentTests(unittest.TestCase):
    def setUp(self):
        self.now = 0.0

    def state(self):
        return engine.AccentState(clock=lambda: self.now)

    def test_all_accents_and_releases(self):
        for base, accent in engine.ACCENTS.items():
            self.now = 0.0
            state = self.state()
            state.event(ord(base), 38, 0)
            self.now = 0.250
            self.assertEqual(state.event(32, 65, 0)[1][-1], ("commit", accent))
            self.assertEqual(state.event(ord(base), 38, state.RELEASE_MASK), (False, []))

    def test_early_space_keeps_plain_letter_and_passes_space(self):
        for base in engine.ACCENTS:
            self.now = 0.0
            state = self.state()
            state.event(ord(base), 38, 0)
            self.now = 0.249
            self.assertEqual(state.event(32, 65, 0),
                             (False, [("clear", ""), ("commit", base)]))
            self.assertEqual(state.event(ord(base), 38, state.RELEASE_MASK), (False, []))

    def test_early_space_does_not_leave_a_late_accent(self):
        state = self.state()
        state.event(ord("a"), 38, 0)
        self.now = 0.050
        state.event(32, 65, 0)
        self.now = 0.500
        self.assertEqual(state.event(32, 65, 0), (False, []))
        self.assertEqual(state.flush(), [])

    def test_repeat_does_not_restart_hold_timer(self):
        state = self.state()
        state.event(ord("a"), 38, 0)
        self.now = 0.200
        self.assertEqual(state.event(ord("a"), 38, 0), (True, []))
        self.now = 0.250
        self.assertEqual(state.event(32, 65, 0)[1][-1], ("commit", "á"))

    def test_release_before_space_never_accents(self):
        state = self.state()
        state.event(ord("n"), 57, 0)
        self.now = 1.0
        self.assertEqual(state.event(ord("n"), 57, state.RELEASE_MASK)[1][-1], ("commit", "n"))
        self.assertEqual(state.event(32, 65, 0), (False, []))

    def test_next_letter_gets_its_own_hold_timer(self):
        state = self.state()
        state.event(ord("a"), 38, 0)
        self.now = 0.300
        state.event(ord("n"), 57, 0)
        self.now = 0.400
        self.assertEqual(state.event(32, 65, 0),
                         (False, [("clear", ""), ("commit", "n")]))

    def test_flush_commits_once_on_disable(self):
        state = engine.AccentState()
        state.event(ord("n"), 57, 0)
        self.assertEqual(state.flush(), [("clear", ""), ("commit", "n")])
        self.assertEqual(state.flush(), [])

    def test_modifiers_cancel_and_repeat(self):
        state = engine.AccentState()
        self.assertEqual(state.event(ord("a"), 38, state.CTRL_MASK), (False, []))
        state.event(ord("a"), 38, 0)
        self.assertEqual(state.event(ord("a"), 38, 0), (True, []))
        state.cancel()
        self.assertEqual(state.flush(), [])


if __name__ == "__main__":
    unittest.main()
