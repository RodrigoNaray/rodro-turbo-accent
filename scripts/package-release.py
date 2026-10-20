#!/usr/bin/env python3
# Copyright © 2026 Rodrigo Naray
# SPDX-License-Identifier: GPL-3.0-only

"""Empaqueta la versión Linux x86_64 sin nombres de usuario ni rutas de la máquina."""
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT).decode().strip()


def main():
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    version = manifest["package"]["version"]
    binary = ROOT / "target/release/rodro-turbo-accent"
    data = binary.read_bytes()
    if data[:4] != b"\x7fELF" or data[4:6] != b"\x02\x01" or data[18:20] != b"\x3e\x00":
        raise SystemExit("Se requiere un ejecutable ELF Linux x86_64 compilado previamente.")
    if bytes(Path.home()) in data or bytes(ROOT) in data:
        raise SystemExit("El binario contiene rutas privadas. Compilá con scripts/build-release.py.")
    epoch = int(os.environ.get("SOURCE_DATE_EPOCH", git("show", "-s", "--format=%ct", "HEAD")))
    metadata = json.dumps({
        "name": "Rodro Turbo Accent",
        "version": version,
        "license": "GPL-3.0-only",
        "copyright": "Copyright © 2026 Rodrigo Naray",
        "source_commit": git("rev-parse", "HEAD"),
        "uncommitted_changes": bool(git("status", "--porcelain")),
        "platform": "linux-x86_64",
        "paths_remapped": True,
    }, indent=2).encode() + b"\n"
    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)
    destination = dist / f"rodro-turbo-accent-{version}-linux-x86_64.tar.gz"

    def normalize(info):
        info.uid = info.gid = 0
        info.uname = info.gname = ""
        info.mtime = epoch
        info.mode = 0o755 if info.name == "rodro-turbo-accent" else 0o644
        info.pax_headers = {}
        return info

    files = {"rodro-turbo-accent": binary}
    for name in (
        "README.md", "README.es.md", "LICENSE", "COPYRIGHT",
        "THIRD_PARTY_NOTICES.md", "THIRD_PARTY_LICENSES.txt",
        "CONTRIBUTING.md", "SECURITY.md", "CHANGELOG.md",
    ):
        files[name] = ROOT / name
    files.update({str(path.relative_to(ROOT)): path for path in sorted((ROOT / "docs").rglob("*.md"))})
    with destination.open("wb") as output:
        with gzip.GzipFile(filename="", fileobj=output, mode="wb", mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as archive:
                for name, path in files.items():
                    archive.add(path, arcname=name, filter=normalize)
                info = normalize(tarfile.TarInfo("BUILD-INFO.json"))
                info.size = len(metadata)
                archive.addfile(info, io.BytesIO(metadata))
    # Confirmar contenido y permisos sin extraer ni ejecutar el instalador.
    with tarfile.open(destination) as archive:
        assert set(archive.getnames()) == {*files, "BUILD-INFO.json"}
        assert archive.getmember("rodro-turbo-accent").mode == 0o755
        for member in archive.getmembers():
            assert member.uid == member.gid == 0 and not member.uname and not member.gname
        for name, path in files.items():
            assert archive.extractfile(name).read() == path.read_bytes()
    digest = hashlib.sha256(destination.read_bytes()).hexdigest()
    destination.with_suffix(destination.suffix + ".sha256").write_text(f"{digest}  {destination.name}\n")
    print(f"Paquete verificado: {destination.name}")


if __name__ == "__main__":
    main()
