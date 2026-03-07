/// Deserialization of Project: Gorgon CDN JSON files into typed structs.
///
/// The CDN uses object maps keyed by "Item_1", "Ability_2", etc.
/// We parse those into HashMaps keyed by integer ID for fast lookup.

use std::collections::HashMap;
use std::path::Path;
use serde::Deserialize;
use tokio::fs;

// ── Module declarations ──────────────────────────────────────────────────────
mod items;
mod skills;
mod abilities;
mod recipes;
mod npcs;
mod effects;
mod areas;
mod attributes;
mod xp_tables;
mod advancement_tables;
mod ability_keywords;
mod ability_dynamic;
mod ai;
mod directed_goals;
mod item_uses;
mod landmarks;
mod lorebooks;
mod player_titles;
mod quests;
mod sources;
mod storage_vaults;
mod tsys;

// ── Re-exports so cdn_commands.rs doesn't need updating ──────────────────────
pub use items::ItemInfo;
pub use skills::SkillInfo;
pub use abilities::AbilityInfo;
pub use recipes::{RecipeInfo, RecipeIngredient, RecipeResultItem};
pub use npcs::{NpcInfo, NpcPreference};

// ── Shared utilities ─────────────────────────────────────────────────────────

/// Parse a CDN-style map file where keys are like "Item_1", "Skill_42", etc.
/// Returns a HashMap<u32, T> keyed by the integer portion.
/// `file_name` is used only for error messages.
pub fn parse_id_map<T: for<'de> Deserialize<'de>>(
    json: &str,
    file_name: &str,
) -> Result<HashMap<u32, T>, String> {
    let raw: HashMap<String, T> = serde_json::from_str(json).map_err(|e| {
        format!("{file_name}: parse error at line {}, col {}: {e}", e.line(), e.column())
    })?;

    let mut out = HashMap::with_capacity(raw.len());
    for (key, value) in raw {
        // Keys are like "Item_1", "Ability_42", "Level_1" etc.
        if let Some(id_str) = key.split('_').last() {
            if let Ok(id) = id_str.parse::<u32>() {
                out.insert(id, value);
            }
        }
    }
    Ok(out)
}

pub fn parse_string_map<T: for<'de> Deserialize<'de>>(
    json: &str,
    file_name: &str,
) -> Result<HashMap<String, T>, String> {
    serde_json::from_str(json).map_err(|e| {
        format!("{file_name}: parse error at line {}, col {}: {e}", e.line(), e.column())
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
pub struct GameData {
    pub version: u32,

    // ── Already implemented ────────────────────────────────────────────
    pub items:     HashMap<u32, items::ItemInfo>,
    pub skills:    HashMap<u32, skills::SkillInfo>,
    pub abilities: HashMap<u32, abilities::AbilityInfo>,
    pub recipes:   HashMap<u32, recipes::RecipeInfo>,
    pub npcs:      HashMap<String, npcs::NpcInfo>,

    // ── New stub fields (populated by load_from_cache) ─────────────────
    pub effects:            HashMap<u32,     effects::EffectInfo>,
    pub areas:              HashMap<String,  areas::AreaInfo>,
    pub attributes:         HashMap<String,  attributes::AttributeInfo>,
    pub xp_tables:          HashMap<u32,     xp_tables::XpTableInfo>,
    pub advancement_tables: HashMap<String,  advancement_tables::AdvancementTableInfo>,
    pub ability_keywords:   HashMap<String,  ability_keywords::AbilityKeywordInfo>,
    pub ability_dynamic:    ability_dynamic::AbilityDynamicData,
    pub ai:                 HashMap<String,  ai::AiInfo>,
    pub directed_goals:     HashMap<String,  directed_goals::DirectedGoalInfo>,
    pub item_uses:          HashMap<String,  item_uses::ItemUseInfo>,
    pub landmarks:          HashMap<String,  landmarks::LandmarkInfo>,
    pub lorebooks:          lorebooks::LorebookData,
    pub player_titles:      HashMap<u32,     player_titles::PlayerTitleInfo>,
    pub quests:             HashMap<String,  quests::QuestInfo>,
    pub sources:            sources::SourcesData,
    pub storage_vaults:     HashMap<String,  storage_vaults::StorageVaultInfo>,
    pub tsys:               tsys::TsysData,

    // ── Cross-type indices ─────────────────────────────────────────────
    pub item_name_index:       HashMap<String, u32>,
    pub skill_name_index:      HashMap<String, u32>,
    pub recipes_by_skill:      HashMap<String, Vec<u32>>,
    pub recipes_producing_item: HashMap<u32, Vec<u32>>,
    pub recipes_using_item:    HashMap<u32, Vec<u32>>,
    pub npcs_by_skill:         HashMap<String, Vec<String>>,
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
            skill_name_index: HashMap::new(),
            recipes_by_skill: HashMap::new(),
            recipes_producing_item: HashMap::new(),
            recipes_using_item: HashMap::new(),
            npcs_by_skill: HashMap::new(),
        }
    }

    pub fn item_by_name(&self, name: &str) -> Option<&items::ItemInfo> {
        let id = self.item_name_index.get(name)?;
        self.items.get(id)
    }

    pub fn skill_by_name(&self, name: &str) -> Option<&skills::SkillInfo> {
        let id = self.skill_name_index.get(name)?;
        self.skills.get(id)
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
    let npcs_json = read_file(&cache_dir.join("npcs.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let effects_json = read_file(&cache_dir.join("effects.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let areas_json = read_file(&cache_dir.join("areas.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let attributes_json = read_file(&cache_dir.join("attributes.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let xp_tables_json = read_file(&cache_dir.join("xptables.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let advancement_tables_json = read_file(&cache_dir.join("advancementtables.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let ability_keywords_json = read_file(&cache_dir.join("abilitykeywords.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let ability_dynamic_dots_json = read_file(&cache_dir.join("abilitydynamicdots.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let ability_dynamic_special_json = read_file(&cache_dir.join("abilitydynamicspecialvalues.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let ai_json = read_file(&cache_dir.join("ai.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let directed_goals_json = read_file(&cache_dir.join("directedgoals.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let item_uses_json = read_file(&cache_dir.join("itemuses.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let landmarks_json = read_file(&cache_dir.join("landmarks.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let lorebooks_json = read_file(&cache_dir.join("lorebooks.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let lorebook_info_json = read_file(&cache_dir.join("lorebookinfo.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let player_titles_json = read_file(&cache_dir.join("playertitles.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let quests_json = read_file(&cache_dir.join("quests.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let sources_abilities_json = read_file(&cache_dir.join("sources_abilities.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let sources_items_json = read_file(&cache_dir.join("sources_items.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let sources_recipes_json = read_file(&cache_dir.join("sources_recipes.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let storage_vaults_json = read_file(&cache_dir.join("storagevaults.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let tsys_client_info_json = read_file(&cache_dir.join("tsysclientinfo.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });
    let tsys_profiles_json = read_file(&cache_dir.join("tsysprofiles.json")).await
        .unwrap_or_else(|e| { eprintln!("Warning: {e}"); String::from("{}") });

    // Parse all data
    let items = items::parse(&items_json)?;
    let skills = skills::parse(&skills_json)?;
    let abilities = abilities::parse(&abilities_json)?;
    let recipes = recipes::parse(&recipes_json)?;
    let npcs = npcs::parse(&npcs_json)?;
    let effects = effects::parse(&effects_json)?;
    let areas = areas::parse(&areas_json)?;
    let attributes = attributes::parse(&attributes_json)?;
    let xp_tables = xp_tables::parse(&xp_tables_json)?;
    let advancement_tables = advancement_tables::parse(&advancement_tables_json)?;
    let ability_keywords = ability_keywords::parse(&ability_keywords_json)?;
    let ability_dynamic = ability_dynamic::AbilityDynamicData::parse(
        &ability_dynamic_dots_json,
        &ability_dynamic_special_json
    )?;
    let ai = ai::parse(&ai_json)?;
    let directed_goals = directed_goals::parse(&directed_goals_json)?;
    let item_uses = item_uses::parse(&item_uses_json)?;
    let landmarks = landmarks::parse(&landmarks_json)?;
    let lorebooks = lorebooks::LorebookData::parse(&lorebooks_json, &lorebook_info_json)?;
    let player_titles = player_titles::parse(&player_titles_json)?;
    let quests = quests::parse(&quests_json)?;
    let sources = sources::SourcesData::parse(
        &sources_abilities_json,
        &sources_items_json,
        &sources_recipes_json
    )?;
    let storage_vaults = storage_vaults::parse(&storage_vaults_json)?;
    let tsys = tsys::TsysData::parse(&tsys_client_info_json, &tsys_profiles_json)?;

    // Build indices
    let item_name_index: HashMap<String, u32> = items
        .iter()
        .map(|(id, item)| (item.name.clone(), *id))
        .collect();

    let skill_name_index: HashMap<String, u32> = skills
        .iter()
        .map(|(id, skill)| (skill.name.clone(), *id))
        .collect();

    let mut recipes_by_skill: HashMap<String, Vec<u32>> = HashMap::new();
    let mut recipes_producing_item: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut recipes_using_item: HashMap<u32, Vec<u32>> = HashMap::new();

    for (recipe_id, recipe) in &recipes {
        if let Some(skill) = &recipe.skill {
            recipes_by_skill.entry(skill.clone()).or_default().push(*recipe_id);
        }
        for item_id in &recipe.result_item_ids {
            recipes_producing_item.entry(*item_id).or_default().push(*recipe_id);
        }
        for item_id in &recipe.ingredient_item_ids {
            recipes_using_item.entry(*item_id).or_default().push(*recipe_id);
        }
    }

    let mut npcs_by_skill: HashMap<String, Vec<String>> = HashMap::new();
    for (npc_key, npc) in &npcs {
        for skill in &npc.trains_skills {
            npcs_by_skill.entry(skill.clone()).or_default().push(npc_key.clone());
        }
    }

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
        skill_name_index,
        recipes_by_skill,
        recipes_producing_item,
        recipes_using_item,
        npcs_by_skill,
    })
}
