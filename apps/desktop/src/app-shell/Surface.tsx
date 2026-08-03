import type { PropsWithChildren } from "react";

export function Surface({ children, className = "" }: PropsWithChildren<{ className?: string }>) {
  return <section className={`fl-surface ${className}`.trim()}>{children}</section>;
}
