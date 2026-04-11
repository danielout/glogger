export interface CombatStats {
  damage: number | null
  power_cost: number | null
  range: number | null
  rage_cost: number | null
  accuracy: number | null
  attributes_that_delta_damage: string[]
  attributes_that_mod_base_damage: string[]
  attributes_that_mod_damage: string[]
  attributes_that_mod_crit_damage: string[]
  attributes_that_delta_power_cost: string[]
  attributes_that_mod_power_cost: string[]
  attributes_that_delta_rage: string[]
  attributes_that_mod_rage: string[]
  attributes_that_delta_taunt: string[]
  attributes_that_mod_taunt: string[]
  extra: Record<string, unknown>
}

export interface AbilityInfo {
  id: number
  name: string
  internal_name: string | null
  description: string | null
  icon_id: number | null
  skill: string | null
  level: number | null
  keywords: string[]

  damage_type: string | null
  reset_time: number | null
  target: string | null
  prerequisite: string | null
  upgrade_of: string | null
  is_harmless: boolean | null
  animation: string | null
  special_info: string | null
  works_underwater: boolean | null
  works_while_falling: boolean | null
  pve: CombatStats | null
  pvp: CombatStats | null
  mana_cost: number | null
  power_cost: number | null
  armor_cost: number | null
  health_cost: number | null
  range: number | null

  // Full raw JSON
  raw_json: Record<string, unknown>
}

export interface AbilityFamily {
  base_internal_name: string
  base_name: string
  icon_id: number | null
  skill: string | null
  damage_type: string | null
  is_monster_ability: boolean
  tier_ids: number[]
}
