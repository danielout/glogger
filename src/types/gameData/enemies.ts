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
