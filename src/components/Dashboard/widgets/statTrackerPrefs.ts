/** Stat Tracker widget preferences — persisted via useViewPrefs. */

export interface StatTrackerPrefs {
  [key: string]: unknown
  /** Attribute names or special computed stat keys to display. */
  trackedStats: string[]
}

/**
 * Special computed stats that aren't raw attributes but are derived from game state.
 * These use a "computed:" prefix to distinguish them from raw attribute names.
 */
export const COMPUTED_STATS: Record<string, { label: string; description: string }> = {
  'computed:total_skill_levels': {
    label: 'Total Skill Levels',
    description: 'Sum of all skill levels (base + bonus)',
  },
  'computed:skill_count': {
    label: 'Skills Known',
    description: 'Number of skills with at least 1 level',
  },
}

export const STAT_TRACKER_DEFAULTS: StatTrackerPrefs = {
  trackedStats: [
    'computed:total_skill_levels',
    'COMBAT_XP_EARNED_MOD',
    'CRAFTING_XP_EARNED_MOD',
    'MAX_INVENTORY_SIZE',
  ],
}
