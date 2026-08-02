// SPDX-License-Identifier: GPL-3.0-or-later
import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Check, CheckCircle2, ChevronRight, CircleAlert, Download, ExternalLink, Folder, Inbox,
  LoaderCircle, Monitor, Moon, Rows2, Rows3, RotateCcw, Search, Settings, Sun, Trash2, X, Zap,
} from "lucide-react";
import { Onboarding } from "./onboarding";
import { THEME_KEY, readTheme, resolveTheme } from "./theme";
import type { ThemeMode } from "./theme";
import "./styles.css";

type Status = "queued" | "downloading" | "completed" | "failed";
type ViewKey = "all" | "active" | "completed" | "failed";
/** Row height preset. Dense tools should ship density as a real choice, not a default. */
type Density = "compact" | "comfortable";
type Item = { id:string; engineId?:string; url:string; name:string; status:Status; downloaded:number; total:number|null; destination:string; speed:number; error:string|null };
type Progress = { id:string; downloaded:number; total:number|null };
type Complete = { id:string; path:string };
type Failure = { id:string; message:string };

const ONBOARDING_KEY = "freeloader.onboarding.v2";
const ONBOARDING_MIGRATED_KEY = "freeloader.onboarding.v3-migrated";
const DESTINATION_KEY = "freeloader.destination";
const DENSITY_KEY = "freeloader.density";
const RELEASES_URL = "https://github.com/HuberLeon007/freeloader/releases";
const UNITS = ["B","KB","MB","GB","TB"] as const;
const VIEWS: {key:ViewKey; label:string}[] = [
  {key:"all",label:"All files"},{key:"active",label:"Active"},{key:"completed",label:"Completed"},{key:"failed",label:"Failed"},
];

function bytes(value:number|null):string { if(value===null||!Number.isFinite(value)) return "--"; let n=Math.max(0,value), i=0; while(n>=1024&&i<UNITS.length-1){n/=1024;i+=1;} return `${n.toFixed(i===0?0:n>=100?0:n>=10?1:2)} ${UNITS[i]}`; }
function speed(value:number):string { return value>0?`${bytes(value)}/s`:"--"; }
function pct(item:Item):number { return item.status==="completed"?100:item.total&&item.total>0?Math.min(100,Math.round(item.downloaded/item.total*100)):0; }
function eta(item:Item):string { if(item.status!=="downloading"||item.total===null||item.speed<=0)return "--"; const seconds=Math.max(0,Math.round((item.total-item.downloaded)/item.speed)); return seconds<60?`${seconds}s left`:seconds<3600?`${Math.round(seconds/60)}m left`:`${(seconds/3600).toFixed(1)}h left`; }
function joinPath(dir:string,name:string):string { const win=dir.includes("\\")&&!dir.includes("/"); return `${dir.replace(/[\\/]+$/g,"")}${win?"\\":"/"}${name}`; }
function parent(path:string):string { const i=Math.max(path.lastIndexOf("/"),path.lastIndexOf("\\")); return i>0?path.slice(0,i):path; }
/** Max filename length so the full path stays under Windows MAX_PATH (260).
 *  We keep 100 chars for the name + 160 for directory + \.part = under 260. */
const MAX_FILENAME = 100;
function filename(raw:string):string { try { const parsed=new URL(raw); const part=parsed.pathname.split("/").filter(Boolean).pop(); const decoded=part?decodeURIComponent(part):""; let name=decoded&&decoded!=="."&&decoded!==".."&&!decoded.includes("/")&&!decoded.includes("\\")?decoded:parsed.hostname||"download"; if(name.length>MAX_FILENAME){const dot=name.lastIndexOf(".");const ext=dot>0?name.slice(dot):"";const stem=name.slice(0,Math.max(0,MAX_FILENAME-ext.length));name=stem+ext;} return name; } catch { return "download"; } }
function extension(name:string):string { const i=name.lastIndexOf("."); return i>0?name.slice(i+1, i+5).toLowerCase():"file"; }
/** Register a Tauri event listener that degrades to a no-op outside the webview. */
function safeListen<T>(event:string,handler:(payload:T)=>void):Promise<()=>void>{return listen<T>(event,e=>handler(e.payload)).catch(()=>()=>{});}
function icon(key:ViewKey):React.JSX.Element { if(key==="active")return <Zap size={15}/>; if(key==="completed")return <CheckCircle2 size={15}/>; if(key==="failed")return <CircleAlert size={15}/>; return <Inbox size={15}/>; }
function host(raw:string):string { try { return new URL(raw).hostname; } catch { return raw; } }
function Status({item}:{item:Item}):React.JSX.Element { if(item.status==="completed")return <span className="pill pill-done"><Check size={12}/>Done</span>; if(item.status==="failed")return <span className="pill pill-failed"><CircleAlert size={12}/>Failed</span>; if(item.status==="downloading")return <span className="pill pill-active"><LoaderCircle className="spin" size={12}/>Downloading</span>; return <span className="pill">Queued</span>; }

type RowProps = { item:Item; open:boolean; onToggle:()=>void; onReveal:()=>void; onRetry:()=>void; onRemove:()=>void };

/** One transfer. Collapsed it is a single scannable line; expanded it reveals
 *  the source and target that would otherwise cost every row a second line. */
function Row({item,open,onToggle,onReveal,onRetry,onRemove}:RowProps):React.JSX.Element {
  const percent=pct(item);
  return <div className={`rowgroup${open?" rowgroup-open":""}`}>
    <div className={`row row-${item.status}`} role="row">
      <div role="cell"><button type="button" className="disclose" onClick={onToggle} aria-expanded={open} aria-label={`Details for ${item.name}`}><ChevronRight size={13}/><span className="ext">{extension(item.name)}</span></button></div>
      <div className="cell-name" role="cell"><strong title={item.name}>{item.name}</strong><span className="cell-host">{host(item.url)}</span></div>
      <div role="cell"><Status item={item}/></div>
      <div className="cell-meter" role="cell"><span className="meter" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent} aria-label={`${item.name} progress`}><span style={{transform:`scaleX(${percent/100})`}}/></span><span className="num lead">{percent}%</span><span className="num dim cell-eta">{item.status==="downloading"?eta(item):""}</span></div>
      <div className="num cell-figure" role="cell">{bytes(item.downloaded)}<span className="num dim">{item.total===null?"":` / ${bytes(item.total)}`}</span></div>
      <div className="num cell-figure" role="cell">{item.status==="downloading"?speed(item.speed):"--"}</div>
      <div className="row-actions" role="cell">{item.status==="completed"&&<button type="button" className="iconbtn" onClick={onReveal} aria-label="Open folder"><ExternalLink size={15}/></button>}{item.status==="failed"&&<button type="button" className="iconbtn" onClick={onRetry} aria-label="Retry"><RotateCcw size={15}/></button>}<button type="button" className="iconbtn" onClick={onRemove} aria-label="Remove"><X size={15}/></button></div>
    </div>
    {open&&<dl className="row-detail"><div><dt>Source</dt><dd>{item.url}</dd></div><div><dt>Saving to</dt><dd>{item.destination}</dd></div>{item.error&&<div><dt>Error</dt><dd className="row-err">{item.error}</dd></div>}</dl>}
  </div>;
}

function App():React.JSX.Element {
  const [themeMode,setThemeMode]=useState<ThemeMode>(readTheme);
  const [onboarding,setOnboarding]=useState(()=>localStorage.getItem(ONBOARDING_KEY)!=="done");
  const [destination,setDestination]=useState(()=>localStorage.getItem(DESTINATION_KEY)||"");
  const [systemFolder,setSystemFolder]=useState("");
  const [view,setView]=useState<ViewKey>("all"); const [query,setQuery]=useState(""); const [url,setUrl]=useState("");
  const [density,setDensity]=useState<Density>(()=>localStorage.getItem(DENSITY_KEY)==="comfortable"?"comfortable":"compact"); const [expanded,setExpanded]=useState<string|null>(null);
  const [items,setItems]=useState<Item[]>([]); const [notice,setNotice]=useState<string|null>(null); const [busy,setBusy]=useState(false); const [settings,setSettings]=useState(false); const [browsers,setBrowsers]=useState<string[]>([]);
  const urlRef=useRef<HTMLInputElement>(null); const filterRef=useRef<HTMLInputElement>(null); const dialogRef=useRef<HTMLDialogElement>(null); const samples=useRef(new Map<string,{at:number; bytes:number}>());
  const theme=resolveTheme(themeMode); const valid=/^https?:\/\/[^\s]+$/i.test(url.trim());
  const counts=useMemo(()=>({all:items.length,active:items.filter(i=>i.status==="queued"||i.status==="downloading").length,completed:items.filter(i=>i.status==="completed").length,failed:items.filter(i=>i.status==="failed").length}),[items]);
  const visible=useMemo(()=>{const needle=query.trim().toLowerCase();return items.filter(i=>{const inView=view==="all"||(view==="active"&&(i.status==="queued"||i.status==="downloading"))||(view===i.status);return inView&&(!needle||i.name.toLowerCase().includes(needle)||i.url.toLowerCase().includes(needle));});},[items,query,view]);
  const destinationLabel=destination||"Choose a folder";

  useEffect(()=>{document.documentElement.dataset.theme=theme;document.documentElement.style.colorScheme=theme;},[theme]);
  useEffect(()=>{localStorage.setItem(THEME_KEY,themeMode);},[themeMode]);
  useEffect(()=>{if(destination)localStorage.setItem(DESTINATION_KEY,destination);},[destination]);
  useEffect(()=>{localStorage.setItem(DENSITY_KEY,density);},[density]);
  useEffect(()=>{let cancelled=false;void invoke<string>("default_download_dir").then(path=>{if(cancelled)return;setSystemFolder(path);setDestination(current=>current||path);}).catch(()=>undefined);return()=>{cancelled=true;};},[]);
  useEffect(()=>{const progress=safeListen<Progress>("download-progress",p=>{const now=Date.now();const previous=samples.current.get(p.id);samples.current.set(p.id,{at:now,bytes:p.downloaded});const instant=previous&&now>previous.at?(p.downloaded-previous.bytes)/((now-previous.at)/1000):0;setItems(current=>current.map(item=>item.id===p.id&&(item.status==="completed"||item.status==="failed")?item:{...item,status:"downloading",downloaded:p.downloaded,total:p.total,speed:instant>0?item.speed*.7+instant*.3:item.speed}));});const complete=safeListen<Complete>("download-complete",p=>{samples.current.delete(p.id);setItems(current=>current.map(item=>item.id===p.id?{...item,status:"completed",destination:p.path,total:item.total??item.downloaded,speed:0,error:null}:item));});const failed=safeListen<Failure>("download-error",p=>{samples.current.delete(p.id);setItems(current=>current.map(item=>item.id===p.id?{...item,status:"failed",speed:0,error:p.message}:item));setNotice(p.message);});return()=>{void progress.then(f=>f());void complete.then(f=>f());void failed.then(f=>f());};},[]);
  useEffect(()=>{const key=(event:KeyboardEvent)=>{const target=event.target;const typing=target instanceof HTMLInputElement||target instanceof HTMLTextAreaElement;if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==="n"){event.preventDefault();urlRef.current?.focus();}if(event.key==="/"&&!typing){event.preventDefault();filterRef.current?.focus();}};window.addEventListener("keydown",key);return()=>window.removeEventListener("keydown",key);},[]);
  useEffect(()=>{const node=dialogRef.current;if(!node)return;if(settings&&!node.open){node.showModal();void detect();}if(!settings&&node.open)node.close();},[settings]);

  const detect=useCallback(async()=>{try{const found=await invoke<string[]>("detect_browsers");setBrowsers(found.map(v=>v.toLowerCase()));}catch{setBrowsers([]);}},[]);
  const browse=useCallback(async()=>{setNotice(null);try{const chosen=await invoke<string|null>("pick_download_dir");if(chosen)setDestination(chosen);}catch(cause){const detail=typeof cause==="string"?cause:cause instanceof Error?cause.message:"Unknown native dialog error";setNotice(`Folder picker failed: ${detail}. You can still enter a path manually.`);}},[]);
  const start=useCallback(async()=>{const raw=url.trim();if(!/^https?:\/\/[^\s]+$/i.test(raw)||busy)return;setBusy(true);setNotice(null);const name=filename(raw);const base=destination||systemFolder||"Downloads";const id=crypto.randomUUID();const target=joinPath(base,name);setItems(current=>[{id,url:raw,name,status:"queued",downloaded:0,total:null,destination:target,speed:0,error:null},...current]);setUrl("");try{const result=await invoke<{id:string;path:string}>("add_download",{input:{url:raw,destinationPath:target,clientRequestId:id}});setItems(current=>current.map(item=>item.id===id?{...item,destination:result.path,engineId:result.id}:item));}catch(cause){const message=typeof cause==="string"?cause:cause instanceof Error?cause.message:"Could not start the download.";setItems(current=>current.map(item=>item.id===id?{...item,status:"failed",error:message}:item));setNotice(message);}finally{setBusy(false);}},[busy,destination,systemFolder,url]);
  const finish=useCallback(()=>{localStorage.setItem(ONBOARDING_KEY,"done");localStorage.setItem(ONBOARDING_MIGRATED_KEY,"1");setOnboarding(false);},[]);
  const replaySetup=useCallback(()=>{localStorage.removeItem(ONBOARDING_KEY);localStorage.removeItem(ONBOARDING_MIGRATED_KEY);setSettings(false);setOnboarding(true);},[]);
  const pickTheme=useCallback((mode:ThemeMode)=>setThemeMode(mode),[]);
  const remove=(id:string)=>{const item=items.find(entry=>entry.id===id);samples.current.delete(id);if(item&&(item.status==="queued"||item.status==="downloading")&&item.engineId){void invoke("cancel_download",{id:item.engineId}).catch(()=>undefined);}setItems(current=>current.filter(entry=>entry.id!==id));};
  const reveal=async(item:Item)=>{try{await invoke("open_in_file_manager",{path:parent(item.destination)});}catch(cause){setNotice(`Could not open the folder: ${String(cause)}`);}};

  if(onboarding)return <Onboarding destination={destination} onDestinationChange={setDestination} themeMode={themeMode} onThemeChange={pickTheme} onFinish={finish}/>;
  const status=[`${visible.length} shown`,counts.active?`${counts.active} in flight`:"",counts.failed?`${counts.failed} failed`:""].filter(Boolean).join(" · ");
  return <div className="shell"><aside className="rail"><div className="brand"><span className="mark"><Download size={16}/></span><span>Freeloader</span></div><nav className="views" aria-label="Views">{VIEWS.map(entry=><button type="button" key={entry.key} className={view===entry.key?"view view-on":"view"} onClick={()=>setView(entry.key)}>{icon(entry.key)}<span className="view-label">{entry.label}</span><span className="view-count">{counts[entry.key]}</span></button>)}</nav><div className="rail-foot"><p className="rail-note">Private by default. No server, no telemetry.</p><button type="button" className="dest" onClick={()=>void browse()}><span className="dest-head"><span>Saving to</span><span>Change</span></span><span className="dest-path" title={destination}>{destinationLabel}</span></button></div></aside><main className="stage"><header className="topbar"><div className="topbar-head"><p className="kicker">Library</p><h1>{VIEWS.find(entry=>entry.key===view)?.label}</h1><p className="statusline">{status||"Ready"}</p></div><label className="finder"><Search size={15}/><span className="sr-only">Filter downloads</span><input ref={filterRef} value={query} placeholder="Filter" onChange={event=>setQuery(event.target.value)}/><kbd>/</kbd></label><div className="topbar-actions"><button type="button" className="iconbtn" onClick={()=>setDensity(current=>current==="compact"?"comfortable":"compact")} aria-label={density==="compact"?"Use comfortable rows":"Use compact rows"} title="Row density">{density==="compact"?<Rows3 size={17}/>:<Rows2 size={17}/>}</button><button type="button" className="iconbtn" onClick={()=>setThemeMode(theme==="dark"?"light":"dark")} aria-label="Toggle theme">{theme==="dark"?<Sun size={17}/>:<Moon size={17}/>}</button><button type="button" className="iconbtn" onClick={()=>setSettings(true)} aria-label="Open settings"><Settings size={17}/></button></div></header><div className="scroller"><form className="composer" onSubmit={event=>{event.preventDefault();void start();}}><div className="composer-row"><input ref={urlRef} className="urlfield" type="url" spellCheck={false} placeholder="Paste a direct HTTP or HTTPS link" value={url} onChange={event=>setUrl(event.target.value)} aria-label="Download link"/><button type="submit" className="button primary" disabled={!valid||busy}>{busy?"Starting":"Download"}<span aria-hidden="true">→</span></button></div><div className="composer-foot"><button type="button" className="destchip" onClick={()=>void browse()} title={destination}><Folder size={13}/><span>{destinationLabel}</span></button><span className="hint">Only the link you submit is used. Credentials stay local.</span>{counts.completed>0&&<button type="button" className="button plain sm" onClick={()=>setItems(current=>current.filter(i=>i.status!=="completed"))}><Trash2 size={13}/>Clear completed</button>}</div></form>{notice&&<div className="notice" role="alert"><CircleAlert size={16}/><span>{notice}</span><button type="button" className="iconbtn" onClick={()=>setNotice(null)} aria-label="Dismiss"><X size={14}/></button></div>}{visible.length===0?<section className="empty"><p className="kicker">{items.length===0?"Getting started":"No matches"}</p><h2>{items.length===0?"Paste a link to begin":"Nothing in this view"}</h2><p className="lede">{items.length===0?"Freeloader streams HTTP and HTTPS transfers straight into your chosen folder. Nothing leaves your machine.":"Clear the filter or switch back to All files."}</p></section>:<div className="queue" role="table" aria-label="Downloads" data-density={density}><div className="queue-head" role="row"><span role="columnheader"><span className="sr-only">Type</span></span><span role="columnheader">Name</span><span role="columnheader">Status</span><span role="columnheader">Progress</span><span className="cell-figure" role="columnheader">Size</span><span className="cell-figure" role="columnheader">Speed</span><span role="columnheader"><span className="sr-only">Actions</span></span></div>{visible.map(item=><Row key={item.id} item={item} open={expanded===item.id} onToggle={()=>setExpanded(current=>current===item.id?null:item.id)} onReveal={()=>void reveal(item)} onRetry={()=>{remove(item.id);setUrl(item.url);urlRef.current?.focus();}} onRemove={()=>remove(item.id)}/>)}</div>}</div></main><dialog className="drawer" ref={dialogRef} onClose={()=>setSettings(false)}><div className="drawer-inner"><header className="drawer-head"><div><p className="eyebrow">Preferences</p><h2>Settings</h2></div><button type="button" className="iconbtn" onClick={()=>setSettings(false)} aria-label="Close"><X size={18}/></button></header><div className="drawer-block"><span className="lab">Download folder</span><div className="pathbox"><Folder size={16}/><input className="pathinput" value={destination} onChange={event=>setDestination(event.target.value)} spellCheck={false}/><button type="button" className="button ghost sm" onClick={()=>void browse()}>Browse</button></div></div><div className="drawer-block"><h3>Appearance</h3><div className="segmented"><button type="button" className={themeMode==="system"?"seg seg-on":"seg"} onClick={()=>pickTheme("system")}><Monitor size={14}/>System</button><button type="button" className={themeMode==="light"?"seg seg-on":"seg"} onClick={()=>pickTheme("light")}><Sun size={14}/>Light</button><button type="button" className={themeMode==="dark"?"seg seg-on":"seg"} onClick={()=>pickTheme("dark")}><Moon size={14}/>Dark</button></div></div><div className="drawer-block"><h3>Browser handoff</h3><p className="hint">Optional. We only look for browser executables on disk.</p>{browsers.length===0?<button type="button" className="button ghost" onClick={()=>void detect()}>Detect browsers</button>:<ul className="detected">{browsers.map(browser=><li key={browser}><Check size={14}/><span>{browser}</span><a href={RELEASES_URL} target="_blank" rel="noreferrer">Extension <ExternalLink size={12}/></a></li>)}</ul>}</div><div className="drawer-block"><h3>Setup</h3><p className="hint">Walk through folder, appearance and browser handoff again.</p><button type="button" className="button ghost" onClick={replaySetup}>Run setup again</button></div><footer className="drawer-foot"><span className="version">v0.1.0 · GPL-3.0-or-later</span><button type="button" className="button primary" onClick={()=>setSettings(false)}>Done</button></footer></div></dialog></div>;
}
createRoot(document.getElementById("root")!).render(<React.StrictMode><App/></React.StrictMode>);
