import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useSkillStore } from './skillStore'
import { useCharacterStore } from './characterStore'
import type { FoodItem, GourmandFoodEntry, GourmandImportResult } from '../types/gourmand'

export const useGourmandStore = defineStore('gourmand', () => {
  // ── State ──────────────────────────────────────────────────────────────────

  const allFoods = ref<FoodItem[]>([])
  const eatenFoods = ref<Map<string, number>>(new Map())
  const reportLoaded = ref(false)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Gourmand skill level
  const manualGourmandLevel = ref<number | null>(null)

  // UI state
  const selectedMeal = ref<FoodItem | null>(null)
  const selectedSnack = ref<FoodItem | null>(null)
  const hideEaten = ref(false)
  const hideUnusable = ref(false)
  const sortMode = ref<'level' | 'alpha' | 'food-level'>('level')
  const sortAsc = ref(true)
  const viewMode = ref<'card' | 'list'>('card')

  // ── Computed: Gourmand level resolution ────────────────────────────────────

  const gourmandLevel = computed<number | null>(() => {
    // Priority: manual override > live session > character snapshot
    if (manualGourmandLevel.value !== null) return manualGourmandLevel.value

    const skillStore = useSkillStore()
    const sessionSkill = skillStore.skills['Gourmand']
    if (sessionSkill) return sessionSkill.currentLevel

    const characterStore = useCharacterStore()
    const snapshotSkill = characterStore.skills.find(s => s.skill_name === 'Gourmand')
    if (snapshotSkill) return snapshotSkill.level

    return null
  })

  const gourmandLevelSource = computed<'manual' | 'session' | 'snapshot' | null>(() => {
    if (manualGourmandLevel.value !== null) return 'manual'

    const skillStore = useSkillStore()
    if (skillStore.skills['Gourmand']) return 'session'

    const characterStore = useCharacterStore()
    if (characterStore.skills.find(s => s.skill_name === 'Gourmand')) return 'snapshot'

    return null
  })

  // ── Computed: Category splits ──────────────────────────────────────────────

  const meals = computed(() => allFoods.value.filter(f => f.food_category === 'Meal'))
  const snacks = computed(() => allFoods.value.filter(f => f.food_category === 'Snack'))
  const instantSnacks = computed(() => allFoods.value.filter(f => f.food_category === 'Instant-Snack'))

  // ── Computed: Progress stats ───────────────────────────────────────────────

  function countEaten(foods: FoodItem[]): number {
    return foods.filter(f => eatenFoods.value.has(f.name)).length
  }

  const mealsEaten = computed(() => countEaten(meals.value))
  const snacksEaten = computed(() => countEaten(snacks.value))
  const instantSnacksEaten = computed(() => countEaten(instantSnacks.value))
  const totalEaten = computed(() => countEaten(allFoods.value))

  const mealProgress = computed(() => meals.value.length ? (mealsEaten.value / meals.value.length) * 100 : 0)
  const snackProgress = computed(() => snacks.value.length ? (snacksEaten.value / snacks.value.length) * 100 : 0)
  const instantSnackProgress = computed(() => instantSnacks.value.length ? (instantSnacksEaten.value / instantSnacks.value.length) * 100 : 0)
  const overallProgress = computed(() => allFoods.value.length ? (totalEaten.value / allFoods.value.length) * 100 : 0)

  // ── Computed: Favorites (top 3 most consumed per category) ─────────────────

  function topEaten(foods: FoodItem[], n = 3): { name: string; count: number }[] {
    return foods
      .filter(f => eatenFoods.value.has(f.name))
      .map(f => ({ name: f.name, count: eatenFoods.value.get(f.name)! }))
      .sort((a, b) => b.count - a.count)
      .slice(0, n)
  }

  const favoriteMeals = computed(() => topEaten(meals.value))
  const favoriteSnacks = computed(() => topEaten(snacks.value))
  const favoriteInstantSnacks = computed(() => topEaten(instantSnacks.value))

  // ── Computed: Uneaten foods ────────────────────────────────────────────────

  const uneatenFoods = computed(() => allFoods.value.filter(f => !eatenFoods.value.has(f.name)))

  // ── Computed: Combined effects for comparison panel ────────────────────────

  const combinedEffects = computed<string[]>(() => {
    const effects: string[] = []
    if (selectedMeal.value) {
      effects.push(...selectedMeal.value.effect_descs)
    }
    if (selectedSnack.value) {
      effects.push(...selectedSnack.value.effect_descs)
    }
    return effects
  })

  // ── Actions ────────────────────────────────────────────────────────────────

  async function loadAllFoods() {
    try {
      loading.value = true
      allFoods.value = await invoke<FoodItem[]>('get_all_foods')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadEatenFoods() {
    try {
      const entries = await invoke<GourmandFoodEntry[]>('get_gourmand_eaten_foods')
      if (entries.length > 0) {
        const map = new Map<string, number>()
        for (const entry of entries) {
          map.set(entry.name, entry.count)
        }
        eatenFoods.value = map
        reportLoaded.value = true
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  async function importReport() {
    error.value = null

    const filePath = await open({
      filters: [{ name: 'Gourmand Report', extensions: ['txt'] }],
    })

    if (!filePath) return null

    try {
      loading.value = true
      const result = await invoke<GourmandImportResult>('import_gourmand_report', {
        filePath: filePath as string,
      })

      // Reload eaten foods from DB after import
      await loadEatenFoods()

      return result
    } catch (e) {
      error.value = String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  function setManualLevel(level: number | null) {
    manualGourmandLevel.value = level
  }

  function selectMeal(food: FoodItem | null) {
    selectedMeal.value = food
  }

  function selectSnack(food: FoodItem | null) {
    selectedSnack.value = food
  }

  function clearSelection() {
    selectedMeal.value = null
    selectedSnack.value = null
  }

  async function exportUneaten() {
    const filePath = await save({
      filters: [{ name: 'Text File', extensions: ['txt'] }],
      defaultPath: 'uneaten-foods.txt',
    })

    if (!filePath) return

    const lines = uneatenFoods.value.map(f => {
      const req = f.gourmand_req !== null ? ` (Gourmand ${f.gourmand_req})` : ''
      return `${f.name} — ${f.food_category} Level ${f.food_level}${req}`
    })

    const content = `Uneaten Foods (${lines.length} remaining)\n\n${lines.join('\n')}\n`

    try {
      await invoke('export_text_file', { filePath, content })
    } catch (e) {
      error.value = `Failed to write export file: ${e}`
    }
  }

  async function tryAutoImport() {
    try {
      const result = await invoke<GourmandImportResult | null>('import_latest_gourmand_report')
      if (result) {
        await loadEatenFoods()
      }
    } catch (e) {
      console.warn('Gourmand auto-import:', e)
    }
  }

  // ── Sort helper ────────────────────────────────────────────────────────────

  function sortedFoods(foods: FoodItem[]): FoodItem[] {
    const sorted = [...foods]
    switch (sortMode.value) {
      case 'level':
        sorted.sort((a, b) => (a.gourmand_req ?? 0) - (b.gourmand_req ?? 0) || a.name.localeCompare(b.name))
        break
      case 'food-level':
        sorted.sort((a, b) => a.food_level - b.food_level || a.name.localeCompare(b.name))
        break
      case 'alpha':
        sorted.sort((a, b) => a.name.localeCompare(b.name))
        break
    }
    return sorted
  }

  return {
    // State
    allFoods,
    eatenFoods,
    reportLoaded,
    loading,
    error,
    manualGourmandLevel,
    selectedMeal,
    selectedSnack,
    hideEaten,
    hideUnusable,
    sortMode,
    sortAsc,
    viewMode,
    // Computed
    gourmandLevel,
    gourmandLevelSource,
    meals,
    snacks,
    instantSnacks,
    mealsEaten,
    snacksEaten,
    instantSnacksEaten,
    totalEaten,
    mealProgress,
    snackProgress,
    instantSnackProgress,
    overallProgress,
    favoriteMeals,
    favoriteSnacks,
    favoriteInstantSnacks,
    uneatenFoods,
    combinedEffects,
    // Actions
    loadAllFoods,
    loadEatenFoods,
    importReport,
    setManualLevel,
    selectMeal,
    selectSnack,
    clearSelection,
    exportUneaten,
    tryAutoImport,
    sortedFoods,
  }
})
