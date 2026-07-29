# SPDX-License-Identifier: GPL-3.0-or-later
[CmdletBinding()]
param([switch]$SkipFrontend)
$ErrorActionPreference = "Stop"

function Require-Command([string]$Name, [string]$Hint) {
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) { throw "$Name is required but was not found. $Hint" }
}

Require-Command cargo "Install Rust from https://rustup.rs/"
Require-Command rustup "Install Rust from https://rustup.rs/"
Require-Command node "Install Node.js 22 LTS."

if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
  $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
  if (Test-Path $vswhere) {
    $installation = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($installation) {
      $devCmd = Join-Path $installation "Common7\Tools\VsDevCmd.bat"
      if (Test-Path $devCmd) {
        $envDump = cmd.exe /c "`"$devCmd`" -arch=x64 -host_arch=x64 && set"
        foreach ($line in $envDump) { if ($line -match "^(.*?)=(.*)$") { Set-Item -Path "Env:$($Matches[1])" -Value $Matches[2] } }
      }
    }
  }
}
if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) { throw "MSVC link.exe is unavailable. Install Visual Studio 2022 Build Tools with Desktop development with C++ and the Windows 10/11 SDK, then rerun from Developer PowerShell." }

cargo fmt --all --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not $SkipFrontend) {
  if (Get-Command pnpm -ErrorAction SilentlyContinue) {
    $pnpmCommand = "pnpm"
  } else {
    Require-Command corepack "Install Node.js 22 LTS, then enable Corepack."
    corepack enable
    $pnpmCommand = "corepack"
  }
  if ($pnpmCommand -eq "pnpm") { & pnpm install --frozen-lockfile=false } else { & corepack pnpm install --frozen-lockfile=false }
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  if ($pnpmCommand -eq "pnpm") { & pnpm --dir apps/desktop typecheck } else { & corepack pnpm --dir apps/desktop typecheck }
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  if ($pnpmCommand -eq "pnpm") { & pnpm --dir apps/desktop build } else { & corepack pnpm --dir apps/desktop build }
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "Freeloader portable checks passed. Native packaging still requires the target OS and architecture." -ForegroundColor Green
