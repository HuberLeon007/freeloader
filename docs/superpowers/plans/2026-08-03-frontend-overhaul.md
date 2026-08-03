# Freeloader Frontend Overhaul Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the monolithic Freeloader frontend and narrow browser handoff with a quiet, keyboard-first, accessible download workstation that supports generic host adapters, batches, persistence, and Chromium plus Firefox capture flows without weakening local-first guarantees.

**Architecture:** Keep the Rust download engine and Tauri v2 shell as the boundary, then add a typed frontend application layer: protocol DTOs, queue store, parser and formatter modules, focused UI surfaces, and a virtualized table. Add a resolver registry with direct HTTP fallback and a `fuckingfast.co` adapter, expose only the smallest new Tauri commands needed for batch control and recovery, and share one token stylesheet plus message contracts between desktop and extensions.

**Tech Stack:** Tauri v2, Rust, React 19, TypeScript 5, Vite, pnpm workspaces, hand-authored CSS, `motion/react`, Radix Primitives, `@tanstack/react-virtual`, Vitest, Testing Library, and axe assertions. No Tailwind, full component kit, remote assets, telemetry, cloud service, macOS support, or new dependency without an explicit task justification.

## Global Constraints

- Work only on `plan/frontend-overhaul-2026-08-03`, created from `main`; leave `main` untouched.
- Use Conventional Commits with DCO sign-off: `Signed-off-by: Leon Huber <leonerwinhube@gmail.com>`.
- Every task finishes with `cargo xtask check`, which must run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, frontend typecheck, and frontend build.
- Keep Tauri v2, Rust, React 19, TypeScript 5, Vite, pnpm workspaces, strict CSP, GPL-3.0-or-later, and Windows/Linux only.
- Keep no account, no cloud, no telemetry, zero unsolicited outbound requests, no remote assets, system typefaces only, and refusal to forward cookies or credentials.
- Keep deferred features deferred: multi-connection segmentation, bandwidth limiting, tray, checksums, and outbound update checks. The UI must not imply they exist.
- Use UI stack `"Segoe UI Variable Text", "Segoe UI", Inter, system-ui, "Noto Sans", sans-serif`; numeric stack `"Cascadia Mono", ui-monospace, "DejaVu Sans Mono", monospace`.
- Use the exact type scale `11 / 12 / 13 / 15 / 19 / 24 / 30`, 4px base spacing with ramp `2 / 4 / 6 / 8 / 12 / 16 / 24 / 32 / 48`, and density row heights compact 32px, cozy 40px, comfortable 52px, cozy default.
- Use inline SVG sprites bundled locally, 1.5px line icons on a 16px grid, no emoji, and no icon-only destructive controls.
- Animate only `transform` and `opacity`; use 120ms state echoes, 180ms element transitions, 260ms surface transitions, `cubic-bezier(0.16, 1, 0.3, 1)` entrances, and `cubic-bezier(0.4, 0, 1, 1)` exits. Stop shimmer and looping animation under reduced motion.
- Wrap the app in `<MotionConfig reducedMotion="user">`; use `AnimatePresence mode="popLayout"` for queue rows, `layout` for batch disclosure, and `useReducedMotion()` when a translation must degrade to a fade.
- Meet WCAG 2.2 AA in both themes: text contrast at least 4.5:1, interface strokes and status shapes at least 3:1, real table headers with `aria-sort`, progressbar values, polite throttled live announcements, and Radix focus management.
- Destructive removal uses a five-second inline undo. Confirm only when deleting bytes from disk.

## File Structure Map

- Modify `apps/desktop/package.json`, `pnpm-lock.yaml`, `xtask/src/main.rs`, `.github/workflows/ci.yml`: permitted dependency additions, unified check gate, and reproducible CI.
- Replace `apps/desktop/src/main.tsx` with `apps/desktop/src/app/App.tsx` and `apps/desktop/src/app/app-shell.tsx`: bootstrapping, theme, onboarding gate, and global shortcuts.
- Create `apps/desktop/src/domain/downloads.ts`, `batches.ts`, `queue-store.ts`, `parser.ts`, `format.ts`, `protocol.ts`: stable DTOs, derived aggregates, persistence, parsing, number behavior, and Tauri invocation wrappers.
- Create `apps/desktop/src/components/ui/{Icon,Button,Field,StatusShape,ProgressMeter,Layer}.tsx` and `components/ui/primitives.ts`: shared primitives with Radix Dialog, DropdownMenu, Checkbox, Tooltip, and ContextMenu.
- Create `apps/desktop/src/components/queue/{QueueTable,QueueHeader,QueueRow,QueueBatchGroup,QueueDetail,QueueEmpty,QueueSkeleton}.tsx` and `components/composer/{AddComposer,MultiLinkPicker}.tsx`: shell surfaces and virtualization.
- Create `apps/desktop/src/components/settings/SettingsDialog.tsx`, `components/onboarding/Onboarding.tsx`, and `components/diagnostics/FailurePanel.tsx`: exact visual agreement across modes.
- Replace `apps/desktop/src/styles.css` and `onboarding.css` with `apps/desktop/src/styles/tokens.css`, `base.css`, `motion.css`, `shell.css`, `queue.css`, `setup.css`, and `extension.css`; keep `apps/desktop/src/theme.ts`, update tests.
- Extend `crates/protocol/src/message.rs`, `validation.rs`, `lib.rs`; create resolver and batch seam modules under the existing desktop adapter and native host paths after inspecting their current names in Task 1.
- Replace `extensions/chromium/{background.js,popup.html,popup.js,manifest.json}` and Firefox equivalents; create shared extension sources under `extensions/shared/` plus `extensions/scripts/build.mjs` and manual install docs under `docs/extensions/`.
- Modify `CHANGELOG.md` and `docs/implementation-status.md` in the task that changes each shipped capability. Do not create a cleanup task for either.

## Task 1: Baseline, CI Diagnosis, and Test Harness

**Files:** Modify `.github/workflows/ci.yml`, `xtask/src/main.rs`, `apps/desktop/package.json`; create `apps/desktop/src/test/setup.ts`, `apps/desktop/vitest.config.ts`; test `apps/desktop/src/test/smoke.test.ts`.

**Interfaces:** Produces `cargo xtask check`, Vitest setup, and a documented baseline failure record for later tasks.

- [ ] **Step 1: Reproduce the current gates.** Run `git switch main`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm install --frozen-lockfile=false`, `pnpm --dir apps/desktop typecheck`, and `pnpm --dir apps/desktop build`. Record the first failing command, exact error, affected file, and whether the failure is CI-only.
- [ ] **Step 2: Write the failing smoke test.** Add `describe("test harness", () => { it("runs in jsdom", () => { expect(document.body).toBeDefined() }) })`; run `pnpm --dir apps/desktop test` and expect the command to fail until Vitest config exists.
- [ ] **Step 3: Add the smallest harness.** Configure Vitest with `environment: "jsdom"`, `setupFiles: ["./src/test/setup.ts"]`, and `globals: true`; add `@testing-library/react`, `@testing-library/jest-dom`, `jsdom`, and `vitest-axe` only if absent. Add `cargo xtask check` orchestration with sequential child commands and nonzero exit propagation. Fix only the diagnosed baseline failure.
- [ ] **Step 4: Verify.** Run the smoke test and `cargo xtask check`; expected output is one passing Vitest test and all four gates green.
- [ ] **Step 5: Commit.** `git add .github/workflows/ci.yml xtask/src/main.rs apps/desktop/package.json apps/desktop/vitest.config.ts apps/desktop/src/test && git commit -s -m "build: establish unified frontend and workspace checks"`.

## Task 2: Design Tokens, Icons, and Shared UI Primitives

**Files:** Replace `apps/desktop/src/styles.css`, `apps/desktop/src/onboarding.css`; create `apps/desktop/src/styles/tokens.css`, `base.css`, `motion.css`, `components/ui/{Icon,Button,Field,StatusShape,ProgressMeter,Layer}.tsx`, `components/ui/primitives.ts`, and accessibility tests for each primitive.

**Interfaces:** `Icon({ name, size, label? }: { name: IconName; size?: number; label?: string })`; `StatusShape({ status }: { status: DownloadStatus })`; `ProgressMeter({ value, label, indeterminate }: { value: number | null; label: string; indeterminate?: boolean })`; `Layer` wraps Radix Dialog or menu surfaces and returns focus to its trigger.

- [ ] **Step 1: Write token and primitive tests.** Assert every control has a visible focus ring, every icon-only control has an accessible name, `ProgressMeter` renders `role="progressbar"` with `aria-valuenow` only when known, and axe reports no violations in light and dark themes.
- [ ] **Step 2: Declare exact light tokens with hex fallback first, then OKLCH.** Use `--surface-page: #fbfaf7; --surface-page: oklch(98.6% 0.006 85)`, `--surface-raised: #fefdfb; --surface-raised: oklch(99.4% 0.004 85)`, `--surface-sunken: #f6f4ef; --surface-sunken: oklch(96.4% 0.008 85)`, `--surface-inset: #f0eee8; --surface-inset: oklch(94.2% 0.010 85)`, `--line-quiet: #e7e4de; --line-quiet: oklch(91.0% 0.007 85)`, `--line: #dcd8d0; --line: oklch(86.5% 0.009 85)`, `--line-strong: #c8c3ba; --line-strong: oklch(78.0% 0.011 85)`, `--ink: #393632; --ink: oklch(24.0% 0.014 85)`, `--ink-secondary: #77736d; --ink-secondary: oklch(46.0% 0.012 85)`, `--ink-tertiary: #a39e96; --ink-tertiary: oklch(62.0% 0.010 85)`, `--ink-disabled: #beb9b1; --ink-disabled: oklch(74.0% 0.008 85)`, `--signal: #3f806b; --signal: oklch(47.0% 0.105 162)`, `--signal-hover: #36715f; --signal-hover: oklch(42.0% 0.100 162)`, `--signal-quiet: #e7f2ed; --signal-quiet: oklch(95.0% 0.022 162)`, `--signal-ink: #f7fbf8; --signal-ink: oklch(98.5% 0.008 162)`, `--danger: #a94a3c; --danger: oklch(52.0% 0.170 27)`, `--danger-quiet: #faeee9; --danger-quiet: oklch(95.5% 0.030 27)`, `--attention: #d2a552; --attention: oklch(72.0% 0.120 72)`, and `--focus-ring: 2px solid var(--signal);` with offset 2px.
- [ ] **Step 3: Add dark tokens.** Use `--surface-page: #292824; --surface-page: oklch(19.5% 0.008 85)`, mirrored warm hues for raised, sunken, inset, and `--line-quiet: #44413c; --line-quiet: oklch(30% 0.010 85)` with lifted line values. Set ink to `#e8e5df; oklch(92% 0.010 85)` and reduce signal chroma by roughly 15 percent, never using `#000` or `#fff`.
- [ ] **Step 4: Implement primitives.** Use inline SVG sprite symbols, Radix `Dialog.Root`, `Dialog.Content` with `onOpenAutoFocus`, `onCloseAutoFocus`, `onEscapeKeyDown`, `DropdownMenu.CheckboxItem` with `ItemIndicator`, `Checkbox.Root`, `Tooltip`, and ContextMenu. Use `font-variant-numeric: tabular-nums` on all byte, speed, percentage, ETA, and count output.
- [ ] **Step 5: Verify and commit.** Run the primitive tests, axe assertions, `cargo xtask check`, then `git add ... && git commit -s -m "feat(ui): establish quiet instrument primitives"`.

## Task 3: Typed Queue Domain, Parsing, Formatting, and Persistence

**Files:** Create `apps/desktop/src/domain/{downloads,batches,parser,format,protocol,queue-store}.ts`; create Vitest tests beside each module.

**Interfaces:** `type DownloadStatus = "queued" | "resolving" | "connecting" | "active" | "paused_user" | "paused_network" | "retrying" | "resuming" | "completed" | "failed" | "cancelled" | "missing" | "destination_unwritable"`; `parseLinkInput(input: string): ParseResult`; `formatBytes(value: bigint | null): string`; `formatSpeed(value: number | null): string`; `formatEta(seconds: number | null): string`; `createBatch(items: ParsedLink[], folderName: string): Batch`; `QueueStore` exposes `hydrate()`, `persist()`, `addBatch()`, `updateDownload()`, `removeWithUndo()`, `undoRemoval()`, and `selectAll()`.

- [ ] **Step 1: Write parser tests.** Cover blank lines, comments, whitespace, malformed schemes, duplicate URLs, duplicate URLs with fragments, valid HTTP/HTTPS URLs, line-numbered feedback, and a 50-item protocol cap.
- [ ] **Step 2: Write formatter and persistence tests.** Assert one binary unit convention, `"—"` for unknown values, speed smoothing over a rolling sample window, ETA hidden until total and speed are trustworthy, JSON round-trip recovery after simulated crash, and removal focus target selection.
- [ ] **Step 3: Implement pure logic.** Deduplicate by normalized URL, preserve first-line order, return `{ accepted, rejected }`, sanitize editable batch folder names, and calculate aggregate `{ total, downloaded, speed, eta, active, failed }` without treating unknown values as zero.
- [ ] **Step 4: Implement persistence.** Store versioned queue snapshots in the existing local SQLite/Tauri persistence seam, not remote storage; write atomically, restore `.part` and restart notices, and retain user density, theme, destination, concurrency, and adapter choice. Keep localStorage only for migration flags.
- [ ] **Step 5: Verify and commit.** Run `pnpm --dir apps/desktop test -- parser format queue-store`, `cargo xtask check`, then `git commit -s -m "feat(queue): add typed parsing and recovery state"`.

## Task 4: Host Adapter Seam and Batch Commands

**Files:** Modify the existing desktop adapter command module and `crates/protocol/src/{message,validation,lib}.rs`; create resolver registry, direct HTTP resolver, `fuckingfast.co` resolver, Rust DTO tests, and frontend invocation tests. Use the exact existing module paths discovered in Task 1 instead of inventing a second adapter crate.

**Interfaces:** Rust `trait HostAdapter { fn matches(&self, url: &Url) -> bool; fn resolve(&self, url: &Url) -> Result<Vec<ResolvedItem>, AdapterError>; }`; `ResolvedItem { direct_url: Url, filename: String, total: Option<u64>, adapter: String }`; frontend `resolveLinks(urls: string[]): Promise<ResolveResult[]>`; `create_batch`, `pause_downloads`, `resume_downloads`, `retry_downloads`, `force_redownload`, and `restore_queue` Tauri commands accept validated IDs and return DTOs.

- [ ] **Step 1: Write protocol tests first.** Add strict serde tests for a new `capture_batch` payload carrying `url`, `suggestedFilename`, `referrer`, `contentType`, and `cookiesIncluded: false`; reject cookies true, unknown fields, invalid schemes, overlong URLs, and more than 50 items. Preserve version 1 compatibility for existing `capture_download` and `ping`.
- [ ] **Step 2: Write adapter contract tests.** Assert unknown hosts use direct HTTP unchanged, `fuckingfast.co` resolves a share page to direct URL, filename, optional size, and adapter name, resolver failures are human-readable, and no request includes cookies or credentials.
- [ ] **Step 3: Implement the registry and commands.** Register direct HTTP first and the `fuckingfast.co` adapter second; make adapters enumerable for the UI; route batch actions to the existing download core without changing its streaming, retry, containment, or atomic rename behavior.
- [ ] **Step 4: Verify end to end.** Run Rust protocol and adapter tests, frontend invocation tests, a local native host capture fixture, then `cargo xtask check`; commit `feat(core): add generic host adapters and batch controls`.

## Task 5: Shell, Composer, and Queue Table

**Files:** Replace `apps/desktop/src/main.tsx`; create `app/{App,app-shell}.tsx`, `components/composer/{AddComposer,MultiLinkPicker}.tsx`, `components/queue/{QueueTable,QueueHeader,QueueRow,QueueBatchGroup,QueueDetail,QueueEmpty,QueueSkeleton}.tsx`, and component tests.

**Interfaces:** `QueueTable({ rows, density, selectedIds, onSelectionChange, onAction }: QueueTableProps)` uses `useVirtualizer({ count, getScrollElement, estimateSize, paddingStart, scrollPaddingStart })` and attaches `ref={rowVirtualizer.measureElement}` plus `data-index`; `QueueRow` renders a real `<tr>` with `<th>` headers, not div roles; `MultiLinkPicker` returns only checked rows and editable folder name.

- [ ] **Step 1: Write interaction tests.** Test Ctrl+V paste parsing, Space pause/resume, Delete removal with inline undo, Ctrl+A, Enter detail disclosure, Escape layer close, Ctrl+, settings, slash filter, focus movement after removal, indeterminate select-all, and empty-after-filter.
- [ ] **Step 2: Implement the shell.** Use rail, header status line, composer well, sticky table header, footer destination, and settings trigger. Keep the empty state as a workbench: paste field, one sentence, destination, no hero or illustration.
- [ ] **Step 3: Implement batches and details.** Render collapsible groups with aggregate progress, transferred bytes, combined speed and ETA; use `AnimatePresence mode="popLayout"` for row insertion/removal, `layout` for group expansion, and inline details for source, adapter, destination, retry count, restart notice, and copyable technical detail.
- [ ] **Step 4: Implement virtualization and states.** Support loading skeleton, parsing, resolving, queued, connecting, active, paused, retrying, resuming, completed, failed, cancelled, missing, and unwritable states. Use `scaleX()` for meters, no width animation, and no toast spam.
- [ ] **Step 5: Verify and commit.** Run Testing Library plus axe tests at 10, 40, and 1,000 rows, then `cargo xtask check`, commit `feat(desktop): rebuild shell and virtualized queue`.

## Task 6: Settings, Onboarding, Diagnostics, and Motion Verification

**Files:** Replace `apps/desktop/src/onboarding.tsx`; create `components/settings/SettingsDialog.tsx`, `components/onboarding/Onboarding.tsx`, `components/diagnostics/FailurePanel.tsx`, and their tests.

**Interfaces:** `SettingsDialog` controls theme, density, destination, concurrency, adapter enumeration, recent folders, browser handoff, and setup replay. `FailurePanel` accepts `{ reason: FailureReason; technicalDetail: string; onRetry; onCopy }`. Onboarding keeps the existing command signatures `suggested_directories`, `inspect_directory({ path })`, `pick_download_dir`, `create_directory({ path })`, and `detect_browsers`.

- [ ] **Step 1: Write tests.** Test Radix dialog focus return, Escape, browser detection disclosure, directory unwritable and missing states, setup keyboard paths, reduced motion, copyable technical detail, and no raw OS error code shown alone.
- [ ] **Step 2: Implement exact shell agreement.** Reuse tokens, typography, button geometry, hairlines, status shapes, cozy density, and copy rules. The only cards are composer and setup stage, never nested.
- [ ] **Step 3: Add MotionConfig and reduced-motion behavior.** Use `MotionConfig reducedMotion="user"`; use `useReducedMotion()` for drawer and setup transitions; remove shimmer entirely when reduced motion is preferred.
- [ ] **Step 4: Verify themes and accessibility.** Run axe in light/dark, keyboard-only Testing Library tests, and screenshot review at 100 percent and 200 percent zoom. Run `cargo xtask check`; commit `feat(desktop): align setup settings and diagnostics`.

## Task 7: Extension Shared UI and Native Messaging

**Files:** Create `extensions/shared/{protocol.js,format.js,styles.css,SingleCapture.jsx,MultiLinkPicker.jsx,NativeHost.js}` and build tests; replace both browser `background.js`, `popup.html`, `popup.js`, and manifests.

**Interfaces:** `NativeHost.send(request: Request): Promise<Response>` uses `browser.runtime.sendNativeMessage(hostName, request)` and rejects with actionable host registration text; request types remain strict `capture_download`, `capture_batch`, and `ping`; `SingleCapture` props are `{ filename, extension, size, contentType, host, adapter, destination, recentFolders, batch, onStart, onQueuePaused, onCancel }`; `MultiLinkPicker` returns checked rows and a folder name.

- [ ] **Step 1: Write extension tests.** Test interception before browser download, cancel handoff to browser, filename extension separation, remembered host/type choices, candidate deduplication, kind chips, text filter, indeterminate select-all, combined summary, keyboard focus, Escape, and native host failure.
- [ ] **Step 2: Implement strict capture transport.** Send only user-selected URLs, suggested metadata, and `cookiesIncluded: false`; never read cookies, history, profiles, passwords, or remote services. Map `No such native application` to host registration instructions.
- [ ] **Step 3: Implement popup surfaces.** Single-link popup is compact and frameless; multi-link picker is a real table with chips, live `N of M selected · combined size`, destination, editable batch folder, and primary send action. Reuse desktop tokens verbatim and use local SVG icons.
- [ ] **Step 4: Implement Chromium MV3 and Firefox manifests.** Chromium uses service worker event listeners and `nativeMessaging`; Firefox uses WebExtensions background script and `nativeMessaging`; keep separate manifests where contract fields differ. Add GitHub Releases ZIP build and manual load-unpacked guide, never Chrome Web Store instructions.
- [ ] **Step 5: Verify and commit.** Run extension unit tests, manifest validation, native host fixture tests, `cargo test --workspace`, `cargo xtask check`, commit `feat(extension): add quiet capture and batch picker`.

## Task 8: Recovery, Force Redownload, and Cross-Surface Polish

**Files:** Modify existing download-core and desktop adapter seams only where required for `restore_queue`, `force_redownload`, batch controls, and file-missing detection; update `CHANGELOG.md` and `docs/implementation-status.md`; create integration tests under each affected crate and frontend recovery tests.

**Interfaces:** `restore_queue(): Promise<DownloadDto[]>`; `force_redownload(ids: string[]): Promise<DownloadDto[]>`; `batch_action(batchId: string, action: "pause" | "resume" | "retry" | "force_redownload"): Promise<Batch>`; `queue_snapshot_version` is bumped once with a migration test.

- [ ] **Step 1: Write recovery tests.** Simulate process termination during `.part` write, restart with Range resume, missing destination, unwritable destination, failed retry attempt count, force-redownload replacing a completed file only after explicit action, and exact aggregate restoration.
- [ ] **Step 2: Implement only the needed seam.** Do not alter engine public behavior beyond adapter and recovery seams; preserve containment checks, retry policy, atomic rename, and credential refusal.
- [ ] **Step 3: Update docs with the changed deliverable.** Add shipped adapter, batch, persistence, concurrency, extension, and recovery capabilities to the current release section; explicitly keep segmentation, limiting, tray, checksums, update checks, and credential forwarding deferred or refused.
- [ ] **Step 4: Verify and commit.** Run all Rust integration tests, frontend recovery tests, `cargo xtask check`, commit `feat(queue): finish recovery and batch actions`.

## Task 9: Release Verification and Plan Self-Review

**Files:** Modify `.github/workflows/ci.yml`, `docs/verification.md`, `README.md`, `CHANGELOG.md` only for verified commands and release instructions; add `apps/desktop/src/test/accessibility.test.ts`, extension fixture tests, and packaging checks.

- [ ] **Step 1: Run the full gate.** `cargo xtask check` must print successful completion for fmt, clippy, workspace tests, frontend typecheck, and frontend build.
- [ ] **Step 2: Run product verification.** Test 1,000 virtualized rows, all keyboard shortcuts, reduced motion, both themes, WCAG contrast, batch aggregate math, crash restore, direct HTTP, `fuckingfast.co`, unknown host fallback, Chromium capture, Firefox capture, native-host unavailable error, no-cookie rejection, and no outbound request in a network-denied fixture.
- [ ] **Step 3: Verify packaging.** Build the Chromium ZIP and Firefox package from local files only, confirm no remote asset URLs, confirm strict CSP remains unchanged, and confirm Windows/Linux targets only.
- [ ] **Step 4: Self-review the plan against the spec.** Check every surface in sections 7, 8, and 9 against Tasks 3 through 8; scan for `TODO`, `TBD`, placeholder prose, invented API signatures, banned gradients, nested cards, row stripes, and em dash characters. Fix the plan or implementation task inline before release.
- [ ] **Step 5: Commit and hand off.** `git add .github docs README.md apps extensions apps/desktop xtask crates && git commit -s -m "chore: verify frontend overhaul release"`; open a pull request from `plan/frontend-overhaul-2026-08-03` into `main`, with `cargo xtask check` output attached.

## Self-Review Findings

- **Spec coverage:** Quiet Instrument tokens, typography, spacing, density, icons, motion, keyboard laws, table semantics, Radix focus management, every listed state, virtualization, generic adapters, `fuckingfast.co`, direct HTTP fallback, parsing, batch naming, aggregate rows, concurrency, persistence, pause/resume/retry/force-redownload, crash recovery, Chromium MV3, Firefox Native Messaging, strict protocol refusal, diagnostics, changelog, implementation status, DCO, branch discipline, CI diagnosis, Vitest, Testing Library, axe, and all four gates are mapped above.
- **Repository grounding:** The current repo is `HuberLeon007/freeloader`, current frontend is `apps/desktop/src/main.tsx` plus `onboarding.tsx`, current extension roots are `extensions/chromium` and `extensions/firefox`, current protocol is `crates/protocol/src/message.rs`, and current CI has separate Rust and frontend jobs but no unified `cargo xtask check`.
- **Research grounding:** Motion docs confirm `MotionConfig reducedMotion="user"`, `useReducedMotion()`, and `layout`; Radix source confirms controlled Dialog focus props and `DropdownMenu.CheckboxItem` patterns; TanStack docs confirm `useVirtualizer`, `measureElement`, `paddingStart`, and dynamic row measurement; Tauri docs confirm `@tauri-apps/api/core` `invoke` and capability permissions; MDN confirms `runtime.sendNativeMessage`, `nativeMessaging`, native host manifests, and actionable missing-host errors.
- **Gaps closed:** The initial repository lookup found no open issues, so Task 1 diagnoses the live CI state by running the exact gates rather than inventing an issue number. The plan deliberately leaves implementation-specific Rust file names to Task 1 inspection where the repository tree was not fully loaded, while all new frontend paths and public interfaces are fixed here.
