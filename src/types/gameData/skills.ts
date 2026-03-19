export interface SkillInfo {
  id: number
  name: string
  internal_name: string
  description: string | null
  icon_id: number | null
  xp_table: string | null
  keywords: string[]

  // Phase 2 typed fields
  combat: boolean | null
  max_bonus_levels: number | null
  parents: unknown[]
  advancement_table: string | null
  guest_level_cap: number | null
  hide_when_zero: boolean | null
  advancement_hints: unknown | null
  rewards: unknown | null
  reports: unknown[] | null

  // Full raw JSON
  raw_json: Record<string, unknown>
}
