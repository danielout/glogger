import { defineStore } from "pinia"
import { ref, computed } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useSettingsStore } from "./settingsStore"
import type {
  BuildPreset,
  BuildPresetMod,
  BuildPresetModInput,
  BuildPresetSlotItem,
  BuildPresetAbility,
  BuildPresetAbilityInput,
  BuildPresetCpRecipe,
  CpRecipeOption,
  SlotTsysPower,
} from "../types/buildPlanner"
import {
  EQUIPMENT_SLOTS,
  ARMOR_SET_SLOTS,
  AUGMENT_CP_COST,
  getRarityDef,
  getArmorTypeFromKeywords,
} from "../types/buildPlanner"
import type { ArmorType } from "../types/buildPlanner"
import type { SkillInfo } from "../types/gameData/skills"
import type { ItemInfo } from "../types/gameData/items"

export const useBuildPlannerStore = defineStore("buildPlanner", () => {
  // ── State ─────────────────────────────────────────────────────────────────

  const presets = ref<BuildPreset[]>([])
  const activePreset = ref<BuildPreset | null>(null)
  const presetMods = ref<BuildPresetMod[]>([])
  const combatSkills = ref<SkillInfo[]>([])
  const selectedSlot = ref<string | null>(null)
  const slotPowers = ref<SlotTsysPower[]>([])
  const loadingPowers = ref(false)
  const modFilter = ref("")
  const slotItems = ref<BuildPresetSlotItem[]>([])
  const presetAbilities = ref<BuildPresetAbility[]>([])
  const activeBar = ref<'primary' | 'secondary' | 'sidebar' | null>(null)
  /** Resolved full item data for slot items (for keyword/armor type detection) */
  const resolvedSlotItems = ref<Record<string, ItemInfo>>({})
  /** CP-consuming recipes assigned to slots in the active preset */
  const slotCpRecipes = ref<BuildPresetCpRecipe[]>([])
  /** Available CP recipes for the currently selected slot */
  const availableCpRecipes = ref<CpRecipeOption[]>([])
  const loadingCpRecipes = ref(false)
  /** Configurable sidebar slot count (default 6, max 12) */
  const sidebarSlotCount = ref(6)

  // ── Computed ──────────────────────────────────────────────────────────────

  function getCharacterId(): string {
    const settings = useSettingsStore()
    return `${settings.settings.activeCharacterName ?? "Unknown"}@${settings.settings.activeServerName ?? "Unknown"}`
  }

  /** Mods for the currently selected slot */
  const selectedSlotMods = computed(() => {
    if (!selectedSlot.value) return []
    return presetMods.value.filter(m => m.equip_slot === selectedSlot.value)
  })

  /** Count of mods per slot */
  const slotModCounts = computed(() => {
    const counts: Record<string, number> = {}
    for (const slot of EQUIPMENT_SLOTS) {
      counts[slot.id] = presetMods.value.filter(m => m.equip_slot === slot.id && !m.is_augment).length
    }
    return counts
  })

  /** Whether each slot has an augment */
  const slotHasAugment = computed(() => {
    const aug: Record<string, boolean> = {}
    for (const slot of EQUIPMENT_SLOTS) {
      aug[slot.id] = presetMods.value.some(m => m.equip_slot === slot.id && m.is_augment)
    }
    return aug
  })

  /** CP recipes assigned to the currently selected slot */
  const selectedSlotCpRecipes = computed(() => {
    if (!selectedSlot.value) return []
    return slotCpRecipes.value.filter(r => r.equip_slot === selectedSlot.value)
  })

  /** Total CP used on a specific slot (augment + cp recipes) */
  function getSlotCpUsed(slotId: string): number {
    let used = 0
    if (slotHasAugment.value[slotId]) used += AUGMENT_CP_COST
    used += slotCpRecipes.value
      .filter(r => r.equip_slot === slotId)
      .reduce((sum, r) => sum + r.cp_cost, 0)
    return used
  }

  /** Max mods for a specific slot based on its rarity (per-slot) */
  function getMaxModsForSlot(slotId: string): number {
    const item = getSlotItem(slotId)
    const rarity = item?.slot_rarity ?? activePreset.value?.target_rarity ?? "Epic"
    return getRarityDef(rarity).totalMods
  }

  /** Max mods for the currently selected slot (convenience for the mod picker) */
  const maxModsPerSlot = computed(() => {
    if (!selectedSlot.value) {
      const rarity = activePreset.value?.target_rarity ?? "Epic"
      return getRarityDef(rarity).totalMods
    }
    return getMaxModsForSlot(selectedSlot.value)
  })

  /** Get the level for a specific slot */
  function getSlotLevel(slotId: string): number {
    const item = getSlotItem(slotId)
    return item?.slot_level ?? activePreset.value?.target_level ?? 90
  }

  /** Get the rarity for a specific slot */
  function getSlotRarity(slotId: string): string {
    const item = getSlotItem(slotId)
    if (item?.slot_rarity) return item.slot_rarity
    // Use slot-specific default (e.g., Uncommon for Belt) or preset target
    const slotDef = EQUIPMENT_SLOTS.find(s => s.id === slotId)
    if (slotDef?.defaultRarity) return slotDef.defaultRarity
    return activePreset.value?.target_rarity ?? "Epic"
  }

  /** Get the primary skill for a specific slot (per-slot override or preset default) */
  function getSlotSkillPrimary(slotId: string): string | null {
    const item = getSlotItem(slotId)
    return item?.slot_skill_primary ?? activePreset.value?.skill_primary ?? null
  }

  /** Get the secondary skill for a specific slot (per-slot override or preset default) */
  function getSlotSkillSecondary(slotId: string): string | null {
    const item = getSlotItem(slotId)
    return item?.slot_skill_secondary ?? activePreset.value?.skill_secondary ?? null
  }

  /** Available powers for the selected slot, filtered by search */
  const filteredPowers = computed(() => {
    if (!modFilter.value) return slotPowers.value
    const q = modFilter.value.toLowerCase()
    return slotPowers.value.filter(p => {
      const name = (p.prefix ?? p.suffix ?? p.internal_name ?? "").toLowerCase()
      const effects = p.effects.join(" ").toLowerCase()
      return name.includes(q) || effects.includes(q)
    })
  })

  /** Get the item assigned to a specific slot */
  function getSlotItem(slotId: string): BuildPresetSlotItem | undefined {
    return slotItems.value.find(si => si.equip_slot === slotId)
  }

  /** Abilities for a specific bar */
  function getBarAbilities(bar: string): BuildPresetAbility[] {
    return presetAbilities.value
      .filter(a => a.bar === bar)
      .sort((a, b) => a.slot_position - b.slot_position)
  }

  /** Count of abilities per bar */
  const barAbilityCounts = computed(() => ({
    primary: presetAbilities.value.filter(a => a.bar === 'primary').length,
    secondary: presetAbilities.value.filter(a => a.bar === 'secondary').length,
    sidebar: presetAbilities.value.filter(a => a.bar === 'sidebar').length,
  }))

  /** Armor type for each slot (derived from resolved item keywords) */
  const slotArmorTypes = computed(() => {
    const types: Record<string, ArmorType | null> = {}
    for (const slot of EQUIPMENT_SLOTS) {
      const slotItem = getSlotItem(slot.id)
      if (!slotItem || slotItem.item_id === 0) {
        types[slot.id] = null
        continue
      }
      const itemInfo = resolvedSlotItems.value[slot.id]
      types[slot.id] = itemInfo ? getArmorTypeFromKeywords(itemInfo.keywords ?? []) : null
    }
    return types
  })

  /** Count of each armor type across armor slots (for 3-piece bonus) */
  const armorTypeCounts = computed(() => {
    const counts: Record<string, number> = {}
    for (const slotId of ARMOR_SET_SLOTS) {
      const type = slotArmorTypes.value[slotId]
      if (type) {
        counts[type] = (counts[type] ?? 0) + 1
      }
    }
    return counts
  })

  /** Build summary: all mods grouped by skill */
  const buildSummary = computed(() => {
    const groups: Record<string, { slot: string; name: string; effects: string[] }[]> = {
      primary: [],
      secondary: [],
      generic: [],
    }

    for (const mod of presetMods.value) {
      const entry = { slot: mod.equip_slot, name: mod.power_name, effects: [] as string[] }
      groups.generic.push(entry)
    }
    return groups
  })

  // ── Actions ───────────────────────────────────────────────────────────────

  async function loadCombatSkills() {
    if (combatSkills.value.length > 0) return
    combatSkills.value = await invoke<SkillInfo[]>("get_combat_skills")
  }

  async function loadPresets() {
    const characterId = getCharacterId()
    presets.value = await invoke<BuildPreset[]>("get_build_presets", { characterId })
  }

  async function createPreset(name: string, skillPrimary?: string, skillSecondary?: string) {
    const characterId = getCharacterId()
    const id = await invoke<number>("create_build_preset", {
      input: {
        character_id: characterId,
        name,
        skill_primary: skillPrimary ?? null,
        skill_secondary: skillSecondary ?? null,
        target_level: 90,
        target_rarity: "Epic",
      },
    })
    await loadPresets()
    // Select the new preset
    const preset = presets.value.find(p => p.id === id)
    if (preset) await selectPreset(preset)
    return id
  }

  async function clonePreset(sourceId: number, newName: string) {
    const id = await invoke<number>("clone_build_preset", {
      presetId: sourceId,
      newName,
    })
    await loadPresets()
    const preset = presets.value.find(p => p.id === id)
    if (preset) await selectPreset(preset)
    return id
  }

  async function selectPreset(preset: BuildPreset) {
    activePreset.value = preset
    selectedSlot.value = null
    activeBar.value = null
    slotPowers.value = []
    const [mods, items, abilities, cpRecipes] = await Promise.all([
      invoke<BuildPresetMod[]>("get_build_preset_mods", { presetId: preset.id }),
      invoke<BuildPresetSlotItem[]>("get_build_preset_slot_items", { presetId: preset.id }),
      invoke<BuildPresetAbility[]>("get_build_preset_abilities", { presetId: preset.id }),
      invoke<BuildPresetCpRecipe[]>("get_build_preset_cp_recipes", { presetId: preset.id }),
    ])
    presetMods.value = mods
    slotItems.value = items
    presetAbilities.value = abilities
    slotCpRecipes.value = cpRecipes
    // Resolve full item data (icons, keywords, etc.) in background
    resolveSlotItemData()
  }

  async function updatePreset(updates: Partial<BuildPreset>) {
    if (!activePreset.value) return
    const input = {
      id: activePreset.value.id,
      name: updates.name ?? activePreset.value.name,
      skill_primary: updates.skill_primary ?? activePreset.value.skill_primary,
      skill_secondary: updates.skill_secondary ?? activePreset.value.skill_secondary,
      target_level: updates.target_level ?? activePreset.value.target_level,
      target_rarity: updates.target_rarity ?? activePreset.value.target_rarity,
      notes: updates.notes ?? activePreset.value.notes,
    }
    await invoke("update_build_preset", { input })
    // Update local state
    Object.assign(activePreset.value, updates)
    await loadPresets()
  }

  async function deletePreset(id: number) {
    await invoke("delete_build_preset", { presetId: id })
    if (activePreset.value?.id === id) {
      activePreset.value = null
      presetMods.value = []
      slotItems.value = []
      presetAbilities.value = []
      selectedSlot.value = null
      activeBar.value = null
      slotPowers.value = []
    }
    await loadPresets()
  }

  async function selectSlot(slotId: string) {
    selectedSlot.value = slotId
    activeBar.value = null
    modFilter.value = ""
    await Promise.all([loadSlotPowers(), loadAvailableCpRecipes()])
  }

  async function loadSlotPowers() {
    if (!selectedSlot.value || !activePreset.value) {
      slotPowers.value = []
      return
    }
    loadingPowers.value = true
    try {
      const level = getSlotLevel(selectedSlot.value)
      slotPowers.value = await invoke<SlotTsysPower[]>("get_tsys_powers_for_slot", {
        skillPrimary: getSlotSkillPrimary(selectedSlot.value),
        skillSecondary: getSlotSkillSecondary(selectedSlot.value),
        equipSlot: selectedSlot.value,
        targetLevel: level,
      })
    } catch (e) {
      console.error("[buildPlanner] Failed to load slot powers:", e)
      slotPowers.value = []
    } finally {
      loadingPowers.value = false
    }
  }

  /** Add a mod to the currently selected slot */
  async function addMod(power: SlotTsysPower, isAugment: boolean = false, tierId?: string) {
    if (!activePreset.value || !selectedSlot.value) return

    // Check if this power is already assigned to this slot
    const existing = presetMods.value.find(
      m => m.equip_slot === selectedSlot.value && m.power_name === (power.internal_name ?? power.key)
    )
    if (existing) return // no duplicates on same slot

    // Check slot capacity
    const slotMods = presetMods.value.filter(m => m.equip_slot === selectedSlot.value && !m.is_augment)
    if (!isAugment && slotMods.length >= maxModsPerSlot.value) return

    // Check augment limit (1 per slot)
    if (isAugment && slotHasAugment.value[selectedSlot.value]) return

    const nextOrder = Math.max(0, ...presetMods.value
      .filter(m => m.equip_slot === selectedSlot.value)
      .map(m => m.sort_order)) + 1

    const effectiveTierId = tierId ?? power.tier_id

    // Add to local state immediately
    presetMods.value.push({
      id: -Date.now(), // temp id
      preset_id: activePreset.value.id,
      equip_slot: selectedSlot.value,
      power_name: power.internal_name ?? power.key,
      tier: effectiveTierId ? parseInt(effectiveTierId.replace("id_", "")) : null,
      is_augment: isAugment,
      sort_order: nextOrder,
    })

    await saveMods()
  }

  /** Change the tier of an existing mod */
  async function changeModTier(mod: BuildPresetMod, tierId: string) {
    const target = presetMods.value.find(m => m === mod)
    if (!target) return
    target.tier = parseInt(tierId.replace("id_", ""))
    await saveMods()
  }

  /** Remove a mod from the build */
  async function removeMod(mod: BuildPresetMod) {
    presetMods.value = presetMods.value.filter(m => m !== mod)
    await saveMods()
  }

  /** Persist all mods to the database */
  async function saveMods() {
    if (!activePreset.value) return
    const modsInput: BuildPresetModInput[] = presetMods.value.map(m => ({
      equip_slot: m.equip_slot,
      power_name: m.power_name,
      tier: m.tier,
      is_augment: m.is_augment,
      sort_order: m.sort_order,
    }))
    await invoke("set_build_preset_mods", {
      presetId: activePreset.value.id,
      mods: modsInput,
    })
    // Reload to get server-assigned IDs
    presetMods.value = await invoke<BuildPresetMod[]>("get_build_preset_mods", {
      presetId: activePreset.value.id,
    })
  }

  /** When skills or level change, reload available powers for selected slot */
  async function onBuildParamsChanged() {
    if (selectedSlot.value) {
      await loadSlotPowers()
    }
  }

  /** Load all slot items for the active preset */
  async function loadSlotItems() {
    if (!activePreset.value) {
      slotItems.value = []
      resolvedSlotItems.value = {}
      return
    }
    slotItems.value = await invoke<BuildPresetSlotItem[]>("get_build_preset_slot_items", {
      presetId: activePreset.value.id,
    })
    await resolveSlotItemData()
  }

  /** Resolve full item data for all slot items (for keywords, armor type, etc.) */
  async function resolveSlotItemData() {
    const resolved: Record<string, ItemInfo> = {}
    for (const si of slotItems.value) {
      if (si.item_id === 0) continue
      try {
        const info = await invoke<ItemInfo | null>("resolve_item", {
          reference: String(si.item_id),
        })
        if (info) {
          resolved[si.equip_slot] = info
        }
      } catch {
        // Item might not resolve
      }
    }
    resolvedSlotItems.value = resolved
  }

  /** Set or replace the base item for a specific slot */
  async function setSlotItem(
    slotId: string,
    itemId: number,
    itemName: string | null,
    slotLevel?: number,
    slotRarity?: string,
    isCrafted?: boolean,
    isMasterwork?: boolean,
  ) {
    if (!activePreset.value) return
    const existingItem = getSlotItem(slotId)
    await invoke("set_build_preset_slot_item", {
      input: {
        preset_id: activePreset.value.id,
        equip_slot: slotId,
        item_id: itemId,
        item_name: itemName,
        slot_level: slotLevel ?? existingItem?.slot_level ?? activePreset.value.target_level ?? 90,
        slot_rarity: slotRarity ?? existingItem?.slot_rarity ?? activePreset.value.target_rarity ?? "Epic",
        is_crafted: isCrafted ?? existingItem?.is_crafted ?? false,
        is_masterwork: isMasterwork ?? existingItem?.is_masterwork ?? false,
      },
    })
    await loadSlotItems()
  }

  /** Update slot properties without changing the item */
  async function updateSlotProps(
    slotId: string,
    props: {
      slot_level?: number
      slot_rarity?: string
      is_crafted?: boolean
      is_masterwork?: boolean
      slot_skill_primary?: string | null
      slot_skill_secondary?: string | null
    },
  ) {
    if (!activePreset.value) return
    const existing = getSlotItem(slotId)
    if (!existing) {
      // If no item set yet, we need to create a placeholder entry
      // Use item_id 0 as "no item" placeholder
      await invoke("set_build_preset_slot_item", {
        input: {
          preset_id: activePreset.value.id,
          equip_slot: slotId,
          item_id: 0,
          item_name: null,
          slot_level: props.slot_level ?? activePreset.value.target_level ?? 90,
          slot_rarity: props.slot_rarity ?? getSlotRarity(slotId),
          is_crafted: props.is_crafted ?? false,
          is_masterwork: props.is_masterwork ?? false,
        },
      })
      // If skills were provided, update them separately
      if (props.slot_skill_primary !== undefined || props.slot_skill_secondary !== undefined) {
        await invoke("update_build_preset_slot_props", {
          presetId: activePreset.value.id,
          equipSlot: slotId,
          slotLevel: null,
          slotRarity: null,
          isCrafted: null,
          isMasterwork: null,
          slotSkillPrimary: props.slot_skill_primary ?? null,
          slotSkillSecondary: props.slot_skill_secondary ?? null,
        })
      }
    } else {
      await invoke("update_build_preset_slot_props", {
        presetId: activePreset.value.id,
        equipSlot: slotId,
        slotLevel: props.slot_level ?? null,
        slotRarity: props.slot_rarity ?? null,
        isCrafted: props.is_crafted ?? null,
        isMasterwork: props.is_masterwork ?? null,
        slotSkillPrimary: props.slot_skill_primary ?? null,
        slotSkillSecondary: props.slot_skill_secondary ?? null,
      })
    }
    await loadSlotItems()
    // Reload powers if this is the selected slot and skills or level changed
    if (slotId === selectedSlot.value && (
      props.slot_level != null || props.slot_skill_primary !== undefined || props.slot_skill_secondary !== undefined
    )) {
      await loadSlotPowers()
    }
  }

  /** Clear the base item for a specific slot */
  async function clearSlotItem(slotId?: string) {
    if (!activePreset.value) return
    const slot = slotId ?? selectedSlot.value
    if (!slot) return
    await invoke("clear_build_preset_slot_item", {
      presetId: activePreset.value.id,
      equipSlot: slot,
    })
    await loadSlotItems()
  }

  /** Select an ability bar for editing */
  function selectBar(bar: 'primary' | 'secondary' | 'sidebar') {
    activeBar.value = bar
    selectedSlot.value = null
    slotPowers.value = []
  }

  /** Set an ability at a specific slot position on the active bar.
   *  Replaces whatever was in that slot (if anything). */
  async function setAbilityAtSlot(bar: 'primary' | 'secondary' | 'sidebar', slotPosition: number, abilityId: number, abilityName: string | null) {
    if (!activePreset.value) return

    // Remove any existing ability at this slot position
    presetAbilities.value = presetAbilities.value.filter(
      a => !(a.bar === bar && a.slot_position === slotPosition)
    )

    // Also remove this exact ability if it's elsewhere on the same bar (no duplicates)
    presetAbilities.value = presetAbilities.value.filter(
      a => !(a.bar === bar && a.ability_id === abilityId)
    )

    presetAbilities.value.push({
      id: -Date.now(),
      preset_id: activePreset.value.id,
      bar,
      slot_position: slotPosition,
      ability_id: abilityId,
      ability_name: abilityName,
    })

    await saveBarAbilities(bar)
  }

  /** Clear a specific slot on a bar */
  async function clearAbilitySlot(bar: 'primary' | 'secondary' | 'sidebar', slotPosition: number) {
    if (!activePreset.value) return
    presetAbilities.value = presetAbilities.value.filter(
      a => !(a.bar === bar && a.slot_position === slotPosition)
    )
    await saveBarAbilities(bar)
  }

  /** Clear all abilities from a bar */
  async function clearBar(bar: 'primary' | 'secondary' | 'sidebar') {
    if (!activePreset.value) return
    presetAbilities.value = presetAbilities.value.filter(a => a.bar !== bar)
    await saveBarAbilities(bar)
  }

  /** Remove an ability from a bar */
  async function removeAbility(ability: BuildPresetAbility) {
    presetAbilities.value = presetAbilities.value.filter(a => a !== ability)
    await saveBarAbilities(ability.bar)
  }

  /** Persist abilities for a specific bar (preserves slot_position values) */
  async function saveBarAbilities(bar: string) {
    if (!activePreset.value) return
    const barAbilities = presetAbilities.value
      .filter(a => a.bar === bar)
      .sort((a, b) => a.slot_position - b.slot_position)

    const input: BuildPresetAbilityInput[] = barAbilities.map(a => ({
      bar: a.bar,
      slot_position: a.slot_position,
      ability_id: a.ability_id,
      ability_name: a.ability_name,
    }))

    await invoke("set_build_preset_abilities", {
      presetId: activePreset.value.id,
      bar,
      abilities: input,
    })

    // Reload to get server-assigned IDs
    presetAbilities.value = await invoke<BuildPresetAbility[]>("get_build_preset_abilities", {
      presetId: activePreset.value.id,
    })
  }

  // ── CP Recipe Actions ───────────────────────────────────────────────────

  async function loadAvailableCpRecipes() {
    if (!selectedSlot.value) {
      availableCpRecipes.value = []
      return
    }
    loadingCpRecipes.value = true
    try {
      availableCpRecipes.value = await invoke<CpRecipeOption[]>("get_cp_recipes_for_slot", {
        equipSlot: selectedSlot.value,
      })
    } finally {
      loadingCpRecipes.value = false
    }
  }

  async function addCpRecipe(recipe: CpRecipeOption) {
    if (!activePreset.value || !selectedSlot.value) return

    slotCpRecipes.value.push({
      id: 0, // placeholder, server assigns
      preset_id: activePreset.value.id,
      equip_slot: selectedSlot.value,
      recipe_id: recipe.recipe_id,
      recipe_name: recipe.recipe_name,
      cp_cost: recipe.cp_cost,
      effect_type: recipe.effect_type,
      effect_key: recipe.effect_key,
      sort_order: selectedSlotCpRecipes.value.length,
    })
    await saveCpRecipes()
  }

  async function removeCpRecipe(recipe: BuildPresetCpRecipe) {
    slotCpRecipes.value = slotCpRecipes.value.filter(r => r !== recipe)
    await saveCpRecipes()
  }

  async function saveCpRecipes() {
    if (!activePreset.value || !selectedSlot.value) return
    const slotRecipes = slotCpRecipes.value
      .filter(r => r.equip_slot === selectedSlot.value)
      .map((r, i) => ({
        equip_slot: r.equip_slot,
        recipe_id: r.recipe_id,
        recipe_name: r.recipe_name,
        cp_cost: r.cp_cost,
        effect_type: r.effect_type,
        effect_key: r.effect_key,
        sort_order: i,
      }))

    await invoke("set_build_preset_cp_recipes", {
      presetId: activePreset.value.id,
      equipSlot: selectedSlot.value,
      recipes: slotRecipes,
    })

    // Reload to get server-assigned IDs
    slotCpRecipes.value = await invoke<BuildPresetCpRecipe[]>("get_build_preset_cp_recipes", {
      presetId: activePreset.value.id,
    })
  }

  return {
    // State
    presets,
    activePreset,
    presetMods,
    combatSkills,
    selectedSlot,
    slotPowers,
    loadingPowers,
    modFilter,
    slotItems,
    presetAbilities,
    activeBar,
    resolvedSlotItems,
    slotCpRecipes,
    availableCpRecipes,
    loadingCpRecipes,
    sidebarSlotCount,
    // Computed
    selectedSlotMods,
    selectedSlotCpRecipes,
    slotModCounts,
    slotHasAugment,
    maxModsPerSlot,
    filteredPowers,
    buildSummary,
    barAbilityCounts,
    slotArmorTypes,
    armorTypeCounts,
    // Actions
    loadCombatSkills,
    loadPresets,
    createPreset,
    clonePreset,
    selectPreset,
    updatePreset,
    deletePreset,
    selectSlot,
    loadSlotPowers,
    addMod,
    removeMod,
    changeModTier,
    saveMods,
    onBuildParamsChanged,
    getSlotItem,
    getSlotLevel,
    getSlotRarity,
    getSlotSkillPrimary,
    getSlotSkillSecondary,
    getMaxModsForSlot,
    setSlotItem,
    updateSlotProps,
    clearSlotItem,
    getBarAbilities,
    selectBar,
    setAbilityAtSlot,
    clearAbilitySlot,
    clearBar,
    removeAbility,
    getSlotCpUsed,
    loadAvailableCpRecipes,
    addCpRecipe,
    removeCpRecipe,
  }
})
