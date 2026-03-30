// ── Build Planner Types ─────────────────────────────────────────────────────

/** Persisted build preset from the database */
export interface BuildPreset {
  id: number
  character_id: string
  name: string
  skill_primary: string | null
  skill_secondary: string | null
  target_level: number
  target_rarity: string
  notes: string | null
  created_at: string
  updated_at: string
}

/** A mod assigned to a specific slot in a build */
export interface BuildPresetMod {
  id: number
  preset_id: number
  equip_slot: string
  power_name: string
  tier: number | null
  is_augment: boolean
  sort_order: number
}

/** A base item assigned to a slot in a build */
export interface BuildPresetSlotItem {
  preset_id: number
  equip_slot: string
  item_id: number
  item_name: string | null
}

/** A saved ability assignment in a build bar */
export interface BuildPresetAbility {
  id: number
  preset_id: number
  bar: 'primary' | 'secondary' | 'sidebar'
  slot_position: number
  ability_id: number
  ability_name: string | null
}

/** Input for saving abilities (no id, server-assigned) */
export interface BuildPresetAbilityInput {
  bar: string
  slot_position: number
  ability_id: number
  ability_name: string | null
}

/** Input for saving mods (no id, server-assigned) */
export interface BuildPresetModInput {
  equip_slot: string
  power_name: string
  tier: number | null
  is_augment: boolean
  sort_order: number
}

/** A TSys power eligible for a slot, returned from the backend query */
export interface SlotTsysPower {
  key: string
  internal_name: string | null
  skill: string | null
  prefix: string | null
  suffix: string | null
  tier_id: string | null
  effects: string[]
  raw_effects: string[]
  min_rarity: string | null
  skill_level_prereq: number | null
}

/** Equipment slot definition for the build planner */
export interface EquipSlotDef {
  id: string
  label: string
  group: 'armor' | 'weapon' | 'jewelry' | 'extra'
}

/** Rarity levels with their mod slot distributions */
export interface RarityDef {
  id: string
  label: string
  totalMods: number
  skillMods: number
  genericMods: number
}

// ── Constants ───────────────────────────────────────────────────────────────

export const EQUIPMENT_SLOTS: EquipSlotDef[] = [
  { id: 'Head', label: 'Head', group: 'armor' },
  { id: 'Chest', label: 'Chest', group: 'armor' },
  { id: 'Legs', label: 'Legs', group: 'armor' },
  { id: 'Hands', label: 'Hands', group: 'armor' },
  { id: 'Feet', label: 'Feet', group: 'armor' },
  { id: 'MainHand', label: 'Main Hand', group: 'weapon' },
  { id: 'OffHand', label: 'Off Hand', group: 'weapon' },
  { id: 'Ring', label: 'Ring', group: 'jewelry' },
  { id: 'Necklace', label: 'Necklace', group: 'jewelry' },
]

export const RARITY_DEFS: RarityDef[] = [
  { id: 'Uncommon', label: 'Uncommon', totalMods: 3, skillMods: 1, genericMods: 2 },
  { id: 'Rare', label: 'Rare', totalMods: 3, skillMods: 2, genericMods: 1 },
  { id: 'Exceptional', label: 'Exceptional', totalMods: 3, skillMods: 3, genericMods: 0 },
  { id: 'Epic', label: 'Epic', totalMods: 4, skillMods: 4, genericMods: 0 },
  { id: 'Legendary', label: 'Legendary', totalMods: 5, skillMods: 5, genericMods: 0 },
]

export const ABILITY_BARS = [
  { id: 'primary' as const, label: 'Primary', slots: 6 },
  { id: 'secondary' as const, label: 'Secondary', slots: 6 },
  { id: 'sidebar' as const, label: 'Sidebar', slots: 10 },
]

export function getRarityDef(rarity: string): RarityDef {
  return RARITY_DEFS.find(r => r.id === rarity) ?? RARITY_DEFS[3] // default to Epic
}

/** Get a display name for a TSys power */
export function getPowerDisplayName(power: SlotTsysPower): string {
  return power.prefix ?? power.suffix ?? power.internal_name ?? power.key
}
