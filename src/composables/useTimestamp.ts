/**
 * Timestamp formatting utilities.
 *
 * All timestamps stored in the database are UTC. These helpers convert
 * UTC strings to the user's local time for display.
 */

/**
 * Parse a UTC timestamp string ("YYYY-MM-DD HH:MM:SS" or ISO) and return
 * a Date object. Appends "Z" if no timezone indicator is present so that
 * the browser interprets it as UTC rather than local.
 */
function parseUtc(timestamp: string): Date {
  // If already has timezone info (Z or +/-offset), use as-is
  if (/[Zz]$/.test(timestamp) || /[+-]\d{2}:\d{2}$/.test(timestamp)) {
    return new Date(timestamp)
  }
  // Replace space with T and append Z for proper UTC parsing
  const iso = timestamp.replace(' ', 'T') + 'Z'
  return new Date(iso)
}

/** Format a UTC timestamp as local time: "HH:MM" (24h) */
export function formatTimeShort(timestamp: string): string {
  const d = parseUtc(timestamp)
  if (isNaN(d.getTime())) return timestamp
  return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false })
}

/** Format a UTC timestamp as local time: "HH:MM:SS" (24h) */
export function formatTimeFull(timestamp: string): string {
  const d = parseUtc(timestamp)
  if (isNaN(d.getTime())) return timestamp
  return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })
}

/** Format a UTC timestamp as local date+time: "Mar 26, 14:30" */
export function formatDateTimeShort(timestamp: string): string {
  const d = parseUtc(timestamp)
  if (isNaN(d.getTime())) return timestamp
  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

/** Format a UTC timestamp as local date: "3/26/2026" (locale-dependent) */
export function formatDate(timestamp: string): string {
  const d = parseUtc(timestamp)
  if (isNaN(d.getTime())) return timestamp
  return d.toLocaleDateString()
}

/** Format a UTC timestamp as a full local datetime: "2026-03-26 14:30:00" */
export function formatDateTimeFull(timestamp: string): string {
  const d = parseUtc(timestamp)
  if (isNaN(d.getTime())) return timestamp
  const year = d.getFullYear()
  const month = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  const hours = String(d.getHours()).padStart(2, '0')
  const minutes = String(d.getMinutes()).padStart(2, '0')
  const seconds = String(d.getSeconds()).padStart(2, '0')
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
}
