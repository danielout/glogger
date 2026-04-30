/**
 * Unified search query parser — TypeScript port of the Rust parser
 * in src-tauri/src/unified_search.rs.
 *
 * Parses Scryfall-inspired structured search syntax:
 *   plain text        → substring match across all fields
 *   "quoted text"     → exact phrase match
 *   type:item         → restrict to entity type
 *   skill:Sword       → entities associated with a skill
 *   area:Serbule      → entities in a zone
 *   level:30-50       → level range
 *   keyword:Food      → item keyword
 *   has:recipe        → items with recipes
 *   slot:MainHand     → equipment slot
 *   name:sword        → restrict match to name only
 *   -keyword:X        → exclusion (negate any filter with -)
 *
 * Multiple filters AND together.
 */

const KNOWN_FILTER_KEYS = new Set([
  "type", "skill", "area", "level", "keyword", "has", "slot", "name",
])

// ── Types ────────────────────────────────────────────────────────────────────

export type SearchFilter =
  | { kind: "type"; value: string }
  | { kind: "skill"; value: string }
  | { kind: "area"; value: string }
  | { kind: "level"; min?: number; max?: number }
  | { kind: "keyword"; value: string }
  | { kind: "has"; value: string }
  | { kind: "slot"; value: string }
  | { kind: "name"; value: string }

export interface ParsedQuery {
  /** Free-text terms (lowercased), ANDed together */
  textTerms: string[]
  /** Exact-phrase matches (lowercased, without quotes) */
  exactPhrases: string[]
  /** Structured filters */
  filters: SearchFilter[]
  /** Negated filters (prefixed with -) */
  negations: SearchFilter[]
}

// ── Parser ───────────────────────────────────────────────────────────────────

export function parseQuery(input: string): ParsedQuery {
  const textTerms: string[] = []
  const exactPhrases: string[] = []
  const filters: SearchFilter[] = []
  const negations: SearchFilter[] = []

  const trimmed = input.trim()
  const len = trimmed.length
  let i = 0

  while (i < len) {
    // Skip whitespace
    if (trimmed[i] === " " || trimmed[i] === "\t") {
      i++
      continue
    }

    // Quoted phrase
    if (trimmed[i] === '"') {
      i++
      const start = i
      while (i < len && trimmed[i] !== '"') i++
      const phrase = trimmed.slice(start, i).trim().toLowerCase()
      if (phrase) exactPhrases.push(phrase)
      if (i < len) i++ // skip closing quote
      continue
    }

    // Collect token (up to next whitespace or quote)
    const start = i
    while (i < len && trimmed[i] !== " " && trimmed[i] !== "\t" && trimmed[i] !== '"') i++
    const token = trimmed.slice(start, i)

    // Check negation prefix
    const negated = token.startsWith("-") && token.length > 1
    const body = negated ? token.slice(1) : token

    // Check for filter key:value
    const colonPos = body.indexOf(":")
    if (colonPos > 0) {
      const key = body.slice(0, colonPos).toLowerCase()
      const value = body.slice(colonPos + 1)

      if (KNOWN_FILTER_KEYS.has(key) && value) {
        const filter = buildFilter(key, value)
        if (filter) {
          ;(negated ? negations : filters).push(filter)
          continue
        }
      }
    }

    // Plain text term (re-add - if it was there but didn't match a filter)
    const fullToken = negated ? `-${body}` : body
    textTerms.push(fullToken.toLowerCase())
  }

  return { textTerms, exactPhrases, filters, negations }
}

function buildFilter(key: string, value: string): SearchFilter | null {
  const lower = value.toLowerCase()
  switch (key) {
    case "type":
      return { kind: "type", value: lower }
    case "skill":
      return { kind: "skill", value: lower }
    case "area":
      return { kind: "area", value: lower }
    case "level": {
      const dash = value.indexOf("-")
      if (dash >= 0) {
        const min = value.slice(0, dash) ? parseInt(value.slice(0, dash), 10) : undefined
        const max = value.slice(dash + 1) ? parseInt(value.slice(dash + 1), 10) : undefined
        return { kind: "level", min: Number.isNaN(min!) ? undefined : min, max: Number.isNaN(max!) ? undefined : max }
      }
      const exact = parseInt(value, 10)
      return Number.isNaN(exact) ? null : { kind: "level", min: exact, max: exact }
    }
    case "keyword":
      return { kind: "keyword", value: lower }
    case "has":
      return { kind: "has", value: lower }
    case "slot":
      return { kind: "slot", value: lower }
    case "name":
      return { kind: "name", value: lower }
    default:
      return null
  }
}

// ── Query helpers ────────────────────────────────────────────────────────────

/** Check if query has any meaningful content (text, phrases, or filters) */
export function isEmptyQuery(q: ParsedQuery): boolean {
  return q.textTerms.length === 0
    && q.exactPhrases.length === 0
    && q.filters.length === 0
    && q.negations.length === 0
}

/** Check if query is only filters with no text */
export function isFilterOnly(q: ParsedQuery): boolean {
  return q.textTerms.length === 0
    && q.exactPhrases.length === 0
    && (q.filters.length > 0 || q.negations.length > 0)
}

/** Get the raw text portion of the query (terms + phrases joined) */
export function getTextQuery(q: ParsedQuery): string {
  return [...q.textTerms, ...q.exactPhrases].join(" ")
}

/** Get a specific filter value */
export function getFilter(q: ParsedQuery, kind: SearchFilter["kind"]): SearchFilter | undefined {
  return q.filters.find(f => f.kind === kind)
}

/** Get all filter values of a kind */
export function getFilters(q: ParsedQuery, kind: SearchFilter["kind"]): SearchFilter[] {
  return q.filters.filter(f => f.kind === kind)
}

/** Get negated filter values of a kind */
export function getNegatedFilters(q: ParsedQuery, kind: SearchFilter["kind"]): SearchFilter[] {
  return q.negations.filter(f => f.kind === kind)
}

// ── Text matching helpers ────────────────────────────────────────────────────

/** Check if a combined text matches all terms and phrases */
export function matchesQuery(searchableText: string, q: ParsedQuery): boolean {
  const lower = searchableText.toLowerCase()
  for (const term of q.textTerms) {
    if (!lower.includes(term)) return false
  }
  for (const phrase of q.exactPhrases) {
    if (!lower.includes(phrase)) return false
  }
  return true
}

/**
 * Build a searchable string from multiple fields.
 * Joins non-empty values with spaces and lowercases.
 */
export function combineFields(...fields: (string | undefined | null)[]): string {
  return fields.filter(Boolean).join(" ")
}
