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

// Cumulative days before each month (non-leap year, close enough for sorting)
const DAYS_BEFORE_MONTH = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334]

/** Convert timestamp to day-of-year (1–365). Handles both full and label formats. */
export function timestampToDayOfYear(ts: string): number {
  const parts = ts.split(/\s+/)
  const monIdx = MONTHS[parts[0]] ? 0 : 1
  const mon = MONTHS[parts[monIdx]] ?? 0
  const day = parseInt(parts[monIdx + 1]) || 0
  return (DAYS_BEFORE_MONTH[mon] ?? 0) + day
}

/** Get the month number (1–12) from a timestamp. */
export function timestampToMonth(ts: string): number {
  const parts = ts.split(/\s+/)
  const monIdx = MONTHS[parts[0]] ? 0 : 1
  return MONTHS[parts[monIdx]] ?? 0
}

/** Get the month name from a timestamp (e.g., "Mar"). */
export function timestampToMonthLabel(ts: string): string {
  const parts = ts.split(/\s+/)
  const monIdx = MONTHS[parts[0]] ? 0 : 1
  return parts[monIdx] ?? ''
}

const MONTH_NAMES = ['', 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']

/** Compute week key (week number within year) from a timestamp. */
export function timestampToWeekKey(ts: string): number {
  const doy = timestampToDayOfYear(ts)
  return Math.floor((doy - 1) / 7)
}

export type Granularity = 'daily' | 'weekly' | 'monthly'

export interface PeriodInfo {
  key: number
  label: string
}

/** Get the period key and label for a timestamp at the given granularity. */
export function timestampToPeriod(ts: string, granularity: Granularity): PeriodInfo {
  if (granularity === 'monthly') {
    const mon = timestampToMonth(ts)
    return { key: mon, label: MONTH_NAMES[mon] ?? '' }
  }
  if (granularity === 'weekly') {
    const weekKey = timestampToWeekKey(ts)
    return { key: weekKey, label: '' } // label filled in by collectPeriods
  }
  // daily
  const dk = timestampToDateKey(ts)
  return { key: dk, label: timestampToDateLabel(ts) }
}

/** Collect unique sorted periods from timestamps. For weekly, computes range labels. */
export function collectPeriods(timestamps: string[], granularity: Granularity): PeriodInfo[] {
  if (granularity === 'weekly') {
    // Group timestamps by week key, then build "Mon DD–Mon DD" labels from min/max date in each week
    const weekMap = new Map<number, { minDoy: number, maxDoy: number, minTs: string, maxTs: string }>()
    for (const ts of timestamps) {
      const wk = timestampToWeekKey(ts)
      const doy = timestampToDayOfYear(ts)
      const existing = weekMap.get(wk)
      if (!existing) {
        weekMap.set(wk, { minDoy: doy, maxDoy: doy, minTs: ts, maxTs: ts })
      } else {
        if (doy < existing.minDoy) { existing.minDoy = doy; existing.minTs = ts }
        if (doy > existing.maxDoy) { existing.maxDoy = doy; existing.maxTs = ts }
      }
    }
    return [...weekMap.entries()]
      .sort((a, b) => a[0] - b[0])
      .map(([key, { minTs, maxTs }]) => {
        const from = timestampToDateLabel(minTs)
        const to = timestampToDateLabel(maxTs)
        return { key, label: from === to ? from : `${from}–${to}` }
      })
  }

  // daily / monthly: use a simple dedup+sort
  const seen = new Map<number, string>()
  for (const ts of timestamps) {
    const p = timestampToPeriod(ts, granularity)
    if (!seen.has(p.key)) seen.set(p.key, p.label)
  }
  return [...seen.entries()]
    .sort((a, b) => a[0] - b[0])
    .map(([key, label]) => ({ key, label }))
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
