export interface TsysBrowserEntry {
  key: string
  internal_name: string | null
  skill: string | null
  slots: string[]
  prefix: string | null
  suffix: string | null
  tiers: Record<string, unknown> | null
  is_unavailable: boolean | null
  is_hidden_from_transmutation: boolean | null
  tier_count: number
  raw_json: Record<string, unknown>
}
