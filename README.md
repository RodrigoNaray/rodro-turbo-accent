# Rodro Turbo Accent

**English** | [Español](README.es.md)

A Linux tool for typing **á, é, í, ó, ú and ñ** while keeping a US keyboard layout. Built with a Rust desktop interface and a Python IBus engine.

Hold a vowel or **N** for at least **250 ms**, then press **Space** without releasing the letter. Uppercase letters work too. For example, holding `a` and pressing Space produces `á`. Pressing Space earlier keeps the plain letter and the space.

## Motivation and status

A US keyboard is convenient for programming, but writing Spanish requires characters it does not provide directly. This personal project explores an interaction inspired by Microsoft PowerToys Quick Accent, using IBus on Linux with GNOME/Wayland, without switching the keyboard layout or reading raw input devices. It is independent and unaffiliated with Microsoft or PowerToys.

**Version 0.5.0 — functional MVP, experimental personal project under development.** The author has confirmed installation, accented input, startup activation and On/Off on Linux Mint with GNOME/Wayland and a US layout. This is not a general stability or compatibility guarantee.

## Current features

- Lowercase and uppercase accented vowels and `ñ` / `Ñ`.
- Fixed 250 ms hold threshold to reduce accidental accents between words.
- Resizable, scrollable GUI: installation, On, Off, status, logs and confirmed uninstallation.
- Per-user installation and startup through `systemd --user`. Closing the window keeps the engine running.
- Asynchronous IBus activation, bounded startup retries and coordinated shutdown.
- Ownership checks when updating or removing installed files.

The GUI is currently in Spanish. Only the README is bilingual.

## Technologies and requirements

| Component | Technology / requirement |
| --- | --- |
| Desktop interface | Rust, eframe/egui 0.31.1, OpenGL renderer |
| Input engine | Python 3, PyGObject (`gi`), IBus 1.0 introspection |
| Session | IBus, D-Bus, `systemd --user`, `journalctl`, `XDG_RUNTIME_DIR` |
| Confirmed desktop | Linux Mint, GNOME on Wayland, US keyboard layout |
| Build | Rust 1.89+, Cargo, C toolchain, pkg-config, windowing/OpenGL development libraries |
| Release scripts | Python 3.11+; Linux x86_64 packaging |

The application checks system requirements but does not install system packages. It requires no sudo, membership in the `input` group or direct access to `/dev/input`. A working IBus connection and a systemd user manager must already be available in the graphical session.

CI builds on Ubuntu 24.04 x86_64; compilation is not evidence of desktop behavior on every Ubuntu installation. Windows, macOS, other desktops and other architectures have not been validated. This repository contains no GNOME Shell extension, Flatpak or `.deb` package.

## Build and run

Install Rust through [rustup](https://rust-lang.org/tools/install/) if needed. On Mint/Ubuntu, the CI dependency list is a reference: `build-essential`, `pkg-config`, `libx11-dev`, `libxi-dev`, `libxcursor-dev`, `libxrandr-dev`, `libxkbcommon-dev`, `libwayland-dev`, `libgl1-mesa-dev`, `ibus`, `dbus-x11`, `python3-gi` and `gir1.2-ibus-1.0`. System package installation is managed separately by the administrator.

```bash
git clone https://github.com/RodrigoNaray/rodro-turbo-accent.git
cd rodro-turbo-accent
python3 scripts/build-release.py
./target/release/rodro-turbo-accent
```

Use `source "$HOME/.cargo/env"` if Cargo is not on your PATH. The build script also checks `~/.cargo/bin/cargo`. It accepts `--offline` when the locked dependencies are cached. For a development build, use `cargo run --locked`.

Opening the GUI displays status. Its management buttons modify the real per-user installation and service; running the GUI is not an isolated test.

## Install and use

1. Build and open the application. Alternatively, extract a CI artifact from **Actions → Compilar Linux**, verify its SHA-256 checksum and run `./rodro-turbo-accent`. Check that the artifact matches the intended source revision. Rust is not required to use a built binary.
2. Click **Instalar** (Install). It copies the executable, engine, launcher and user service, then enables and starts the engine. **Actualizar / Reinstalar** updates an existing installation.
3. **On** enables accented input and automatic startup. **Off** stops it and disables startup. Off restores US when this engine is selected; another selected engine is preserved.
4. Hold a supported letter for at least 250 ms and press Space. Release the letter before typing the next one. Space is consumed to produce the accent; press Space again if you need a word separator.
5. **Actualizar estado** refreshes service/IBus status. **Logs** shows the last 80 journal entries for the service. **Desinstalar** asks for confirmation, stops the service and removes application-owned files.

Closing the window leaves the service active. The GUI waits for a pending management operation before closing. Uninstallation does not remove your checkout, downloaded archive, extracted package or unrelated files.

The internal engine ID and service remain `quick-accent-local` for compatibility with the former name. Installation uses `~/.local/share/quick-accent-local/`, `~/.local/bin/quick-accent-local-ui`, `~/.config/systemd/user/quick-accent-local.service` and `~/.local/share/applications/quick-accent-local.desktop`.

## Known limitations and recovery

- The hold threshold is fixed, with no settings screen. Holding a supported letter suppresses its normal autorepeat.
- Input depends on IBus integration in the target application. Password/PIN detection relies on applications reporting their content type; it is not a guarantee for every sensitive field.
- For 30 seconds after engine startup, activation can replace US if GNOME selects it. Manually choosing US during that window can also be overridden; use **Off** to disable the engine. Another selected engine suspends retries.
- Registration is dynamic: `ibus list-engine` may omit `quick-accent-local` even while it works. Status distinguishes active registration from the persistent catalog.
- On can wait up to 35 seconds for selection confirmation. A running service alone does not prove accented input is active.

Project code does not transmit keystrokes over the network or log typed text. It keeps the US layout and does not edit GNOME's `sources` or `mru-sources`. See [security and privacy](SECURITY.md).

If needed, these recovery commands select US and disable the service:

```bash
ibus engine xkb:us::eng
systemctl --user disable --now quick-accent-local.service
```

## Tests and structure

```bash
python3 scripts/check-publication.py
python3 src/assets/quick_accent.py --self-test
python3 -m unittest discover -s tests -v
/usr/bin/python3 tests/ibus_smoke.py
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Rust tests use temporary HOME/runtime directories and mocked management commands. The IBus smoke test uses a private D-Bus and IBus with temporary directories; it does not change the real GNOME configuration or systemd service. It requires the system Python interpreter with GI and permission to create local sockets.

| Path | Purpose |
| --- | --- |
| `src/main.rs` | GUI and background management tasks |
| `src/linux.rs` | Linux installation, service and IBus controls |
| `src/assets/` | Embedded Python engine and systemd user unit |
| `tests/` | Python unit tests and isolated IBus integration |
| `scripts/` | Privacy checks, release build and packaging |
| `.github/workflows/` | Linux build and test workflow |
| `docs/` | Publication, licensing and historical review |

After the release build, `python3 scripts/package-release.py` writes an ignored archive and checksum in `dist/`. Build with `scripts/build-release.py` first: it remaps personal compiler paths, while packaging verifies the binary and normalizes archive ownership. Package metadata records the commit and uncommitted state. Review [distribution obligations](docs/LICENSING.md) before sharing binaries.

## Bugs and contributions

Use [GitHub Issues](https://github.com/RodrigoNaray/rodro-turbo-accent/issues) for reproducible bugs or suggestions. Include version, distribution, GNOME/session details, layout, reproduction steps, expected and actual behavior. Review logs before attaching them. Follow [SECURITY.md](SECURITY.md) for security issues.

Contributions are welcome: discuss changes in an issue, keep pull requests focused and run the checks above. Read [CONTRIBUTING.md](CONTRIBUTING.md); preserve the working engine and legacy installation identifiers.

## License, author and optional support

Copyright © 2026 Rodrigo Naray.

Original project code is licensed under **GNU GPL version 3 only (`GPL-3.0-only`)**, without an “or later” option. [LICENSE](LICENSE) contains the complete, unmodified text. Third-party dependencies and fonts retain their own licenses and copyrights; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and the [licensing review](docs/LICENSING.md).

Further reading: [publication checklist](docs/PUBLICATION.md), [distribution](docs/DISTRIBUTION.md), [changelog](CHANGELOG.md), [historical 0.4 review](docs/v0.4-review.md). Supporting documents are in English or Spanish as indicated by their content.
