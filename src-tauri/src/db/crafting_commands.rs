use super::DbPool;
use serde::{Deserialize, Serialize};
/// Crafting helper project persistence commands
use std::collections::HashMap;
use tauri::State;

// ── Input types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub notes: Option<String>,
    pub group_name: Option<String>,
    pub fee_config: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProjectInput {
    pub id: i64,
    pub name: String,
    pub notes: String,
    pub group_name: Option<String>,
    pub fee_config: Option<String>,
    pub customer_provides: Option<String>,
}

#[derive(Deserialize)]
pub struct AddProjectEntryInput {
    pub project_id: i64,
    pub recipe_id: i64,
    pub recipe_name: String,
    pub quantity: i32,
    pub target_stock: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateProjectEntryInput {
    pub id: i64,
    pub quantity: i32,
    pub expanded_ingredient_ids: Vec<i64>,
    pub target_stock: Option<i32>,
}

#[derive(Deserialize)]
pub struct ReorderEntriesInput {
    pub project_id: i64,
    pub entry_ids: Vec<i64>,
}

// ── Output types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct CraftingProject {
    pub id: i64,
    pub name: String,
    pub notes: String,
    pub group_name: Option<String>,
    pub fee_config: String,
    pub customer_provides: String,
    pub created_at: String,
    pub updated_at: String,
    pub entries: Vec<CraftingProjectEntry>,
}

#[derive(Serialize)]
pub struct CraftingProjectEntry {
    pub id: i64,
    pub project_id: i64,
    pub recipe_id: i64,
    pub recipe_name: String,
    pub quantity: i32,
    pub sort_order: i32,
    pub expanded_ingredient_ids: Vec<i64>,
    pub target_stock: Option<i32>,
}

#[derive(Serialize)]
pub struct CraftingProjectSummary {
    pub id: i64,
    pub name: String,
    pub notes: String,
    pub group_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub entry_count: i64,
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_crafting_project(
    db: State<'_, DbPool>,
    input: CreateProjectInput,
) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let default_fee = r#"{"per_craft_fee":0,"material_pct":0,"material_pct_basis":"total","flat_fee":0}"#;
    let fee_config = input.fee_config.as_deref().unwrap_or(default_fee);

    conn.execute(
        "INSERT INTO crafting_projects (name, notes, group_name, fee_config) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![input.name, input.notes.unwrap_or_default(), input.group_name, fee_config],
    )
    .map_err(|e| format!("Failed to create crafting project: {e}"))?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn get_crafting_projects(db: State<'_, DbPool>) -> Result<Vec<CraftingProjectSummary>, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.notes, p.group_name, datetime(p.created_at), datetime(p.updated_at),
                (SELECT COUNT(*) FROM crafting_project_entries WHERE project_id = p.id)
         FROM crafting_projects p
         ORDER BY p.updated_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CraftingProjectSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                notes: row.get(2)?,
                group_name: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                entry_count: row.get(6)?,
            })
        })
        .map_err(|e| format!("Query failed: {e}"))?;

    let mut projects = Vec::new();
    for row in rows {
        projects.push(row.map_err(|e| format!("Row parse error: {e}"))?);
    }

    Ok(projects)
}

#[tauri::command]
pub fn get_crafting_project(
    db: State<'_, DbPool>,
    project_id: i64,
) -> Result<CraftingProject, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let project = conn
        .query_row(
            "SELECT id, name, notes, group_name, fee_config, customer_provides, datetime(created_at), datetime(updated_at)
         FROM crafting_projects WHERE id = ?1",
            [project_id],
            |row| {
                Ok(CraftingProject {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    notes: row.get(2)?,
                    group_name: row.get(3)?,
                    fee_config: row.get(4)?,
                    customer_provides: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    entries: Vec::new(),
                })
            },
        )
        .map_err(|e| format!("Project not found: {e}"))?;

    let mut entry_stmt = conn.prepare(
        "SELECT id, project_id, recipe_id, recipe_name, quantity, sort_order, expanded_ingredient_ids, target_stock
         FROM crafting_project_entries
         WHERE project_id = ?1
         ORDER BY sort_order ASC"
    ).map_err(|e| format!("Failed to prepare entry query: {e}"))?;

    let entry_rows = entry_stmt
        .query_map([project_id], |row| {
            let ids_json: String = row.get(6)?;
            let expanded_ids: Vec<i64> = serde_json::from_str(&ids_json).unwrap_or_default();
            Ok(CraftingProjectEntry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                recipe_id: row.get(2)?,
                recipe_name: row.get(3)?,
                quantity: row.get(4)?,
                sort_order: row.get(5)?,
                expanded_ingredient_ids: expanded_ids,
                target_stock: row.get(7)?,
            })
        })
        .map_err(|e| format!("Entry query failed: {e}"))?;

    let mut project = project;
    for row in entry_rows {
        project
            .entries
            .push(row.map_err(|e| format!("Entry row error: {e}"))?);
    }

    Ok(project)
}

#[tauri::command]
pub fn update_crafting_project(
    db: State<'_, DbPool>,
    input: UpdateProjectInput,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute(
        "UPDATE crafting_projects SET name = ?1, notes = ?2, group_name = ?3, fee_config = COALESCE(?4, fee_config), customer_provides = COALESCE(?5, customer_provides), updated_at = CURRENT_TIMESTAMP
         WHERE id = ?6",
        rusqlite::params![input.name, input.notes, input.group_name, input.fee_config, input.customer_provides, input.id],
    )
    .map_err(|e| format!("Failed to update project: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn delete_crafting_project(db: State<'_, DbPool>, project_id: i64) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    conn.execute("DELETE FROM crafting_projects WHERE id = ?1", [project_id])
        .map_err(|e| format!("Failed to delete project: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn add_project_entry(
    db: State<'_, DbPool>,
    input: AddProjectEntryInput,
) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Get next sort_order
    let next_order: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM crafting_project_entries WHERE project_id = ?1",
        [input.project_id],
        |row| row.get(0),
    ).map_err(|e| format!("Failed to get sort order: {e}"))?;

    conn.execute(
        "INSERT INTO crafting_project_entries (project_id, recipe_id, recipe_name, quantity, sort_order, target_stock)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![input.project_id, input.recipe_id, input.recipe_name, input.quantity, next_order, input.target_stock],
    ).map_err(|e| format!("Failed to add entry: {e}"))?;

    // Touch the project's updated_at
    conn.execute(
        "UPDATE crafting_projects SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        [input.project_id],
    )
    .ok();

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_project_entry(
    db: State<'_, DbPool>,
    input: UpdateProjectEntryInput,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let ids_json = serde_json::to_string(&input.expanded_ingredient_ids)
        .map_err(|e| format!("Failed to serialize expanded_ingredient_ids: {e}"))?;
    conn.execute(
        "UPDATE crafting_project_entries SET quantity = ?1, expanded_ingredient_ids = ?2, target_stock = ?3
         WHERE id = ?4",
        rusqlite::params![input.quantity, ids_json, input.target_stock, input.id],
    )
    .map_err(|e| format!("Failed to update entry: {e}"))?;

    // Touch the project's updated_at
    conn.execute(
        "UPDATE crafting_projects SET updated_at = CURRENT_TIMESTAMP
         WHERE id = (SELECT project_id FROM crafting_project_entries WHERE id = ?1)",
        [input.id],
    )
    .ok();

    Ok(())
}

#[tauri::command]
pub fn remove_project_entry(db: State<'_, DbPool>, entry_id: i64) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Touch the project's updated_at before deleting
    conn.execute(
        "UPDATE crafting_projects SET updated_at = CURRENT_TIMESTAMP
         WHERE id = (SELECT project_id FROM crafting_project_entries WHERE id = ?1)",
        [entry_id],
    )
    .ok();

    conn.execute(
        "DELETE FROM crafting_project_entries WHERE id = ?1",
        [entry_id],
    )
    .map_err(|e| format!("Failed to remove entry: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn reorder_project_entries(
    db: State<'_, DbPool>,
    input: ReorderEntriesInput,
) -> Result<(), String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    for (index, entry_id) in input.entry_ids.iter().enumerate() {
        conn.execute(
            "UPDATE crafting_project_entries SET sort_order = ?1
             WHERE id = ?2 AND project_id = ?3",
            rusqlite::params![index as i32, entry_id, input.project_id],
        )
        .map_err(|e| format!("Failed to reorder entry: {e}"))?;
    }

    conn.execute(
        "UPDATE crafting_projects SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        [input.project_id],
    )
    .ok();

    Ok(())
}

#[tauri::command]
pub fn duplicate_crafting_project(db: State<'_, DbPool>, project_id: i64) -> Result<i64, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Get original project
    let (name, notes, group_name, fee_config, customer_provides): (String, String, Option<String>, String, String) = conn
        .query_row(
            "SELECT name, notes, group_name, fee_config, customer_provides FROM crafting_projects WHERE id = ?1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| format!("Project not found: {e}"))?;

    // Create copy
    conn.execute(
        "INSERT INTO crafting_projects (name, notes, group_name, fee_config, customer_provides) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![format!("{name} (copy)"), notes, group_name, fee_config, customer_provides],
    )
    .map_err(|e| format!("Failed to duplicate project: {e}"))?;

    let new_id = conn.last_insert_rowid();

    // Copy entries
    conn.execute(
        "INSERT INTO crafting_project_entries (project_id, recipe_id, recipe_name, quantity, sort_order, expanded_ingredient_ids, target_stock)
         SELECT ?1, recipe_id, recipe_name, quantity, sort_order, expanded_ingredient_ids, target_stock
         FROM crafting_project_entries
         WHERE project_id = ?2
         ORDER BY sort_order",
        rusqlite::params![new_id, project_id],
    ).map_err(|e| format!("Failed to copy entries: {e}"))?;

    Ok(new_id)
}

// ── Material availability ───────────────────────────────────────────────────

#[derive(Serialize)]
pub struct VaultStock {
    pub vault_name: String,
    pub quantity: i64,
}

#[derive(Serialize)]
pub struct MaterialAvailability {
    pub item_type_id: i64,
    pub item_name: String,
    pub inventory_quantity: i64,
    pub storage_quantity: i64,
    pub vault_breakdown: Vec<VaultStock>,
    pub total_available: i64,
}

/// Check material availability across the persisted game state inventory (from log events),
/// the latest storage snapshot, and item name lookups. Takes a list of item type IDs to check.
#[tauri::command]
pub fn check_material_availability(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
    item_type_ids: Vec<i64>,
) -> Result<Vec<MaterialAvailability>, String> {
    if item_type_ids.is_empty() {
        return Ok(Vec::new());
    }

    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    let mut results: HashMap<i64, MaterialAvailability> = HashMap::new();

    // Initialize all requested IDs
    for &id in &item_type_ids {
        results.insert(
            id,
            MaterialAvailability {
                item_type_id: id,
                item_name: String::new(),
                inventory_quantity: 0,
                storage_quantity: 0,
                vault_breakdown: Vec::new(),
                total_available: 0,
            },
        );
    }

    // ── 1. Query persisted inventory from game_state_inventory (log-driven) ────
    {
        let placeholders: Vec<String> = item_type_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 2))
            .collect();
        let placeholders_str = placeholders.join(",");

        let sql = format!(
            "SELECT item_type_id, item_name, SUM(stack_size) as qty
             FROM game_state_inventory
             WHERE character_name = ?1 AND item_type_id IN ({})
             GROUP BY item_type_id",
            placeholders_str
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare game state inventory query: {e}"))?;

        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params.push(Box::new(character_name.clone()));
        for id in &item_type_ids {
            params.push(Box::new(*id));
        }
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok((
                    row.get::<_, i64>(0)?,    // item_type_id
                    row.get::<_, String>(1)?, // item_name
                    row.get::<_, i64>(2)?,    // qty
                ))
            })
            .map_err(|e| format!("Game state inventory query failed: {e}"))?;

        for row in rows {
            let (type_id, item_name, qty) = row.map_err(|e| format!("Row parse error: {e}"))?;

            if let Some(entry) = results.get_mut(&type_id) {
                entry.item_name = item_name;
                entry.inventory_quantity = qty;
            }
        }
    }

    // ── 2. Query storage vaults from latest snapshot ───────────────────────────
    let latest_snapshot_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM character_item_snapshots
         WHERE character_name = ?1 AND server_name = ?2
         ORDER BY snapshot_timestamp DESC LIMIT 1",
            rusqlite::params![character_name, server_name],
            |row| row.get(0),
        )
        .ok();

    if let Some(snapshot_id) = latest_snapshot_id {
        let placeholders: Vec<String> = item_type_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 2))
            .collect();
        let placeholders_str = placeholders.join(",");

        let sql = format!(
            "SELECT type_id, item_name, storage_vault, SUM(stack_size) as qty
             FROM character_snapshot_items
             WHERE item_snapshot_id = ?1 AND type_id IN ({}) AND is_in_inventory = 0
             GROUP BY type_id, storage_vault",
            placeholders_str
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare availability query: {e}"))?;

        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params.push(Box::new(snapshot_id));
        for id in &item_type_ids {
            params.push(Box::new(*id));
        }
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok((
                    row.get::<_, i64>(0)?,    // type_id
                    row.get::<_, String>(1)?, // item_name
                    row.get::<_, String>(2)?, // storage_vault
                    row.get::<_, i64>(3)?,    // qty
                ))
            })
            .map_err(|e| format!("Availability query failed: {e}"))?;

        for row in rows {
            let (type_id, item_name, vault, qty) =
                row.map_err(|e| format!("Row parse error: {e}"))?;

            if let Some(entry) = results.get_mut(&type_id) {
                if entry.item_name.is_empty() {
                    entry.item_name = item_name;
                }
                entry.storage_quantity += qty;
                let vault_name = if vault.is_empty() {
                    "Unknown".to_string()
                } else {
                    vault
                };
                entry.vault_breakdown.push(VaultStock {
                    vault_name,
                    quantity: qty,
                });
            }
        }
    }

    // ── 3. Fill in item names and compute totals ───────────────────────────────
    for (&id, entry) in results.iter_mut() {
        if entry.item_name.is_empty() {
            let name: Option<String> = conn
                .query_row("SELECT name FROM items WHERE id = ?1", [id], |row| {
                    row.get(0)
                })
                .ok();
            entry.item_name = name.unwrap_or_else(|| format!("Item #{}", id));
        }
        entry.total_available = entry.inventory_quantity + entry.storage_quantity;
    }

    Ok(item_type_ids
        .iter()
        .filter_map(|id| results.remove(id))
        .collect())
}

// ── Work order data from snapshot ────────────────────────────────────────────

#[derive(Serialize)]
pub struct WorkOrderData {
    pub active: Vec<String>,
    pub completed: Vec<String>,
    /// TypeIDs of work order scroll items found in inventory/storage
    pub inventory_item_ids: Vec<u32>,
}

/// Extract ActiveWorkOrders, CompletedWorkOrders, and inventory work order scroll items.
#[tauri::command]
pub fn get_work_orders_from_snapshot(
    db: State<'_, DbPool>,
    character_name: String,
    server_name: String,
) -> Result<WorkOrderData, String> {
    let conn = db
        .get()
        .map_err(|e| format!("Database connection error: {e}"))?;

    // Get active/completed work orders from character snapshot
    let raw_json: Option<String> = conn
        .query_row(
            "SELECT raw_json FROM character_snapshots
         WHERE character_name = ?1 AND server_name = ?2
         ORDER BY snapshot_timestamp DESC LIMIT 1",
            rusqlite::params![character_name, server_name],
            |row| row.get(0),
        )
        .ok();

    let (active, completed) = if let Some(raw) = raw_json {
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|e| format!("Failed to parse snapshot JSON: {e}"))?;

        let active = parsed
            .get("ActiveWorkOrders")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let completed = parsed
            .get("CompletedWorkOrders")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        (active, completed)
    } else {
        (Vec::new(), Vec::new())
    };

    // Get work order scroll items from the latest inventory snapshot
    let inventory_item_ids: Vec<u32> = conn
        .query_row(
            "SELECT id FROM character_item_snapshots
         WHERE character_name = ?1 AND server_name = ?2
         ORDER BY snapshot_timestamp DESC LIMIT 1",
            rusqlite::params![character_name, server_name],
            |row| row.get::<_, i64>(0),
        )
        .ok()
        .map(|snapshot_id| {
            let mut stmt = conn
                .prepare(
                    "SELECT DISTINCT type_id FROM character_snapshot_items
             WHERE item_snapshot_id = ?1
               AND (item_name LIKE 'Work Order for %' OR item_name LIKE 'Scroll\\_%' ESCAPE '\\')",
                )
                .unwrap();
            stmt.query_map(rusqlite::params![snapshot_id], |row| row.get::<_, u32>(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        })
        .unwrap_or_default();

    Ok(WorkOrderData {
        active,
        completed,
        inventory_item_ids,
    })
}
