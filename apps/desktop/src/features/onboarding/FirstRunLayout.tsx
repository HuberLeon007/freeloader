import type { PropsWithChildren } from 'react';
export function FirstRunLayout({children}:{children:PropsWithChildren['children']}){return <div className="setup"><aside className="setup-rail"><strong>Freeloader</strong><p className="hint">Local-first downloads. No account, cloud, or telemetry.</p></aside><main className="setup-main">{children}</main></div>}
