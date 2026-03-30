use super::DbPool;
use crate::settings::{resolve_item_value, SettingsManager};
use serde::Serialize;
/// Tauri commands for aggregate views across all characters on a server.
use std::sync::Arc;
use tauri::State;

// ── Response Types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AggregateInventoryItem {
    pub item_name: String,
    pub total_stack_size: i64,
    pub character_count: i32,
    pub characters: Vec<CharacterItemBreakdown>,
}

#[derive(Serialize)]
pub struct CharacterItemBreakdown {
    pub character_name: String,
    pub stack_size: i64,
}

#[derive(Serialize)]
pub struct AggregateCurrency {
    pub currency_name: String,
    pub total_amount: f64,
    pub characters: Vec<CharacterCurrencyBreakdown>,
}

#[derive(Serialize)]
pub struct CharacterCurrencyBreakdown {
    pub character_name: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct AggregateWealth {
    pub total_currency: f64,
    pub total_market_value: i64,
    pub grand_total: i64,
    pub currencies: Vec<AggregateCurrency>,
    pub per_character: Vec<CharacterWealth>,
}

#[derive(Serialize)]
pub struct CharacterWealth {
    pub character_name: String,
    pub currency_total: f64,
    pub market_value_total: i64,
}

#[derive(Serialize)]
pub struct AggregateSkillSummary {
    pub skill_name: String,
    pub characters: Vec<CharacterSkillEntry>,
}

#[derive(Serialize)]
pub struct CharacterSkillEntry {
    pub character_name: String,
    pub level: i32,
    pub xp: i64,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_aggregate_inventory(
    db: State<'_, DbPool>,
    server_name: String,
) -> Result<Vec<AggregateInventoryItem>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;

    // Get all inventory items for the server, grouped by item_name
    let mut stmt = conn
        .prepare(
            "SELECT item_name, character_name, SUM(stack_size) as total
         FROM game_state_inventory
         WHERE server_name = ?1
         GROUP BY item_name, character_name
         ORDER BY item_name, character_name",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows: Vec<(String, String, i64)> = stmt
        .query_map(rusqlite::params![server_name], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("Query error: {e}"))?
        .flatten()
        .collect();

    // Group by item_name
    let mut items: std::collections::BTreeMap<String, Vec<CharacterItemBreakdown>> =
        std::collections::BTreeMap::new();
    for (item_name, character_name, stack_size) in rows {
        items
            .entry(item_name)
            .or_default()
            .push(CharacterItemBreakdown {
                character_name,
                stack_size,
            });
    }

    let result: Vec<AggregateInventoryItem> = items
        .into_iter()
        .map(|(item_name, characters)| {
            let total_stack_size: i64 = characters.iter().map(|c| c.stack_size).sum();
            let character_count = characters.len() as i32;
            AggregateInventoryItem {
                item_name,
                total_stack_size,
                character_count,
                characters,
            }
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub fn get_aggregate_wealth(
    db: State<'_, DbPool>,
    settings_manager: State<'_, Arc<SettingsManager>>,
    server_name: String,
) -> Result<AggregateWealth, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;

    // 1. Aggregate currencies
    let mut cur_stmt = conn
        .prepare(
            "SELECT currency_name, character_name, amount
         FROM game_state_currencies
         WHERE server_name = ?1
         ORDER BY currency_name, character_name",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let cur_rows: Vec<(String, String, f64)> = cur_stmt
        .query_map(rusqlite::params![server_name], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("Query error: {e}"))?
        .flatten()
        .collect();

    let mut currency_map: std::collections::BTreeMap<String, Vec<CharacterCurrencyBreakdown>> =
        std::collections::BTreeMap::new();
    for (currency_name, character_name, amount) in &cur_rows {
        currency_map
            .entry(currency_name.clone())
            .or_default()
            .push(CharacterCurrencyBreakdown {
                character_name: character_name.clone(),
                amount: *amount,
            });
    }

    let currencies: Vec<AggregateCurrency> = currency_map
        .into_iter()
        .map(|(currency_name, characters)| {
            let total_amount: f64 = characters.iter().map(|c| c.amount).sum();
            AggregateCurrency {
                currency_name,
                total_amount,
                characters,
            }
        })
        .collect();

    let total_currency: f64 = currencies.iter().map(|c| c.total_amount).sum();

    // 2. Compute inventory value using item valuation mode
    let settings = settings_manager.get();
    let market_mode = &settings.market_price_mode;
    let valuation_mode = &settings.item_valuation_mode;
    let market_server = if market_mode == "universal" {
        "*".to_string()
    } else {
        server_name.clone()
    };

    let mut inv_stmt = conn
        .prepare(
            "SELECT i.character_name, i.item_name, i.stack_size,
                COALESCE(m.market_value, 0) as mv,
                COALESCE(items.value, 0) as vendor_val
         FROM game_state_inventory i
         LEFT JOIN market_values m ON m.item_type_id = i.item_type_id AND m.server_name = ?2
         LEFT JOIN items ON items.id = i.item_type_id
         WHERE i.server_name = ?1",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let inv_rows: Vec<(String, i64, f64, f64)> = inv_stmt
        .query_map(rusqlite::params![server_name, market_server], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
            ))
        })
        .map_err(|e| format!("Query error: {e}"))?
        .flatten()
        .collect();

    // Per-character inventory value totals (using valuation mode)
    let mut char_market: std::collections::BTreeMap<String, i64> =
        std::collections::BTreeMap::new();
    for (character_name, stack_size, market_val, vendor_val) in &inv_rows {
        let effective = resolve_item_value(valuation_mode, *vendor_val, *market_val);
        *char_market.entry(character_name.clone()).or_default() +=
            (*stack_size as f64 * effective) as i64;
    }

    let total_market_value: i64 = char_market.values().sum();

    // 3. Per-character currency totals
    let mut char_currency: std::collections::BTreeMap<String, f64> =
        std::collections::BTreeMap::new();
    for (_, character_name, amount) in &cur_rows {
        *char_currency.entry(character_name.clone()).or_default() += amount;
    }

    // 4. Merge per-character wealth
    let mut all_chars: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for k in char_currency.keys() {
        all_chars.insert(k.clone());
    }
    for k in char_market.keys() {
        all_chars.insert(k.clone());
    }

    let per_character: Vec<CharacterWealth> = all_chars
        .into_iter()
        .map(|name| {
            let ct = *char_currency.get(&name).unwrap_or(&0.0);
            let mv = *char_market.get(&name).unwrap_or(&0);
            CharacterWealth {
                character_name: name,
                currency_total: ct,
                market_value_total: mv,
            }
        })
        .collect();

    let grand_total = total_currency as i64 + total_market_value;

    Ok(AggregateWealth {
        total_currency,
        total_market_value,
        grand_total,
        currencies,
        per_character,
    })
}

#[tauri::command]
pub fn get_aggregate_skills(
    db: State<'_, DbPool>,
    server_name: String,
) -> Result<Vec<AggregateSkillSummary>, String> {
    let conn = db.get().map_err(|e| format!("Database error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT skill_name, character_name, level, xp
         FROM game_state_skills
         WHERE server_name = ?1
         ORDER BY skill_name, character_name",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows: Vec<(String, String, i32, i64)> = stmt
        .query_map(rusqlite::params![server_name], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| format!("Query error: {e}"))?
        .flatten()
        .collect();

    let mut skills: std::collections::BTreeMap<String, Vec<CharacterSkillEntry>> =
        std::collections::BTreeMap::new();
    for (skill_name, character_name, level, xp) in rows {
        skills
            .entry(skill_name)
            .or_default()
            .push(CharacterSkillEntry {
                character_name,
                level,
                xp,
            });
    }

    Ok(skills
        .into_iter()
        .map(|(skill_name, characters)| AggregateSkillSummary {
            skill_name,
            characters,
        })
        .collect())
}
