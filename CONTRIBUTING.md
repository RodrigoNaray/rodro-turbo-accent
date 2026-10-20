# Contributing to Rodro Turbo Accent

Copyright © 2026 Rodrigo Naray. SPDX-License-Identifier: GPL-3.0-only.

This is a functional, experimental personal MVP. Read [README.md](README.md) or [README.es.md](README.es.md) for its supported behavior and known limitations.

## Report a bug or propose a change

Open an [issue](https://github.com/RodrigoNaray/rodro-turbo-accent/issues) with the application version, distribution, GNOME version/session type, US layout details, reproduction steps, expected result and actual result. Include reviewed logs only when useful; do not attach private text, credentials or personal filesystem paths. For security issues follow [SECURITY.md](SECURITY.md).

Discuss changes affecting input behavior before implementing them. Keep pull requests focused and explain what changed, why and how it was verified. Do not announce unsupported platforms or unmeasured performance.

## Development checks

Follow the build and test commands in the README. Rust tests use temporary HOME/runtime directories and mocked commands. The IBus smoke test uses a private D-Bus/IBus. Automated tests must not operate on the real desktop's services or GNOME configuration. Explain the effects and obtain consent before a manual installation or input-engine test.

Preserve the separation between Rust GUI and Linux backend. Internal `quick-accent-local` identifiers exist for upgrade compatibility; changing them requires an independently designed and tested migration. A Windows backend is not implemented.

Keep README.md and README.es.md equivalent when updating behavior or instructions. Dependency or embedded-resource changes also require refreshing the license review and third-party notices.

## Privacy and licensing

Use a GitHub noreply address for commits if you do not want to expose your email. This repository's publication check currently accepts only GitHub noreply author/committer identities in HEAD and its ancestors, so configure it before opening a pull request:

```bash
git config --local user.email 103074621+RodrigoNaray@users.noreply.github.com
```

That privacy check is a repository contribution process, not an extra condition on use, modification or redistribution of the GPL software.

Original contributions are supplied under GPL-3.0-only, the project's existing license. Retain existing third-party copyrights and license notices. Do not add code/resources whose terms conflict with the project; explain provenance and licensing of any material you introduce. There is no CLA, payment requirement or additional usage restriction.

Do not commit private `.env` files, keys, Git backups, local executables or build artifacts. Keep Cargo.lock in version control. Release builds should use the existing path-remapping and packaging scripts; binary distribution additionally needs the Corresponding Source described in [docs/LICENSING.md](docs/LICENSING.md).
