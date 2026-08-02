// SPDX-License-Identifier: GPL-3.0-or-later
import React, { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ArrowLeft,
  ArrowRight,
  Check,
  CircleAlert,
  Download,
  FolderOpen,
  FolderPlus,
  LoaderCircle,
  Monitor,
  Moon,
  Radar,
  Sun,
} from "lucide-react";
import type { ThemeMode } from "./theme";
import "./onboarding.css";

type Suggestion = { label: string; path: string; hint: string };
type DirectoryReport = { path: string; exists: boolean; writable: boolean };
type Step = { id: string; ordinal: string; label: string; note: string };

type Props = {
  destination: string;
  onDestinationChange: (value: string) => void;
  themeMode: ThemeMode;
  onThemeChange: (mode: ThemeMode) => void;
  onFinish: () => void;
};

const INTRO: Step = {
  id: "intro",
  ordinal: "01",
  label: "What you are running",
  note: "Read once",
};

const STEPS: readonly Step[] = [
  INTRO,
  { id: "location", ordinal: "02", label: "Where files land", note: "Required" },
  { id: "appearance", ordinal: "03", label: "How it looks", note: "Changeable" },
  { id: "handoff", ordinal: "04", label: "Browser handoff", note: "Optional" },
];

const FACTS: { term: string; detail: string }[] = [
  { term: "No account", detail: "There is no service behind this app to sign up to." },
  { term: "No background service", detail: "Nothing keeps running once you close the window." },
  { term: "No telemetry", detail: "The only requests made are the transfers you start." },
  { term: "Your disk", detail: "Files are streamed straight into the folder you pick." },
];

const THEME_OPTIONS: { id: ThemeMode; label: string; detail: string }[] = [
  { id: "system", label: "System", detail: "Follows the OS" },
  { id: "dark", label: "Dark", detail: "Near-black canvas" },
  { id: "light", label: "Light", detail: "Paper white" },
];

function browserLabel(key: string): string {
  if (key === "edge") return "Microsoft Edge";
  if (key === "firefox") return "Firefox";
  if (key === "chromium") return "Chromium based";
  return key;
}

function browserDetail(key: string): string {
  if (key === "chromium") return "Chrome, Brave, Vivaldi and relatives";
  if (key === "firefox") return "Firefox and forks that keep Native Messaging";
  return "Found on your PATH";
}

function themeIcon(mode: ThemeMode): React.JSX.Element {
  if (mode === "dark") return <Moon size={13} aria-hidden="true" />;
  if (mode === "light") return <Sun size={13} aria-hidden="true" />;
  return <Monitor size={13} aria-hidden="true" />;
}

/** A miniature of the app window, used to preview a theme by showing it. */
function Frame({ tone, clipped }: { tone: "dark" | "light"; clipped: boolean }): React.JSX.Element {
  return (
    <span className={`frame frame-${tone}${clipped ? " frame-clip" : ""}`}>
      <span className="frame-rail" />
      <span className="frame-body">
        <span className="frame-line frame-line-wide" />
        <span className="frame-line" />
        <span className="frame-bar">
          <i />
        </span>
      </span>
    </span>
  );
}

export function Onboarding(props: Props): React.JSX.Element {
  const { destination, onDestinationChange, themeMode, onThemeChange, onFinish } = props;

  const [index, setIndex] = useState(0);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [report, setReport] = useState<DirectoryReport | null>(null);
  const [browsers, setBrowsers] = useState<string[] | null>(null);
  const [scanning, setScanning] = useState(false);
  const [working, setWorking] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  const headingRef = useRef<HTMLHeadingElement>(null);
  const seeded = useRef(false);

  const step = STEPS[index] ?? INTRO;
  const last = index === STEPS.length - 1;
  const blocked =
    step.id === "location" &&
    (destination.trim().length === 0 || report === null || (report !== null && !report.writable));

  useEffect(() => {
    if (seeded.current) return;
    seeded.current = true;
    let cancelled = false;
    void (async () => {
      try {
        const found = await invoke<Suggestion[]>("suggested_directories");
        if (cancelled) return;
        setSuggestions(found);
        const first = found[0];
        if (destination.trim().length === 0 && first !== undefined) {
          onDestinationChange(first.path);
        }
      } catch {
        if (!cancelled) setSuggestions([]);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [destination, onDestinationChange]);

  useEffect(() => {
    const path = destination.trim();
    if (path.length === 0) {
      setReport(null);
      return;
    }
    let cancelled = false;
    const timer = window.setTimeout(() => {
      void (async () => {
        try {
          const result = await invoke<DirectoryReport>("inspect_directory", { path });
          if (!cancelled) setReport(result);
        } catch {
          if (!cancelled) setReport(null);
        }
      })();
    }, 220);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [destination]);

  useEffect(() => {
    headingRef.current?.focus({ preventScroll: true });
  }, [index]);

  const browse = useCallback(async (): Promise<void> => {
    setProblem(null);
    try {
      const picked = await invoke<string | null>("pick_download_dir");
      if (picked !== null && picked.length > 0) onDestinationChange(picked);
    } catch {
      setProblem("The system folder picker did not open. Type the path instead.");
    }
  }, [onDestinationChange]);

  const createFolder = useCallback(async (): Promise<void> => {
    setWorking(true);
    setProblem(null);
    try {
      const result = await invoke<DirectoryReport>("create_directory", {
        path: destination.trim(),
      });
      setReport(result);
    } catch (cause) {
      setProblem(
        cause instanceof Error ? cause.message : "That folder could not be created.",
      );
    } finally {
      setWorking(false);
    }
  }, [destination]);

  const scan = useCallback(async (): Promise<void> => {
    setScanning(true);
    try {
      const found = await invoke<string[]>("detect_browsers");
      setBrowsers(found.map((entry) => entry.toLowerCase()));
    } catch {
      setBrowsers([]);
    } finally {
      setScanning(false);
    }
  }, []);

  const advance = useCallback((): void => {
    if (blocked) return;
    if (last) {
      onFinish();
      return;
    }
    setIndex((value) => Math.min(value + 1, STEPS.length - 1));
  }, [blocked, last, onFinish]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent): void => {
      const target = event.target as HTMLElement | null;
      if (event.key === "Escape") {
        event.preventDefault();
        onFinish();
        return;
      }
      if (event.key !== "Enter") return;
      if (target instanceof HTMLButtonElement || target instanceof HTMLAnchorElement) return;
      event.preventDefault();
      advance();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [advance, onFinish]);

  return (
    <div className="setup">
      <div className="setup-progress" aria-hidden="true">
        <span style={{ transform: `scaleX(${(index + 1) / STEPS.length})` }} />
      </div>

      <aside className="setup-rail">
        <div className="brand brand-lg">
          <span className="brand-mark" aria-hidden="true">
            <Download size={18} />
          </span>
          <span className="brand-name">Freeloader</span>
        </div>

        <ol className="setup-steps" aria-label="Setup progress">
          {STEPS.map((entry, position) => {
            const state =
              position < index ? "setup-step-done" : position === index ? "setup-step-now" : "";
            return (
              <li
                key={entry.id}
                className={`setup-step ${state}`.trim()}
                aria-current={position === index ? "step" : undefined}
              >
                <span className="setup-ordinal">
                  {position < index ? <Check size={13} aria-hidden="true" /> : entry.ordinal}
                </span>
                <span className="setup-step-text">
                  <strong>{entry.label}</strong>
                  <span className="setup-step-note">{entry.note}</span>
                </span>
              </li>
            );
          })}
        </ol>

        <p className="setup-assure">
          Every answer is stored on this machine and can be changed later in Settings.
        </p>
      </aside>

      <main className="setup-main">
        <button className="setup-skip" onClick={onFinish}>
          Skip setup
        </button>

        <div className="setup-scroll">
          <div className="setup-body" key={step.id}>
            {step.id === "intro" && (
              <>
                <p className="kicker">Welcome</p>
                <h1 ref={headingRef} tabIndex={-1}>
                  Downloads that never leave your machine.
                </h1>
                <p className="lede">
                  Freeloader streams HTTP and HTTPS transfers straight to disk. Four questions and
                  you are done. None of them are about you.
                </p>
                <dl className="facts">
                  {FACTS.map((fact) => (
                    <div key={fact.term}>
                      <dt>{fact.term}</dt>
                      <dd>{fact.detail}</dd>
                    </div>
                  ))}
                </dl>
              </>
            )}

            {step.id === "location" && (
              <>
                <p className="kicker">Save location</p>
                <h1 ref={headingRef} tabIndex={-1}>
                  Pick the folder files land in.
                </h1>
                <p className="lede">
                  Everything you download goes here unless you change it for a single transfer.
                </p>

                <div className="path">
                  <span className="path-glyph" aria-hidden="true">
                    <FolderOpen size={17} />
                  </span>
                  <span className="path-value">
                    <label>
                      <span className="sr-only">Download folder</span>
                      <input
                        value={destination}
                        spellCheck={false}
                        placeholder="Choose a folder"
                        onChange={(event) => onDestinationChange(event.target.value)}
                      />
                    </label>
                    <span className="path-state" role="status">
                      {report === null && <span className="path-idle">Checking this path</span>}
                      {report !== null && report.writable && (
                        <span className="path-ok">
                          <Check size={12} aria-hidden="true" />
                          Ready to write
                        </span>
                      )}
                      {report !== null && !report.exists && (
                        <span className="path-warn">
                          <CircleAlert size={12} aria-hidden="true" />
                          This folder does not exist yet
                        </span>
                      )}
                      {report !== null && report.exists && !report.writable && (
                        <span className="path-bad">
                          <CircleAlert size={12} aria-hidden="true" />
                          No permission to write here
                        </span>
                      )}
                    </span>
                  </span>
                  <span className="path-actions">
                    {report !== null && !report.exists && (
                      <button
                        className="button ghost"
                        disabled={working}
                        onClick={() => void createFolder()}
                      >
                        {working ? (
                          <LoaderCircle className="spin" size={14} aria-hidden="true" />
                        ) : (
                          <FolderPlus size={14} aria-hidden="true" />
                        )}
                        Create it
                      </button>
                    )}
                    <button className="button ghost" onClick={() => void browse()}>
                      Browse
                    </button>
                  </span>
                </div>

                {suggestions.length > 0 && (
                  <>
                    <p className="setup-caption">Common choices</p>
                    <div className="picks" role="group" aria-label="Suggested folders">
                      {suggestions.map((entry) => (
                        <button
                          key={entry.path}
                          className={entry.path === destination ? "pick pick-on" : "pick"}
                          aria-pressed={entry.path === destination}
                          title={entry.hint}
                          onClick={() => onDestinationChange(entry.path)}
                        >
                          {entry.path === destination && <Check size={13} aria-hidden="true" />}
                          {entry.label}
                        </button>
                      ))}
                    </div>
                  </>
                )}

                {problem !== null && (
                  <p className="setup-problem" role="alert">
                    <CircleAlert size={14} aria-hidden="true" />
                    {problem}
                  </p>
                )}
              </>
            )}

            {step.id === "appearance" && (
              <>
                <p className="kicker">Appearance</p>
                <h1 ref={headingRef} tabIndex={-1}>
                  Match your desktop, or ignore it.
                </h1>
                <p className="lede">
                  Both themes are drawn by hand rather than inverted from one another. System
                  switches the moment your OS does.
                </p>

                <div className="themes" role="radiogroup" aria-label="Theme">
                  {THEME_OPTIONS.map((option) => (
                    <button
                      key={option.id}
                      role="radio"
                      aria-checked={themeMode === option.id}
                      className={themeMode === option.id ? "theme theme-on" : "theme"}
                      onClick={() => onThemeChange(option.id)}
                    >
                      <span className="frame-stack" aria-hidden="true">
                        <Frame tone={option.id === "dark" ? "dark" : "light"} clipped={false} />
                        {option.id === "system" && <Frame tone="dark" clipped={true} />}
                      </span>
                      <span className="theme-name">
                        {themeIcon(option.id)}
                        {option.label}
                      </span>
                      <span className="theme-detail">{option.detail}</span>
                    </button>
                  ))}
                </div>
              </>
            )}

            {step.id === "handoff" && (
              <>
                <p className="kicker">Browser handoff</p>
                <h1 ref={headingRef} tabIndex={-1}>
                  Send links straight from your browser.
                </h1>
                <p className="lede">
                  Optional, and deliberately dull. Freeloader looks for browser executables on your
                  PATH. Profiles, history, cookies and saved passwords are never opened.
                </p>

                <div className="scan">
                  <button className="button ghost" disabled={scanning} onClick={() => void scan()}>
                    {scanning ? (
                      <LoaderCircle className="spin" size={14} aria-hidden="true" />
                    ) : (
                      <Radar size={14} aria-hidden="true" />
                    )}
                    {browsers === null ? "Look for browsers" : "Scan again"}
                  </button>
                  {browsers !== null && (
                    <span className="setup-caption">{browsers.length} found on PATH</span>
                  )}
                </div>

                {browsers !== null && browsers.length > 0 && (
                  <ul className="found">
                    {browsers.map((browser) => (
                      <li key={browser}>
                        <Check size={15} aria-hidden="true" />
                        <span>
                          <strong>{browserLabel(browser)}</strong>
                          <em>{browserDetail(browser)}</em>
                        </span>
                      </li>
                    ))}
                  </ul>
                )}

                {browsers !== null && browsers.length === 0 && (
                  <p className="none">
                    Nothing on your PATH looked like a browser. That is fine: paste links into
                    Freeloader directly and install the extension later from Settings.
                  </p>
                )}
              </>
            )}
          </div>
        </div>

        <footer className="setup-foot">
          <button
            className="button quiet"
            disabled={index === 0}
            onClick={() => setIndex((value) => Math.max(value - 1, 0))}
          >
            <ArrowLeft size={14} aria-hidden="true" />
            Back
          </button>

          <span className="setup-hint">
            <kbd>Enter</kbd>
            continue
            <span className="setup-dot" aria-hidden="true" />
            <kbd>Esc</kbd>
            skip
          </span>

          <button className="button primary" disabled={blocked} onClick={advance}>
            {last ? "Open Freeloader" : "Continue"}
            <ArrowRight size={15} aria-hidden="true" />
          </button>
        </footer>
      </main>
    </div>
  );
}
