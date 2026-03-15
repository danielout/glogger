export interface ItemInfo {
  id: number
  name: string
  description: string | null
  icon_id: number | null
  value: number | null
  max_stack_size: number | null
  keywords: string[]
  effect_descs: string[]

  // Phase 1 typed fields
  internal_name: string | null
  food_desc: string | null
  equip_slot: string | null
  num_uses: number | null
  skill_reqs: Record<string, number> | null
  behaviors: unknown[] | null
  bestow_recipes: unknown[] | null
  bestow_ability: string | null
  bestow_quest: string | null
  bestow_title: number | null
  craft_points: number | null
  crafting_target_level: number | null
  tsys_profile: string | null

  // Full raw JSON
  raw_json: Record<string, unknown>
}
