# Dependency security follow-up — October 19, 2026

## Immediate fix: rustls

- The October 19 CI dependency audit found four vulnerability advisories affecting `rustls 0.23.45`:
  [RUSTSEC-2026-0346](https://rustsec.org/advisories/RUSTSEC-2026-0346),
  [RUSTSEC-2026-0347](https://rustsec.org/advisories/RUSTSEC-2026-0347),
  [RUSTSEC-2026-0348](https://rustsec.org/advisories/RUSTSEC-2026-0348),
  and [RUSTSEC-2026-0349](https://rustsec.org/advisories/RUSTSEC-2026-0349).
- The common patched version is **0.23.46**.
- The lockfile is updated only for `rustls`; its new checksum
  `48e13bd8c0e9365c43cfa5c9e8f9ad49d3c8444926c9aac819e0e4dc503c8fdf`
  was verified against the official
  [crates.io index entry](https://github.com/rust-lang/crates.io-index/blob/master/ru/st/rustls)
  for `rustls 0.23.46`.
- Existing resolved `rustls-webpki 0.103.15`, `rustls-pki-types 1.15.1`,
  and `aws-lc-rs 1.18.1` satisfy the new release's dependency constraints.
  This fix does not change dependencies for PdfCraft's other crates.
- Validate with `cargo metadata --locked`, platform CI and cargo-deny. Do not
  consider the fix verified until actual CI results confirm it.

## Remaining informational advisories: Linux accessibility dependency chain

The same CI audit reports:

| Advisory | Locked package | Type |
| --- | --- | --- |
| [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370) | `zvariant 5.15.0` | Unmaintained |
| [RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371) | `zbus_names 4.3.4` | Unmaintained |
| [RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372) | `zvariant_derive 5.15.0` | Unmaintained |

**These are maintenance-status advisories, not reports of an exploit in these
particular versions.** Do not confuse them with the four rustls vulnerabilities.

The locked chain reported by cargo-deny is broadly:
`eframe 0.36.2` → `egui-winit 0.36.2` → `accesskit_winit 0.32.2`
→ `accesskit_unix 0.21.1` / `accesskit_atspi_common 0.18.1` →
`atspi 0.29` / `zbus 5.19.0` → `zvariant`, `zbus_names`, etc.

The upstream crates were consolidated into `zbus 6`, but a direct `zbus`
major-version bump is **not** a safe drop-in change while the AccessKit/AT-SPI
integration explicitly depends on version 5. Furthermore, the newest
`accesskit_unix 0.24.0` as of this audit still depends on `zbus 5.19`
([published dependency listing](https://docs.rs/crate/accesskit_unix/0.24.0)).
Even newer egui versions therefore do not guarantee removal of the notices.

### Remediation path

1. Check whether upstream AccessKit/AT-SPI has published a `zbus 6`-
   compatible release and whether egui/winit supports it.
2. Test a full compatible dependency upgrade separately with keyboard access,
   Linux screen-reader/AT-SPI behavior and macOS/Windows smoke tests.
3. In the interim **keep the advisories visible in CI**. Do not quietly disable
   Linux accessibility support, suppress the advisories without an explicit
   risk acceptance, or make a major dependency bump without testing.

### Related build and tests

- Original failed dependency scan:
  [CI run 39359261565](https://github.com/eeveeuzumaki/pdfcraft/actions/runs/39359261565).
- Initial PDF round-trip regression tests:
  [PR #7](https://github.com/eeveeuzumaki/pdfcraft/pull/7).
- This is a dependency maintenance task; the code for PDF editing is unchanged.
