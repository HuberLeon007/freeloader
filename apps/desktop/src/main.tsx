// SPDX-License-Identifier: GPL-3.0-or-later
import React, { useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  ArrowUpRight,
  Check,
  CheckCircle2,
  CircleAlert,
  Download,
  FolderOpen,
  LoaderCircle,
  Moon,
  Plus,
  Search,
  Settings,
  Sun,
  X,
} from "lucide-react";
import "./styles.css";

type Status = "queued" | "downloading" | "completed" | "failed" | "cancelled";
type View = "all" | "active" | "completed";
type DownloadItem = {
  id: string;
  name: string;
  status: Status;
  progress: number;
  size: string;
  destination: string;
};
type ProgressEvent = { id: string; downloaded: number; total: number | null };
type CompleteEvent = { id: string; path: string };
type ErrorEvent = { id: string; message: string };
type Browser = { name: string; store: string; detected: boolean };

const GITHUB_RELEASES = "https://github.com/HuberLeon007/freeloader/releases";

function formatBytes(value: number | null): string {
  if (value === null) return "Unknown size";
  if (value < 1024) return `${value} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let amount = value;
  let unit = -1;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  return `${amount.toFixed(amount >= 10 ? 0 : 1)} ${units[unit]}`;
}

function statusLabel(status: Status): string {
  return status === "downloading" ? "Downloading" : status.charAt(0).toUpperCase() + status.slice(1);
}

function StatusMark({ status }: { status: Status }): React.JSX.Element {
  if (status === "completed") return <CheckCircle2 size={17} aria-hidden="true" />;
  if (status === "failed") return <CircleAlert size={17} aria-hidden="true" />;
  if (status === "downloading") return <LoaderCircle className="spin" size={17} aria-hidden="true" />;
  return <span className="status-square" aria-hidden="true" />;
}

function App(): React.JSX.Element {
  const [dark, setDark] = useState(() => localStorage.getItem("freeloader.theme") !== "light");
  const [view, setView] = useState<View>("all");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [onboarding, setOnboarding] = useState(() => localStorage.getItem("freeloader.onboarding") !== "done");
  const [onboardingStep, setOnboardingStep] = useState(0);
  const [query, setQuery] = useState("");
  const [url, setUrl] = useState("");
  const [destination, setDestination] = useState("Downloads");
  const [items, setItems] = useState<DownloadItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [browsers, setBrowsers] = useState<Browser[]>([]);
  const settingsCloseRef = useRef<HTMLButtonElement>(null);

  const activeCount = items.filter((item) => item.status === "downloading").length;
  const completedCount = items.filter((item) => item.status === "completed").length;
  const validUrl = /^https?:\/\/[^\s]+$/i.test(url);
  const visibleItems = useMemo(() => items.filter((item) => {
    const matchesView = view === "all" || (view === "active" ? item.status === "downloading" : item.status === "completed");
    return matchesView && item.name.toLowerCase().includes(query.toLowerCase());
  }), [items, query, view]);

  useEffect(() => {
    const unlistenProgress = listen<ProgressEvent>("download-progress", (event) => {
      const value = event.payload;
      setItems((current) => current.map((item) => item.id === value.id ? {
        ...item,
        status: "downloading",
        progress: value.total ? Math.round((value.downloaded / value.total) * 100) : 0,
        size: value.total ? `${formatBytes(value.downloaded)} / ${formatBytes(value.total)}` : formatBytes(value.downloaded),
      } : item));
    });
    const unlistenComplete = listen<CompleteEvent>("download-complete", (event) => {
      const value = event.payload;
      setItems((current) => current.map((item) => item.id === value.id ? {
        ...item,
        status: "completed",
        progress: 100,
        destination: value.path,
      } : item));
    });
    const unlistenError = listen<ErrorEvent>("download-error", (event) => {
      const value = event.payload;
      setItems((current) => current.map((item) => item.id === value.id ? { ...item, status: "failed" } : item));
      setError((current) => current ?? value.message);
    });
    return () => {
      void unlistenProgress.then((dispose) => dispose());
      void unlistenComplete.then((dispose) => dispose());
      void unlistenError.then((dispose) => dispose());
    };
  }, []);

  useEffect(() => {
    if (!settingsOpen) return;
    const handleKeyDown = (event: KeyboardEvent): void => {
      if (event.key === "Escape") setSettingsOpen(false);
    };
    window.addEventListener("keydown", handleKeyDown);
    settingsCloseRef.current?.focus();
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [settingsOpen]);

  const completeOnboarding = (): void => {
    localStorage.setItem("freeloader.onboarding", "done");
    setOnboarding(false);
  };

  const detectBrowsers = async (): Promise<void> => {
    try {
      const found = await invoke<string[]>("detect_browsers");
      const names = new Set(found.map((value) => value.toLowerCase()));
      setBrowsers([
        { name: "Firefox", store: "Firefox Add-ons", detected: names.has("firefox") },
        { name: "Microsoft Edge", store: "Edge Add-ons", detected: names.has("edge") },
        { name: "Chromium", store: "GitHub Releases", detected: names.has("chromium") || names.has("chrome") },
      ]);
    } catch {
      setBrowsers([]);
    }
  };

  const addDownload = async (): Promise<void> => {
    if (!validUrl || submitting) return;
    setSubmitting(true);
    setError(null);
    const parsedUrl = new URL(url);
    const filename = parsedUrl.pathname.split("/").filter(Boolean).pop() || "download";
    const localId = crypto.randomUUID();
    setItems((current) => [...current, {
      id: localId,
      name: filename,
      status: "queued",
      progress: 0,
      size: "Preparing",
      destination,
    }]);
    const requestedUrl = url;
    setUrl("");
    try {
      const result = await invoke<{ id: string; path: string }>("add_download", {
        input: { url: requestedUrl, destinationPath: `${destination}/${filename}`, clientRequestId: localId },
      });
      setItems((current) => current.map((item) => item.id === localId ? {
        ...item,
        destination: result.path,
      } : item));
    } catch (cause) {
      setItems((current) => current.map((item) => item.id === localId ? { ...item, status: "failed" } : item));
      setError(cause instanceof Error ? cause.message : "Download failed. Check the URL and try again.");
    } finally {
      setSubmitting(false);
    }
  };

  const toggleTheme = (): void => {
    setDark((current) => {
      const next = !current;
      localStorage.setItem("freeloader.theme", next ? "dark" : "light");
      return next;
    });
  };

  const appClass = dark ? "app dark" : "app";

  if (onboarding) {
    return (
      <div className={appClass}>
        <div className="onboarding-shell">
          <section className="onboarding-panel" role="dialog" aria-modal="true" aria-labelledby="onboarding-title">
            <div className="workmark"><span className="workmark-symbol">F</span><span>Freeloader</span></div>
            <div className="step-track" aria-label={`Setup step ${onboardingStep + 1} of 3`}>
              {[0, 1, 2].map((step) => <span className={step <= onboardingStep ? "step active" : "step"} key={step} />)}
            </div>
            {onboardingStep === 0 && <div className="onboarding-copy">
              <p className="section-kicker">Local download utility</p>
              <h1 id="onboarding-title">Your files. Your machine.</h1>
              <p>Freeloader streams direct HTTP and HTTPS downloads to disk. No account, cloud service, or tracking layer.</p>
            </div>}
            {onboardingStep === 1 && <div className="onboarding-copy">
              <p className="section-kicker">Storage location</p>
              <h1 id="onboarding-title">Choose where files land.</h1>
              <label className="field-label" htmlFor="onboarding-destination">Default folder</label>
              <input id="onboarding-destination" className="field-input mono" value={destination} onChange={(event) => setDestination(event.target.value)} />
              <p>Change this later in Settings whenever you need to.</p>
            </div>}
            {onboardingStep === 2 && <div className="onboarding-copy">
              <p className="section-kicker">Browser handoff</p>
              <h1 id="onboarding-title">Send links from your browser.</h1>
              <p>Optional integration. Freeloader only checks executable locations and never reads browser profiles.</p>
              <button className="button secondary" onClick={() => void detectBrowsers()}>Detect browsers</button>
              {browsers.filter((browser) => browser.detected).map((browser) => <div className="browser-line" key={browser.name}><Check size={15} aria-hidden="true" />{browser.name}</div>)}
            </div>}
            <div className="onboarding-actions">
              <button className="button ghost" onClick={completeOnboarding}>Skip setup</button>
              {onboardingStep < 2 ? <button className="button primary" onClick={() => setOnboardingStep((step) => step + 1)}>Continue <ArrowUpRight size={15} aria-hidden="true" /></button> : <button className="button primary" onClick={completeOnboarding}>Open Freeloader <ArrowUpRight size={15} aria-hidden="true" /></button>}
            </div>
          </section>
        </div>
      </div>
    );
  }

  return (
    <div className={appClass}>
      <header className="topbar">
        <div className="workmark"><span className="workmark-symbol">F</span><span>Freeloader</span></div>
        <nav className="topnav" aria-label="Primary navigation">
          <button className={view === "all" ? "topnav-link active" : "topnav-link"} onClick={() => setView("all")}>All files <span>{items.length}</span></button>
          <button className={view === "active" ? "topnav-link active" : "topnav-link"} onClick={() => setView("active")}>Active <span>{activeCount}</span></button>
          <button className={view === "completed" ? "topnav-link active" : "topnav-link"} onClick={() => setView("completed")}>Completed <span>{completedCount}</span></button>
        </nav>
        <div className="topbar-actions">
          <button className="icon-button" aria-label={dark ? "Switch to light theme" : "Switch to dark theme"} onClick={toggleTheme}>{dark ? <Sun size={17} /> : <Moon size={17} />}</button>
          <button className="icon-button" aria-label="Open settings" onClick={() => { setSettingsOpen(true); void detectBrowsers(); }}><Settings size={17} /></button>
        </div>
      </header>

      <main className="workbench">
        <section className="hero-row">
          <div>
            <p className="section-kicker">Download workbench</p>
            <h1>Move files<br /><em>forward.</em></h1>
          </div>
          <div className="hero-note"><span className="live-mark" aria-hidden="true" />Local only. Direct to disk.</div>
        </section>

        <section className="composer" aria-label="Add a download">
          <div className="composer-label"><Plus size={16} aria-hidden="true" /><span>New download</span></div>
          <div className="composer-fields">
            <label className="composer-url">
              <span className="field-caption">URL</span>
              <input value={url} onChange={(event) => setUrl(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") void addDownload(); }} placeholder="Paste a direct HTTP or HTTPS URL" type="url" />
            </label>
            <label className="composer-destination">
              <FolderOpen size={15} aria-hidden="true" />
              <span className="field-caption">Save to</span>
              <input className="mono" value={destination} onChange={(event) => setDestination(event.target.value)} aria-label="Destination folder" />
            </label>
            <button className="button primary composer-submit" disabled={!validUrl || submitting} onClick={() => void addDownload()}>{submitting ? "Starting" : "Start download"}<ArrowUpRight size={15} aria-hidden="true" /></button>
          </div>
          <p className="composer-hint">Press Enter to start. Cookies and authorization headers are never forwarded.</p>
        </section>

        <div className="section-heading">
          <div><h2>{view === "all" ? "Your files" : view === "active" ? "In progress" : "Completed files"}</h2><span className="result-count">{visibleItems.length} shown</span></div>
          <label className="filter-box"><Search size={15} aria-hidden="true" /><span className="sr-only">Filter files</span><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Filter files" /></label>
        </div>

        {error && <div className="error-line" role="alert"><CircleAlert size={16} aria-hidden="true" /><span>{error}</span><button className="icon-button" aria-label="Dismiss error" onClick={() => setError(null)}><X size={15} /></button></div>}

        {visibleItems.length === 0 ? <section className="blank-state">
          <div className="blank-glyph"><Download size={20} aria-hidden="true" /></div>
          <h2>{items.length === 0 ? "Nothing in the queue" : "No matching files"}</h2>
          <p>{items.length === 0 ? "Paste a URL above to put your first file on disk." : "Try another filter or return to all files."}</p>
          {items.length > 0 && <button className="button secondary" onClick={() => { setQuery(""); setView("all"); }}>Show all files</button>}
        </section> : <section className="download-list" aria-label="Downloads">
          {visibleItems.map((item, index) => <article className="download-row" key={item.id}>
            <span className="row-index mono">{String(index + 1).padStart(2, "0")}</span>
            <div className="row-main">
              <div className="row-title"><Download size={17} aria-hidden="true" /><strong title={item.name}>{item.name}</strong></div>
              <div className="row-path mono"><FolderOpen size={13} aria-hidden="true" />{item.destination}</div>
            </div>
            <div className={`row-status status-${item.status}`}><StatusMark status={item.status} /><span>{statusLabel(item.status)}</span></div>
            <div className="row-progress"><div className="progress-track"><span style={{ width: `${item.progress}%` }} /></div><span className="mono progress-number">{item.progress}%</span></div>
            <span className="row-size mono">{item.size}</span>
            <button className="icon-button row-remove" aria-label={`Remove ${item.name}`} onClick={() => setItems((current) => current.filter((candidate) => candidate.id !== item.id))}><X size={15} /></button>
          </article>)}
        </section>}
      </main>

      {settingsOpen && <div className="modal-backdrop" onClick={(event) => { if (event.target === event.currentTarget) setSettingsOpen(false); }}>
        <section className="settings-panel" role="dialog" aria-modal="true" aria-labelledby="settings-title">
          <div className="panel-header"><div><p className="section-kicker">Preferences</p><h2 id="settings-title">Settings</h2></div><button ref={settingsCloseRef} className="icon-button" aria-label="Close settings" onClick={() => setSettingsOpen(false)}><X size={18} /></button></div>
          <div className="settings-block"><label className="field-label" htmlFor="settings-destination">Default destination</label><input id="settings-destination" className="field-input mono" value={destination} onChange={(event) => setDestination(event.target.value)} /></div>
          <div className="settings-block"><h3>Browser integration</h3><p className="panel-copy">Only executable locations are checked. Browser profiles, history, cookies, and credentials stay untouched.</p>{browsers.filter((browser) => browser.detected).map((browser) => <div className="browser-line" key={browser.name}><Check size={15} aria-hidden="true" /><span>{browser.name}</span><a href={GITHUB_RELEASES} target="_blank" rel="noreferrer">{browser.store}<ArrowUpRight size={13} /></a></div>)}{browsers.every((browser) => !browser.detected) && <button className="button secondary" onClick={() => void detectBrowsers()}>Detect browsers</button>}</div>
          <div className="panel-footer"><button className="button primary" onClick={() => setSettingsOpen(false)}>Done</button></div>
        </section>
      </div>}
    </div>
  );
}

createRoot(document.getElementById("root") as HTMLElement).render(<App />);
