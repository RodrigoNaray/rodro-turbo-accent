#!/usr/bin/env python3
"""Compila sin incluir rutas privadas del equipo en los diagnósticos del binario."""
import argparse
import os
from pathlib import Path
import shlex
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true", help="Usar dependencias ya descargadas")
    args = parser.parse_args()
    cargo = shutil.which("cargo")
    if not cargo:
        candidate = Path.home() / ".cargo/bin/cargo"
        if candidate.is_file():
            cargo = str(candidate)
    if not cargo:
        parser.error("No se encontró Cargo. Instalá Rust con rustup antes de compilar.")
    env = os.environ.copy()
    encoded = env.get("CARGO_ENCODED_RUSTFLAGS")
    flags = encoded.split("\x1f") if encoded else shlex.split(env.get("RUSTFLAGS", ""))
    flags += [
        f"--remap-path-prefix={Path.home()}=/build-user",
        f"--remap-path-prefix={ROOT}=.",
    ]
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    env.pop("RUSTFLAGS", None)
    command = [cargo, "build", "--release", "--locked"]
    if args.offline:
        command.append("--offline")
    subprocess.run(command, cwd=ROOT, env=env, check=True)


if __name__ == "__main__":
    main()
