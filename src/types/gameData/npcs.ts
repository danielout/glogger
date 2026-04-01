export interface NpcPreference {
  name: string | null
  desire: string
  keywords: string[]
  pref: number
}

export interface NpcInfo {
  key: string
  name: string
  desc: string | null
  area_name: string | null
  area_friendly_name: string | null
  trains_skills: string[]
  preferences: NpcPreference[]
  gift_favor_tiers: string[]

  // Phase 4 typed fields
  pos: unknown | null
  services: unknown[] | null

  // Full raw JSON
  raw_json: Record<string, unknown>
}
