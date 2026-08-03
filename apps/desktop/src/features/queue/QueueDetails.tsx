import type { QueueItem } from './queue-types';
export function QueueDetails({item}:{item:QueueItem}){return <dl><div><dt>Source</dt><dd>{item.url}</dd></div><div><dt>Destination</dt><dd>{item.destination}</dd></div>{item.error&&<div><dt>Error</dt><dd>{item.error}</dd></div>}</dl>}
