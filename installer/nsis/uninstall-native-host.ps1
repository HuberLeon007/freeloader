# SPDX-License-Identifier: GPL-3.0-or-later
[CmdletBinding(SupportsShouldProcess)]
param([string[]]$Browsers = @("Google\\Chrome", "Microsoft\\Edge", "BraveSoftware\\Brave-Browser", "Chromium", "Vivaldi", "Mozilla"))
$hostName = "io.freeloader.host"
foreach ($browser in $Browsers) {
  $key = "HKCU:\Software\$browser\NativeMessagingHosts\$hostName"
  if (Test-Path $key -and $PSCmdlet.ShouldProcess($key, "Remove native host registration")) {
    Remove-Item -LiteralPath $key -Recurse -Force
  }
}
