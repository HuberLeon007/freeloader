// SPDX-License-Identifier: GPL-3.0-or-later
import React, { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Download, FolderOpen, Moon, Plus, Search, Settings, Sun, X } from "lucide-react";
import "./styles.css";

type Status = "queued" | "downloading" | "completed" | "failed" | "cancelled";
type DownloadItem = { id: string; name: string; status: Status; progress: number; size: string; destination: string };
type ProgressEvent = { id: string; downloadedBytes: number; totalBytes: number | null };
type AddDownloadInput = { url: string; destinationPath: string; conflictPolicy: "rename" | "overwrite" | "error" };

function formatBytes(value: number | null): string {
  if (value === null) return "Unknown";
  if (value < 1024) return `${value} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let amount = value;
  let unit = -1;
  while (amount >= 1024 && unit < units.length - 1) { amount /= 1024; unit += 1; }
  return `${amount.toFixed(amount >= 10 ? 0 : 1)} ${units[unit]}`;
}

function App(): React.JSX.Element {
  const [dark, setDark] = useState(true);
  const [dialog, setDialog] = useState(false);
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<DownloadItem[]>([]);
  const [url, setUrl] = useState("");
  const [destination, setDestination] = useState("Downloads");
  const [error, setError] = useState<string | null>(null);
  const validUrl = /^https?:\/\/[^\s]+$/i.test(url);
  const filtered = useMemo(() => items.filter((item) => item.name.toLowerCase().includes(query.toLowerCase())), [items, query]);

  useEffect(() => {
    let active = true;
    const unlisten = listen<ProgressEvent>("download-progress", (event) => {
      if (!active) return;
      const value = event.payload;
      setItems((current) => current.map((item) => item.id === value.id ? { ...item, status: "downloading", progress: value.totalBytes ? Math.round(value.downloadedBytes / value.totalBytes * 100) : 0, size: value.totalBytes ? `${formatBytes(value.downloadedBytes)} / ${formatBytes(value.totalBytes)}` : formatBytes(value.downloadedBytes) } : item));
    });
    return () => { active = false; void unlisten.then((dispose) => dispose()); };
  }, []);

  const add = async (): Promise<void> => {
    if (!validUrl) return;
    setError(null);
    const filename = new URL(url).pathname.split("/").filter(Boolean).pop() || "download.bin";
    const input: AddDownloadInput = { url, destinationPath: `${destination}/${filename}`, conflictPolicy: "rename" };
    const localId = crypto.randomUUID();
    setItems((current) => [...current, { id: localId, name: filename, status: "queued", progress: 0, size: "Starting", destination }]);
    setUrl("");
    setDialog(false);
    try {
      const result = await invoke<{ id: string; path: string }>("add_download", { input });
      setItems((current) => current.map((item) => item.id === localId ? { ...item, id: result.id, status: "completed", progress: 100, destination: result.path } : item));
    } catch (cause) {
      setItems((current) => current.map((item) => item.id === localId ? { ...item, status: "failed" } : item));
      setError(cause instanceof Error ? cause.message : "The download failed. Check the URL and destination, then try again.");
    }
  };

  return <div className={dark ? "app dark" : "app"}>
    <aside className="rail" aria-label="Primary navigation"><div className="brand"><Download size={18} aria-hidden="true" /><span>Freeloader</span></div><nav><button className="nav active">All downloads</button><button className="nav">Active</button><button className="nav">Queued</button><button className="nav">Completed</button><button className="nav">Failed</button><button className="nav"><Settings size={16} aria-hidden="true" />Settings</button></nav><button className="theme" onClick={() => setDark((value) => !value)} aria-label={dark ? "Use light theme" : "Use dark theme"}>{dark ? <Sun size={16} /> : <Moon size={16} />}</button></aside>
    <main className="main"><header className="toolbar"><label className="search"><Search size={16} aria-hidden="true" /><span className="sr-only">Search downloads</span><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search downloads" /></label><button className="primary" onClick={() => setDialog(true)}><Plus size={16} aria-hidden="true" />Add download</button></header><section className="content"><div className="heading"><div><p className="eyebrow">All downloads</p><h1>Your downloads</h1></div><p className="muted">{items.length} item{items.length === 1 ? "" : "s"}</p></div>{error && <p className="error" role="alert">{error}</p>}{filtered.length === 0 ? <div className="empty"><Download size={28} aria-hidden="true" /><h2>Nothing here yet</h2><p>Add a direct HTTP or HTTPS URL to start your first download.</p><button className="primary" onClick={() => setDialog(true)}><Plus size={16} />Add download</button></div> : <div className="list" role="table" aria-label="Downloads">{filtered.map((item) => <div className="row" role="row" key={item.id}><div className="file" role="cell"><Download size={16} aria-hidden="true" /><strong>{item.name}</strong></div><span className={`status ${item.status}`} role="cell">{item.status}</span><div className="progress" role="cell" aria-label={`${item.progress}% complete`}><span style={{ width: `${item.progress}%` }} /></div><span className="muted" role="cell">{item.size}</span><span className="muted" role="cell"><FolderOpen size={14} aria-hidden="true" />{item.destination}</span><button className="icon" aria-label={`Remove ${item.name}`} onClick={() => setItems((current) => current.filter((candidate) => candidate.id !== item.id))}><X size={15} /></button></div>)}</div>}</section></main>
    {dialog && <div className="overlay" role="presentation"><section className="dialog" role="dialog" aria-modal="true" aria-labelledby="add-title"><div className="dialog-header"><div><p className="eyebrow">New transfer</p><h2 id="add-title">Add download</h2></div><button className="icon" aria-label="Close dialog" onClick={() => setDialog(false)}><X size={18} /></button></div><label className="field">URL<input autoFocus value={url} onChange={(event) => setUrl(event.target.value)} placeholder="https://example.com/file.zip" type="url" /></label><label className="field">Destination folder<input value={destination} onChange={(event) => setDestination(event.target.value)} placeholder="Downloads" /></label><p className="hint">Only direct HTTP and HTTPS URLs are accepted. Cookies and private headers are never sent.</p><div className="dialog-actions"><button className="secondary" onClick={() => setDialog(false)}>Cancel</button><button className="primary" onClick={() => void add()} disabled={!validUrl}><Plus size={16} />Download</button></div></section></div>}
  </div>;
}

createRoot(document.getElementById("root") as HTMLElement).render(<App />);
