export interface AbilityInfo {
  id: number
  name: string
  description: string | null
  icon_id: number | null
  skill: string | null
  level: number | null
  keywords: string[]

  // Phase 3 typed fields
  damage_type: string | null
  reset_time: number | null
  target: string | null
  prerequisite: string | null
  is_harmless: boolean | null
  animation: string | null
  special_info: string | null
  works_underwater: boolean | null
  works_while_falling: boolean | null
  pve: unknown | null
  pvp: unknown | null
  mana_cost: number | null
  power_cost: number | null
  armor_cost: number | null
  health_cost: number | null
  range: number | null

  // Full raw JSON
  raw_json: Record<string, unknown>
}
