export interface RecipeIngredient {
  item_id: number
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
}
