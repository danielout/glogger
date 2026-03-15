export interface EffectInfo {
  id: number
  name: string | null
  desc: string | null
  icon_id: number | null
  display_mode: string | null
  duration: unknown | null
  keywords: string[]
  ability_keywords: string[]
  stacking_type: string | null
  stacking_priority: number | null
  particle: string | null
  raw_json: Record<string, unknown>
}
