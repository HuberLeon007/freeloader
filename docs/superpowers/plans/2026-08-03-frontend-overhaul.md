# Freeloader Frontend Overhaul Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the monolithic Freeloader frontend and narrow browser handoff with a quiet, keyboard-first, accessible download workstation that supports generic host adapters, batches, persistence, and Chromium plus Firefox capture flows without weakening local-first guarantees.

**Architecture:** Keep the Rust download engine and Tauri v2 shell as the boundary, then add a typed frontend application layer: protocol DTOs, queue store, parser and formatter modules, focused UI surfaces, and a virtualized table. Add a resolver registry with direct HTTP fallback and a `fuckingfast.co` adapter, expose only the smallest new Tauri commands needed for batch control and recovery, and share one token stylesheet plus message contracts between desktop and extensions.

**Tech Stack:** Tauri v2, Rust, React 19, TypeScript 5, Vite, pnpm workspaces, hand-authored CSS, `motion/react`, Radix Primitives, `@tanstack/react-virtual`, Vitest, Testing Library, and `axe` assertions. No Tailwind, full component kit, remote assets, telemetry, cloud service, macOS support, or new dependency without an explicit task justification.

## Global Constraints

- Work only on `plan/frontend-overhaul-2026-08-03`, created from `main`; leave `main` untouched.
- Use Conventional Commits with DCO sign-off: `Signed-off-by: Leon Huber <leonerwinhube@gmail.com>`.
- Every task finishes with `cargo xtask check`, which must run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, frontend typecheck, and frontend build.
- Keep Tauri v2, Rust, React 19, TypeScript 5, Vite, pnpm workspaces, strict CSP, GPL-3.0-or-later, and Windows/Linux only.
- Keep no account, no cloud, no telemetry, zero unsolicited outbound requests, no remote assets, system typefaces only, and refusal to forward cookies or credentials.
- Keep deferred features deferred: multi-connection segmentation, bandwidth limiting, tray, checksums, and outbound update checks. The UI must not imply they exist.
- Use system UI stack `