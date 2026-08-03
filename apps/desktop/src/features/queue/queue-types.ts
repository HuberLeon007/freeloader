export type QueueStatus="queued"|"downloading"|"completed"|"failed";
export type QueueItem={id:string;url:string;name:string;status:QueueStatus;downloaded:number;total:number|null;destination:string;speed:number;error:string|null;adapterId?:string;batchId?:string};
export type QueueBatch={id:string;name:string;itemIds:string[]};
export type QueueSnapshot={items:QueueItem[];batches:QueueBatch[];selectedIds:string[];settings:{density:"compact"|"comfortable";concurrency:number}};
