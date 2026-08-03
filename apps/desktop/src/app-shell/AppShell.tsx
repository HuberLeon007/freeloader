import type { PropsWithChildren, ReactNode } from "react";
import { BottomRail } from "./BottomRail";
import { TopRail } from "./TopRail";

export function AppShell({ title, status, actions, footer, children }: PropsWithChildren<{ title: string; status?: string; actions?: ReactNode; footer?: ReactNode }>) {
  return <div className="fl-shell"><TopRail title={title} status={status} actions={actions} /><main className="fl-content">{children}</main><BottomRail>{footer}</BottomRail></div>;
}
