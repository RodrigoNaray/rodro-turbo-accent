#!/usr/bin/env python3
# Copyright © 2026 Rodrigo Naray
# SPDX-License-Identifier: GPL-3.0-only

"""Comprobación local básica de privacidad: autoría, archivos y patrones de secretos.

No imprime valores detectados. Revisa HEAD y sus ancestros; no publica ni cambia Git.
"""
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SECRET_RULES = {
    "private-key": re.compile(r"-----BEGIN (?:RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY(?: BLOCK)?-----"),
    "github-token": re.compile(r"\b(?:gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{40,})\b"),
    "aws-key": re.compile(r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b"),
    "google-key": re.compile(r"\bAIza[A-Za-z0-9_-]{30,}\b"),
    "slack-token": re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{12,}"),
    "openai-key": re.compile(r"\bsk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{20,}"),
    "jwt": re.compile(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}"),
    "credential-url": re.compile(r"\b[a-z][a-z0-9+.-]*://[^\s/@:]+:[^\s/@]+@"),
    "credential-literal": re.compile(r'''(?i)\b(?:api[_-]?key|access[_-]?token|client[_-]?secret|password|passwd)\b\s*[:=]\s*["'][^\s"'\n,;]{8,}["']'''),
    "personal-path": re.compile(r"(?:/home|/Users)/[A-Za-z0-9_.-]+/"),
}
PRIVATE_NAME = re.compile(r"(?i)(?:^\.env(?:\.|$)|\.(?:pem|key|p12|pfx|keystore|bundle)$|^(?:id_rsa|id_ed25519|credentials\.json|\.netrc|\.npmrc)$)")


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def scan(label, data, errors):
    text = data.decode("utf-8", errors="replace")
    for name, pattern in SECRET_RULES.items():
        for match in pattern.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            errors.add(f"{label}:{line}: {name} (valor oculto)")


def is_private_path(name):
    path = Path(name)
    return (bool(PRIVATE_NAME.search(path.name)) and path.name != ".env.example"
            or any(part in (".aws", ".ssh", ".secrets", ".private") for part in path.parts))


def main():
    errors = set()
    commits = git("rev-list", "HEAD").decode().splitlines()
    for commit in commits:
        identities = git("show", "-s", "--format=%ae%n%ce", commit).decode().splitlines()
        if any(not (email.endswith("@users.noreply.github.com") or email == "noreply@github.com") for email in identities):
            errors.add(f"commit {commit[:12]}: correo de autor o committer no anónimo (valor oculto)")
    files = set(git("ls-files", "--cached", "--others", "--exclude-standard", "-z").decode().split("\0")) - {""}
    for name in files:
        if is_private_path(name):
            errors.add(f"archivo privado: {name}")
        path = ROOT / name
        if path.is_file():
            scan(name, path.read_bytes(), errors)
    blobs = 0
    for line in git("rev-list", "--objects", "HEAD").decode().splitlines():
        oid, _, name = line.partition(" ")
        if git("cat-file", "-t", oid).strip() != b"blob":
            continue
        blobs += 1
        if is_private_path(name):
            errors.add(f"archivo privado en historial: {oid[:12]}:{name}")
        scan(f"historial {oid[:12]}:{name}", git("cat-file", "blob", oid), errors)
    if errors:
        print("Revisar antes de publicar:", file=sys.stderr)
        for error in sorted(errors):
            print(f"- {error}", file=sys.stderr)
        return 1
    print(f"Comprobación básica OK: {len(files)} archivos, {len(commits)} commits, {blobs} blobs de HEAD.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
