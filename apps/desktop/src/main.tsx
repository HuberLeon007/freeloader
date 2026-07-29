// SPDX-License-Identifier: GPL-3.0-or-later
import React, { useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { Download, FolderOpen, Moon, Plus, Search, Settings, Sun, X } from "lucide-react";
import "./styles.css";

type Status = "queued" | "downloading" | "completed" | "failed";
type DownloadItem = { id: string; name: string; status: Status; progress: number; size: string; destination: string };

function App(): React.JSX.Element {
  const [dark, setDark] = useState(true);
  const [dialog, setDialog] = useState(false);
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<DownloadItem[]>([]);
  const [url, setUrl] = useState("");
  const filtered = useMemo(() => items.filter((item) => item.name.toLowerCase().includes(query.toLowerCase())), [items, query]);
  const add = (): void => {
    if (!/^https?:\/\//i.test(url)) return;
    const name = url.split("/").pop() || "download";
    setItems((current) => [...current, { id: crypto.randomUUID(), name, status: "queued", progress: 0, size: "Unknown", destination: "Downloads" }]);
    setUrl("");
    setDialog(false);
  };
  return <div className={dark ? "app dark" : "app"}>
    <aside className="rail" aria-label="Primary navigation"><div className="brand"><Download size={18} aria-hidden="true" /><span>Freeloader</span></div><nav><button className="nav active">All downloads</button><button className="nav">Active</button><button className="nav">Queued</button><button className="nav">Completed</button><button className="nav">Failed</button><button className="nav"><Settings size={16} aria-hidden="true" />Settings</button></nav><button className="theme" onClick={() => setDark((value) => !value)} aria-label={dark ? "Use light theme" : "Use dark theme"}>{dark ? <Sun size={16} /> : <Moon size={16} />}</button></aside>
    <main className="main"><header className="toolbar"><label className="search"><Search size={16} aria-hidden="true" /><span className="sr-only">Search downloads</span><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search downloads" /></label><button className="primary" onClick={() => setDialog(true)}><Plus size={16} aria-hidden="true" />Add download</button></header><section className="content"><div className="heading"><div><p className="eyebrow">All downloads</p><h1>Your downloads</h1></div><p className="muted">{items.length} item{items.length === 1 ? "" : "s"}</p></div>{filtered.length === 0 ? <div className="empty"><Download size={28} aria-hidden="true" /><h2>Nothing here yet</h2><p>Add a direct HTTP or HTTPS URL to start your first download.</p><button className="primary" onClick={() => setDialog(true)}><Plus size={16} />Add download</button></div> : <div className="list" role="table" aria-label="Downloads">{filtered.map((item) => <div className="row" role="row" key={item.id}><div className="file" role="cell"><Download size={16} aria-hidden="true" /><strong>{item.name}</strong></div><span className={`status ${item.status}`} role="cell">{item.status}</span><div className="progress" role="cell" aria-label={`${item.progress}% complete`}><span style={{ width: `${item.progress}%` }} /></div><span className="muted" role="cell">{item.size}</span><span className="muted" role="cell"><FolderOpen size={14} aria-hidden="true" />{item.destination}</span><button className="icon" aria-label={`Remove ${item.name}`} onClick={() => setItems((current) => current.filter((candidate) => candidate.id !== item.id))}><X size={15} /></button></div>)}</div>}</section></main>
    {dialog && <div className="overlay" role="presentation"><section className="dialog" role="dialog" aria-modal="true" aria-labelledby="add-title"><div className="dialog-header"><div><p className="eyebrow">New transfer</p><h2 id="add-title">Add download</h2></div><button className="icon" aria-label="Close dialog" onClick={() => setDialog(false)}><X size={18} /></button></div><label className="field">URL<input autoFocus value={url} onChange={(event) => setUrl(event.target.value)} placeholder="https://example.com/file.zip" type="url" /></label><p className="hint">Only direct HTTP and HTTPS URLs are accepted. Cookies and private headers are never sent.</p><div className="dialog-actions"><button className="secondary" onClick={() => setDialog(false)}>Cancel</button><button className="primary" onClick={add} disabled={!/^https?:\/\//i.test(url)}><Plus size={16} />Download</button></div></section></div>}
  </div>;
}

createRoot(document.getElementById("root") as HTMLElement).render(<App />);
