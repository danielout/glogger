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
  slot_level: number
  slot_rarity: string
  is_crafted: boolean
  is_masterwork: boolean
  slot_skill_primary: string | null
  slot_skill_secondary: string | null
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

/** A CP-consuming recipe option returned from CDN query */
export interface CpRecipeOption {
  recipe_id: number
  recipe_name: string
  icon_id: number | null
  skill: string | null
  skill_level_req: number | null
  cp_cost: number
  effect_type: 'shamanic_infusion' | 'crafting_enhancement'
  effect_key: string
  effect_description: string
}

/** A CP recipe assigned to a slot in a saved build */
export interface BuildPresetCpRecipe {
  id: number
  preset_id: number
  equip_slot: string
  recipe_id: number
  recipe_name: string | null
  cp_cost: number
  effect_type: string
  effect_key: string
  sort_order: number
}

/** A TSys power eligible for a slot, returned from the backend query */
export interface TsysTierSummary {
  tier_id: string
  min_level: number
  max_level: number
  min_rarity: string | null
  skill_level_prereq: number | null
  effects: string[]
  icon_id: number | null
}

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
  icon_id: number | null
  available_tiers: TsysTierSummary[]
}

/** Equipment slot definition for the build planner */
export interface EquipSlotDef {
  id: string
  label: string
  group: 'armor' | 'weapon' | 'jewelry' | 'extra'
  /** Index into RARITY_DEFS for the highest allowed rarity. Undefined = no cap. */
  maxRarityIndex?: number
  /** Default rarity for this slot (overrides the preset's target rarity). */
  defaultRarity?: string
}

/** Rarity levels with their mod slot distributions.
 *  primarySlots/secondarySlots represent the default distribution of skill-specific
 *  mod slots. Any skill slot can be replaced by generic/endurance. */
export interface RarityDef {
  id: string
  label: string
  totalMods: number
  /** Number of mod slots defaulting to primary skill */
  primarySlots: number
  /** Number of mod slots defaulting to secondary skill */
  secondarySlots: number
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
  { id: 'Belt', label: 'Belt', group: 'extra', maxRarityIndex: 1, defaultRarity: 'Uncommon' },
]

export const RARITY_DEFS: RarityDef[] = [
  { id: 'Common', label: 'Common', totalMods: 0, primarySlots: 0, secondarySlots: 0 },
  { id: 'Uncommon', label: 'Uncommon', totalMods: 3, primarySlots: 1, secondarySlots: 0 },
  { id: 'Rare', label: 'Rare', totalMods: 3, primarySlots: 1, secondarySlots: 1 },
  { id: 'Exceptional', label: 'Exceptional', totalMods: 3, primarySlots: 2, secondarySlots: 1 },
  { id: 'Epic', label: 'Epic', totalMods: 4, primarySlots: 2, secondarySlots: 2 },
  { id: 'Legendary', label: 'Legendary', totalMods: 5, primarySlots: 3, secondarySlots: 2 },
]

export const ABILITY_BARS = [
  { id: 'primary' as const, label: 'Primary', slots: 6 },
  { id: 'secondary' as const, label: 'Secondary', slots: 6 },
  { id: 'sidebar' as const, label: 'Sidebar', slots: 10 },
]

export function getRarityDef(rarity: string): RarityDef {
  return RARITY_DEFS.find(r => r.id === rarity) ?? RARITY_DEFS[4] // default to Epic
}

/** Get the allowed rarities for a specific slot (belt is limited to Common/Uncommon) */
export function getAllowedRarities(slot: EquipSlotDef): RarityDef[] {
  if (slot.maxRarityIndex != null) {
    return RARITY_DEFS.slice(0, slot.maxRarityIndex + 1)
  }
  // Non-belt slots: skip Common (index 0) since regular equipment can't be Common in the planner
  return RARITY_DEFS.slice(1)
}

/** Get the default rarity for a slot (belt defaults to Uncommon, others use preset target) */
export function getDefaultRarityForSlot(slot: EquipSlotDef): string {
  return slot.defaultRarity ?? 'Epic'
}

/** Cost in crafting points to apply an augment */
export const AUGMENT_CP_COST = 100

/** Calculate total crafting points budget for a slot based on its properties.
 *  Mastercrafted/foretold legendaries are a flat 160 CP.
 *  Otherwise: crafted = 120, dropped = 100. */
export function getSlotCraftingPoints(slotItem: BuildPresetSlotItem | undefined): number {
  if (!slotItem) return 0
  if (slotItem.is_masterwork) return 160
  return slotItem.is_crafted ? 120 : 100
}

/** Armor type keywords used to detect material type */
export const ARMOR_TYPE_KEYWORDS = ['ClothArmor', 'LeatherArmor', 'MetalArmor', 'OrganicArmor'] as const
export type ArmorType = 'Cloth' | 'Leather' | 'Metal' | 'Organic'

/** Map keyword to display name */
export function getArmorTypeFromKeywords(keywords: string[]): ArmorType | null {
  if (keywords.includes('ClothArmor')) return 'Cloth'
  if (keywords.includes('LeatherArmor')) return 'Leather'
  if (keywords.includes('MetalArmor')) return 'Metal'
  if (keywords.includes('OrganicArmor')) return 'Organic'
  return null
}

/** Armor slots that count toward 3-piece set bonus */
export const ARMOR_SET_SLOTS = ['Head', 'Chest', 'Legs', 'Hands', 'Feet']

/** Get max mods for a specific rarity */
export function getMaxModsForRarity(rarity: string): number {
  return getRarityDef(rarity).totalMods
}

/** Get a display name for a TSys power */
export function getPowerDisplayName(power: SlotTsysPower): string {
  return power.prefix ?? power.suffix ?? power.internal_name ?? power.key
}
