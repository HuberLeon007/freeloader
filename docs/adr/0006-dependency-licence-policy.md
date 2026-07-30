# ADR 0006: dependency licence policy

- **Status:** accepted
- **Date:** 2026-02-14
- **Applies to:** every Cargo and npm dependency of the shipped artifacts, and every build-time dependency that contributes code to them

## Context

Freeloader is distributed under GPL-3.0-or-later. Every dependency that ends up in a shipped binary or bundle becomes part of a combined work, so its licence has to be compatible with distributing that combined work under the GPL. Getting this wrong is not a style problem: it makes a release undistributable, and it is discovered by users rather than by the maintainers.

Two classes of licence cause trouble specifically:

- **Network-copyleft and source-available licences.** AGPL-3.0 imposes obligations beyond the GPL that the project does not want to accept for a desktop application. SSPL, BUSL, the Elastic Licence, and "free for non-commercial use" terms such as CC-BY-NC are not free software licences at all; a GPL-3.0-or-later release cannot include them.
- **Unclear or missing licence metadata.** A crate that declares no licence, or declares one that the scanner cannot match with confidence, is indistinguishable from a licence violation until somebody reads the repository by hand.

Review by hand does not scale and does not survive dependency updates, so the policy has to be machine-enforced on every pull request.

## Decision

The licence policy is expressed in `deny.toml` and enforced by `cargo deny check` in CI. Nothing is allowed implicitly.

**Allowed** (permissive, or copyleft compatible with a GPL-3.0-or-later combined work):

`MIT`, `MIT-0`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Zlib`, `BSL-1.0`, `CC0-1.0`, `MPL-2.0`, `Unicode-3.0`, `Unicode-DFS-2016`, `GPL-3.0`, `LGPL-2.1`, `LGPL-3.0`.

**Denied**, by the simple mechanism of not appearing in the allow list:

`AGPL-3.0` and any other network-copyleft licence, `SSPL`, `BUSL-1.1`, the Elastic Licence, `CC-BY-NC` and every other non-commercial variant, every source-available licence that restricts use or redistribution, and every dependency whose licence expression cannot be determined.

Supporting rules:

- `confidence-threshold = 0.93`. A licence text that only loosely matches a known licence counts as undetermined, not as allowed.
- `unknown-registry = "deny"` and `unknown-git = "deny"`. Dependencies come from `crates.io`; a git or vendored source needs a superseding ADR.
- `[bans] wildcards = "deny"`. A wildcard version requirement makes the licence set of a build unpredictable.
- `[[licenses.clarify]]` is used only to pin the licence text of a crate whose metadata is ambiguous, with a file hash so the clarification cannot silently drift. `ring` is the current entry: `MIT AND ISC AND OpenSSL` refers to its own licence file, not to a link against OpenSSL (see ADR 0002).
- npm dependencies follow the same rule set. Any package under a denied licence is replaced rather than exempted.

**Exceptions.** An exception requires a new ADR under `docs/adr/` that names the dependency, the licence, why no compliant alternative exists, and what obligations the exception creates for distribution. Only after that ADR exists may an entry be added to `licenses.exceptions` in `deny.toml`, and the entry must reference the ADR. `exceptions` is empty today, and an empty list is the expected steady state.

## Consequences

**Positive**

- A licence violation fails the pull request that introduced it, names the offending dependency, and is fixed while the change is still in review.
- The distribution question for a release is answered by a command, not by an argument.
- The `about.toml` attribution output stays derivable from a known-good licence set.

**Negative**

- Some otherwise attractive libraries are unavailable, and a compliant alternative may cost more implementation work.
- A dependency that relicenses to AGPL or SSPL in a later version blocks its own update until it is replaced. The build fails loudly rather than shipping the new terms.
- Every new allowed licence needs a deliberate decision, which slows down first-time additions.

## References

- `deny.toml`, section `[licenses]`
- `about.toml` for the attribution bundle
- ADR 0002: rustls only, no OpenSSL anywhere in the dependency graph
- Requirements 22.1, 22.6, 22.7
