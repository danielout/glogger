// Re-export all game data types from their modules
export type { ItemInfo } from './items'
export type { SkillInfo } from './skills'
export type { AbilityInfo } from './abilities'
export type { RecipeIngredient, RecipeResultItem, RecipeInfo } from './recipes'
export type {
  QuestInfo,
  QuestData,
  QuestObjective,
  QuestReward,
  QuestRewardItem,
  QuestRequirement
} from './quests'
export type { NpcPreference, NpcInfo } from './npcs'
export type { EffectInfo } from './effects'
export type { PlayerTitleInfo } from './playerTitles'
export type { SourceEntry, QuestSummary, EntitySources } from './sources'

// CacheStatus lives here since it's not really a game data type
export interface CacheStatus {
  cached_version: number | null
  remote_version: number | null
  up_to_date: boolean
  item_count: number
  skill_count: number
}
