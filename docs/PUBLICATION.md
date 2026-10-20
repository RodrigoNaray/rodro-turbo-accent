# Public repository preparation

Copyright © 2026 Rodrigo Naray. SPDX-License-Identifier: GPL-3.0-only.

## Identity and proposed GitHub metadata

Repository: `RodrigoNaray/rodro-turbo-accent`. Product: **Rodro Turbo Accent**. This is a functional, experimental personal MVP for typing accented characters with a US keyboard on Linux. Its Rust GUI manages a Python IBus engine and a systemd user service.

Suggested About description:

> Type accented characters with a US keyboard on Linux. Rust/egui GUI, Python IBus engine and per-user service.

Suggested topics: `rust`, `linux`, `ibus`, `wayland`, `gnome`, `keyboard`, `diacritics`, `spanish`, `egui`, `python`, `systemd`, `gplv3`.

These suggestions have not been applied to GitHub.

## Local security findings

The review of the starting revision `afea7c6` covered 21 tracked files, seven commits reachable from main, all local refs, and all 122 Git objects (64 blobs, 46 trees, 12 commits), including unreachable objects. The basic scanner found no credential patterns or personal filesystem paths in main or its reachable history. Its authors and committers use the owner's GitHub noreply address. Current and historical tracked filenames were checked for private files; generated binaries, caches and temporary files are ignored.

**Private local remnants remain:** five unreachable commits retain the previously replaced institutional email, and two unreachable blobs contain personal filesystem paths. They do not belong to main, origin/main or another current ref. No values are reproduced here. The finding was reported before continuing preparation. Do not publish `.git`, private backups or reflogs, and do not use `push --mirror`. No history rewrite, reflog expiry or garbage collection was performed for this preparation.

The earlier history rewrite and remote update were separate, already authorized operations. The previous statement that origin/main still contained the old history was obsolete and has been removed. A private backup of the old history exists outside the repository and must remain private. Local cleanup, if desired, should be planned separately with an independently verified backup; it is unnecessary for publishing only the reviewed branch.

Pattern scanning is a limited check, not proof that no secret exists. Third-party copyright notices intentionally preserve upstream author information. The project's public author name and GitHub identity are deliberate.

This review does not inspect GitHub's old commit caches, pull requests, logs, artifacts, secrets, attachments or other remote branches not available locally. Changing a branch does not erase copies retained elsewhere. Revoke any real credential if exposure is later found.

## Manual checklist before changing visibility

1. Review the local diff, GPL-3.0-only choice, copyright and third-party notices. Keep the complete GNU license text unchanged.
2. Run `python3 scripts/check-publication.py` and the README checks on the exact final revision. Review and commit with the configured noreply identity.
3. Review the hosted repository's branches/tags, old PRs, cached links, Actions logs/artifacts and attachments for the previously exposed email or other private data. If removal is needed on GitHub, plan it separately; no remote cleanup is performed here.
4. Enable Issues for bug reports and check GitHub email privacy, secret scanning/push protection availability and private vulnerability reporting. Never share tokens or private logs in issues.
5. Confirm the default branch and hosted files match the reviewed local revision when you explicitly choose to upload it. Do not merge the old history back into the clean branch.
6. Run the existing **Compilar Linux** workflow on that revision and inspect its results. Local checks cannot confirm a hosted run that has not occurred.
7. If distributing binaries, first satisfy [Corresponding Source and resource-license obligations](LICENSING.md). Old ignored `dist/` archives may represent earlier code/license states: rebuild from the reviewed revision, do not reuse an older archive.
8. Manually validate the resulting application in the target Mint/GNOME session, with informed consent for installation and input-engine changes. Add actual screenshots only after checking their contents for personal data.
9. Set the About description/topics and change visibility manually only when ready. No visibility change, push or release is part of this preparation.

## Optional presentation improvements

Consider one screenshot of the current GUI, a short real demonstration of `a → á` and `n → ñ`, and an On/Off demonstration in a nonsensitive text editor. Show the actual Spanish interface and the tested desktop. Avoid terminal paths, account names and private text in the capture. No visual assets or performance claims have been invented.
