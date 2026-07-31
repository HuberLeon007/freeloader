# Implementation status

This branch is implemented directly in the repository, without delegating code generation to an external coding agent. The implementation proceeds in vertical slices: workspace, protocol, download core, desktop adapter, UI, native messaging, extensions, installers, and release verification.

No feature is marked complete until it has executable tests and a documented verification command. Unsupported host-specific checks are recorded explicitly rather than represented by fake green results. See [verification.md](verification.md) for which commands run anywhere and which need the target operating system and architecture.

## Deferred to a later spec

The following features exist in v0.1 as a defined seam only. Nothing in the user interface suggests they already work, and no placeholder control pretends otherwise.

| Feature | State in v0.1 | Seam it will attach to |
| --- | --- | --- |
| Multi-connection segmentation | not implemented; every transfer is a single stream | `DownloadStrategy`, which a segmenting implementation can implement without changing any call site |
| Bandwidth limiting | not implemented; exactly one pass-through implementation ships, and it documents that it does not throttle | `RateLimiter` |
| Tray integration | not implemented | window and lifecycle handling in the Tauri adapter |
| Cookie and credential forwarding | not implemented and deliberately refused; the native host rejects a request that asks for it | Native Messaging request validation in `crates/protocol` |
| Checksum verification | not implemented; exactly zero checksums are verified | `ChecksumVerifier` |
| Outbound update check | not implemented; the setting is persisted and triggers exactly zero outbound requests | the persisted update-check setting |

## Permanently out of scope

DRM circumvention, bypassing paywalls or login walls, extraction from streaming sites, YouTube ripping, and macOS support. These are not deferred; they will not be added.
