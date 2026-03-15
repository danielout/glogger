import type { ItemInfo } from './items'

export interface SourceEntry {
  source_type: string
  skill: string | null
  npc: string | null
  item_type_id: number | null
  quest_id: number | null
  recipe_id: number | null
  hang_out_id: number | null
  friendly_name: string | null
  extra: Record<string, unknown>
}

export interface QuestSummary {
  key: string
  name: string
}

export interface EntitySources {
  cdn_sources: SourceEntry[]
  bestowed_by_items: ItemInfo[]
  rewarded_by_quests: QuestSummary[]
}
