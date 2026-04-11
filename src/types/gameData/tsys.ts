export interface TsysTierInfo {
  effect_descs: string[]
  min_level: number | null
  max_level: number | null
  min_rarity: string | null
  skill_level_prereq: number | null
}

export interface TsysBrowserEntry {
  key: string
  internal_name: string | null
  skill: string | null
  slots: string[]
  prefix: string | null
  suffix: string | null
  tiers: Record<string, TsysTierInfo>
  is_unavailable: boolean | null
  is_hidden_from_transmutation: boolean | null
  tier_count: number
  raw_json: Record<string, unknown>
}
