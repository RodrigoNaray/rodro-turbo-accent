"""La protección de publicación se comprueba en un repositorio Git temporal."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class PublicationTests(unittest.TestCase):
    def test_private_email_and_removed_credentials_are_rejected_without_printing_values(self):
        with tempfile.TemporaryDirectory(prefix="rodro-publication-test-") as folder:
            base = Path(folder)
            (base / "scripts").mkdir()
            shutil.copy(ROOT / "scripts/check-publication.py", base / "scripts/check-publication.py")
            shutil.copy(ROOT / ".gitignore", base / ".gitignore")
            (base / "README.md").write_text("Public example\n")
            env = os.environ.copy()
            env["GIT_CONFIG_GLOBAL"] = os.devnull
            env["GIT_CONFIG_NOSYSTEM"] = "1"
            for key in ("GIT_AUTHOR_EMAIL", "GIT_COMMITTER_EMAIL", "GIT_AUTHOR_NAME", "GIT_COMMITTER_NAME"):
                env.pop(key, None)

            def git(*args):
                return subprocess.run(["git", "-c", "core.hooksPath=" + os.devnull, *args],
                                      cwd=base, env=env, check=True, capture_output=True)

            def check():
                return subprocess.run([sys.executable, "scripts/check-publication.py"],
                                      cwd=base, env=env, capture_output=True, text=True, timeout=10)

            git("init", "--quiet", "--initial-branch=main")
            git("config", "user.name", "Privacy check example")
            git("config", "user.email", "author@example.invalid")
            git("add", ".")
            git("commit", "--quiet", "-m", "Initial example")
            result = check()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("correo", result.stderr)
            self.assertNotIn("author@example.invalid", result.stderr)

            git("config", "user.email", "1+example@users.noreply.github.com")
            git("commit", "--quiet", "--amend", "--no-edit", "--reset-author")
            self.assertEqual(check().returncode, 0)
            # Un secreto ignorado no se publicaría; si se fuerza su inclusión, debe bloquearse.
            fake_token = "ghp_" + "A" * 36
            (base / ".env").write_text("API_KEY=" + fake_token + "\n")
            self.assertEqual(check().returncode, 0)
            git("add", "--force", ".env")
            result = check()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("github-token", result.stderr)
            self.assertNotIn(fake_token, result.stderr)
            git("commit", "--quiet", "-m", "Synthetic leaked credential")
            git("rm", "--quiet", ".env")
            git("commit", "--quiet", "-m", "Remove synthetic credential")
            result = check()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("historial", result.stderr)
            self.assertNotIn(fake_token, result.stderr)


if __name__ == "__main__":
    unittest.main()
