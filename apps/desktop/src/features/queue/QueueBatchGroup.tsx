import type { QueueBatch, QueueItem } from './queue-types';
import { QueueRow } from './QueueRow';
export function QueueBatchGroup({batch,items,selectedIds,onSelect,onRemove}:{batch:QueueBatch;items:QueueItem[];selectedIds:string[];onSelect:(id:string)=>void;onRemove:(id:string)=>void}){return <section aria-label={batch.name}><h3>{batch.name}</h3>{items.filter(item=>batch.itemIds.includes(item.id)).map(item=><QueueRow key={item.id} item={item} selected={selectedIds.includes(item.id)} onSelect={()=>onSelect(item.id)} onRemove={()=>onRemove(item.id)}/>)}</section>}
