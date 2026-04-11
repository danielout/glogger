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
  /** Equipment slots this power can appear on */
  slots: string[]
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

/** Valid mod configurations per rarity.
 *  Each config is [mainSkillCount, auxSkillCount, genericCount].
 *  "main" is whichever combat skill ends up with more mods.
 *  "aux" is the other combat skill. "generic" includes endurance. */
export type ModConfig = [main: number, aux: number, generic: number]

export const RARITY_CONFIGS: Record<string, ModConfig[]> = {
  Common: [],
  Uncommon: [
    [1, 0, 2],
    [0, 0, 3],
  ],
  Rare: [
    [2, 0, 1],
    [1, 0, 2],
    [0, 0, 3],
  ],
  Exceptional: [
    [2, 1, 0],
    [2, 0, 1],
    [1, 1, 1],
    [0, 0, 3],
  ],
  Epic: [
    [2, 2, 0],
    [2, 1, 1],
    [2, 0, 2],
    [0, 0, 4],
  ],
  Legendary: [
    [3, 2, 0],
    [3, 1, 1],
    [3, 0, 2],
    [0, 0, 5],
  ],
}

/** Given the current mod counts on an item, determine what types of mods can still be added.
 *  Returns { canAddSkillMod: boolean, canAddGeneric: boolean, mustBeSkill: boolean }
 *  where mustBeSkill means the remaining slots MUST be skill mods (no generic allowed). */
export interface SlotConstraints {
  /** Can a mod from an existing combat skill be added? (any existing skill) */
  canAddSkillMod: boolean
  /** Can a generic/endurance mod be added? */
  canAddGeneric: boolean
  /** Can a mod from a NEW (third) combat skill be added? */
  canAddNewSkill: boolean
  /** Which existing skills can still accept more mods */
  growableSkills: Set<string>
  /** Remaining empty slot count */
  emptySlots: number
  /** Which configs are still reachable */
  validConfigs: ModConfig[]
}

/**
 * Compute what mods can be placed given the current state.
 * @param rarity - item rarity ID
 * @param skillCounts - map of combat skill name → count of mods from that skill
 * @param genericCount - count of generic/endurance mods
 */
export function computeSlotConstraints(
  rarity: string,
  skillCounts: Map<string, number>,
  genericCount: number,
): SlotConstraints {
  const configs = RARITY_CONFIGS[rarity] ?? []
  const totalMods = getRarityDef(rarity).totalMods
  const usedSlots = [...skillCounts.values()].reduce((a, b) => a + b, 0) + genericCount
  const emptySlots = totalMods - usedSlots

  // Get sorted skill counts (highest first)
  const sorted = [...skillCounts.entries()].sort((a, b) => b[1] - a[1])
  const skill1Name = sorted[0]?.[0] ?? null
  const skill1Count = sorted[0]?.[1] ?? 0
  const skill2Name = sorted[1]?.[0] ?? null
  const skill2Count = sorted[1]?.[1] ?? 0
  const numSkills = sorted.filter(([, c]) => c > 0).length

  const fits = (m: number, a: number, g: number) =>
    skill1Count <= m && skill2Count <= a && genericCount <= g &&
    m + a + g === totalMods

  if (emptySlots <= 0) {
    // All slots filled — check if current state matches any valid config
    const isValid = configs.some(([main, aux, gen]) =>
      fits(main, aux, gen) || fits(aux, main, gen)
    )
    return {
      canAddSkillMod: false, canAddGeneric: false, canAddNewSkill: false,
      growableSkills: new Set(), emptySlots: 0,
      validConfigs: isValid ? configs.filter(([main, aux, gen]) => fits(main, aux, gen) || fits(aux, main, gen)) : [],
    }
  }

  // Find which configs are still reachable.
  // For each config [main, aux, gen], try assigning skill1 as main or aux.

  const validConfigs = configs.filter(([main, aux, gen]) =>
    fits(main, aux, gen) || fits(aux, main, gen)
  )

  let canAddSkillMod = false
  let canAddGeneric = false
  let canAddNewSkill = false
  const growableSkills = new Set<string>()

  for (const [main, aux, gen] of validConfigs) {
    const tryAssignment = (m: number, a: number, g: number) => {
      if (skill1Count <= m && skill2Count <= a && genericCount <= g) {
        if (skill1Count < m && skill1Name) { canAddSkillMod = true; growableSkills.add(skill1Name) }
        if (skill2Count < a && skill2Name) { canAddSkillMod = true; growableSkills.add(skill2Name) }
        if (genericCount < g) canAddGeneric = true
        if (numSkills < 2 && a > 0 && skill2Count === 0) canAddNewSkill = true
        if (numSkills === 0 && m > 0) canAddNewSkill = true
      }
    }
    tryAssignment(main, aux, gen)
    tryAssignment(aux, main, gen)
  }

  return { canAddSkillMod, canAddGeneric, canAddNewSkill, growableSkills, emptySlots, validConfigs }
}

export const ABILITY_BARS = [
  { id: 'primary' as const, label: 'Primary', slots: 6 },
  { id: 'secondary' as const, label: 'Secondary', slots: 6 },
  { id: 'sidebar' as const, label: 'Sidebar', slots: 6 },
]

/** Maximum number of sidebar slots a user can configure */
export const MAX_SIDEBAR_SLOTS = 12

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

/** Rarity text color classes (for dropdowns, labels, etc.) */
export function getRarityTextColor(rarity: string): string {
  switch (rarity) {
    case 'Legendary': return 'text-yellow-400'
    case 'Epic': return 'text-purple-400'
    case 'Exceptional': return 'text-blue-400'
    case 'Rare': return 'text-emerald-400'
    case 'Uncommon': return 'text-text-primary'
    case 'Common': return 'text-text-dim'
    default: return 'text-text-muted'
  }
}

/** Rarity border color classes (for slot indicators, cards, etc.) */
export function getRarityBorderColor(rarity: string): string {
  switch (rarity) {
    case 'Legendary': return 'border-yellow-500/60'
    case 'Epic': return 'border-purple-500/60'
    case 'Exceptional': return 'border-blue-500/60'
    case 'Rare': return 'border-emerald-500/60'
    case 'Uncommon': return 'border-border-default'
    case 'Common': return 'border-border-default/50'
    default: return 'border-border-default'
  }
}

/** Cost in crafting points to apply an augment */
export const AUGMENT_CP_COST = 100

/** Calculate total crafting points budget for a slot based on its properties.
 *  Belts get 0 CP regardless of crafted/masterwork status.
 *  Mastercrafted/foretold legendaries are a flat 160 CP.
 *  Otherwise: crafted = 120, dropped = 100. */
export function getSlotCraftingPoints(slotItem: BuildPresetSlotItem | undefined): number {
  if (!slotItem) return 0
  if (slotItem.equip_slot === 'Belt') return 0
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
