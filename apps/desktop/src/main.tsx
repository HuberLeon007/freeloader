// SPDX-License-Identifier: GPL-3.0-or-later
import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  ArrowUpRight,
  Check,
  CheckCircle2,
  CircleAlert,
  Download,
  ExternalLink,
  FolderOpen,
  Inbox,
  LoaderCircle,
  Monitor,
  Moon,
  RotateCcw,
  Search,
  Settings,
  ShieldCheck,
  Sun,
  Trash2,
  X,
  Zap,
} from "lucide-react";
import { Onboarding } from "./onboarding";
import { readTheme, resolveTheme, THEME_KEY, type ThemeMode } from "./theme";
import "./styles.css";

type Status = "queued" | "downloading" | "completed" | "failed";
type ViewKey = "all" | "active" | "completed" | "failed";

type DownloadItem = {
  id: string;
  url: string;
  name: string;
  status: Status;
  downloaded: number;
  total: number | null;
  destination: string;
  speed: number;
  error: string | null;
};

type ProgressPayload = { id: string; downloaded: number; total: number | null };
type CompletePayload = { id: string; path: string };
type ErrorPayload = { id: string; message: string };
type Sample = { at: number; bytes: number };

const ONBOARDING_KEY = "freeloader.onboarding";
const DESTINATION_KEY = "freeloader.destination";
const RELEASES_URL = "https://github.com/HuberLeon007/freeloader/releases";

const BYTE_UNITS = ["B", "KB", "MB", "GB", "TB"] as const;

function formatBytes(value: number | null): string {
  if (value === null || !Number.isFinite(value)) return "--";
  let amount = Math.max(value, 0);
  let unit = 0;
  while (amount >= 1024 && unit < BYTE_UNITS.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  const label = BYTE_UNITS[unit] ?? "B";
  const digits = unit === 0 ? 0 : amount >= 100 ? 0 : amount >= 10 ? 1 : 2;
  return `${amount.toFixed(digits)} ${label}`;
}

function formatSpeed(bytesPerSecond: number): string {
  if (!Number.isFinite(bytesPerSecond) || bytesPerSecond <= 0) return "--";
  return `${formatBytes(bytesPerSecond)}/s`;
}

function formatEta(item: DownloadItem): string {
  if (item.total === null || item.speed <= 0) return "--";
  const remaining = item.total - item.downloaded;
  if (remaining <= 0) return "almost done";
  const seconds = Math.round(remaining / item.speed);
  if (seconds < 60) return `${seconds}s left`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m left`;
  return `${(seconds / 3600).toFixed(1)}h left`;
}

function percentOf(item: DownloadItem): number {
  if (item.status === "completed") return 100;
  if (item.total === null || item.total <= 0) return 0;
  return Math.min(100, Math.round((item.downloaded / item.total) * 100));
}

function parentDirectory(path: string): string {
  const cut = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return cut > 0 ? path.slice(0, cut) : path;
}

/** Join using the separator the destination already speaks. */
function joinPath(base: string, child: string): string {
  if (base.length === 0) return child;
  const separator = base.includes("\\") && !base.includes("/") ? "\\" : "/";
  const trimmed = base.endsWith("/") || base.endsWith("\\") ? base.slice(0, -1) : base;
  return `${trimmed}${separator}${child}`;
}

function filenameFrom(rawUrl: string): string {
  try {
    const parsed = new URL(rawUrl);
    const segments = parsed.pathname.split("/").filter(Boolean);
    const last = segments.length > 0 ? segments[segments.length - 1] : undefined;
    return decodeURIComponent(last ?? "") || parsed.hostname || "download";
  } catch {
    return "download";
  }
}

function extensionOf(name: string): string {
  const dot = name.lastIndexOf(".");
  if (dot <= 0 || dot === name.length - 1) return "file";
  return name.slice(dot + 1).toLowerCase().slice(0, 4);
}

const VIEWS: { key: ViewKey; label: string }[] = [
  { key: "all", label: "All files" },
  { key: "active", label: "Active" },
  { key: "completed", label: "Completed" },
  { key: "failed", label: "Failed" },
];

function viewIcon(key: ViewKey): React.JSX.Element {
  if (key === "active") return <Zap size={16} aria-hidden="true" />;
  if (key === "completed") return <CheckCircle2 size={16} aria-hidden="true" />;
  if (key === "failed") return <CircleAlert size={16} aria-hidden="true" />;
  return <Inbox size={16} aria-hidden="true" />;
}

function StatusPill({ item }: { item: DownloadItem }): React.JSX.Element {
  if (item.status === "completed") {
    return (
      <span className="pill pill-done">
        <CheckCircle2 size={13} aria-hidden="true" />
        Done
      </span>
    );
  }
  if (item.status === "failed") {
    return (
      <span className="pill pill-failed">
        <CircleAlert size={13} aria-hidden="true" />
        Failed
      </span>
    );
  }
  if (item.status === "downloading") {
    return (
      <span className="pill pill-active">
        <LoaderCircle className="spin" size={13} aria-hidden="true" />
        Downloading
      </span>
    );
  }
  return <span className="pill pill-queued">Queued</span>;
}

function App(): React.JSX.Element {
  const [themeMode, setThemeMode] = useState<ThemeMode>(readTheme);
  const [view, setView] = useState<ViewKey>("all");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [onboarding, setOnboarding] = useState(
    () => localStorage.getItem(ONBOARDING_KEY) !== "done",
  );
  const [query, setQuery] = useState("");
  const [url, setUrl] = useState("");
  const [destination, setDestination] = useState(() => localStorage.getItem(DESTINATION_KEY) ?? "");
  const [items, setItems] = useState<DownloadItem[]>([]);
  const [notice, setNotice] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [browsers, setBrowsers] = useState<string[]>([]);

  const urlRef = useRef<HTMLInputElement>(null);
  const filterRef = useRef<HTMLInputElement>(null);
  const drawerCloseRef = useRef<HTMLButtonElement>(null);
  const samples = useRef(new Map<string, Sample>());

  const theme = resolveTheme(themeMode);
  const validUrl = /^https?:\/\/[^\s]+$/i.test(url.trim());

  const counts = useMemo(
    () => ({
      all: items.length,
      active: items.filter((item) => item.status === "downloading" || item.status === "queued")
        .length,
      completed: items.filter((item) => item.status === "completed").length,
      failed: items.filter((item) => item.status === "failed").length,
    }),
    [items],
  );

  const throughput = useMemo(
    () =>
      items
        .filter((item) => item.status === "downloading")
        .reduce((total, item) => total + item.speed, 0),
    [items],
  );

  const storedBytes = useMemo(
    () =>
      items
        .filter((item) => item.status === "completed")
        .reduce((total, item) => total + item.downloaded, 0),
    [items],
  );

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return items.filter((item) => {
      const matchesView =
        view === "all" ||
        (view === "active" && (item.status === "downloading" || item.status === "queued")) ||
        (view === "completed" && item.status === "completed") ||
        (view === "failed" && item.status === "failed");
      if (!matchesView) return false;
      if (needle.length === 0) return true;
      return item.name.toLowerCase().includes(needle) || item.url.toLowerCase().includes(needle);
    });
  }, [items, query, view]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
  }, [theme]);

  useEffect(() => {
    if (themeMode !== "system" || !window.matchMedia) return;
    const media = window.matchMedia("(prefers-color-scheme: light)");
    const sync = (): void => setThemeMode("system");
    media.addEventListener("change", sync);
    return () => media.removeEventListener("change", sync);
  }, [themeMode]);

  // A relative default would resolve against the process working directory, so
  // the OS gets asked instead.
  useEffect(() => {
    if (destination.trim().length > 0) return;
    let cancelled = false;
    void (async () => {
      try {
        const resolved = await invoke<string>("default_download_dir");
        if (!cancelled && resolved.length > 0) setDestination(resolved);
      } catch {
        /* setup asks for a folder anyway */
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [destination]);

  useEffect(() => {
    if (destination.length === 0) return;
    localStorage.setItem(DESTINATION_KEY, destination);
  }, [destination]);

  useEffect(() => {
    const unlistenProgress = listen<ProgressPayload>("download-progress", (event) => {
      const payload = event.payload;
      const now = Date.now();
      const previous = samples.current.get(payload.id);
      samples.current.set(payload.id, { at: now, bytes: payload.downloaded });
      const elapsed = previous ? (now - previous.at) / 1000 : 0;
      const delta = previous ? payload.downloaded - previous.bytes : 0;
      const instant = elapsed > 0.05 && delta > 0 ? delta / elapsed : null;
      setItems((current) =>
        current.map((item) =>
          item.id === payload.id
            ? {
                ...item,
                status: "downloading",
                downloaded: payload.downloaded,
                total: payload.total,
                speed: instant === null ? item.speed : item.speed * 0.7 + instant * 0.3,
              }
            : item,
        ),
      );
    });

    const unlistenComplete = listen<CompletePayload>("download-complete", (event) => {
      const payload = event.payload;
      samples.current.delete(payload.id);
      setItems((current) =>
        current.map((item) =>
          item.id === payload.id
            ? {
                ...item,
                status: "completed",
                destination: payload.path,
                total: item.total ?? item.downloaded,
                speed: 0,
                error: null,
              }
            : item,
        ),
      );
    });

    const unlistenError = listen<ErrorPayload>("download-error", (event) => {
      const payload = event.payload;
      samples.current.delete(payload.id);
      setItems((current) =>
        current.map((item) =>
          item.id === payload.id
            ? { ...item, status: "failed", speed: 0, error: payload.message }
            : item,
        ),
      );
      setNotice(payload.message);
    });

    return () => {
      void unlistenProgress.then((dispose) => dispose());
      void unlistenComplete.then((dispose) => dispose());
      void unlistenError.then((dispose) => dispose());
    };
  }, []);

  const detectBrowsers = useCallback(async (): Promise<void> => {
    try {
      const found = await invoke<string[]>("detect_browsers");
      setBrowsers(found.map((entry) => entry.toLowerCase()));
    } catch {
      setBrowsers([]);
    }
  }, []);

  const browseForFolder = useCallback(async (): Promise<void> => {
    try {
      const picked = await invoke<string | null>("pick_download_dir");
      if (picked !== null && picked.length > 0) setDestination(picked);
    } catch {
      setNotice("The system folder picker did not open.");
    }
  }, []);

  const startDownload = useCallback(
    async (rawUrl: string): Promise<void> => {
      const trimmed = rawUrl.trim();
      if (!/^https?:\/\/[^\s]+$/i.test(trimmed)) return;
      setSubmitting(true);
      setNotice(null);
      const name = filenameFrom(trimmed);
      const target = joinPath(destination, name);
      const id = crypto.randomUUID();
      setItems((current) => [
        {
          id,
          url: trimmed,
          name,
          status: "queued",
          downloaded: 0,
          total: null,
          destination: target,
          speed: 0,
          error: null,
        },
        ...current,
      ]);
      try {
        const result = await invoke<{ id: string; path: string }>("add_download", {
          input: {
            url: trimmed,
            destinationPath: target,
            clientRequestId: id,
          },
        });
        setItems((current) =>
          current.map((item) => (item.id === id ? { ...item, destination: result.path } : item)),
        );
      } catch (cause) {
        const message =
          cause instanceof Error
            ? cause.message
            : typeof cause === "string"
              ? cause
              : "Could not start the download. Check the URL and try again.";
        setItems((current) =>
          current.map((item) =>
            item.id === id ? { ...item, status: "failed", error: message } : item,
          ),
        );
        setNotice(message);
      } finally {
        setSubmitting(false);
      }
    },
    [destination],
  );

  const revealItem = useCallback(async (item: DownloadItem): Promise<void> => {
    try {
      await invoke("open_in_file_manager", { path: parentDirectory(item.destination) });
    } catch {
      setNotice("Could not open the folder on this system.");
    }
  }, []);

  const removeItem = useCallback((id: string): void => {
    samples.current.delete(id);
    setItems((current) => current.filter((item) => item.id !== id));
  }, []);

  const clearFinished = useCallback((): void => {
    setItems((current) => current.filter((item) => item.status !== "completed"));
  }, []);

  useEffect(() => {
    if (onboarding) return;
    const onKeyDown = (event: KeyboardEvent): void => {
      const target = event.target as HTMLElement | null;
      const typing = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
      if (event.key === "Escape") {
        setSettingsOpen(false);
        return;
      }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "n") {
        event.preventDefault();
        urlRef.current?.focus();
        return;
      }
      if (event.key === "/" && !typing) {
        event.preventDefault();
        filterRef.current?.focus();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onboarding]);

  useEffect(() => {
    if (!settingsOpen) return;
    const previous = document.activeElement as HTMLElement | null;
    drawerCloseRef.current?.focus();
    void detectBrowsers();
    return () => previous?.focus();
  }, [detectBrowsers, settingsOpen]);

  useEffect(() => {
    if (notice === null) return;
    const timer = window.setTimeout(() => setNotice(null), 8000);
    return () => window.clearTimeout(timer);
  }, [notice]);

  const pickTheme = useCallback((mode: ThemeMode): void => {
    localStorage.setItem(THEME_KEY, mode);
    setThemeMode(mode);
  }, []);

  const cycleTheme = (): void => pickTheme(theme === "dark" ? "light" : "dark");

  const finishOnboarding = useCallback((): void => {
    localStorage.setItem(ONBOARDING_KEY, "done");
    setOnboarding(false);
  }, []);

  if (onboarding) {
    return (
      <Onboarding
        destination={destination}
        onDestinationChange={setDestination}
        themeMode={themeMode}
        onThemeChange={pickTheme}
        onFinish={finishOnboarding}
      />
    );
  }

  return (
    <div className="shell">
      <aside className="rail">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            <Download size={16} />
          </span>
          <span className="brand-name">Freeloader</span>
        </div>

        <nav className="rail-nav" aria-label="Views">
          {VIEWS.map((entry) => (
            <button
              key={entry.key}
              className={view === entry.key ? "rail-link rail-link-on" : "rail-link"}
              aria-current={view === entry.key ? "page" : undefined}
              onClick={() => setView(entry.key)}
            >
              {viewIcon(entry.key)}
              <span className="rail-label">{entry.label}</span>
              <span className="rail-count">{counts[entry.key]}</span>
            </button>
          ))}
        </nav>

        <div className="rail-foot">
          <div className="assurance">
            <ShieldCheck size={15} aria-hidden="true" />
            <div>
              <strong>No network surface</strong>
              <span>No server, no telemetry</span>
            </div>
          </div>
        </div>
      </aside>

      <div className="stage">
        <header className="stage-top">
          <div className="stage-title">
            <h1>{VIEWS.find((entry) => entry.key === view)?.label ?? "All files"}</h1>
            <p>
              {visible.length} shown
              {counts.active > 0 ? ` · ${counts.active} in flight` : ""}
            </p>
          </div>

          <label className="finder">
            <Search size={15} aria-hidden="true" />
            <span className="sr-only">Filter downloads</span>
            <input
              ref={filterRef}
              value={query}
              placeholder="Filter"
              onChange={(event) => setQuery(event.target.value)}
            />
            <kbd>/</kbd>
          </label>

          <div className="stage-actions">
            <button
              className="icon-button"
              aria-label={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
              onClick={cycleTheme}
            >
              {theme === "dark" ? <Sun size={17} /> : <Moon size={17} />}
            </button>
            <button
              className="icon-button"
              aria-label="Open settings"
              onClick={() => setSettingsOpen(true)}
            >
              <Settings size={17} />
            </button>
          </div>
        </header>

        <div className="stage-body">
          <section className="composer" aria-label="Add a download">
            <div className="composer-main">
              <label className="composer-url">
                <span className="sr-only">Download URL</span>
                <input
                  ref={urlRef}
                  value={url}
                  type="url"
                  spellCheck={false}
                  placeholder="Paste a direct HTTP or HTTPS link"
                  onChange={(event) => setUrl(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key !== "Enter") return;
                    void startDownload(url);
                    setUrl("");
                  }}
                />
              </label>
              <button
                className="button primary"
                disabled={!validUrl || submitting}
                onClick={() => {
                  void startDownload(url);
                  setUrl("");
                }}
              >
                {submitting ? "Starting" : "Download"}
                <ArrowUpRight size={15} aria-hidden="true" />
              </button>
            </div>
            <div className="composer-meta">
              <label className="composer-dest">
                <FolderOpen size={14} aria-hidden="true" />
                <span className="sr-only">Destination folder</span>
                <input
                  className="mono"
                  value={destination}
                  onChange={(event) => setDestination(event.target.value)}
                />
              </label>
              <button className="linkish" onClick={() => void browseForFolder()}>
                Browse
              </button>
              <span className="composer-hint">
                Cookies and authorization headers are never forwarded.
              </span>
            </div>
          </section>

          <section className="summary" aria-label="Session summary">
            <div className="summary-card">
              <span className="summary-key">Throughput</span>
              <strong className="summary-value">{formatSpeed(throughput)}</strong>
            </div>
            <div className="summary-card">
              <span className="summary-key">Written to disk</span>
              <strong className="summary-value">{formatBytes(storedBytes)}</strong>
            </div>
            <div className="summary-card">
              <span className="summary-key">Queue</span>
              <strong className="summary-value">
                {counts.active} active · {counts.failed} failed
              </strong>
            </div>
            <button
              className="button quiet summary-action"
              disabled={counts.completed === 0}
              onClick={clearFinished}
            >
              <Trash2 size={14} aria-hidden="true" />
              Clear completed
            </button>
          </section>

          {notice !== null && (
            <div className="notice" role="alert">
              <CircleAlert size={16} aria-hidden="true" />
              <span>{notice}</span>
              <button className="icon-button" aria-label="Dismiss" onClick={() => setNotice(null)}>
                <X size={14} />
              </button>
            </div>
          )}

          {visible.length === 0 ? (
            <section className="empty">
              <span className="empty-glyph" aria-hidden="true">
                <Download size={22} />
              </span>
              <h2>{items.length === 0 ? "Nothing queued yet" : "No matches here"}</h2>
              <p>
                {items.length === 0
                  ? "Paste a link above and Freeloader will stream it straight to disk."
                  : "Try a different filter, or head back to all files."}
              </p>
              {items.length > 0 && (
                <button
                  className="button ghost"
                  onClick={() => {
                    setQuery("");
                    setView("all");
                  }}
                >
                  Show everything
                </button>
              )}
            </section>
          ) : (
            <ul className="queue" aria-label="Downloads">
              {visible.map((item) => {
                const percent = percentOf(item);
                return (
                  <li className={`row row-${item.status}`} key={item.id}>
                    <span className="row-type mono" aria-hidden="true">
                      {extensionOf(item.name)}
                    </span>

                    <div className="row-body">
                      <div className="row-head">
                        <strong className="row-name" title={item.name}>
                          {item.name}
                        </strong>
                        <StatusPill item={item} />
                      </div>
                      <p className="row-path mono" title={item.destination}>
                        {item.destination}
                      </p>
                      {item.error !== null && <p className="row-error">{item.error}</p>}
                    </div>

                    <div className="row-progress">
                      <div
                        className="track"
                        role="progressbar"
                        aria-valuemin={0}
                        aria-valuemax={100}
                        aria-valuenow={percent}
                        aria-label={`${item.name} progress`}
                      >
                        <span style={{ width: `${percent}%` }} />
                      </div>
                      <div className="row-stats mono">
                        <span>{percent}%</span>
                        <span>
                          {formatBytes(item.downloaded)} / {formatBytes(item.total)}
                        </span>
                        <span>
                          {item.status === "downloading"
                            ? formatSpeed(item.speed)
                            : formatEta(item)}
                        </span>
                      </div>
                    </div>

                    <div className="row-actions">
                      {item.status === "completed" && (
                        <button
                          className="icon-button"
                          aria-label={`Show ${item.name} in file manager`}
                          onClick={() => void revealItem(item)}
                        >
                          <ExternalLink size={15} />
                        </button>
                      )}
                      {item.status === "failed" && (
                        <button
                          className="icon-button"
                          aria-label={`Retry ${item.name}`}
                          onClick={() => {
                            removeItem(item.id);
                            void startDownload(item.url);
                          }}
                        >
                          <RotateCcw size={15} />
                        </button>
                      )}
                      <button
                        className="icon-button"
                        aria-label={`Remove ${item.name}`}
                        onClick={() => removeItem(item.id)}
                      >
                        <X size={15} />
                      </button>
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </div>
      </div>

      {settingsOpen && (
        <div
          className="scrim"
          onClick={(event) => {
            if (event.target === event.currentTarget) setSettingsOpen(false);
          }}
        >
          <section
            className="drawer"
            role="dialog"
            aria-modal="true"
            aria-labelledby="settings-title"
          >
            <header className="drawer-head">
              <div>
                <p className="kicker">Preferences</p>
                <h2 id="settings-title">Settings</h2>
              </div>
              <button
                ref={drawerCloseRef}
                className="icon-button"
                aria-label="Close settings"
                onClick={() => setSettingsOpen(false)}
              >
                <X size={18} />
              </button>
            </header>

            <div className="drawer-block">
              <label className="field">
                <span className="field-label">Default destination</span>
                <input
                  className="input mono"
                  value={destination}
                  onChange={(event) => setDestination(event.target.value)}
                />
              </label>
              <button className="linkish" onClick={() => void browseForFolder()}>
                Choose a folder
              </button>
            </div>

            <div className="drawer-block">
              <h3>Appearance</h3>
              <div className="segmented" role="group" aria-label="Theme">
                <button
                  className={themeMode === "system" ? "segment segment-on" : "segment"}
                  onClick={() => pickTheme("system")}
                >
                  <Monitor size={14} aria-hidden="true" />
                  System
                </button>
                <button
                  className={themeMode === "dark" ? "segment segment-on" : "segment"}
                  onClick={() => pickTheme("dark")}
                >
                  <Moon size={14} aria-hidden="true" />
                  Dark
                </button>
                <button
                  className={themeMode === "light" ? "segment segment-on" : "segment"}
                  onClick={() => pickTheme("light")}
                >
                  <Sun size={14} aria-hidden="true" />
                  Light
                </button>
              </div>
            </div>

            <div className="drawer-block">
              <h3>Browser integration</h3>
              <p className="drawer-copy">
                Only executable locations on your PATH are checked. Profiles, history, cookies and
                credentials are never touched.
              </p>
              {browsers.length > 0 ? (
                <ul className="detected">
                  {browsers.map((browser) => (
                    <li key={browser}>
                      <Check size={14} aria-hidden="true" />
                      <span>{browser}</span>
                      <a href={RELEASES_URL} target="_blank" rel="noreferrer">
                        Get extension
                        <ArrowUpRight size={13} aria-hidden="true" />
                      </a>
                    </li>
                  ))}
                </ul>
              ) : (
                <button className="button ghost" onClick={() => void detectBrowsers()}>
                  Detect browsers
                </button>
              )}
            </div>

            <footer className="drawer-foot">
              <span className="drawer-version mono">v0.1.0 · GPL-3.0-or-later</span>
              <button className="button primary" onClick={() => setSettingsOpen(false)}>
                Done
              </button>
            </footer>
          </section>
        </div>
      )}
    </div>
  );
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}
