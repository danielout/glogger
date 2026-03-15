import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useSettingsStore } from './settingsStore'
import type {
  CharacterInfo,
  CharacterSnapshotSummary,
  SnapshotSkillLevel,
  SnapshotNpcFavor,
  SnapshotRecipeCompletion,
  SnapshotStat,
  SnapshotCurrency,
  ImportResult,
  SkillDiff,
  InventorySnapshotSummary,
  SnapshotItem,
  InventoryImportResult,
  InventorySummary,
} from '../types/database'

export const useCharacterStore = defineStore('character', () => {
  const characters = ref<CharacterInfo[]>([])
  const snapshots = ref<CharacterSnapshotSummary[]>([])
  const selectedCharacter = ref<CharacterInfo | null>(null)
  const selectedSnapshot = ref<CharacterSnapshotSummary | null>(null)
  const skills = ref<SnapshotSkillLevel[]>([])
  const npcFavor = ref<SnapshotNpcFavor[]>([])
  const recipes = ref<SnapshotRecipeCompletion[]>([])
  const stats = ref<SnapshotStat[]>([])
  const currencies = ref<SnapshotCurrency[]>([])
  const skillDiffs = ref<SkillDiff[]>([])
  const lastImport = ref<ImportResult | null>(null)
  const error = ref<string | null>(null)
  const loading = ref(false)
  let reportWatchInterval: ReturnType<typeof setInterval> | null = null

  // Inventory state
  const inventorySnapshots = ref<InventorySnapshotSummary[]>([])
  const selectedInventorySnapshot = ref<InventorySnapshotSummary | null>(null)
  const inventoryItems = ref<SnapshotItem[]>([])
  const inventorySummary = ref<InventorySummary | null>(null)
  const lastInventoryImport = ref<InventoryImportResult | null>(null)

  // Aggregated owned item counts by item name (summed across all stacks/vaults)
  const ownedItemCounts = computed<Record<string, number>>(() => {
    const counts: Record<string, number> = {}
    for (const item of inventoryItems.value) {
      const name = item.item_name
      counts[name] = (counts[name] ?? 0) + item.stack_size
    }
    return counts
  })

  async function importCharacterReport(): Promise<ImportResult | null> {
    const settingsStore = useSettingsStore()
    error.value = null

    const filePath = await open({
      filters: [{ name: 'Character Report', extensions: ['json'] }],
      defaultPath: settingsStore.settings.gameDataPath || undefined,
    })

    if (!filePath) return null

    try {
      loading.value = true
      const result = await invoke<ImportResult>('import_character_report', {
        filePath: filePath as string,
      })
      lastImport.value = result

      // Refresh character list after import
      await loadCharacters()

      return result
    } catch (e) {
      error.value = String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function loadCharacters() {
    try {
      characters.value = await invoke<CharacterInfo[]>('get_characters')
    } catch (e) {
      error.value = String(e)
    }
  }

  async function initForActiveCharacter() {
    const settingsStore = useSettingsStore()
    const characterName = settingsStore.settings.activeCharacterName
    const serverName = settingsStore.settings.activeServerName

    if (!characterName) return

    error.value = null
    loading.value = true

    try {
      // Try to auto-import the latest report from the Reports folder
      await invoke('import_latest_report_for_character', { characterName })
    } catch (e) {
      // Non-fatal — the report may not exist on disk
      console.warn('Auto-import latest report:', e)
    }

    try {
      // Load all characters and find the active one
      await loadCharacters()
      const activeChar = characters.value.find(
        c => c.character_name === characterName && (!serverName || c.server_name === serverName)
      )

      if (activeChar) {
        selectedCharacter.value = activeChar
        await loadSnapshots(activeChar.character_name, activeChar.server_name)

        // Auto-select the most recent snapshot
        if (snapshots.value.length > 0) {
          await selectSnapshot(snapshots.value[0])
        }
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }

    // Also init inventory data (non-blocking)
    initInventoryForActiveCharacter()
  }

  async function selectCharacter(character: CharacterInfo) {
    selectedCharacter.value = character
    selectedSnapshot.value = null
    skills.value = []
    npcFavor.value = []
    recipes.value = []
    stats.value = []
    currencies.value = []
    skillDiffs.value = []
    await loadSnapshots(character.character_name, character.server_name)
  }

  async function loadSnapshots(characterName: string, serverName?: string) {
    try {
      snapshots.value = await invoke<CharacterSnapshotSummary[]>(
        'get_character_snapshots',
        { characterName, serverName },
      )
    } catch (e) {
      error.value = String(e)
    }
  }

  async function selectSnapshot(snapshot: CharacterSnapshotSummary) {
    selectedSnapshot.value = snapshot
    skillDiffs.value = []
    await loadSnapshotDetails(snapshot.id)
  }

  async function loadSnapshotDetails(snapshotId: number) {
    try {
      loading.value = true
      const [skillsResult, favorResult, recipesResult, statsResult, currenciesResult] =
        await Promise.all([
          invoke<SnapshotSkillLevel[]>('get_snapshot_skills', { snapshotId }),
          invoke<SnapshotNpcFavor[]>('get_snapshot_npc_favor', { snapshotId }),
          invoke<SnapshotRecipeCompletion[]>('get_snapshot_recipes', { snapshotId }),
          invoke<SnapshotStat[]>('get_snapshot_stats', { snapshotId }),
          invoke<SnapshotCurrency[]>('get_snapshot_currencies', { snapshotId }),
        ])
      skills.value = skillsResult
      npcFavor.value = favorResult
      recipes.value = recipesResult
      stats.value = statsResult
      currencies.value = currenciesResult
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function compareSnapshots(oldId: number, newId: number) {
    try {
      skillDiffs.value = await invoke<SkillDiff[]>('compare_snapshots', {
        snapshotIdOld: oldId,
        snapshotIdNew: newId,
      })
    } catch (e) {
      error.value = String(e)
    }
  }

  async function pollForNewReports() {
    const settingsStore = useSettingsStore()
    const characterName = settingsStore.settings.activeCharacterName
    if (!characterName) return

    try {
      const result = await invoke<ImportResult | null>(
        'import_latest_report_for_character',
        { characterName },
      )

      if (result) {
        // New report was imported — refresh snapshot list and auto-select newest
        lastImport.value = result
        await loadCharacters()

        const serverName = settingsStore.settings.activeServerName
        const activeChar = characters.value.find(
          c => c.character_name === characterName && (!serverName || c.server_name === serverName)
        )
        if (activeChar) {
          selectedCharacter.value = activeChar
          await loadSnapshots(activeChar.character_name, activeChar.server_name)
          if (snapshots.value.length > 0) {
            await selectSnapshot(snapshots.value[0])
          }
        }
      }
    } catch (e) {
      // Silent — polling errors shouldn't disrupt the UI
      console.warn('Report watch poll error:', e)
    }

    // Also poll for inventory reports
    try {
      const invResult = await invoke<InventoryImportResult | null>(
        'import_latest_inventory_for_character',
        { characterName },
      )

      if (invResult) {
        lastInventoryImport.value = invResult
        const serverName = settingsStore.settings.activeServerName
        await loadInventorySnapshots(characterName, serverName || undefined)
        if (inventorySnapshots.value.length > 0) {
          await selectInventorySnapshot(inventorySnapshots.value[0])
        }
      }
    } catch (e) {
      console.warn('Inventory watch poll error:', e)
    }
  }

  function startReportWatching() {
    stopReportWatching()
    const settingsStore = useSettingsStore()
    if (!settingsStore.settings.autoWatchReports) return

    const intervalMs = settingsStore.settings.reportWatchIntervalSeconds * 1000
    reportWatchInterval = setInterval(pollForNewReports, intervalMs)
  }

  function stopReportWatching() {
    if (reportWatchInterval !== null) {
      clearInterval(reportWatchInterval)
      reportWatchInterval = null
    }
  }

  // ── Inventory Actions ──────────────────────────────────────────────────────

  async function importInventoryReport(): Promise<InventoryImportResult | null> {
    const settingsStore = useSettingsStore()
    error.value = null

    const filePath = await open({
      filters: [{ name: 'Inventory Report', extensions: ['json'] }],
      defaultPath: settingsStore.settings.gameDataPath || undefined,
    })

    if (!filePath) return null

    try {
      loading.value = true
      const result = await invoke<InventoryImportResult>('import_inventory_report', {
        filePath: filePath as string,
      })
      lastInventoryImport.value = result

      // Refresh inventory snapshots after import
      if (selectedCharacter.value) {
        await loadInventorySnapshots(
          selectedCharacter.value.character_name,
          selectedCharacter.value.server_name,
        )
      }

      return result
    } catch (e) {
      error.value = String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function loadInventorySnapshots(characterName: string, serverName?: string) {
    try {
      inventorySnapshots.value = await invoke<InventorySnapshotSummary[]>(
        'get_inventory_snapshots',
        { characterName, serverName },
      )
    } catch (e) {
      error.value = String(e)
    }
  }

  async function selectInventorySnapshot(snapshot: InventorySnapshotSummary) {
    selectedInventorySnapshot.value = snapshot
    try {
      loading.value = true
      const [items, summary] = await Promise.all([
        invoke<SnapshotItem[]>('get_snapshot_items', { snapshotId: snapshot.id }),
        invoke<InventorySummary>('get_inventory_summary', { snapshotId: snapshot.id }),
      ])
      inventoryItems.value = items
      inventorySummary.value = summary
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function initInventoryForActiveCharacter() {
    const settingsStore = useSettingsStore()
    const characterName = settingsStore.settings.activeCharacterName
    if (!characterName) return

    // Try auto-import
    try {
      await invoke('import_latest_inventory_for_character', { characterName })
    } catch (e) {
      console.warn('Auto-import inventory:', e)
    }

    // Load snapshots for the active character
    const serverName = settingsStore.settings.activeServerName
    await loadInventorySnapshots(characterName, serverName || undefined)

    // Auto-select most recent snapshot
    if (inventorySnapshots.value.length > 0) {
      await selectInventorySnapshot(inventorySnapshots.value[0])
    }
  }

  return {
    characters,
    snapshots,
    selectedCharacter,
    selectedSnapshot,
    skills,
    npcFavor,
    recipes,
    stats,
    currencies,
    skillDiffs,
    lastImport,
    error,
    loading,
    importCharacterReport,
    loadCharacters,
    initForActiveCharacter,
    selectCharacter,
    loadSnapshots,
    selectSnapshot,
    loadSnapshotDetails,
    compareSnapshots,
    startReportWatching,
    stopReportWatching,
    // Inventory
    inventorySnapshots,
    selectedInventorySnapshot,
    inventoryItems,
    inventorySummary,
    lastInventoryImport,
    ownedItemCounts,
    importInventoryReport,
    loadInventorySnapshots,
    selectInventorySnapshot,
    initInventoryForActiveCharacter,
  }
})
