// SPDX-License-Identifier: GPL-3.0-or-later
export type DownloadStatus = "queued" | "downloading" | "completed" | "failed" | "paused";
export type DownloadItem = {
  id: string;
  url: string;
  name: string;
  status: DownloadStatus;
  downloaded: number;
  total: number | null;
  destination: string;
  speed: number;
  error: string | null;
  batchId: string;
};

export function parseLinks(input: string): { links: string[]; invalid: string[] } {
  const seen = new Set<string>();
  const links: string[] = [];
  const invalid: string[] = [];
  for (const value of input.split(/[\r\n,\s]+/).map((part) => part.trim()).filter(Boolean)) {
    try {
      const url = new URL(value);
      if (url.protocol !== "http:" && url.protocol !== "https:") throw new Error("scheme");
      if (seen.has(url.href)) continue;
      seen.add(url.href);
      links.push(url.href);
    } catch {
      invalid.push(value);
    }
  }
  return { links, invalid };
}

export function filename(raw: string): string {
  try {
    const url = new URL(raw);
    const value = decodeURIComponent(url.pathname.split("/").filter(Boolean).pop() || "");
    return (value || url.hostname || "download").replace(/[<>:"/\\|?*-\u001f]/g, "_").slice(0, 100) || "download";
  } catch {
    return "download";
  }
}

export function groupItems(items: DownloadItem[]) {
  const groups = new Map<string, DownloadItem[]>();
  for (const item of items) groups.set(item.batchId, [...(groups.get(item.batchId) || []), item]);
  return [...groups.entries()].map(([id, values]) => ({
    id,
    name: values[0]?.batchId === "default" ? "Unsorted" : values[0]?.batchId || "Batch",
    items: values,
    downloaded: values.reduce((sum, item) => sum + item.downloaded, 0),
    total: values.some((item) => item.total === null) ? null : values.reduce((sum, item) => sum + (item.total || 0), 0),
    speed: values.reduce((sum, item) => sum + item.speed, 0),
  }));
}

export function percent(downloaded: number, total: number | null, status: DownloadStatus) {
  if (status === "completed") return 100;
  return total && total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
}
