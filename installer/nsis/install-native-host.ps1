# SPDX-License-Identifier: GPL-3.0-or-later
[CmdletBinding(SupportsShouldProcess)]
param(
  [Parameter(Mandatory=$true)][string]$ManifestPath,
  [string[]]$Browsers = @("Google\\Chrome", "Microsoft\\Edge", "BraveSoftware\\Brave-Browser", "Chromium", "Vivaldi", "Mozilla")
)

$hostName = "io.freeloader.host"
if (-not (Test-Path -LiteralPath $ManifestPath -PathType Leaf)) { throw "Manifest not found: $ManifestPath" }
$manifest = (Resolve-Path -LiteralPath $ManifestPath).Path
foreach ($browser in $Browsers) {
  $key = "HKCU:\Software\$browser\NativeMessagingHosts\$hostName"
  if ($PSCmdlet.ShouldProcess($key, "Register native host")) {
    New-Item -Path $key -Force | Out-Null
    New-ItemProperty -Path $key -Name '(default)' -Value $manifest -PropertyType String -Force | Out-Null
  }
}
Write-Output "Registered $hostName for $($Browsers.Count) browser families."
