// TypeScript types for the Farming Calculator feature

// === Active Session State ===

export interface FarmingSkillEntry {
  baseline: number
  baselineTnl: number
  gained: number
  level: number
  tnl: number
  levelsGained: number
}

export interface FarmingFavorEntry {
  npcId: number
  delta: number
}

export interface FarmingSession {
  name: string
  notes: string
  startTime: string          // "HH:MM:SS"
  endTime: string | null
  isPaused: boolean
  pauseStartTime: string | null
  totalPausedSeconds: number

  // XP tracking keyed by skill name
  skillXp: Record<string, FarmingSkillEntry>

  // Item tracking — net quantity change keyed by item_name
  itemDeltas: Record<string, number>

  // Items the user wants to hide from the display
  ignoredItems: Set<string>

  // Favor tracking keyed by npc_name
  favorDeltas: Record<string, FarmingFavorEntry>

  // Gold earned from vendor sales
  vendorGold: number
}

// === Activity Log ===

export type FarmingLogKind =
  | 'session-start'
  | 'item-gained'
  | 'item-lost'
  | 'xp-gain'
  | 'level-up'
  | 'favor-change'
  | 'vendor-sale'
  | 'session-end'

export interface FarmingLogEntry {
  kind: FarmingLogKind
  timestamp: string
  label: string
  detail?: string
}

// === Persistence (save to DB) ===

export interface SaveFarmingSessionInput {
  name: string
  notes: string
  start_time: string
  end_time: string | null
  elapsed_seconds: number
  total_paused_seconds: number
  vendor_gold: number
  skills: Array<{ skill_name: string; xp_gained: number; levels_gained: number }>
  items: Array<{ item_name: string; net_quantity: number }>
  favors: Array<{ npc_name: string; npc_id: number; delta: number }>
}

// === Historical (loaded from DB) ===

export interface HistoricalFarmingSession {
  id: number
  name: string
  notes: string
  start_time: string
  end_time: string | null
  elapsed_seconds: number
  total_paused_seconds: number
  vendor_gold: number
  created_at: string
  skills: Array<{ skill_name: string; xp_gained: number; levels_gained: number }>
  items: Array<{ item_name: string; net_quantity: number }>
  favors: Array<{ npc_name: string; npc_id: number; delta: number }>
}
