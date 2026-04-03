use chrono::Local;
use serde::Deserialize;
/// Deserialization of Project: Gorgon CDN JSON files into typed structs.
///
/// The CDN uses object maps keyed by "Item_1", "Ability_2", etc.
/// We parse those into HashMaps keyed by integer ID for fast lookup.
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// Timestamped log line for startup diagnostics.
macro_rules! startup_log {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", Local::now().format("%H:%M:%S%.3f"), format!($($arg)*));
    };
}

// ── Module declarations ──────────────────────────────────────────────────────
mod abilities;
mod ability_dynamic;
mod ability_keywords;
mod advancement_tables;
mod ai;
mod areas;
mod attributes;
mod directed_goals;
mod effects;
mod item_uses;
mod items;
mod landmarks;
mod lorebooks;
mod npcs;
mod player_titles;
mod quests;
mod recipes;
mod skills;
mod sources;
mod storage_vaults;
mod tsys;
mod xp_tables;

// ── Re-exports so cdn_commands.rs doesn't need updating ──────────────────────
pub use abilities::AbilityInfo;
pub use areas::AreaInfo;
pub use effects::EffectInfo;
pub use item_uses::ItemUseInfo;
pub use items::ItemInfo;
pub use npcs::NpcInfo;
pub use player_titles::PlayerTitleInfo;
pub use quests::QuestInfo;
pub use recipes::RecipeInfo;
pub use skills::SkillInfo;
pub use sources::SourceEntry;
pub use tsys::TsysClientInfo;
pub use xp_tables::XpTableInfo;

// ── Shared utilities ─────────────────────────────────────────────────────────

/// Parse a CDN-style map file where keys are like "Item_1", "Skill_42", etc.
/// Returns a HashMap<u32, T> keyed by the integer portion.
/// `file_name` is used only for error messages.
pub fn parse_id_map<T: for<'de> Deserialize<'de>>(
    json: &str,
    file_name: &str,
) -> Result<HashMap<u32, T>, String> {
    let raw: HashMap<String, T> = serde_json::from_str(json).map_err(|e| {
        format!(
            "{file_name}: parse error at line {}, col {}: {e}",
            e.line(),
            e.column()
        )
    })?;

    let mut out = HashMap::with_capacity(raw.len());
    let mut skipped = 0;
    for (key, value) in raw {
        // Keys are like "Item_1", "Ability_42", "Level_1" etc.
        if let Some(id_str) = key.split('_').last() {
            if let Ok(id) = id_str.parse::<u32>() {
                out.insert(id, value);
            } else {
                skipped += 1;
            }
        } else {
            skipped += 1;
        }
    }
    if skipped > 0 {
        eprintln!("{file_name}: Warning: skipped {skipped} entries with invalid keys");
    }
    Ok(out)
}

pub fn parse_string_map<T: for<'de> Deserialize<'de>>(
    json: &str,
    file_name: &str,
) -> Result<HashMap<String, T>, String> {
    serde_json::from_str(json).map_err(|e| {
        format!(
            "{file_name}: parse error at line {}, col {}: {e}",
            e.line(),
            e.column()
        )
    })
}

pub async fn read_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))
}

// ── GameData struct ──────────────────────────────────────────────────────────

/// All CDN data loaded into memory. Held in Tauri managed state.
/// Built once on startup; replaced on CDN refresh.
#[allow(dead_code)]
pub struct GameData {
    pub version: u32,

    // ── Already implemented ────────────────────────────────────────────
    pub items: HashMap<u32, items::ItemInfo>,
    pub skills: HashMap<u32, skills::SkillInfo>,
    pub abilities: HashMap<u32, abilities::AbilityInfo>,
    pub recipes: HashMap<u32, recipes::RecipeInfo>,
    pub npcs: HashMap<String, npcs::NpcInfo>,

    // ── New stub fields (populated by load_from_cache) ─────────────────
    pub effects: HashMap<u32, effects::EffectInfo>,
    pub areas: HashMap<String, areas::AreaInfo>,
    pub attributes: HashMap<String, attributes::AttributeInfo>,
    pub xp_tables: HashMap<u32, xp_tables::XpTableInfo>,
    pub advancement_tables: HashMap<String, advancement_tables::AdvancementTableInfo>,
    pub ability_keywords: HashMap<String, ability_keywords::AbilityKeywordInfo>,
    pub ability_dynamic: ability_dynamic::AbilityDynamicData,
    pub ai: HashMap<String, ai::AiInfo>,
    pub directed_goals: HashMap<String, directed_goals::DirectedGoalInfo>,
    pub item_uses: HashMap<String, item_uses::ItemUseInfo>,
    pub landmarks: HashMap<String, landmarks::LandmarkInfo>,
    pub lorebooks: lorebooks::LorebookData,
    pub player_titles: HashMap<u32, player_titles::PlayerTitleInfo>,
    pub quests: HashMap<String, quests::QuestInfo>,
    pub sources: sources::SourcesData,
    pub storage_vaults: HashMap<String, storage_vaults::StorageVaultInfo>,
    pub tsys: tsys::TsysData,

    // ── Cross-type indices ─────────────────────────────────────────────
    pub item_name_index: HashMap<String, u32>,
    pub item_internal_name_index: HashMap<String, u32>,
    pub skill_name_index: HashMap<String, u32>,
    pub recipes_by_skill: HashMap<String, Vec<u32>>,
    pub recipes_producing_item: HashMap<u32, Vec<u32>>,
    pub recipes_using_item: HashMap<u32, Vec<u32>>,
    pub recipe_name_index: HashMap<String, u32>,
    pub recipe_internal_name_index: HashMap<String, u32>,
    pub npcs_by_skill: HashMap<String, Vec<String>>,
    pub quest_internal_name_index: HashMap<String, String>,

    // ── Entity resolution indices ────────────────────────────────────────
    pub skill_internal_name_index: HashMap<String, u32>,
    pub npc_name_index: HashMap<String, String>,
    pub area_name_index: HashMap<String, String>,
    /// Ability display name → ability ID (for combat log resolution)
    pub ability_name_index: HashMap<String, u32>,

    // ── Source cross-reference indices ───────────────────────────────────
    /// ability key (e.g. "ability_1002") → item IDs that bestow it
    pub items_bestowing_ability: HashMap<String, Vec<u32>>,
    /// recipe key string from BestowRecipes → item IDs that bestow it
    pub items_bestowing_recipe: HashMap<String, Vec<u32>>,
    /// quest key → item IDs that bestow it
    pub items_bestowing_quest: HashMap<String, Vec<u32>>,
    /// item key (e.g. "item_12345") → quest keys that reward it
    pub quests_rewarding_item: HashMap<String, Vec<String>>,

    // ── Cross-reference indices for data browser linking ──────────────
    /// item name (lowercase) → Vec<(npc_key, desire, pref_value)>
    pub npc_favor_by_item_name: HashMap<String, Vec<(String, String, f32)>>,
    /// item keyword → Vec<(npc_key, desire, pref_value)>
    pub npc_favor_by_keyword: HashMap<String, Vec<(String, String, f32)>>,
    /// npc key → quest keys (via FavorNpc field)
    pub quests_by_npc: HashMap<String, Vec<String>>,
    /// skill internal name → quest keys (work order quests)
    pub quests_by_work_order_skill: HashMap<String, Vec<String>>,
    /// item keyword → recipe IDs that accept this keyword as ingredient
    pub recipes_by_ingredient_keyword: HashMap<String, Vec<u32>>,
    /// NPC key → item IDs sold by that NPC (from Vendor/Barter source entries)
    pub vendor_items_by_npc: HashMap<String, Vec<u32>>,
    /// item ID → NPC keys that sell/barter it
    pub vendors_for_item: HashMap<u32, Vec<String>>,
}

impl GameData {
    pub fn empty() -> Self {
        Self {
            version: 0,
            items: HashMap::new(),
            skills: HashMap::new(),
            abilities: HashMap::new(),
            recipes: HashMap::new(),
            npcs: HashMap::new(),
            effects: HashMap::new(),
            areas: HashMap::new(),
            attributes: HashMap::new(),
            xp_tables: HashMap::new(),
            advancement_tables: HashMap::new(),
            ability_keywords: HashMap::new(),
            ability_dynamic: ability_dynamic::AbilityDynamicData::empty(),
            ai: HashMap::new(),
            directed_goals: HashMap::new(),
            item_uses: HashMap::new(),
            landmarks: HashMap::new(),
            lorebooks: lorebooks::LorebookData::empty(),
            player_titles: HashMap::new(),
            quests: HashMap::new(),
            sources: sources::SourcesData::empty(),
            storage_vaults: HashMap::new(),
            tsys: tsys::TsysData::empty(),
            item_name_index: HashMap::new(),
            item_internal_name_index: HashMap::new(),
            skill_name_index: HashMap::new(),
            recipes_by_skill: HashMap::new(),
            recipes_producing_item: HashMap::new(),
            recipes_using_item: HashMap::new(),
            recipe_name_index: HashMap::new(),
            recipe_internal_name_index: HashMap::new(),
            npcs_by_skill: HashMap::new(),
            quest_internal_name_index: HashMap::new(),
            skill_internal_name_index: HashMap::new(),
            npc_name_index: HashMap::new(),
            area_name_index: HashMap::new(),
            ability_name_index: HashMap::new(),
            items_bestowing_ability: HashMap::new(),
            items_bestowing_recipe: HashMap::new(),
            items_bestowing_quest: HashMap::new(),
            quests_rewarding_item: HashMap::new(),
            npc_favor_by_item_name: HashMap::new(),
            npc_favor_by_keyword: HashMap::new(),
            quests_by_npc: HashMap::new(),
            quests_by_work_order_skill: HashMap::new(),
            recipes_by_ingredient_keyword: HashMap::new(),
            vendor_items_by_npc: HashMap::new(),
            vendors_for_item: HashMap::new(),
        }
    }

    pub fn item_by_name(&self, name: &str) -> Option<&items::ItemInfo> {
        let id = self.item_name_index.get(name)?;
        self.items.get(id)
    }

    pub fn item_by_internal_name(&self, name: &str) -> Option<&items::ItemInfo> {
        let id = self.item_internal_name_index.get(name)?;
        self.items.get(id)
    }

    pub fn quest_by_internal_name(&self, name: &str) -> Option<&quests::QuestInfo> {
        let key = self.quest_internal_name_index.get(name)?;
        self.quests.get(key)
    }

    pub fn skill_by_name(&self, name: &str) -> Option<&skills::SkillInfo> {
        let id = self.skill_name_index.get(name)?;
        self.skills.get(id)
    }

    pub fn recipe_by_name(&self, name: &str) -> Option<&recipes::RecipeInfo> {
        let id = self.recipe_name_index.get(name)?;
        self.recipes.get(id)
    }

    // ── Unified entity resolvers ─────────────────────────────────────────
    // Each accepts any known reference form and returns the entity.

    /// Resolve any string reference to an ItemInfo.
    /// Tries: numeric ID → display name → internal name.
    pub fn resolve_item(&self, reference: &str) -> Option<&items::ItemInfo> {
        if let Ok(id) = reference.parse::<u32>() {
            if let Some(item) = self.items.get(&id) {
                return Some(item);
            }
        }
        if let Some(item) = self.item_by_name(reference) {
            return Some(item);
        }
        self.item_by_internal_name(reference)
    }

    /// Resolve any string reference to a SkillInfo.
    /// Tries: numeric ID → display name → internal name.
    pub fn resolve_skill(&self, reference: &str) -> Option<&skills::SkillInfo> {
        if let Ok(id) = reference.parse::<u32>() {
            if let Some(skill) = self.skills.get(&id) {
                return Some(skill);
            }
        }
        if let Some(skill) = self.skill_by_name(reference) {
            return Some(skill);
        }
        let id = self.skill_internal_name_index.get(reference)?;
        self.skills.get(id)
    }

    /// Resolve any string reference to a RecipeInfo.
    /// Tries: numeric ID → display name → internal name.
    pub fn resolve_recipe(&self, reference: &str) -> Option<&recipes::RecipeInfo> {
        if let Ok(id) = reference.parse::<u32>() {
            if let Some(recipe) = self.recipes.get(&id) {
                return Some(recipe);
            }
        }
        if let Some(recipe) = self.recipe_by_name(reference) {
            return Some(recipe);
        }
        let id = self.recipe_internal_name_index.get(reference)?;
        self.recipes.get(id)
    }

    /// Resolve any string reference to a QuestInfo.
    /// Tries: CDN key → internal name.
    pub fn resolve_quest(&self, reference: &str) -> Option<&quests::QuestInfo> {
        if let Some(quest) = self.quests.get(reference) {
            return Some(quest);
        }
        self.quest_by_internal_name(reference)
    }

    /// Resolve any string reference to an NpcInfo.
    /// Tries: CDN key → display name.
    pub fn resolve_npc(&self, reference: &str) -> Option<&npcs::NpcInfo> {
        if let Some(npc) = self.npcs.get(reference) {
            return Some(npc);
        }
        let key = self.npc_name_index.get(reference)?;
        self.npcs.get(key)
    }

    /// Resolve any string reference to an AreaInfo.
    /// Tries: CDN key → friendly name → short friendly name.
    pub fn resolve_area(&self, reference: &str) -> Option<&areas::AreaInfo> {
        if let Some(area) = self.areas.get(reference) {
            return Some(area);
        }
        let key = self.area_name_index.get(reference)?;
        self.areas.get(key)
    }

    /// Resolve any string reference to an AbilityInfo.
    /// Tries: numeric ID → display name.
    pub fn resolve_ability(&self, reference: &str) -> Option<&abilities::AbilityInfo> {
        if let Ok(id) = reference.parse::<u32>() {
            if let Some(ability) = self.abilities.get(&id) {
                return Some(ability);
            }
        }
        let id = self.ability_name_index.get(reference)?;
        self.abilities.get(id)
    }
}

// ── Loading from disk ─────────────────────────────────────────────────────────

/// Load all cached JSON files from disk into a `GameData` instance.
pub async fn load_from_cache(cache_dir: &Path, version: u32) -> Result<GameData, String> {
    // Read all existing files
    let items_json = read_file(&cache_dir.join("items.json")).await?;
    let skills_json = read_file(&cache_dir.join("skills.json")).await?;
    let abilities_json = read_file(&cache_dir.join("abilities.json")).await?;
    let recipes_json = read_file(&cache_dir.join("recipes.json")).await?;

    // Read new stub files with graceful fallback
    let npcs_json = read_file(&cache_dir.join("npcs.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let effects_json = read_file(&cache_dir.join("effects.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let areas_json = read_file(&cache_dir.join("areas.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let attributes_json = read_file(&cache_dir.join("attributes.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let xp_tables_json = read_file(&cache_dir.join("xptables.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let advancement_tables_json = read_file(&cache_dir.join("advancementtables.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let ability_keywords_json = read_file(&cache_dir.join("abilitykeywords.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let ability_dynamic_dots_json = read_file(&cache_dir.join("abilitydynamicdots.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let ability_dynamic_special_json =
        read_file(&cache_dir.join("abilitydynamicspecialvalues.json"))
            .await
            .unwrap_or_else(|e| {
                eprintln!("Warning: {e}");
                String::from("{}")
            });
    let ai_json = read_file(&cache_dir.join("ai.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let directed_goals_json = read_file(&cache_dir.join("directedgoals.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let item_uses_json = read_file(&cache_dir.join("itemuses.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let landmarks_json = read_file(&cache_dir.join("landmarks.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let lorebooks_json = read_file(&cache_dir.join("lorebooks.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let lorebook_info_json = read_file(&cache_dir.join("lorebookinfo.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let player_titles_json = read_file(&cache_dir.join("playertitles.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let quests_json = read_file(&cache_dir.join("quests.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let sources_abilities_json = read_file(&cache_dir.join("sources_abilities.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let sources_items_json = read_file(&cache_dir.join("sources_items.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let sources_recipes_json = read_file(&cache_dir.join("sources_recipes.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let storage_vaults_json = read_file(&cache_dir.join("storagevaults.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let tsys_client_info_json = read_file(&cache_dir.join("tsysclientinfo.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });
    let tsys_profiles_json = read_file(&cache_dir.join("tsysprofiles.json"))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: {e}");
            String::from("{}")
        });

    // Parse all data
    startup_log!("Parsing game data files...");

    let items = items::parse(&items_json)?;
    startup_log!("  items: {} entries", items.len());

    let skills = skills::parse(&skills_json)?;
    startup_log!("  skills: {} entries", skills.len());

    let abilities = abilities::parse(&abilities_json)?;
    startup_log!("  abilities: {} entries", abilities.len());

    let recipes = recipes::parse(&recipes_json)?;
    startup_log!("  recipes: {} entries", recipes.len());

    let npcs = npcs::parse(&npcs_json)?;
    startup_log!("  npcs: {} entries", npcs.len());

    let effects = effects::parse(&effects_json)?;
    startup_log!("  effects: {} entries", effects.len());

    let areas = areas::parse(&areas_json)?;
    startup_log!("  areas: {} entries", areas.len());

    let attributes = attributes::parse(&attributes_json)?;
    startup_log!("  attributes: {} entries", attributes.len());

    let xp_tables = xp_tables::parse(&xp_tables_json)?;
    startup_log!("  xp_tables: {} entries", xp_tables.len());

    let advancement_tables = advancement_tables::parse(&advancement_tables_json)?;
    startup_log!("  advancement_tables: {} entries", advancement_tables.len());

    let ability_keywords = ability_keywords::parse(&ability_keywords_json)?;
    startup_log!("  ability_keywords: {} entries", ability_keywords.len());

    let ability_dynamic = ability_dynamic::AbilityDynamicData::parse(
        &ability_dynamic_dots_json,
        &ability_dynamic_special_json,
    )?;
    startup_log!("  ability_dynamic: parsed");

    let ai = ai::parse(&ai_json)?;
    startup_log!("  ai: {} entries", ai.len());

    let directed_goals = directed_goals::parse(&directed_goals_json)?;
    startup_log!("  directed_goals: {} entries", directed_goals.len());

    let item_uses = item_uses::parse(&item_uses_json)?;
    startup_log!("  item_uses: {} entries", item_uses.len());

    let landmarks = landmarks::parse(&landmarks_json)?;
    startup_log!("  landmarks: {} entries", landmarks.len());

    let lorebooks = lorebooks::LorebookData::parse(&lorebooks_json, &lorebook_info_json)?;
    startup_log!("  lorebooks: parsed");

    let player_titles = player_titles::parse(&player_titles_json)?;
    startup_log!("  player_titles: {} entries", player_titles.len());

    let quests = quests::parse(&quests_json)?;
    startup_log!("  quests: {} entries", quests.len());

    let sources = sources::SourcesData::parse(
        &sources_abilities_json,
        &sources_items_json,
        &sources_recipes_json,
    )?;
    startup_log!("  sources: parsed");

    let storage_vaults = storage_vaults::parse(&storage_vaults_json)?;
    startup_log!("  storage_vaults: {} entries", storage_vaults.len());

    let tsys = tsys::TsysData::parse(&tsys_client_info_json, &tsys_profiles_json)?;
    startup_log!("  tsys: parsed");

    // Build indices
    let item_name_index: HashMap<String, u32> = items
        .iter()
        .map(|(id, item)| (item.name.clone(), *id))
        .collect();

    let item_internal_name_index: HashMap<String, u32> = items
        .iter()
        .filter_map(|(id, item)| item.internal_name.as_ref().map(|n| (n.clone(), *id)))
        .collect();

    let skill_name_index: HashMap<String, u32> = skills
        .iter()
        .map(|(id, skill)| (skill.name.clone(), *id))
        .collect();

    // Map internal skill names (e.g. "JewelryCrafting") to display names (e.g. "Jewelry Crafting")
    let skill_internal_to_display: HashMap<String, String> = skills
        .values()
        .map(|skill| (skill.internal_name.clone(), skill.name.clone()))
        .collect();

    let mut recipes_by_skill: HashMap<String, Vec<u32>> = HashMap::new();
    let mut recipes_producing_item: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut recipes_using_item: HashMap<u32, Vec<u32>> = HashMap::new();

    for (recipe_id, recipe) in &recipes {
        if let Some(skill) = &recipe.skill {
            // Use display name if available, fall back to raw skill name
            let display_name = skill_internal_to_display
                .get(skill)
                .cloned()
                .unwrap_or_else(|| skill.clone());
            recipes_by_skill
                .entry(display_name)
                .or_default()
                .push(*recipe_id);
        }
        for item_id in &recipe.result_item_ids {
            recipes_producing_item
                .entry(*item_id)
                .or_default()
                .push(*recipe_id);
        }
        for item_id in &recipe.ingredient_item_ids {
            recipes_using_item
                .entry(*item_id)
                .or_default()
                .push(*recipe_id);
        }
    }

    let recipe_name_index: HashMap<String, u32> = recipes
        .iter()
        .map(|(id, recipe)| (recipe.name.clone(), *id))
        .collect();

    let recipe_internal_name_index: HashMap<String, u32> = recipes
        .iter()
        .filter_map(|(id, recipe)| {
            recipe
                .internal_name
                .as_ref()
                .map(|iname| (iname.clone(), *id))
        })
        .collect();

    let mut npcs_by_skill: HashMap<String, Vec<String>> = HashMap::new();
    for (npc_key, npc) in &npcs {
        for skill in &npc.trains_skills {
            npcs_by_skill
                .entry(skill.clone())
                .or_default()
                .push(npc_key.clone());
        }
    }

    // Build quest internal name → key index
    let mut quest_internal_name_index: HashMap<String, String> = HashMap::new();
    for (quest_key, quest) in &quests {
        if !quest.internal_name.is_empty() {
            quest_internal_name_index.insert(quest.internal_name.clone(), quest_key.clone());
        }
    }

    // Build entity resolution indices
    let skill_internal_name_index: HashMap<String, u32> = skills
        .iter()
        .map(|(id, skill)| (skill.internal_name.clone(), *id))
        .collect();

    let npc_name_index: HashMap<String, String> = npcs
        .iter()
        .map(|(key, npc)| (npc.name.clone(), key.clone()))
        .collect();

    let mut area_name_index: HashMap<String, String> = HashMap::new();
    for (key, area) in &areas {
        if let Some(ref name) = area.friendly_name {
            area_name_index.insert(name.clone(), key.clone());
        }
        if let Some(ref name) = area.short_friendly_name {
            area_name_index.insert(name.clone(), key.clone());
        }
    }

    // Build source cross-reference indices
    let mut items_bestowing_ability: HashMap<String, Vec<u32>> = HashMap::new();
    let mut items_bestowing_recipe: HashMap<String, Vec<u32>> = HashMap::new();
    let mut items_bestowing_quest: HashMap<String, Vec<u32>> = HashMap::new();

    for (item_id, item) in &items {
        if let Some(ref ability_key) = item.bestow_ability {
            items_bestowing_ability
                .entry(ability_key.clone())
                .or_default()
                .push(*item_id);
        }
        if let Some(ref quest_key) = item.bestow_quest {
            items_bestowing_quest
                .entry(quest_key.clone())
                .or_default()
                .push(*item_id);
        }
        if let Some(ref bestow_recipes) = item.bestow_recipes {
            for recipe_val in bestow_recipes {
                // BestowRecipes entries can be strings like "recipe_1234"
                if let Some(recipe_key) = recipe_val.as_str() {
                    items_bestowing_recipe
                        .entry(recipe_key.to_string())
                        .or_default()
                        .push(*item_id);
                }
            }
        }
    }

    let mut quests_rewarding_item: HashMap<String, Vec<String>> = HashMap::new();
    for (quest_key, quest) in &quests {
        if let Some(reward_items) = quest.raw.get("Rewards_Items").and_then(|v| v.as_array()) {
            for reward in reward_items {
                if let Some(item_key) = reward.get("Item").and_then(|v| v.as_str()) {
                    quests_rewarding_item
                        .entry(item_key.to_string())
                        .or_default()
                        .push(quest_key.clone());
                }
            }
        }
    }

    // Build NPC favor reverse indices: item name → NPCs, keyword → NPCs
    let mut npc_favor_by_item_name: HashMap<String, Vec<(String, String, f32)>> = HashMap::new();
    let mut npc_favor_by_keyword: HashMap<String, Vec<(String, String, f32)>> = HashMap::new();
    for (npc_key, npc) in &npcs {
        for pref in &npc.preferences {
            let entry = (npc_key.clone(), pref.desire.clone(), pref.pref);
            if let Some(ref name) = pref.name {
                npc_favor_by_item_name
                    .entry(name.to_lowercase())
                    .or_default()
                    .push(entry.clone());
            }
            for keyword in &pref.keywords {
                npc_favor_by_keyword
                    .entry(keyword.clone())
                    .or_default()
                    .push(entry.clone());
            }
        }
    }

    // Build quest reverse indices: NPC → quests, skill → work order quests
    let mut quests_by_npc: HashMap<String, Vec<String>> = HashMap::new();
    let mut quests_by_work_order_skill: HashMap<String, Vec<String>> = HashMap::new();
    for (quest_key, quest) in &quests {
        if let Some(favor_npc) = quest.raw.get("FavorNpc").and_then(|v| v.as_str()) {
            // FavorNpc is a path like "AreaName/NPC_Foo" — extract the NPC key
            if let Some(npc_key) = favor_npc.split('/').last() {
                quests_by_npc
                    .entry(npc_key.to_string())
                    .or_default()
                    .push(quest_key.clone());
            }
        }
        if let Some(skill) = quest.raw.get("WorkOrderSkill").and_then(|v| v.as_str()) {
            quests_by_work_order_skill
                .entry(skill.to_string())
                .or_default()
                .push(quest_key.clone());
        }
    }

    // Build recipe keyword ingredient index: keyword → recipe IDs
    let mut recipes_by_ingredient_keyword: HashMap<String, Vec<u32>> = HashMap::new();
    for (recipe_id, recipe) in &recipes {
        for ingredient in &recipe.ingredients {
            if ingredient.item_id.is_none() {
                for keyword in &ingredient.item_keys {
                    recipes_by_ingredient_keyword
                        .entry(keyword.clone())
                        .or_default()
                        .push(*recipe_id);
                }
            }
        }
    }

    // Build NPC vendor inventory index: NPC key → item IDs (from Vendor/Barter sources)
    let mut vendor_items_by_npc: HashMap<String, Vec<u32>> = HashMap::new();
    for (&item_id, source_info) in &sources.items {
        for entry in &source_info.entries {
            if entry.source_type == "Vendor" || entry.source_type == "Barter" {
                if let Some(ref npc_key) = entry.npc {
                    vendor_items_by_npc
                        .entry(npc_key.clone())
                        .or_default()
                        .push(item_id);
                }
            }
        }
    }
    // Sort each NPC's item list for consistent ordering
    for items in vendor_items_by_npc.values_mut() {
        items.sort_unstable();
        items.dedup();
    }

    // Build reverse index: item ID → NPC keys that sell it
    let mut vendors_for_item: HashMap<u32, Vec<String>> = HashMap::new();
    for (npc_key, item_ids) in &vendor_items_by_npc {
        for &item_id in item_ids {
            vendors_for_item
                .entry(item_id)
                .or_default()
                .push(npc_key.clone());
        }
    }
    for npc_keys in vendors_for_item.values_mut() {
        npc_keys.sort();
        npc_keys.dedup();
    }

    let ability_name_index: HashMap<String, u32> = abilities
        .iter()
        .map(|(id, ability)| (ability.name.clone(), *id))
        .collect();

    startup_log!("Game data indices built");

    Ok(GameData {
        version,
        items,
        skills,
        abilities,
        recipes,
        npcs,
        effects,
        areas,
        attributes,
        xp_tables,
        advancement_tables,
        ability_keywords,
        ability_dynamic,
        ai,
        directed_goals,
        item_uses,
        landmarks,
        lorebooks,
        player_titles,
        quests,
        sources,
        storage_vaults,
        tsys,
        item_name_index,
        item_internal_name_index,
        skill_name_index,
        recipes_by_skill,
        recipes_producing_item,
        recipes_using_item,
        recipe_name_index,
        recipe_internal_name_index,
        npcs_by_skill,
        quest_internal_name_index,
        skill_internal_name_index,
        npc_name_index,
        area_name_index,
        items_bestowing_ability,
        items_bestowing_recipe,
        items_bestowing_quest,
        quests_rewarding_item,
        npc_favor_by_item_name,
        npc_favor_by_keyword,
        quests_by_npc,
        quests_by_work_order_skill,
        recipes_by_ingredient_keyword,
        vendor_items_by_npc,
        vendors_for_item,
        ability_name_index,
    })
}
