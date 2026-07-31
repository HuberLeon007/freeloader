// SPDX-License-Identifier: GPL-3.0-or-later
import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  ArrowLeft,
  ArrowRight,
  Check,
  CheckCircle2,
  CircleAlert,
  Download,
  ExternalLink,
  Folder,
  Inbox,
  LoaderCircle,
  Monitor,
  Moon,
  RotateCcw,
  Search,
  Settings,
  Sun,
  Trash2,
  X,
  Zap,
} from "lucide-react";
import "./styles.css";

type Status = "queued" | "downloading" | "completed" | "failed";
type ViewKey = "all" | "active" | "completed" | "failed";
type ThemeMode = "system" | "dark" | "light";

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

const THEME_KEY = "freeloader.theme";
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
  if (item.status !== "downloading") return "--";
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

function joinPath(directory: string, name: string): string {
  const windows = directory.includes("\\") && !directory.includes("/");
  const trimmed = directory.replace(/[\\/]+$/, "");
  return `${trimmed}${windows ? "\\" : "/"}${name}`;
}

function shortPath(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  if (parts.length <= 2) return path;
  return `…/${parts.slice(-2).join("/")}`;
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

function resolveTheme(mode: ThemeMode): "dark" | "light" {
  if (mode !== "system") return mode;
  if (typeof window === "undefined" || !window.matchMedia) return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function readTheme(): ThemeMode {
  const stored = localStorage.getItem(THEME_KEY);
  return stored === "dark" || stored === "light" || stored === "system" ? stored : "system";
}

const VIEWS: { key: ViewKey; label: string }[] = [
  { key: "all", label: "All files" },
  { key: "active", label: "Active" },
  { key: "completed", label: "Completed" },
  { key: "failed", label: "Failed" },
];

const THEME_CHOICES: { mode: ThemeMode; name: string; note: string }[] = [
  { mode: "system", name: "System", note: "Follows the OS setting" },
  { mode: "light", name: "Light", note: "Warm paper, daylight" },
  { mode: "dark", name: "Dark", note: "Low glare, long sessions" },
];

const STEPS: { key: string; label: string; title: string }[] = [
  { key: "welcome", label: "Welcome", title: "Downloads that never leave your machine." },
  { key: "location", label: "Location", title: "Where should files land?" },
  { key: "appearance", label: "Appearance", title: "Pick a look." },
  { key: "browsers", label: "Browsers", title: "Send links from your browser." },
];

function viewIcon(key: ViewKey): React.JSX.Element {
  if (key === "active") return <Zap size={15} aria-hidden="true" />;
  if (key === "completed") return <CheckCircle2 size={15} aria-hidden="true" />;
  if (key === "failed") return <CircleAlert size={15} aria-hidden="true" />;
  return <Inbox size={15} aria-hidden="true" />;
}

function StatusPill({ item }: { item: DownloadItem }): React.JSX.Element {
  if (item.status === "completed") {
    return (
      <span className="pill pill-done">
        <Check size={12} aria-hidden="true" />
        Done
      </span>
    );
  }
  if (item.status === "failed") {
    return (
      <span className="pill pill-failed">
        <CircleAlert size={12} aria-hidden="true" />
        Failed
      </span>
    );
  }
  if (item.status === "downloading") {
    return (
      <span className="pill pill-active">
        <LoaderCircle className="spin" size={12} aria-hidden="true" />
        Downloading
      </span>
    );
  }
  return <span className="pill">Queued</span>;
}

type OnboardingProps = {
  destination: string;
  suggestions: string[];
  themeMode: ThemeMode;
  browsers: string[];
  onDestination: (value: string) => void;
  onBrowse: () => void;
  onTheme: (mode: ThemeMode) => void;
  onDetect: () => void;
  onDone: () => void;
};

function Onboarding(props: OnboardingProps): React.JSX.Element | null {
  const [index, setIndex] = useState(0);
  const { onDone } = props;
  const last = index === STEPS.length - 1;

  const advance = useCallback((): void => {
    if (index >= STEPS.length - 1) {
      onDone();
      return;
    }
    setIndex(index + 1);
  }, [index, onDone]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === "Escape") {
        event.preventDefault();
        onDone();
        return;
      }
      if (event.key !== "Enter") return;
      const target = event.target;
      if (target instanceof HTMLButtonElement || target instanceof HTMLAnchorElement) return;
      event.preventDefault();
      advance();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [advance, onDone]);

  const current = STEPS[index];
  if (current === undefined) return null;

  return (
    <div className="ob">
      <aside className="ob-side">
        <div className="brand">
          <span className="mark" aria-hidden="true">
            <Download size={16} />
          </span>
          <span className="brand-name">Freeloader</span>
        </div>

        <ol className="ob-steps" aria-label={`Step ${index + 1} of ${STEPS.length}`}>
          {STEPS.map((entry, position) => {
            const state = position === index ? "on" : position < index ? "done" : "";
            return (
              <li key={entry.key} className={state} aria-current={position === index}>
                <span className="dot" aria-hidden="true">
                  {position < index ? <Check size={12} /> : position + 1}
                </span>
                <span className="ob-step-label">{entry.label}</span>
              </li>
            );
          })}
        </ol>

        <p className="ob-note">
          Nothing here is sent anywhere. Every answer is stored on this machine and can be changed
          later in settings.
        </p>
      </aside>

      <section className="ob-main">
        <div className="ob-head">
          <p className="eyebrow">
            Step {index + 1} of {STEPS.length}
          </p>
          <h1>{current.title}</h1>
        </div>

        <div className="ob-body" key={current.key}>
          {current.key === "welcome" && (
            <ul className="facts">
              <li>
                <strong>Streams straight to disk</strong>
                Large files never pass through the interface, so memory stays flat no matter how
                big the transfer is.
              </li>
              <li>
                <strong>Resumes when the server allows it</strong>
                Plain HTTP range requests. A partial file keeps its <code>.part</code> suffix until
                the last byte arrives.
              </li>
              <li>
                <strong>No account, no server, no telemetry</strong>
                The only network traffic Freeloader produces is the download you asked for.
              </li>
            </ul>
          )}

          {current.key === "location" && (
            <div className="stack">
              <div>
                <span className="lab" id="destination-label">
                  Save downloads to
                </span>
                <div className="pathbox">
                  <Folder size={16} aria-hidden="true" />
                  <input
                    className="pathinput"
                    aria-labelledby="destination-label"
                    spellCheck={false}
                    value={props.destination}
                    onChange={(event) => props.onDestination(event.target.value)}
                  />
                  <button type="button" className="btn ghost sm" onClick={props.onBrowse}>
                    Browse
                  </button>
                </div>
                {props.suggestions.length > 0 && (
                  <div className="chips">
                    {props.suggestions.map((candidate) => (
                      <button
                        type="button"
                        key={candidate}
                        className={candidate === props.destination ? "chip chip-on" : "chip"}
                        onClick={() => props.onDestination(candidate)}
                      >
                        {shortPath(candidate)}
                      </button>
                    ))}
                  </div>
                )}
              </div>
              <p className="hint">
                Freeloader writes to a temporary file in this folder and renames it once the
                transfer completes. A name that already exists gets a numbered suffix, so an
                existing file is never overwritten.
              </p>
            </div>
          )}

          {current.key === "appearance" && (
            <div className="stack">
              <div className="themes" role="radiogroup" aria-label="Appearance">
                {THEME_CHOICES.map((choice) => (
                  <button
                    type="button"
                    key={choice.mode}
                    role="radio"
                    aria-checked={props.themeMode === choice.mode}
                    className={props.themeMode === choice.mode ? "theme theme-on" : "theme"}
                    onClick={() => props.onTheme(choice.mode)}
                  >
                    <span className={`swatch swatch-${choice.mode}`} aria-hidden="true">
                      <i />
                      <i />
                      <i />
                    </span>
                    <span className="theme-name">{choice.name}</span>
                    <span className="theme-note">{choice.note}</span>
                  </button>
                ))}
              </div>
              <p className="hint">
                Contrast targets WCAG 2.2 AA in both themes. System follows the desktop preference
                and switches with it while the app is running.
              </p>
            </div>
          )}

          {current.key === "browsers" && (
            <div className="stack">
              <p className="hint">
                Optional. Freeloader checks which browser executables sit on your PATH so it can
                point you at the matching extension. Profiles, history, cookies and saved
                credentials are never read.
              </p>
              {props.browsers.length > 0 ? (
                <ul className="detected">
                  {props.browsers.map((browser) => (
                    <li key={browser}>
                      <Check size={14} aria-hidden="true" />
                      <span>{browser}</span>
                      <a href={RELEASES_URL} target="_blank" rel="noreferrer">
                        Get extension
                        <ExternalLink size={12} aria-hidden="true" />
                      </a>
                    </li>
                  ))}
                </ul>
              ) : (
                <div>
                  <button type="button" className="btn ghost" onClick={props.onDetect}>
                    Look for browsers
                  </button>
                </div>
              )}
            </div>
          )}
        </div>

        <footer className="ob-foot">
          <button
            type="button"
            className="btn ghost"
            disabled={index === 0}
            onClick={() => setIndex(index - 1)}
          >
            <ArrowLeft size={15} aria-hidden="true" />
            Back
          </button>
          <button type="button" className="btn plain" onClick={onDone}>
            Skip setup
          </button>
          <button type="button" className="btn primary" onClick={advance}>
            {last ? "Open Freeloader" : "Continue"}
            <ArrowRight size={15} aria-hidden="true" />
          </button>
        </footer>
      </section>
    </div>
  );
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
  const [destination, setDestination] = useState(
    () => localStorage.getItem(DESTINATION_KEY) ?? "",
  );
  const [systemFolder, setSystemFolder] = useState("");
  const [items, setItems] = useState<DownloadItem[]>([]);
  const [notice, setNotice] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [browsers, setBrowsers] = useState<string[]>([]);

  const urlRef = useRef<HTMLInputElement>(null);
  const filterRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLDialogElement>(null);
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

  const suggestions = useMemo(() => {
    if (systemFolder.length === 0) return [];
    return [systemFolder, joinPath(systemFolder, "Freeloader")];
  }, [systemFolder]);

  const statusLine = useMemo(() => {
    const parts = [`${visible.length} shown`];
    if (counts.active > 0) parts.push(`${counts.active} in flight`);
    if (throughput > 0) parts.push(formatSpeed(throughput));
    if (storedBytes > 0) parts.push(`${formatBytes(storedBytes)} written`);
    if (counts.failed > 0) parts.push(`${counts.failed} failed`);
    return parts.join(" · ");
  }, [counts.active, counts.failed, storedBytes, throughput, visible.length]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
  }, [theme]);

  useEffect(() => {
    if (themeMode !== "system" || !window.matchMedia) return;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const sync = (): void => setThemeMode("system");
    media.addEventListener("change", sync);
    return () => media.removeEventListener("change", sync);
  }, [themeMode]);

  useEffect(() => {
    if (destination.length === 0) return;
    localStorage.setItem(DESTINATION_KEY, destination);
  }, [destination]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      let resolved = "Downloads";
      try {
        resolved = await invoke<string>("default_download_dir");
      } catch {
        resolved = "Downloads";
      }
      if (cancelled) return;
      setSystemFolder(resolved);
      setDestination((current) => (current.length > 0 ? current : resolved));
    })();
    return () => {
      cancelled = true;
    };
  }, []);

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

  const browse = useCallback(async (): Promise<void> => {
    try {
      const chosen = await invoke<string | null>("pick_directory", { start: destination });
      if (typeof chosen === "string" && chosen.length > 0) setDestination(chosen);
    } catch {
      setNotice("The system folder picker is unavailable here. Type the path instead.");
    }
  }, [destination]);

  const startDownload = useCallback(
    async (rawUrl: string): Promise<void> => {
      const trimmed = rawUrl.trim();
      if (!/^https?:\/\/[^\s]+$/i.test(trimmed)) return;
      setSubmitting(true);
      setNotice(null);
      const name = filenameFrom(trimmed);
      const target = joinPath(destination.length > 0 ? destination : "Downloads", name);
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
          input: { url: trimmed, destinationPath: target, clientRequestId: id },
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
              : "Could not start the download. Check the link and try again.";
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
    const onKeyDown = (event: KeyboardEvent): void => {
      const target = event.target;
      const typing = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
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
  }, []);

  useEffect(() => {
    const node = dialogRef.current;
    if (node === null) return;
    if (settingsOpen && !node.open) {
      node.showModal();
      void detectBrowsers();
    }
    if (!settingsOpen && node.open) node.close();
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

  const finishOnboarding = useCallback((): void => {
    localStorage.setItem(ONBOARDING_KEY, "done");
    setOnboarding(false);
  }, []);

  if (onboarding) {
    return (
      <Onboarding
        destination={destination}
        suggestions={suggestions}
        themeMode={themeMode}
        browsers={browsers}
        onDestination={setDestination}
        onBrowse={() => void browse()}
        onTheme={pickTheme}
        onDetect={() => void detectBrowsers()}
        onDone={finishOnboarding}
      />
    );
  }

  const destinationLabel = destination.length > 0 ? shortPath(destination) : "Choose a folder";

  return (
    <div className="shell">
      <aside className="rail">
        <div className="brand">
          <span className="mark" aria-hidden="true">
            <Download size={16} />
          </span>
          <span className="brand-name">Freeloader</span>
        </div>

        <nav className="views" aria-label="Views">
          {VIEWS.map((entry) => (
            <button
              type="button"
              key={entry.key}
              className={view === entry.key ? "view view-on" : "view"}
              aria-current={view === entry.key ? "page" : undefined}
              onClick={() => setView(entry.key)}
            >
              {viewIcon(entry.key)}
              <span className="view-label">{entry.label}</span>
              <span className="view-count">{counts[entry.key]}</span>
            </button>
          ))}
        </nav>

        <div className="rail-foot">
          <p className="rail-note">No account, no server, no telemetry.</p>
          <button type="button" className="dest" onClick={() => void browse()}>
            <span className="dest-head">
              <span>Saving to</span>
              <span className="dest-cta">Change</span>
            </span>
            <span className="dest-path" title={destination}>
              {destinationLabel}
            </span>
          </button>
        </div>
      </aside>

      <main className="stage">
        <header className="topbar">
          <div>
            <h1>{VIEWS.find((entry) => entry.key === view)?.label ?? "All files"}</h1>
            <p className="statusline">{statusLine}</p>
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

          <div className="topbar-actions">
            <button
              type="button"
              className="iconbtn"
              aria-label={theme === "dark" ? "Switch to the light theme" : "Switch to the dark theme"}
              onClick={() => pickTheme(theme === "dark" ? "light" : "dark")}
            >
              {theme === "dark" ? <Sun size={17} /> : <Moon size={17} />}
            </button>
            <button
              type="button"
              className="iconbtn"
              aria-label="Open settings"
              onClick={() => setSettingsOpen(true)}
            >
              <Settings size={17} />
            </button>
          </div>
        </header>

        <div className="scroller">
          <form
            className="composer"
            onSubmit={(event) => {
              event.preventDefault();
              void startDownload(url);
              setUrl("");
            }}
          >
            <div className="composer-row">
              <span className="sr-only" id="url-label">
                Download link
              </span>
              <input
                ref={urlRef}
                className="urlfield"
                type="url"
                spellCheck={false}
                aria-labelledby="url-label"
                placeholder="Paste a direct HTTP or HTTPS link"
                value={url}
                onChange={(event) => setUrl(event.target.value)}
              />
              <button type="submit" className="btn primary" disabled={!validUrl || submitting}>
                {submitting ? "Starting" : "Download"}
                <ArrowRight size={15} aria-hidden="true" />
              </button>
            </div>
            <div className="composer-foot">
              <button
                type="button"
                className="destchip"
                onClick={() => void browse()}
                title={destination}
              >
                <Folder size={13} aria-hidden="true" />
                <span>{destinationLabel}</span>
              </button>
              <span className="hint">Cookies and authorization headers are never forwarded.</span>
              {counts.completed > 0 && (
                <button type="button" className="btn plain sm" onClick={clearFinished}>
                  <Trash2 size={13} aria-hidden="true" />
                  Clear completed
                </button>
              )}
            </div>
          </form>

          {notice !== null && (
            <div className="notice" role="alert">
              <CircleAlert size={16} aria-hidden="true" />
              <span>{notice}</span>
              <button
                type="button"
                className="iconbtn"
                aria-label="Dismiss"
                onClick={() => setNotice(null)}
              >
                <X size={14} />
              </button>
            </div>
          )}

          {visible.length === 0 ? (
            <section className="empty">
              <h2>{items.length === 0 ? "Nothing queued" : "No matches in this view"}</h2>
              <p>
                {items.length === 0
                  ? `Paste a direct link above. Freeloader probes it for size and resume support, then streams it into ${destinationLabel}.`
                  : "Clear the filter or switch back to all files."}
              </p>
              {items.length === 0 ? (
                <div className="empty-keys">
                  <span>
                    <kbd>Ctrl</kbd> <kbd>N</kbd> link field
                  </span>
                  <span>
                    <kbd>/</kbd> filter
                  </span>
                </div>
              ) : (
                <button
                  type="button"
                  className="btn ghost"
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
                    <span className="ext" aria-hidden="true">
                      {extensionOf(item.name)}
                    </span>

                    <div className="row-main">
                      <div className="row-title">
                        <strong title={item.name}>{item.name}</strong>
                        <StatusPill item={item} />
                      </div>
                      <p className="row-sub" title={item.destination}>
                        {item.destination}
                      </p>
                      {item.error !== null && <p className="row-err">{item.error}</p>}
                    </div>

                    <div className="row-num">
                      <span className="num lead">{percent}%</span>
                      <span className="num dim">{formatBytes(item.downloaded)}</span>
                    </div>

                    <div className="row-num">
                      <span className="num">
                        {item.status === "downloading" ? formatSpeed(item.speed) : "--"}
                      </span>
                      <span className="num dim">
                        {item.status === "downloading" ? formatEta(item) : formatBytes(item.total)}
                      </span>
                    </div>

                    <div className="row-actions">
                      {item.status === "completed" && (
                        <button
                          type="button"
                          className="iconbtn"
                          aria-label={`Show ${item.name} in the file manager`}
                          onClick={() => void revealItem(item)}
                        >
                          <ExternalLink size={15} />
                        </button>
                      )}
                      {item.status === "failed" && (
                        <button
                          type="button"
                          className="iconbtn"
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
                        type="button"
                        className="iconbtn"
                        aria-label={`Remove ${item.name}`}
                        onClick={() => removeItem(item.id)}
                      >
                        <X size={15} />
                      </button>
                    </div>

                    <div
                      className="row-bar"
                      role="progressbar"
                      aria-valuemin={0}
                      aria-valuemax={100}
                      aria-valuenow={percent}
                      aria-label={`${item.name} progress`}
                    >
                      <span style={{ transform: `scaleX(${percent / 100})` }} />
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </div>
      </main>

      <dialog
        className="drawer"
        ref={dialogRef}
        aria-labelledby="settings-title"
        onClose={() => setSettingsOpen(false)}
        onClick={(event) => {
          if (event.target === dialogRef.current) setSettingsOpen(false);
        }}
      >
        <div className="drawer-inner">
          <header className="drawer-head">
            <div>
              <p className="eyebrow">Preferences</p>
              <h2 id="settings-title">Settings</h2>
            </div>
            <button
              type="button"
              className="iconbtn"
              aria-label="Close settings"
              onClick={() => setSettingsOpen(false)}
            >
              <X size={18} />
            </button>
          </header>

          <div className="drawer-block">
            <span className="lab" id="settings-destination">
              Download folder
            </span>
            <div className="pathbox">
              <Folder size={16} aria-hidden="true" />
              <input
                className="pathinput"
                aria-labelledby="settings-destination"
                spellCheck={false}
                value={destination}
                onChange={(event) => setDestination(event.target.value)}
              />
              <button type="button" className="btn ghost sm" onClick={() => void browse()}>
                Browse
              </button>
            </div>
          </div>

          <div className="drawer-block">
            <h3>Appearance</h3>
            <div className="segmented" role="group" aria-label="Theme">
              <button
                type="button"
                className={themeMode === "system" ? "seg seg-on" : "seg"}
                onClick={() => pickTheme("system")}
              >
                <Monitor size={14} aria-hidden="true" />
                System
              </button>
              <button
                type="button"
                className={themeMode === "light" ? "seg seg-on" : "seg"}
                onClick={() => pickTheme("light")}
              >
                <Sun size={14} aria-hidden="true" />
                Light
              </button>
              <button
                type="button"
                className={themeMode === "dark" ? "seg seg-on" : "seg"}
                onClick={() => pickTheme("dark")}
              >
                <Moon size={14} aria-hidden="true" />
                Dark
              </button>
            </div>
          </div>

          <div className="drawer-block">
            <h3>Browser integration</h3>
            <p className="hint">
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
                      <ExternalLink size={12} aria-hidden="true" />
                    </a>
                  </li>
                ))}
              </ul>
            ) : (
              <div>
                <button type="button" className="btn ghost" onClick={() => void detectBrowsers()}>
                  Look for browsers
                </button>
              </div>
            )}
          </div>

          <footer className="drawer-foot">
            <span className="version">v0.1.0 · GPL-3.0-or-later</span>
            <button type="button" className="btn primary" onClick={() => setSettingsOpen(false)}>
              Done
            </button>
          </footer>
        </div>
      </dialog>
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
