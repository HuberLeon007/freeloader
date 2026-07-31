// SPDX-License-Identifier: GPL-3.0-or-later
//! Repository automation, so a fresh clone is one command away from a window.
//!
//! The desktop app is a Rust workspace and a pnpm workspace at the same time,
//! which means starting it involves a Node install, an icon generation step and
//! the Tauri CLI, in that order. Getting the order wrong fails as a Rust build
//! error about missing icons, which is a terrible first impression. `cargo dev`
//! is an alias for `cargo run --package xtask -- dev` and does the whole thing.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const HELP: &str = "\
Freeloader repository tasks

  cargo dev            Install what is missing, then open the desktop app
  cargo xtask dev      The same thing, spelled out
  cargo xtask build    Bundle installers for this platform
  cargo xtask check    Run the four gates CI runs
  cargo xtask icons    Regenerate the bundle icons
";

fn main() -> ExitCode {
    let task = env::args().nth(1).unwrap_or_else(|| String::from("dev"));
    let outcome = match task.as_str() {
        "dev" => dev(),
        "build" => build(),
        "check" => check(),
        "icons" => icons(&workspace_root()),
        "help" | "-h" | "--help" => {
            print!("{HELP}");
            Ok(())
        }
        other => Err(format!("unknown task `{other}`\n\n{HELP}")),
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("\nx {message}");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root, resolved at compile time so the task works from any
/// subdirectory.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn step(message: &str) {
    println!("\n:: {message}");
}

/// Run a command and turn a non-zero exit into a readable error.
///
/// pnpm ships as `pnpm.cmd` on Windows, which `CreateProcess` will not resolve,
/// so it goes through the shell there.
fn run(program: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    let mut command = if cfg!(windows) && program == "pnpm" {
        let mut shell = Command::new("cmd");
        shell.arg("/C").arg(program);
        shell
    } else {
        Command::new(program)
    };

    let status = command
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|error| {
            format!(
                "could not start `{program}`: {error}\n  Install it and make sure it is on your PATH."
            )
        })?;

    if status.success() {
        return Ok(());
    }
    let printable = args.join(" ");
    Err(format!("`{program} {printable}` failed"))
}

/// Install the Node side only when it is actually missing.
fn ensure_dependencies(root: &Path) -> Result<(), String> {
    if root.join("node_modules").is_dir() && root.join("apps/desktop/node_modules").is_dir() {
        return Ok(());
    }
    step("Installing workspace dependencies (first run only)");
    run("pnpm", &["install"], root)
}

/// Generate the bundle icons that `tauri::generate_context!` reads at compile
/// time. Nothing binary is checked in, so this has to happen before cargo runs.
fn icons(root: &Path) -> Result<(), String> {
    step("Generating bundle icons");
    run("node", &["scripts/generate-icons.mjs"], root)
}

fn dev() -> Result<(), String> {
    let root = workspace_root();
    ensure_dependencies(&root)?;
    icons(&root)?;
    step("Starting Freeloader. The first build takes a few minutes; the window opens on its own.");
    run("pnpm", &["--dir", "apps/desktop", "run", "app:dev"], &root)
}

fn build() -> Result<(), String> {
    let root = workspace_root();
    ensure_dependencies(&root)?;
    icons(&root)?;
    step("Bundling installers");
    run(
        "pnpm",
        &["--dir", "apps/desktop", "run", "app:build"],
        &root,
    )
}

fn check() -> Result<(), String> {
    let root = workspace_root();
    ensure_dependencies(&root)?;
    icons(&root)?;
    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));

    step("cargo fmt --all --check");
    run(&cargo, &["fmt", "--all", "--check"], &root)?;

    step("cargo clippy --workspace --all-targets -- -D warnings");
    run(
        &cargo,
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &root,
    )?;

    step("cargo test --workspace");
    run(&cargo, &["test", "--workspace"], &root)?;

    step("frontend typecheck and build");
    run("pnpm", &["--dir", "apps/desktop", "run", "build"], &root)
}
