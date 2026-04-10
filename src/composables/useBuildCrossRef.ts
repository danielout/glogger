import { computed } from 'vue'
import { useBuildPlannerStore } from '../stores/buildPlannerStore'

/** Strip trailing rank number from ability name (e.g. "Pound To Slag 9" -> "Pound To Slag") */
function baseAbilityName(name: string): string {
  return name.replace(/\s+\d+$/, '')
}

/**
 * Provides cross-referencing between mods and abilities in the build planner.
 *
 * - `abilityBaseNames`: deduplicated set of base ability names from assigned abilities
 * - `abilityModCounts`: map of base ability name -> count of slot powers whose effects mention it
 * - `powerAbilities`: map of power key -> list of ability names the power's effects mention
 * - `isAbilityRelated`: check if a power's effects reference any assigned ability
 */
export function useBuildCrossRef() {
  const store = useBuildPlannerStore()

  /** Deduplicated base ability names from all assigned abilities */
  const abilityBaseNames = computed(() => {
    const names = new Map<string, string>()
    for (const a of store.presetAbilities) {
      if (!a.ability_name) continue
      const base = baseAbilityName(a.ability_name)
      if (!names.has(base) || a.ability_name > names.get(base)!) {
        names.set(base, a.ability_name)
      }
    }
    return names
  })

  /**
   * For each slot power currently loaded, which assigned abilities do its effects reference?
   * Returns a Map of power key -> array of display ability names.
   */
  const powerAbilities = computed(() => {
    const result = new Map<string, string[]>()
    if (abilityBaseNames.value.size === 0) return result

    for (const power of store.slotPowers) {
      const key = power.internal_name ?? power.key
      const matched: string[] = []

      for (const [baseName, displayName] of abilityBaseNames.value) {
        const baseNameLower = baseName.toLowerCase()
        const effectsMatch = power.effects.some(e => e.toLowerCase().includes(baseNameLower))
          || power.raw_effects.some(e => e.toLowerCase().includes(baseNameLower))
        if (effectsMatch) {
          matched.push(displayName)
        }
      }

      if (matched.length > 0) {
        result.set(key, matched)
      }
    }
    return result
  })

  /**
   * Count of mods (from slotPowers) that reference each assigned ability.
   * Returns a Map of base ability name -> count.
   */
  const abilityModCounts = computed(() => {
    const counts = new Map<string, number>()
    if (abilityBaseNames.value.size === 0) return counts

    for (const [baseName] of abilityBaseNames.value) {
      const baseNameLower = baseName.toLowerCase()
      let count = 0
      for (const power of store.slotPowers) {
        const mentions = power.effects.some(e => e.toLowerCase().includes(baseNameLower))
          || power.raw_effects.some(e => e.toLowerCase().includes(baseNameLower))
        if (mentions) count++
      }
      counts.set(baseName, count)
    }
    return counts
  })

  /**
   * Check if a power's effects reference any assigned ability.
   * Useful for filtering out mods not related to the build's abilities.
   */
  function isAbilityRelated(powerKey: string): boolean {
    return powerAbilities.value.has(powerKey)
  }

  /**
   * Get the ability names a power's effects reference.
   */
  function getAbilitiesForPower(powerKey: string): string[] {
    return powerAbilities.value.get(powerKey) ?? []
  }

  return {
    abilityBaseNames,
    powerAbilities,
    abilityModCounts,
    isAbilityRelated,
    getAbilitiesForPower,
  }
}
