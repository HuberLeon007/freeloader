import { create } from "zustand";
import type { QueueBatch, QueueItem, QueueSnapshot } from "./queue-types";

type QueueState = QueueSnapshot & { add: (item: QueueItem) => void; remove: (id: string) => void; toggleSelected: (id: string) => void; selectAll: () => void; clearSelection: () => void; saveSnapshot: () => QueueSnapshot; loadSnapshot: (snapshot: QueueSnapshot) => void };
const empty: QueueSnapshot = { items: [], batches: [], selectedIds: [], settings: { density: "compact", concurrency: 3 } };

export const useQueueStore = create<QueueState>((set, get) => ({ ...empty,
  add: (item) => set((state) => ({ items: [item, ...state.items] })),
  remove: (id) => set((state) => ({ items: state.items.filter((item) => item.id !== id), selectedIds: state.selectedIds.filter((selected) => selected !== id), batches: state.batches.map((batch) => ({ ...batch, itemIds: batch.itemIds.filter((itemId) => itemId !== id) })) })),
  toggleSelected: (id) => set((state) => ({ selectedIds: state.selectedIds.includes(id) ? state.selectedIds.filter((itemId) => itemId !== id) : [...state.selectedIds, id] })),
  selectAll: () => set((state) => ({ selectedIds: state.items.map((item) => item.id) })),
  clearSelection: () => set({ selectedIds: [] }),
  saveSnapshot: () => { const { items, batches, selectedIds, settings } = get(); return { items, batches, selectedIds, settings }; },
  loadSnapshot: (snapshot) => set({ ...snapshot }),
}));

export function createQueueSnapshot(overrides: Partial<QueueSnapshot> = {}): QueueSnapshot { return { ...empty, ...overrides }; }
export type { QueueBatch };
