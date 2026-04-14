// TypeScript shapes that mirror the Rust serde structs in
// src-tauri/src/db/stall_tracker_commands.rs and src-tauri/src/stall_aggregations.rs.
//
// Keep this file in sync when the Rust types change.
//
// Naming convention:
// - Response types use snake_case because the Rust structs have no
//   `rename_all` attribute (StallEvent, StallStats, RevenueResult, ...).
// - Request/param types use camelCase because the Rust structs are
//   `#[serde(rename_all = "camelCase")]` (StallEventsFilters, StallEventsParams,
//   StallRevenueParams, StallInventoryParams).

export interface StallEvent {
  id: number;
  event_timestamp: string;
  event_at: string | null;
  log_timestamp: string;
  log_title: string;
  action: string;
  player: string;
  owner: string | null;
  item: string | null;
  quantity: number;
  price_unit: number | null;
  price_total: number | null;
  raw_message: string;
  entry_index: number;
  ignored: boolean;
  created_at: string;
}

export interface StallEventsPage {
  rows: StallEvent[];
  total_count: number;
}

export interface StallStats {
  total_sales: number;
  total_revenue: number;
  unique_buyers: number;
  unique_items: number;
}

export interface StallFilterOptions {
  buyers: string[];
  players: string[];
  items: string[];
  dates: string[];
  actions: string[];
}

/** Filter shape sent to backend. camelCase to match Rust serde rename. */
export interface StallEventsFilters {
  owner?: string | null;
  action?: string | null;
  player?: string | null;
  item?: string | null;
  dateFrom?: string | null;
  dateTo?: string | null;
  includeIgnored?: boolean | null;
}

/** Params for `get_stall_events`. Filters are flattened into the top level. */
export interface StallEventsParams extends StallEventsFilters {
  sortBy?: string | null;
  sortDir?: 'asc' | 'desc' | null;
  limit?: number | null;
  offset?: number | null;
  forceAction?: string | null;
}

// ── Revenue (aggregations) ──────────────────────────────────────────────

export type Granularity = 'daily' | 'weekly' | 'monthly';

export interface RevenuePeriod {
  key: string;
  label: string;
}

export interface RevenueCell {
  item: string;
  period_key: string;
  revenue: number;
}

export interface RevenueResult {
  periods: RevenuePeriod[];
  items: string[];
  cells: RevenueCell[];
  /** Per-item totals as `[item, total]` pairs, sorted by item. */
  row_totals: [string, number][];
  /** Per-period totals as `[period_key, total]` pairs, sorted by key. */
  col_totals: [string, number][];
  grand_total: number;
}

export interface StallRevenueParams {
  owner?: string | null;
  granularity?: Granularity | null;
  dateFrom?: string | null;
  dateTo?: string | null;
  player?: string | null;
  item?: string | null;
}

// ── Inventory (aggregations) ────────────────────────────────────────────

export interface PriceTier {
  qty: number;
  /** `null` for unpriced tiers (added but never made visible). */
  price: number | null;
}

export interface InventoryItem {
  item: string;
  quantity: number;
  price_tiers: PriceTier[];
  estimated_value: number;
  period_sold: number;
  period_revenue: number;
  avg_per_day: number;
  last_sold_at: string | null;
  last_activity_at: string | null;
  /** Most recent priced event seen for this item — survives the sellout
   * collapse so Recently Sold Out can still display a last price. */
  last_known_price: number | null;
}

export interface InventoryResult {
  items: InventoryItem[];
  /** Distinct dates with activity, **newest-first**. */
  active_dates: string[];
  estimated_value: number;
  total_sold: number;
  avg_daily_revenue: number;
}

export interface StallInventoryParams {
  owner?: string | null;
  periodDays?: number | null;
}

// ── Import / Export ─────────────────────────────────────────────────────

export interface ImportResult {
  total_entries: number;
  new_entries: number;
  /** The owner the rows were stamped with — parser hint or current_owner fallback. */
  effective_owner: string | null;
  /** True when the file had no owner actions and was claimed for the active character. */
  owner_claimed: boolean;
}

export interface ExportResult {
  files_written: number;
  events_exported: number;
}
