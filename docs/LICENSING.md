# Licensing review

Copyright © 2026 Rodrigo Naray. SPDX-License-Identifier: GPL-3.0-only.

## Original work

The original code, scripts and documentation are offered under **GNU General Public License version 3 only**, identified as `GPL-3.0-only`. There is no option to choose a later GPL version. This does not edit historical commits or revoke permissions granted by earlier distributions under a different license.

`LICENSE` is the complete official text obtained from https://www.gnu.org/licenses/gpl-3.0.txt, without changes. Its SHA-256 is `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`. The Free Software Foundation copyright and illustrative application examples inside that text remain intact. The project's author notice and version choice are in `COPYRIGHT`, source headers, Cargo metadata and both README files.

No noncommercial-use condition, extra redistribution restriction or custom exception has been added. Third parties retain their own rights and license terms.

## Dependency review

Cargo.lock and `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-unknown-linux-gnu` were cross-checked with `cargo tree --locked --offline --target x86_64-unknown-linux-gnu`. The selected tree contains **147 dependencies**, including build dependencies and procedural macros. The broader resolver also includes optional/unselected packages. Enabled eframe features: `default_fonts`, `glow`, `x11`, `wayland`.

| License family found | Assessment |
| --- | --- |
| MIT, ISC, 0BSD, BSD-2-Clause, BSD-3-Clause | GPLv3-compatible permissive terms; retain required copyright and license notices. |
| Apache-2.0 | Compatible with GPLv3, including its patent terms; preserve required notices. |
| Zlib / Unlicense alternatives | These dependencies offer a compatible alternative such as MIT or Apache-2.0. |
| Unicode-3.0 | Permissive Unicode data/software terms; preserve original copyright and permission notices. |
| `OR` / legacy `MIT/Apache-2.0` expressions | A GPL-compatible alternative may be selected. |
| `AND` expressions | All relevant terms apply, including Unicode data and fonts. |

No incompatible **code** license was identified in this selected Linux tree. This covers the inspected versions, target and features, not every feature/target in Cargo.lock or future updates. Original legal texts were inspected in addition to manifests. [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) maps packages and versions to original notices in [THIRD_PARTY_LICENSES.txt](../THIRD_PARTY_LICENSES.txt).

## Embedded fonts and resources

`epaint_default_fonts` embeds unmodified font files as byte resources:

| Font | Retained terms |
| --- | --- |
| Hack Regular | MIT; includes DejaVu public-domain work and Bitstream Vera terms |
| Noto Emoji Regular | SIL Open Font License 1.1 |
| Ubuntu Light | Ubuntu Font Licence 1.0 |
| emoji-icon-font | MIT |

These are separately licensed font resources, not GPL-relicensed program code. OFL and Ubuntu Font Licence permit bundling/embedding while retaining their own conditions and author notices. **Do not describe these fonts as relicensed under GPL-3.0-only.** The inventory preserves their original notices and TTF copyright/license metadata. Bitstream Vera and Ubuntu terms need particular attention when modifying, extracting or separately redistributing the fonts. No font was changed here.

Resource bundling is the basis of this assessment; these font licenses are not interchangeable with GPL for linked program code. Before a public binary release, verify the actual artifact's treatment of embedded fonts under both sets of conditions; obtain a specific licensing review if the distribution model changes. The project's original source can be GPL-licensed without claiming a single license for every resource.

The tracked tree contains no copied PowerToys source, Microsoft artwork, screenshots, custom icon pack or external media. The GUI draws its arrow; the launcher references a system icon by name. Inspiration does not imply affiliation or copied code.

## Runtime and CI tools

Python, PyGObject, IBus, GLib, systemd and graphics/windowing libraries come from the distribution, not the release archive. Their licenses and notices stay with those packages. Python uses PSF terms; PyGObject, IBus and GLib use LGPL terms. Systemd has components under LGPL/GPL and other stated upstream terms. Review the actual packages again if bundling them in another distribution format.

GitHub Actions are externally supplied CI tools; their implementation is not copied into this application. Existing checkout, toolchain and artifact-upload actions are retained.

## Before distributing binaries

The archive now carries GPL text, copyright, both README files, contribution/security documents and original third-party notices. That alone does **not** satisfy every GPL binary-distribution obligation.

Provide complete Corresponding Source by a method allowed in GPLv3 section 6: the exact project revision, build/installation scripts and required dependency sources, with their notices. A practical process is to prepare a matching source archive with vendored locked dependencies and build instructions, verify that it builds, and offer it alongside the binary with equally accessible download instructions. The current script does not generate that source bundle; a GitHub source ZIP does not include Cargo dependency sources. A checksum or Cargo.lock alone is not a source offer.

The installer copies the executable and engine but does not install a separate license/notices directory. Keep the complete distribution archive and accompanying notices available; review installed-package notice placement before distribution packaging. Installer behavior was preserved in this task.

## Primary references

- [Official GNU GPL v3](https://www.gnu.org/licenses/gpl-3.0.en.html)
- [GNU license list](https://www.gnu.org/licenses/license-list.html)
- [Apache Software Foundation: GPL compatibility](https://www.apache.org/licenses/GPL-compatibility)
- [SIL OFL FAQ: bundling and embedding](https://openfontlicense.org/ofl-faq/)
- [Canonical: Ubuntu Font Licence](https://canonical.com/legal/font-licence)
- [egui 0.31.1 font resources](https://github.com/emilk/egui/tree/0.31.1/crates/epaint_default_fonts)
