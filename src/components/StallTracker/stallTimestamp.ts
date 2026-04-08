const MONTHS: Record<string, number> = {
  Jan: 1, Feb: 2, Mar: 3, Apr: 4, May: 5, Jun: 6,
  Jul: 7, Aug: 8, Sep: 9, Oct: 10, Nov: 11, Dec: 12,
}

/** Convert "Sat Mar 28 15:39" to a sortable number (MMDDHHMM). */
export function timestampToSortKey(ts: string): number {
  const parts = ts.split(/\s+/)
  if (parts.length < 4) return 0
  const mon = MONTHS[parts[1]] ?? 0
  const day = parseInt(parts[2]) || 0
  const [hh, mm] = (parts[3] ?? '0:0').split(':').map(Number)
  return mon * 1000000 + day * 10000 + (hh || 0) * 100 + (mm || 0)
}

/** Convert "Sat Mar 28 15:39" or "Mar 28" to date-only sort key (MMDD). */
export function timestampToDateKey(ts: string): number {
  const parts = ts.split(/\s+/)
  // "Mar 28" (label format) — month at index 0
  // "Sat Mar 28 15:39" (full format) — month at index 1
  const monIdx = MONTHS[parts[0]] ? 0 : 1
  const mon = MONTHS[parts[monIdx]] ?? 0
  const day = parseInt(parts[monIdx + 1]) || 0
  return mon * 100 + day
}

/** Extract "Mon DD" date label from "Sat Mar 28 15:39" → "Mar 28". */
export function timestampToDateLabel(ts: string): string {
  const parts = ts.split(/\s+/)
  if (parts.length < 3) return ts
  return `${parts[1]} ${parts[2]}`
}

/** Get unique date labels from timestamps, sorted chronologically. */
export function uniqueDates(timestamps: string[]): string[] {
  const seen = new Map<number, string>()
  for (const ts of timestamps) {
    const key = timestampToDateKey(ts)
    if (key && !seen.has(key)) {
      seen.set(key, timestampToDateLabel(ts))
    }
  }
  return [...seen.entries()]
    .sort((a, b) => a[0] - b[0])
    .map(([, label]) => label)
}
