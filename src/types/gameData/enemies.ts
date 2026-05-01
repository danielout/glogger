export interface EnemyInfo {
  key: string
  strategy: string | null
  mobility_type: string | null
  comment: string | null
  swimming: boolean
  uncontrolled_pet: boolean
  ability_count: number
  ability_names: string[]
}

/** Monster entry from translation strings, enriched with AI data when available. */
export interface MonsterEntry {
  key: string
  name: string
  area_key: string | null
  area_name: string | null
  strategy: string | null
  mobility_type: string | null
  comment: string | null
  swimming: boolean | null
  uncontrolled_pet: boolean | null
  ability_count: number | null
  ability_names: string[] | null
}
