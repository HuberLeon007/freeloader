// SPDX-License-Identifier: GPL-3.0-or-later

/// Theme preference as chosen by the user. `system` tracks the OS.
export type ThemeMode = "system" | "dark" | "light";

export const THEME_KEY = "freeloader.theme";

/** Turn a preference into the theme that should actually render. */
export function resolveTheme(mode: ThemeMode): "dark" | "light" {
  if (mode !== "system") return mode;
  if (typeof window === "undefined" || !window.matchMedia) return "dark";
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

/** Read the stored preference, defaulting to following the OS. */
export function readTheme(): ThemeMode {
  const stored = localStorage.getItem(THEME_KEY);
  return stored === "dark" || stored === "light" || stored === "system" ? stored : "system";
}
