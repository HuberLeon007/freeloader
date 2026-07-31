// SPDX-License-Identifier: GPL-3.0-or-later
//! Developer task runner for the Freeloader workspace.
//!
//! `cargo dev` installs the frontend dependencies when they are missing,
//! generates the bundle icons and hands over to the Tauri CLI, which compiles
//! the Rust core and opens the desktop window. `cargo dev build` bundles
//! installers instead. The crate deliberately has no dependencies so the alias
//! stays instant on a cold checkout.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_else(|| "dev".to_owned());
    let root = repo_root();

    let outcome = match task.as_str() {
        "dev" => dev(&root),
        "build" => build(&root),
        "icons" => icons(&root),
        "setup" => install_dependencies(&root),
        other => Err(format!(
            "unknown task `{other}`. Available tasks: dev, build, icons, setup."
        )),
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("\nxtask: {message}");
            eprintln!("Freeloader needs Node 22 and pnpm 10. `corepack enable` provides pnpm.");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root, one directory above this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Start the app in development mode.
fn dev(root: &Path) -> Result<(), String> {
    ensure_dependencies(root)?;
    icons(root)?;
    step("starting Freeloader, the window opens once the Rust core is built");
    run(root, "pnpm", &["--dir", "apps/desktop", "run", "app:dev"])
}

/// Bundle installers for the current platform.
fn build(root: &Path) -> Result<(), String> {
    ensure_dependencies(root)?;
    icons(root)?;
    step("bundling installers into target/release/bundle");
    run(root, "pnpm", &["--dir", "apps/desktop", "run", "app:build"])
}

/// Regenerate the bundle icons that `tauri::generate_context!` reads.
fn icons(root: &Path) -> Result<(), String> {
    step("generating application icons");
    run(root, "node", &["scripts/generate-icons.mjs"])
}

fn ensure_dependencies(root: &Path) -> Result<(), String> {
    let installed =
        root.join("node_modules").is_dir() && root.join("apps/desktop/node_modules").is_dir();
    if installed {
        return Ok(());
    }
    install_dependencies(root)
}

fn install_dependencies(root: &Path) -> Result<(), String> {
    step("installing frontend dependencies");
    run(root, "pnpm", &["install"])
}

fn step(message: &str) {
    println!("\x1b[36m>\x1b[0m {message}");
}

/// Run a program from the workspace root and wait for it.
///
/// Windows installs pnpm and other Node tooling as `.cmd` shims, which
/// `Command` will not resolve from the bare name, so both spellings are tried
/// before reporting the program as missing.
fn run(root: &Path, name: &str, args: &[&str]) -> Result<(), String> {
    for spelling in [name.to_owned(), format!("{name}.cmd")] {
        match Command::new(spelling).current_dir(root).args(args).status() {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => return Err(format!("`{name} {}` failed with {status}", args.join(" "))),
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("could not start `{name}`: {error}")),
        }
    }
    Err(format!("`{name}` was not found on PATH"))
}
