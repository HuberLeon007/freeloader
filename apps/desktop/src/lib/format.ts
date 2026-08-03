// SPDX-License-Identifier: GPL-3.0-or-later

const UNITS = ["B", "KB", "MB", "GB", "TB"] as const;

export function formatBytes(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value)) return "--";
  let amount = Math.max(0, value);
  let unit = 0;
  while (amount >= 1024 && unit < UNITS.length - 1) { amount /= 1024; unit += 1; }
  const digits = unit === 0 ? 0 : amount >= 100 ? 0 : amount >= 10 ? 1 : 2;
  return `${amount.toFixed(digits)} ${UNITS[unit]}`;
}

export function formatSpeed(bytesPerSecond: number | null | undefined): string {
  return bytesPerSecond != null && bytesPerSecond > 0 ? `${formatBytes(bytesPerSecond)}/s` : "--";
}

export function formatEta(seconds: number | null | undefined): string {
  if (seconds == null || !Number.isFinite(seconds) || seconds < 0) return "—";
  if (seconds < 60) return `${Math.round(seconds)}s`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
  return `${(seconds / 3600).toFixed(1)}h`;
}

export function isKnownNumeric(value: string): boolean {
  return /^\s*\d+(?:\.\d+)?(?:\s*[A-Za-z%/]+)?\s*$/.test(value);
}
