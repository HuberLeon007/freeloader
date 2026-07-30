# ADR 0002: rustls only, no OpenSSL anywhere in the dependency graph

- **Status:** accepted
- **Date:** 2026-02-14
- **Applies to:** every shipped crate, in particular `freeloader-download-core`, `freeloader-desktop` and `freeloader-native-host`

## Context

Freeloader downloads over HTTPS, so a TLS stack is mandatory. In the Rust ecosystem there are two realistic options:

1. `native-tls`, which resolves to SChannel on Windows and OpenSSL on Linux, and which pulls `openssl-sys` into the graph as soon as any crate enables it.
2. `rustls`, a pure-Rust TLS implementation with `ring` or `aws-lc-rs` as its cryptographic backend.

The project targets four triples: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`, `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`. `openssl-sys` needs a C toolchain, a matching set of headers and a discoverable OpenSSL installation for each of them. On `aarch64-pc-windows-msvc` that combination is the part of the build that breaks first, and the failure surfaces in the linker rather than in Cargo, which makes it expensive to diagnose. Cross-compilation and CI runners make the same problem worse, because each runner image ships a different OpenSSL version and layout.

A second consideration is the trust boundary. Native Messaging already exposes the application to browser-supplied input; adding a large C dependency to the same binary widens the memory-unsafe surface for no functional gain. The shipped crates use `#![forbid(unsafe_code)]`, and a pure-Rust TLS stack keeps that guarantee meaningful further down the graph.

`rustls` covers what the download path needs: TLS 1.2 and 1.3, SNI, ALPN, session resumption, and certificate verification against a root store. Nothing in the requirements depends on an OpenSSL-specific feature such as engine support, FIPS modules or a system-wide OpenSSL configuration file.

## Decision

Freeloader uses `rustls` as its only TLS implementation.

- `reqwest` is configured with `default-features = false` and the `rustls-tls` feature; the `default-tls` and `native-tls` features are never enabled.
- `openssl`, `openssl-sys` and `native-tls` are listed in the `[bans] deny` table of `deny.toml`, so `cargo deny check` fails if any dependency reintroduces them, directly or transitively.
- The root certificate source is an explicit choice in code, not an inherited default, so the same trust decision applies on Windows and on Linux.
- Adding a dependency that requires OpenSSL is a blocking change. It needs a superseding ADR, not a `deny.toml` exception.

## Consequences

**Positive**

- All four target triples build without a C toolchain or a system OpenSSL, which is what makes the `aarch64` builds practical.
- CI and cross-compilation stop depending on runner-image details.
- The memory-unsafe surface of the shipped binaries stays small and the `forbid(unsafe_code)` posture keeps its value.
- TLS behaviour is identical across platforms, so a bug reproduces on any developer machine.

**Negative**

- Freeloader does not honour a system-wide OpenSSL policy. On distributions that centrally configure allowed protocol versions or cipher suites through OpenSSL, Freeloader's TLS configuration is independent of that policy.
- `rustls` is stricter than OpenSSL about malformed certificates and legacy servers. A server that a browser still tolerates may be rejected. This is accepted: the failure is visible and reported as a transport error rather than silently downgraded.
- A crate that only supports `native-tls` cannot be adopted without replacing it or contributing `rustls` support upstream.
- The `ring` licence combination (`MIT AND ISC AND OpenSSL`) needs an explicit clarification entry in `deny.toml`. That entry documents a licence text, not a link against OpenSSL.

## References

- `deny.toml`, sections `[bans]` and `[[licenses.clarify]]`
- `docs/security-model.md`
- ADR 0006: dependency licence policy
