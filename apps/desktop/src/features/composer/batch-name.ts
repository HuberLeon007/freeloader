import type { CandidateLink } from "./paste-parser";

export function proposeBatchName(candidates: CandidateLink[]): string {
  const hosts = candidates.filter((candidate) => candidate.valid && candidate.host).map((candidate) => candidate.host as string);
  if (hosts.length === 0) return "New downloads";
  const counts = new Map<string, number>();
  for (const host of hosts) counts.set(host, (counts.get(host) ?? 0) + 1);
  const dominant = [...counts.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))[0]?.[0];
  return dominant ? `${dominant} downloads` : "New downloads";
}
