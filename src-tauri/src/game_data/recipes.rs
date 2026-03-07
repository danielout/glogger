use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::parse_id_map;

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawRecipeIngredient {
    #[serde(rename = "ItemCode")]
    pub item_code: Option<u32>,

    #[serde(rename = "StackSize")]
    pub stack_size: Option<f32>,

    #[serde(rename = "ItemKeys")]
    pub item_keys: Option<Vec<String>>,

    #[serde(rename = "Desc")]
    pub desc: Option<String>,

    #[serde(rename = "ChanceToConsume")]
    pub chance_to_consume: Option<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawRecipeResultItem {
    #[serde(rename = "ItemCode")]
    pub item_code: Option<u32>,

    #[serde(rename = "StackSize")]
    pub stack_size: Option<f32>,

    #[serde(rename = "PercentChance")]
    pub percent_chance: Option<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawRecipe {
    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(rename = "InternalName")]
    pub internal_name: Option<String>,

    #[serde(rename = "IconId")]
    pub icon_id: Option<u32>,

    #[serde(rename = "Skill")]
    pub skill: Option<String>,

    #[serde(rename = "SkillLevelReq")]
    pub skill_level_req: Option<f32>,

    #[serde(rename = "Ingredients")]
    pub ingredients: Option<Vec<RawRecipeIngredient>>,

    #[serde(rename = "ResultItems")]
    pub result_items: Option<Vec<RawRecipeResultItem>>,

    #[serde(rename = "ProtoResultItems")]
    pub proto_result_items: Option<Vec<RawRecipeResultItem>>,

    #[serde(rename = "Keywords")]
    pub keywords: Option<Vec<String>>,

    #[serde(rename = "RewardSkill")]
    pub reward_skill: Option<String>,

    #[serde(rename = "RewardSkillXp")]
    pub reward_skill_xp: Option<f32>,

    #[serde(rename = "RewardSkillXpFirstTime")]
    pub reward_skill_xp_first_time: Option<f32>,

    #[serde(rename = "PrereqRecipe")]
    pub prereq_recipe: Option<String>,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct RecipeIngredient {
    pub item_id: u32,
    pub stack_size: f32,
    pub chance_to_consume: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RecipeResultItem {
    pub item_id: u32,
    pub stack_size: f32,
    pub percent_chance: Option<f32>,
}

/// A single recipe definition.
#[derive(Debug, Serialize, Clone)]
pub struct RecipeInfo {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub internal_name: Option<String>,
    pub icon_id: Option<u32>,
    pub skill: Option<String>,
    pub skill_level_req: Option<f32>,
    pub ingredients: Vec<RecipeIngredient>,
    pub result_items: Vec<RecipeResultItem>,
    pub reward_skill: Option<String>,
    pub reward_skill_xp: Option<f32>,
    pub reward_skill_xp_first_time: Option<f32>,
    pub prereq_recipe: Option<String>,
    pub keywords: Vec<String>,
    pub ingredient_item_ids: Vec<u32>,
    pub result_item_ids: Vec<u32>,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<u32, RecipeInfo>, String> {
    let raw_recipes: HashMap<u32, RawRecipe> = parse_id_map(json, "recipes.json")?;

    let mut recipes = HashMap::with_capacity(raw_recipes.len());
    for (id, raw) in raw_recipes {
        let ingredients: Vec<RecipeIngredient> = raw
            .ingredients
            .unwrap_or_default()
            .iter()
            .filter_map(|i| {
                i.item_code.map(|item_id| RecipeIngredient {
                    item_id,
                    stack_size: i.stack_size.unwrap_or(1.0),
                    chance_to_consume: i.chance_to_consume,
                })
            })
            .collect();

        let result_items: Vec<RecipeResultItem> = raw
            .result_items
            .unwrap_or_default()
            .iter()
            .filter_map(|r| {
                r.item_code.map(|item_id| RecipeResultItem {
                    item_id,
                    stack_size: r.stack_size.unwrap_or(1.0),
                    percent_chance: r.percent_chance,
                })
            })
            .collect();

        // Also include proto-result items (pre-enchant base items)
        let proto_result_items: Vec<RecipeResultItem> = raw
            .proto_result_items
            .unwrap_or_default()
            .iter()
            .filter_map(|r| {
                r.item_code.map(|item_id| RecipeResultItem {
                    item_id,
                    stack_size: r.stack_size.unwrap_or(1.0),
                    percent_chance: r.percent_chance,
                })
            })
            .collect();

        let ingredient_item_ids: Vec<u32> = ingredients.iter().map(|i| i.item_id).collect();

        let mut result_item_ids: Vec<u32> = result_items.iter().map(|r| r.item_id).collect();
        result_item_ids.extend(proto_result_items.iter().map(|r| r.item_id));

        recipes.insert(id, RecipeInfo {
            id,
            name: raw.name.unwrap_or_else(|| format!("Unknown Recipe {id}")),
            description: raw.description,
            internal_name: raw.internal_name,
            skill: raw.skill,
            skill_level_req: raw.skill_level_req,
            icon_id: raw.icon_id,
            ingredients,
            result_items,
            reward_skill: raw.reward_skill,
            reward_skill_xp: raw.reward_skill_xp,
            reward_skill_xp_first_time: raw.reward_skill_xp_first_time,
            prereq_recipe: raw.prereq_recipe,
            keywords: raw.keywords.unwrap_or_default(),
            ingredient_item_ids,
            result_item_ids,
        });
    }

    Ok(recipes)
}
