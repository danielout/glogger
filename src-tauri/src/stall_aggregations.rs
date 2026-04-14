//! Pure aggregation logic for stall tracker Revenue and Inventory tabs.
//!
//! Runs in Rust so the IPC bridge only ships small summary structures
//! instead of raw event rows. No database access in this module — callers
//! fetch events from SQLite and feed them in.

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use serde::Serialize;
use std::collections::BTreeMap;

// ── Revenue ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum Granularity {
    Daily,
    Weekly,
    Monthly,
}

impl Granularity {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            "monthly" => Some(Self::Monthly),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct RevenuePeriod {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct RevenueCell {
    pub item: String,
    pub period_key: String,
    pub revenue: i64,
}

#[derive(Debug, Serialize)]
pub struct RevenueResult {
    pub periods: Vec<RevenuePeriod>,
    pub items: Vec<String>,
    pub cells: Vec<RevenueCell>,
    pub row_totals: Vec<(String, i64)>,
    pub col_totals: Vec<(String, i64)>,
    pub grand_total: i64,
}

pub struct RevenueEvent {
    pub event_at: NaiveDateTime,
    pub item: String,
    pub revenue: i64,
}

fn trim_day(s: String) -> String {
    // chrono's %e pads with a leading space; we don't want that in labels.
    s.trim().replace("  ", " ")
}

/// `show_year` controls whether daily/weekly labels include the year suffix.
/// Enabled when the dataset spans multiple calendar years so otherwise-
/// identical labels like "Apr 13" don't collide across years.
fn period_for(dt: NaiveDateTime, g: Granularity, show_year: bool) -> (String, String) {
    match g {
        Granularity::Daily => {
            let key = dt.format("%Y-%m-%d").to_string();
            let base = trim_day(dt.format("%b %e").to_string());
            let label = if show_year {
                format!("{} {}", base, dt.format("%Y"))
            } else {
                base
            };
            (key, label)
        }
        Granularity::Weekly => {
            let iso = dt.date().iso_week();
            let key = format!("{}-W{:02}", iso.year(), iso.week());
            let monday = NaiveDate::from_isoywd_opt(iso.year(), iso.week(), chrono::Weekday::Mon)
                .unwrap_or_else(|| dt.date());
            let sunday = monday + chrono::Duration::days(6);
            let base = format!(
                "{} – {}",
                trim_day(monday.format("%b %e").to_string()),
                trim_day(sunday.format("%b %e").to_string()),
            );
            let label = if show_year {
                format!("{} {}", base, iso.year())
            } else {
                base
            };
            (key, label)
        }
        Granularity::Monthly => {
            // Monthly labels already include year — no change needed.
            let key = dt.format("%Y-%m").to_string();
            let label = dt.format("%b %Y").to_string();
            (key, label)
        }
    }
}

pub fn aggregate_revenue(
    events: impl IntoIterator<Item = RevenueEvent>,
    granularity: Granularity,
) -> RevenueResult {
    // Collect up front so we can detect year-span before labeling.
    let events: Vec<RevenueEvent> = events.into_iter().collect();
    let mut years = std::collections::HashSet::new();
    for e in &events {
        years.insert(e.event_at.date().year());
    }
    let show_year = years.len() > 1;

    let mut cells: BTreeMap<(String, String), i64> = BTreeMap::new();
    let mut period_labels: BTreeMap<String, String> = BTreeMap::new();
    let mut row_totals: BTreeMap<String, i64> = BTreeMap::new();
    let mut col_totals: BTreeMap<String, i64> = BTreeMap::new();
    let mut grand_total: i64 = 0;

    for e in events {
        let (pk, pl) = period_for(e.event_at, granularity, show_year);
        period_labels.entry(pk.clone()).or_insert(pl);
        *cells.entry((e.item.clone(), pk.clone())).or_insert(0) += e.revenue;
        *row_totals.entry(e.item).or_insert(0) += e.revenue;
        *col_totals.entry(pk).or_insert(0) += e.revenue;
        grand_total += e.revenue;
    }

    // BTreeMap iteration is already sorted. Period keys (YYYY-MM-DD, YYYY-Www,
    // YYYY-MM) all sort correctly lexicographically.
    let periods = period_labels
        .into_iter()
        .map(|(key, label)| RevenuePeriod { key, label })
        .collect();
    let items: Vec<String> = row_totals.keys().cloned().collect();
    let cells_vec = cells
        .into_iter()
        .map(|((item, period_key), revenue)| RevenueCell {
            item,
            period_key,
            revenue,
        })
        .collect();
    let row_totals_vec = row_totals.into_iter().collect();
    let col_totals_vec = col_totals.into_iter().collect();

    RevenueResult {
        periods,
        items,
        cells: cells_vec,
        row_totals: row_totals_vec,
        col_totals: col_totals_vec,
        grand_total,
    }
}

// ── Inventory ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct PriceTier {
    pub qty: i64,
    pub price: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct InventoryItem {
    pub name: String,
    pub quantity: i64,
    pub price_tiers: Vec<PriceTier>,
    pub last_price: Option<i64>,
    pub value: i64,
    pub last_sold_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub period_sold: i64,
    pub period_revenue: i64,
    pub avg_per_day: f64,
}

#[derive(Debug, Serialize)]
pub struct InventoryResult {
    pub items: Vec<InventoryItem>,
    pub estimated_value: i64,
    pub total_sold: i64,
    pub avg_daily_revenue: i64,
    /// Distinct activity dates (YYYY-MM-DD), newest first. Used by the frontend
    /// for "sold out in last N active days" windows to match the old semantics
    /// of counting distinct activity dates rather than calendar days.
    pub active_dates: Vec<String>,
}

pub struct InventoryEvent {
    pub event_at: NaiveDateTime,
    pub action: String,
    pub item: String,
    pub quantity: i64,
    pub price_unit: Option<f64>,
    pub price_total: Option<i64>,
}

#[derive(Default)]
struct ItemState {
    tiers: Vec<PriceTier>,
    last_price: Option<i64>,
    last_sold_at: Option<NaiveDateTime>,
    last_activity_at: Option<NaiveDateTime>,
    period_sold: i64,
    period_revenue: i64,
}

fn total_qty(tiers: &[PriceTier]) -> i64 {
    tiers.iter().map(|t| t.qty).sum()
}

/// Aggregate shop-log events into per-item inventory snapshots.
///
/// `events` MUST be sorted chronologically (event_at ASC, id ASC as tiebreaker).
/// `period_days`:
///   - Positive finite → take the N most-recent distinct activity dates as the
///     sales window; divisor for `avg_per_day` is `period_days`.
///   - Zero / negative / very large → "all time"; window is every distinct
///     activity date in the dataset; divisor is `max(distinct_date_count, 1)`.
///
/// This preserves the old JS semantics of counting *active* dates, so sparse
/// logs with gaps don't have their averages diluted.
pub fn aggregate_inventory(
    events: impl IntoIterator<Item = InventoryEvent>,
    period_days: f64,
) -> InventoryResult {
    let events: Vec<InventoryEvent> = events.into_iter().collect();

    // Collect distinct activity dates (YYYY-MM-DD), then pick the window.
    let mut distinct_dates: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for e in &events {
        distinct_dates.insert(e.event_at.format("%Y-%m-%d").to_string());
    }
    let dates_desc: Vec<String> = distinct_dates.iter().rev().cloned().collect();

    let all_time = !period_days.is_finite() || period_days <= 0.0 || period_days >= 99_999.0;
    let window_n = if all_time {
        dates_desc.len()
    } else {
        (period_days as usize).min(dates_desc.len())
    };
    let window_dates: std::collections::HashSet<&str> = dates_desc
        .iter()
        .take(window_n)
        .map(|s| s.as_str())
        .collect();
    let divisor: f64 = if all_time {
        dates_desc.len().max(1) as f64
    } else {
        period_days
    };

    let mut items: BTreeMap<String, ItemState> = BTreeMap::new();

    for e in events {
        let state = items.entry(e.item.clone()).or_default();
        if state.last_activity_at.map_or(true, |prev| e.event_at > prev) {
            state.last_activity_at = Some(e.event_at);
        }

        match e.action.as_str() {
            "added" => {
                if total_qty(&state.tiers) < 0 {
                    state.tiers.clear();
                }
                state.tiers.push(PriceTier {
                    qty: e.quantity,
                    price: None,
                });
            }
            "visible" | "configured" => {
                if let Some(pu) = e.price_unit {
                    let price = pu.round() as i64;
                    state.last_price = Some(price);
                    let mut remaining = e.quantity;
                    let mut found_unpriced = false;
                    let mut splits: Vec<PriceTier> = Vec::new();
                    for tier in state.tiers.iter_mut() {
                        if remaining <= 0 {
                            break;
                        }
                        if tier.price.is_none() && tier.qty > 0 {
                            let apply = tier.qty.min(remaining);
                            if apply == tier.qty {
                                tier.price = Some(price);
                            } else {
                                tier.qty -= apply;
                                splits.push(PriceTier {
                                    qty: apply,
                                    price: Some(price),
                                });
                            }
                            remaining -= apply;
                            found_unpriced = true;
                        }
                    }
                    state.tiers.extend(splits);
                    if !found_unpriced && !state.tiers.is_empty() {
                        let last = state.tiers.len() - 1;
                        state.tiers[last].price = Some(price);
                    }
                }
            }
            "bought" => {
                if state
                    .last_sold_at
                    .map_or(true, |prev| e.event_at > prev)
                {
                    state.last_sold_at = Some(e.event_at);
                }
                let price = e.price_unit.map(|p| p.round() as i64);
                if let Some(p) = price {
                    state.last_price = Some(p);
                }
                let event_date = e.event_at.format("%Y-%m-%d").to_string();
                if window_dates.contains(event_date.as_str()) {
                    state.period_sold += e.quantity;
                    state.period_revenue += e.price_total.unwrap_or(0);
                }
                let mut remaining = e.quantity;
                if let Some(p) = price {
                    for tier in state.tiers.iter_mut() {
                        if remaining <= 0 {
                            break;
                        }
                        if tier.price == Some(p) && tier.qty > 0 {
                            let take = tier.qty.min(remaining);
                            tier.qty -= take;
                            remaining -= take;
                        }
                    }
                }
                if remaining > 0 {
                    for tier in state.tiers.iter_mut() {
                        if remaining <= 0 {
                            break;
                        }
                        if tier.qty > 0 {
                            let take = tier.qty.min(remaining);
                            tier.qty -= take;
                            remaining -= take;
                        }
                    }
                }
                if remaining > 0 {
                    state.tiers.push(PriceTier {
                        qty: -remaining,
                        price,
                    });
                }
                state.tiers.retain(|t| t.qty != 0);
            }
            "removed" => {
                let mut remaining = e.quantity;
                for tier in state.tiers.iter_mut().rev() {
                    if remaining <= 0 {
                        break;
                    }
                    if tier.qty > 0 {
                        let take = tier.qty.min(remaining);
                        tier.qty -= take;
                        remaining -= take;
                    }
                }
                if remaining > 0 {
                    state.tiers.push(PriceTier {
                        qty: -remaining,
                        price: None,
                    });
                }
                state.tiers.retain(|t| t.qty != 0);
            }
            _ => {}
        }
    }

    // Materialize result items.
    let mut result_items: Vec<InventoryItem> = Vec::with_capacity(items.len());
    let mut total_sold: i64 = 0;
    let mut total_period_revenue: i64 = 0;
    let mut estimated_value: i64 = 0;

    for (name, state) in items {
        // Collapse same-price tiers, drop non-positive.
        let mut merged: BTreeMap<Option<i64>, i64> = BTreeMap::new();
        for tier in &state.tiers {
            if tier.qty <= 0 {
                continue;
            }
            *merged.entry(tier.price).or_insert(0) += tier.qty;
        }
        let mut price_tiers: Vec<PriceTier> = merged
            .into_iter()
            .map(|(price, qty)| PriceTier { qty, price })
            .collect();
        price_tiers.sort_by_key(|t| t.price.unwrap_or(0));

        let quantity: i64 = price_tiers.iter().map(|t| t.qty).sum();
        let value: i64 = price_tiers
            .iter()
            .map(|t| t.qty * t.price.unwrap_or(0))
            .sum();

        total_sold += state.period_sold;
        total_period_revenue += state.period_revenue;
        if quantity > 0 {
            estimated_value += value;
        }

        result_items.push(InventoryItem {
            name,
            quantity,
            price_tiers,
            last_price: state.last_price,
            value,
            last_sold_at: state.last_sold_at.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_activity_at: state
                .last_activity_at
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            period_sold: state.period_sold,
            period_revenue: state.period_revenue,
            avg_per_day: state.period_sold as f64 / divisor,
        });
    }

    let avg_daily_revenue = (total_period_revenue as f64 / divisor).round() as i64;

    InventoryResult {
        items: result_items,
        estimated_value,
        total_sold,
        avg_daily_revenue,
        active_dates: dates_desc,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn revenue_daily_pivot() {
        let events = vec![
            RevenueEvent {
                event_at: dt("2026-04-13 10:00:00"),
                item: "Quality Reins".into(),
                revenue: 4500,
            },
            RevenueEvent {
                event_at: dt("2026-04-13 11:00:00"),
                item: "Quality Reins".into(),
                revenue: 4500,
            },
            RevenueEvent {
                event_at: dt("2026-04-14 09:00:00"),
                item: "Great Saddle".into(),
                revenue: 5000,
            },
        ];
        let r = aggregate_revenue(events, Granularity::Daily);
        assert_eq!(r.periods.len(), 2);
        assert_eq!(r.periods[0].key, "2026-04-13");
        assert_eq!(r.periods[1].key, "2026-04-14");
        assert_eq!(r.grand_total, 14000);
        assert_eq!(r.items, vec!["Great Saddle", "Quality Reins"]);
        assert_eq!(r.cells.len(), 2);
    }

    #[test]
    fn revenue_monthly_collapses_days() {
        let events = vec![
            RevenueEvent {
                event_at: dt("2026-04-01 10:00:00"),
                item: "A".into(),
                revenue: 100,
            },
            RevenueEvent {
                event_at: dt("2026-04-30 10:00:00"),
                item: "A".into(),
                revenue: 200,
            },
            RevenueEvent {
                event_at: dt("2026-05-01 10:00:00"),
                item: "A".into(),
                revenue: 300,
            },
        ];
        let r = aggregate_revenue(events, Granularity::Monthly);
        assert_eq!(r.periods.len(), 2);
        assert_eq!(r.periods[0].key, "2026-04");
        assert_eq!(r.cells[0].revenue, 300); // A in April
        assert_eq!(r.cells[1].revenue, 300); // A in May
    }

    #[test]
    fn inventory_add_then_sell() {
        let events = vec![
            InventoryEvent {
                event_at: dt("2026-04-13 10:00:00"),
                action: "added".into(),
                item: "Reins".into(),
                quantity: 5,
                price_unit: None,
                price_total: None,
            },
            InventoryEvent {
                event_at: dt("2026-04-13 10:01:00"),
                action: "visible".into(),
                item: "Reins".into(),
                quantity: 5,
                price_unit: Some(4500.0),
                price_total: None,
            },
            InventoryEvent {
                event_at: dt("2026-04-13 15:00:00"),
                action: "bought".into(),
                item: "Reins".into(),
                quantity: 2,
                price_unit: Some(4500.0),
                price_total: Some(9000),
            },
        ];
        let r = aggregate_inventory(events, 7.0);
        assert_eq!(r.items.len(), 1);
        let item = &r.items[0];
        assert_eq!(item.quantity, 3);
        assert_eq!(item.price_tiers.len(), 1);
        assert_eq!(item.price_tiers[0].price, Some(4500));
        assert_eq!(item.period_sold, 2);
        assert_eq!(item.period_revenue, 9000);
        assert_eq!(r.estimated_value, 13500);
    }

    fn sparse_events() -> Vec<InventoryEvent> {
        vec![
            InventoryEvent {
                event_at: dt("2026-04-01 10:00:00"),
                action: "added".into(),
                item: "A".into(),
                quantity: 10,
                price_unit: None,
                price_total: None,
            },
            InventoryEvent {
                event_at: dt("2026-04-01 10:01:00"),
                action: "visible".into(),
                item: "A".into(),
                quantity: 10,
                price_unit: Some(1000.0),
                price_total: None,
            },
            InventoryEvent {
                event_at: dt("2026-04-02 10:00:00"),
                action: "bought".into(),
                item: "A".into(),
                quantity: 3,
                price_unit: Some(1000.0),
                price_total: Some(3000),
            },
            InventoryEvent {
                event_at: dt("2026-04-10 10:00:00"),
                action: "bought".into(),
                item: "A".into(),
                quantity: 2,
                price_unit: Some(1000.0),
                price_total: Some(2000),
            },
        ]
    }

    #[test]
    fn inventory_period_scoped_to_recent_active_dates() {
        // 3 distinct active dates: 04-01, 04-02, 04-10. Take 1 most recent = 04-10.
        let r = aggregate_inventory(sparse_events(), 1.0);
        assert_eq!(r.items[0].quantity, 5);
        assert_eq!(r.items[0].period_sold, 2);
        assert_eq!(r.items[0].period_revenue, 2000);
        // active_dates returned newest first
        assert_eq!(r.active_dates, vec!["2026-04-10", "2026-04-02", "2026-04-01"]);
    }

    #[test]
    fn inventory_period_spans_multiple_active_dates() {
        // Take 2 most recent distinct dates: 04-10 and 04-02. Both buys count.
        let r = aggregate_inventory(sparse_events(), 2.0);
        assert_eq!(r.items[0].period_sold, 5);
        assert_eq!(r.items[0].period_revenue, 5000);
    }

    #[test]
    fn inventory_all_time_uses_distinct_date_count_as_divisor() {
        // 3 distinct dates → divisor=3; 5 units sold total → avg_per_day ≈ 1.666
        let r = aggregate_inventory(sparse_events(), 100_000.0);
        assert_eq!(r.items[0].period_sold, 5);
        assert_eq!(r.items[0].period_revenue, 5000);
        assert!((r.items[0].avg_per_day - (5.0 / 3.0)).abs() < 1e-9);
        // avg_daily_revenue = 5000/3 ≈ 1667
        assert_eq!(r.avg_daily_revenue, 1667);
    }
}
