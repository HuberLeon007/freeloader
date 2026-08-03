import type { ReactNode } from "react";
export function TopRail({title,status,actions}:{title:string;status?:string;actions?:ReactNode}){return <header className="fl-top-rail"><div><p className="eyebrow">Library</p><h1>{title}</h1>{status&&<p className="statusline">{status}</p>}</div>{actions&&<div className="fl-top-actions">{actions}</div>}</header>;}
