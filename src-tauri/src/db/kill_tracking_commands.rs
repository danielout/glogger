/// Kill tracking queries — enemy kill stats and loot drop rates
use super::DbPool;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct EnemyLootDrop {
    pub item_name: String,
    pub total_quantity: i64,
    pub times_dropped: i64,
    /// How many kills had this item drop (times_dropped / total_kills)
    pub drop_rate: f64,
}

#[derive(Serialize)]
pub struct EnemyKillStats {
    pub enemy_name: String,
    pub total_kills: i64,
    pub loot: Vec<EnemyLootDrop>,
}

#[tauri::command]
pub fn get_enemy_kill_stats(
    db: State<'_, DbPool>,
    enemy_name: String,
) -> Result<EnemyKillStats, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Total kills for this enemy name
    let total_kills: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM enemy_kills WHERE enemy_name = ?1",
            [&enemy_name],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if total_kills == 0 {
        return Ok(EnemyKillStats {
            enemy_name,
            total_kills: 0,
            loot: Vec::new(),
        });
    }

    // Aggregate loot drops: item_name, total quantity, number of distinct kills that dropped it
    let mut stmt = conn
        .prepare(
            "SELECT l.item_name,
                    SUM(l.quantity) as total_qty,
                    COUNT(DISTINCT l.kill_id) as times_dropped
             FROM enemy_kill_loot l
             JOIN enemy_kills k ON l.kill_id = k.id
             WHERE k.enemy_name = ?1
             GROUP BY l.item_name
             ORDER BY times_dropped DESC, total_qty DESC",
        )
        .map_err(|e| format!("Failed to prepare loot query: {e}"))?;

    let loot_rows = stmt
        .query_map([&enemy_name], |row| {
            let times_dropped: i64 = row.get(2)?;
            Ok(EnemyLootDrop {
                item_name: row.get(0)?,
                total_quantity: row.get(1)?,
                times_dropped,
                drop_rate: times_dropped as f64 / total_kills as f64,
            })
        })
        .map_err(|e| format!("Loot query failed: {e}"))?;

    let mut loot = Vec::new();
    for row in loot_rows {
        loot.push(row.map_err(|e| format!("Loot row error: {e}"))?);
    }

    Ok(EnemyKillStats {
        enemy_name,
        total_kills,
        loot,
    })
}
