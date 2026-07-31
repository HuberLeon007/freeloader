#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

HOST_NAME="io.freeloader.host"
HOST_BINARY="${FREELOADER_NATIVE_HOST_BINARY:-${HOME}/.local/bin/freeloader-native-host}"
MANIFEST_DIRS=(
  "${HOME}/.config/google-chrome/NativeMessagingHosts"
  "${HOME}/.config/chromium/NativeMessagingHosts"
  "${HOME}/.config/microsoft-edge/NativeMessagingHosts"
  "${HOME}/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts"
  "${HOME}/.config/vivaldi/NativeMessagingHosts"
  "${HOME}/.mozilla/native-messaging-hosts"
)

if [[ ! -x "${HOST_BINARY}" ]]; then
  echo "Native host binary not found: ${HOST_BINARY}" >&2
  exit 2
fi

for dir in "${MANIFEST_DIRS[@]}"; do
  mkdir -p "${dir}"
  umask 077
  printf '{"name":"%s","description":"Freeloader Native Messaging host","path":"%s","type":"stdio","allowed_origins":[] ,"allowed_extensions":[]}' "${HOST_NAME}" "${HOST_BINARY}" > "${dir}/${HOST_NAME}.json"
done

echo "Native host manifests installed. Exact extension IDs must be added before browser communication is enabled."
