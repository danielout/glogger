import { ref, computed, type Ref } from "vue"
import {
  parseQuery,
  matchesQuery,
  isEmptyQuery,
  getFilter,
  getFilters,
  getNegatedFilters,
  type ParsedQuery,
} from "../utils/SearchParser"

// ── Types ────────────────────────────────────────────────────────────────────

/**
 * Defines how an entity's fields map to search features.
 * Each browser provides one of these to describe its data.
 */
export interface EntitySearchConfig<T> {
  /**
   * Build the combined searchable text for an entity.
   * This text is matched against free-text terms and exact phrases.
   */
  searchText: (item: T) => string

  /**
   * Optional: apply structured filters that go beyond text matching.
   * Return false to exclude the item. Called after text matching passes.
   * The parsed query is provided so the function can check any filters.
   */
  applyFilters?: (item: T, query: ParsedQuery) => boolean
}

// ── Composable ───────────────────────────────────────────────────────────────

/**
 * Shared search composable for data browsers.
 *
 * Parses the unified search syntax and filters an array of entities
 * client-side. Browsers keep their existing data loading — this only
 * replaces the filter logic.
 *
 * @param items - Reactive ref to the full entity array
 * @param config - Describes how fields map to search
 * @returns query ref (bind to input), filtered items, parsed query, loading state
 */
export function useDataBrowserSearch<T>(
  items: Ref<T[]>,
  config: EntitySearchConfig<T>,
) {
  const query = ref("")
  const parsedQuery = computed(() => parseQuery(query.value))

  const filtered = computed(() => {
    const q = parsedQuery.value

    // Empty query → show all items (most browsers show full list by default)
    if (isEmptyQuery(q)) {
      return items.value
    }

    return items.value.filter(item => {
      // Text matching against combined searchable text
      const hasText = q.textTerms.length > 0 || q.exactPhrases.length > 0
      if (hasText) {
        const searchable = config.searchText(item)
        if (!matchesQuery(searchable, q)) return false
      }

      // Structured filters
      if (config.applyFilters) {
        if (!config.applyFilters(item, q)) return false
      }

      return true
    })
  })

  /** Whether the query has any structured filters active */
  const hasFilters = computed(() =>
    parsedQuery.value.filters.length > 0 || parsedQuery.value.negations.length > 0
  )

  /** Whether the query has text content */
  const hasText = computed(() =>
    parsedQuery.value.textTerms.length > 0 || parsedQuery.value.exactPhrases.length > 0
  )

  return {
    query,
    parsedQuery,
    filtered,
    hasFilters,
    hasText,
  }
}

// ── Reusable filter helpers ──────────────────────────────────────────────────

/**
 * Check keyword filters against an entity's keyword list.
 * Handles both positive (keyword:X) and negative (-keyword:X) filters.
 */
export function checkKeywordFilters(
  keywords: string[],
  query: ParsedQuery,
): boolean {
  const required = getFilters(query, "keyword")
  const excluded = getNegatedFilters(query, "keyword")

  // Check exclusions
  for (const f of excluded) {
    if (f.kind !== "keyword") continue
    if (keywords.some(k => k.toLowerCase() === f.value)) return false
  }

  // Check requirements
  for (const f of required) {
    if (f.kind !== "keyword") continue
    if (!keywords.some(k => k.toLowerCase() === f.value)) return false
  }

  return true
}

/**
 * Check skill filter against an entity's skill field.
 */
export function checkSkillFilter(
  skill: string | undefined | null,
  query: ParsedQuery,
): boolean {
  const f = getFilter(query, "skill")
  if (!f || f.kind !== "skill") return true
  if (!skill) return false
  return skill.toLowerCase().includes(f.value)
}

/**
 * Check area filter against an entity's area field.
 */
export function checkAreaFilter(
  area: string | undefined | null,
  query: ParsedQuery,
): boolean {
  const f = getFilter(query, "area")
  if (!f || f.kind !== "area") return true
  if (!area) return false
  return area.toLowerCase().includes(f.value)
}

/**
 * Check level filter against a numeric level.
 */
export function checkLevelFilter(
  level: number | undefined | null,
  query: ParsedQuery,
): boolean {
  const f = getFilter(query, "level")
  if (!f || f.kind !== "level") return true
  if (level == null) return false
  if (f.min != null && level < f.min) return false
  if (f.max != null && level > f.max) return false
  return true
}

/**
 * Check slot filter against an entity's equip slot.
 */
export function checkSlotFilter(
  slot: string | undefined | null,
  query: ParsedQuery,
): boolean {
  const f = getFilter(query, "slot")
  if (!f || f.kind !== "slot") return true
  if (!slot) return false
  return slot.toLowerCase().includes(f.value)
}

/**
 * Check name filter — restricts text match to name field only.
 */
export function checkNameFilter(
  name: string,
  query: ParsedQuery,
): boolean {
  const f = getFilter(query, "name")
  if (!f || f.kind !== "name") return true
  return name.toLowerCase().includes(f.value)
}
