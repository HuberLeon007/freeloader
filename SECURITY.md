# Security policy

## Reporting a vulnerability

Report vulnerabilities privately. Do not open a public issue, and do not describe the problem in a pull request before it is fixed.

**Preferred channel:** [open a private security advisory](https://github.com/HuberLeon007/freeloader/security/advisories/new) on GitHub. Only the maintainers can read it.

If GitHub advisories are unavailable to you, contact the repository owner [@HuberLeon007](https://github.com/HuberLeon007) directly and ask for a private channel before sending any detail.

Please include:

- affected version or commit, and the operating system and architecture,
- component: download engine, Tauri adapter, frontend, native messaging host, browser extension, or installer,
- reproduction steps, ideally minimal, plus what you expected and what happened,
- impact as you see it, and any proof of concept you already have.

**What to expect:** acknowledgement within 7 days, an initial assessment within 14 days, and progress updates until the issue is closed. Fixes are released as soon as a tested one exists. Please allow up to 90 days before public disclosure, and let us know if you plan to publish earlier so the timing can be coordinated. Credit is given in the release notes unless you prefer otherwise.

## Supported versions

Freeloader is in early development. Only the current development branch and the most recent release receive fixes. There are no long-term support branches yet.

## In scope

- Path traversal, filename handling and containment escapes when writing downloads.
- The Native Messaging boundary: framing, payload limits, schema validation, extension allowlists, and host manifest registration.
- Privilege escalation or arbitrary code execution through the installer or the native host registration.
- Handling of untrusted input: URLs, redirects, response headers, filenames, referrers and browser messages.
- Unexpected outbound network activity, or any leak of local data. Freeloader has no telemetry and starts no HTTP server; a finding that contradicts either is a security bug.
- Tauri adapter configuration: content security policy, capabilities and exposed commands.

## Out of scope

- Reports produced by a scanner without a demonstrated impact on Freeloader.
- Vulnerabilities in third-party dependencies that Freeloader does not reach. Report those upstream; tell us if Freeloader is affected.
- Denial of service that requires local administrative access, or physical access to an unlocked machine.
- Attacks that require a user to install a modified build.
- Missing hardening that has no exploitable consequence.

## Permanently out of scope as a feature

The following are not omissions to be reported, requested or patched. They will not be added to Freeloader, and a pull request that adds them is rejected on sight:

- **DRM circumvention** of any kind.
- **Bypassing paywalls, login walls or other access restrictions**, including forwarding cookies, tokens or credentials in order to reach content the user cannot otherwise reach.
- Extraction from streaming sites, and YouTube ripping.

Freeloader downloads resources that the user can already reach with an ordinary HTTP(S) request. It does not forward cookies or `Authorization` headers, and the native host refuses a request that asks for it. See [docs/security-model.md](docs/security-model.md).

## Security properties we intend to keep

- No shipped binary starts an HTTP server, localhost listener or WebSocket bridge.
- No telemetry, analytics or remote crash reporting.
- Logs stay local and redact query strings.
- TLS is `rustls` only; OpenSSL and `native-tls` are banned in `deny.toml` ([ADR 0002](docs/adr/0002-rustls-only.md)).
- Shipped crates use `#![forbid(unsafe_code)]`.
- Native host registration is per-user and is removed on uninstall.

A change that weakens one of these without an ADR is treated as a defect.
