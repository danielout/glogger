export interface RecipeIngredient {
  item_id: number | null
  item_keys: string[]
  description: string | null
  stack_size: number
  chance_to_consume: number | null
}

export interface RecipeResultItem {
  item_id: number
  stack_size: number
  percent_chance: number | null
}

export interface RecipeInfo {
  id: number
  name: string
  description: string | null
  internal_name: string | null
  icon_id: number | null
  skill: string | null
  skill_level_req: number | null
  ingredients: RecipeIngredient[]
  result_items: RecipeResultItem[]
  reward_skill: string | null
  reward_skill_xp: number | null
  reward_skill_xp_first_time: number | null
  prereq_recipe: string | null
  keywords: string[]
  ingredient_item_ids: number[]
  result_item_ids: number[]

  // Phase 2 typed fields
  result_effects: string[]
  usage_delay: number | null
  reward_skill_xp_drop_off_level: number | null
  sort_skill: string | null
  action_label: string | null
  shares_name_with_item_id: number | null

  // Full raw JSON
  raw_json: Record<string, unknown>
}
