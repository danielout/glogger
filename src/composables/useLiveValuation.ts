// Reactive item valuation — resolves unit_value for a LootSummaryRow
// using the live marketStore + the user's valuation-mode preference.
// When a market price changes, any computed that calls liveUnitValue()
// recomputes automatically via Vue reactivity.
//
// Used by SessionSummary and HistoricalTab to keep revenue/profit in
// sync with the Market Prices tab without re-fetching from the backend.

import { computed, type ComputedRef } from 'vue'
import { useMarketStore } from '../stores/marketStore'
import { useSettingsStore } from '../stores/settingsStore'
import type { LootSummaryRow } from '../stores/surveyTrackerStore'

/** Resolve the effective unit value for a loot row using live market data.
 *  Returns the backend's snapshot when no market data is available. */
export function liveUnitValue(row: LootSummaryRow): number | null {
  const marketStore = useMarketStore()
  const settingsStore = useSettingsStore()
  const mode = settingsStore.settings.itemValuationMode

  const market = row.item_type_id !== null
    ? (marketStore.valuesByItemId[row.item_type_id]?.market_value ?? null)
    : null

  if (market !== null) {
    if (mode === 'market_only') return market
    // For combined modes, take the higher of fresh market and the
    // backend's vendor-based snapshot.
    return Math.max(market, row.unit_value ?? 0)
  }
  // No market price — backend's snapshot (vendor-based) is authoritative.
  return row.unit_value
}

/** Enrich a loot-summary array with live pricing. Returns a new array
 *  (same items, updated unit_value / total_value). Reactive — call
 *  inside a `computed()` to auto-update on market changes. */
export function liveEnrichedRows(rows: LootSummaryRow[]): LootSummaryRow[] {
  return rows.map(row => {
    const uv = liveUnitValue(row)
    return {
      ...row,
      unit_value: uv,
      total_value: uv !== null ? uv * row.total_qty : null,
    }
  })
}

/** Compute total revenue from enriched rows. */
export function liveRevenue(rows: LootSummaryRow[]): number {
  return rows.reduce((sum, r) => sum + (r.total_value ?? 0), 0)
}

/** Compute profit from enriched rows + a cost total. */
export function liveProfit(rows: LootSummaryRow[], costTotal: number): number {
  return liveRevenue(rows) - costTotal
}

/** Create a reactive profit computed for a single HistoricalSessionRow.
 *  Reads marketStore reactively so it updates when prices change. */
export function useRowProfit(
  getRows: () => LootSummaryRow[],
  getCost: () => number,
): ComputedRef<number> {
  return computed(() => {
    const enriched = liveEnrichedRows(getRows())
    return liveProfit(enriched, getCost())
  })
}
