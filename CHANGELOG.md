# Changelog

All notable changes to Freeloader are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Entries are grouped as Added, Changed, Deprecated, Removed, Fixed and Security. Until v0.1.0 is released, everything lands under Unreleased.

## [Unreleased]

### Added

- `cargo dev` starts the app from a fresh clone: it installs the frontend dependencies when they are missing, generates the bundle icons, compiles the Rust core and opens the window. `cargo dev build`, `cargo dev icons` and `cargo dev setup` cover the remaining developer tasks. The task runner is the dependency-free `xtask` crate.
- `default_download_dir` and `pick_directory` commands, so the setup flow can propose the real OS download folder and open the native folder picker instead of storing a relative path.
- The first-run flow configures appearance as well, with previews of the system, light and dark themes.
- `CONTRIBUTING.md` with the DCO sign-off requirement, the Conventional Commits convention and the four commit-gate checks.
- `CODE_OF_CONDUCT.md`, `SECURITY.md` and this changelog.
- Issue templates, a pull request template, a Dependabot configuration for Cargo, npm and GitHub Actions, and `CODEOWNERS`.
- `docs/adr/0002-rustls-only.md` and `docs/adr/0006-dependency-licence-policy.md`, the two decisions `deny.toml` refers to.
- A `package.json` for `extensions/chromium` and `extensions/firefox`, so both are real pnpm packages.
- `.gitattributes`, so line endings are normalised for every clone: LF everywhere, CRLF for `ps1`, `bat`, `cmd`, `nsi` and `nsh`.

### Changed

- Rebuilt the interface on a new design system: warm paper neutrals with a single indigo accent, light as the resolved default, and structure from hairlines and spacing rather than stacked cards. Colours are declared as a hex fallback followed by the OKLCH value so older WebKitGTK builds still render a complete theme.
- The download queue is a hairline list with tabular figures and a progress edge per row, and the three summary cards collapsed into one status line in the header.
- Settings is a native `<dialog>`, so focus trapping, Escape and the backdrop are handled by the platform.
- Motion is limited to transform and opacity with exponential ease-out curves, and is disabled entirely under `prefers-reduced-motion`.
- `docs/implementation-status.md` is valid Markdown again and names the features deferred to a later spec.
- `pnpm-workspace.yaml` lists only directories that contain a `package.json`; the non-existent `extensions/shared` entry is gone.
- `.gitignore` excludes `*.exe`, so build output cannot be committed by accident.
- `deny.toml` states the licence policy explicitly, bans `openssl`, `openssl-sys` and `native-tls`, and points at the two ADRs that explain both decisions.

## Notes

The initial release is not tagged yet. Target platforms are Windows x64, Windows ARM64, Linux x64 and Linux ARM64; treat an artifact as unsupported until its own build and smoke test have run on that platform, as described in [docs/verification.md](docs/verification.md).
