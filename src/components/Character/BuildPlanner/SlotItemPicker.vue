<template>
  <div class="flex flex-col gap-1.5">
    <div class="flex items-center gap-2">
      <h4 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Base Item</h4>
    </div>

    <!-- Currently selected item -->
    <div v-if="currentItem && currentItem.item_id !== 0" class="flex items-center gap-2 px-2 py-1.5 bg-surface-elevated border border-border-default rounded text-sm group">
      <ItemInline :reference="String(currentItem.item_id)" :show-icon="true" />
      <span class="flex-1" />
      <button
        class="text-red-400/60 hover:text-red-400 text-xs opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
        title="Remove item"
        @click="store.clearSlotItem()">
        x
      </button>
    </div>

    <!-- Browse / search controls -->
    <div class="flex flex-col gap-1">
      <!-- Search input -->
      <input
        v-model="query"
        type="text"
        placeholder="Search by name..."
        class="w-full bg-surface-elevated border border-border-default rounded px-2 py-1 text-xs text-text-primary"
        @input="onFilterChange" />

      <!-- Filter row -->
      <div class="flex items-center gap-1.5 flex-wrap">
        <!-- Skill filter -->
        <select
          v-model="filterSkill"
          class="bg-surface-elevated border border-border-default rounded px-1.5 py-0.5 text-[10px] text-text-secondary min-w-0"
          @change="onFilterChange">
          <option value="">Any skill</option>
          <option v-for="skill in availableSkills" :key="skill" :value="skill">{{ skill }}</option>
        </select>

        <!-- Level range -->
        <div class="flex items-center gap-0.5">
          <input
            v-model.number="filterLevelMin"
            type="number"
            placeholder="Lv"
            min="1"
            max="125"
            class="bg-surface-elevated border border-border-default rounded px-1 py-0.5 text-[10px] text-text-secondary w-10 text-center"
            @change="onFilterChange" />
          <span class="text-text-dim text-[10px]">–</span>
          <input
            v-model.number="filterLevelMax"
            type="number"
            placeholder="Lv"
            min="1"
            max="125"
            class="bg-surface-elevated border border-border-default rounded px-1 py-0.5 text-[10px] text-text-secondary w-10 text-center"
            @change="onFilterChange" />
        </div>

        <!-- Clear filters -->
        <button
          v-if="hasActiveFilters"
          class="text-[10px] text-text-dim hover:text-accent-red cursor-pointer"
          @click="clearFilters">
          Clear
        </button>

        <span class="flex-1" />
        <span class="text-[10px] text-text-dim">{{ results.length }} items</span>
      </div>
    </div>

    <!-- Results list -->
    <div class="flex-1 overflow-y-auto max-h-80 border border-border-default rounded">
      <div v-if="loading" class="px-2 py-3 text-xs text-text-muted text-center">
        Loading items...
      </div>
      <div v-else-if="results.length === 0" class="px-2 py-3 text-xs text-text-dim text-center">
        No items found{{ query ? ` for "${query}"` : '' }}
      </div>
      <button
        v-for="item in results"
        :key="item.id"
        class="w-full text-left px-2 py-1.5 text-xs cursor-pointer flex items-center gap-2 border-b border-border-default/50 last:border-b-0 transition-colors"
        :class="currentItem?.item_id === item.id
          ? 'bg-accent-gold/15 hover:bg-accent-gold/20'
          : 'hover:bg-surface-hover'"
        @click="selectItem(item)">
        <GameIcon :icon-id="item.icon_id" :alt="item.name" size="xs" />
        <div class="flex-1 min-w-0">
          <div class="text-text-primary truncate">{{ item.name }}</div>
          <div class="flex items-center gap-2 text-[10px] text-text-dim">
            <span v-if="item.skill_reqs">
              <template v-for="(level, skill) in item.skill_reqs" :key="String(skill)">
                {{ skill }} {{ level }}
              </template>
            </span>
            <span v-if="getItemArmorType(item)" class="px-1 rounded" :class="armorTypeBadge(getItemArmorType(item)!)">
              {{ getItemArmorType(item) }}
            </span>
          </div>
        </div>
        <div class="flex flex-col items-end gap-0.5 shrink-0">
          <span v-if="item.craft_points" class="text-[10px] text-text-dim">
            {{ item.craft_points }}cp
          </span>
          <span v-if="item.crafting_target_level" class="text-[10px] text-text-dim">
            Lv{{ item.crafting_target_level }}
          </span>
        </div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { getArmorTypeFromKeywords } from '../../../types/buildPlanner'
import type { ArmorType } from '../../../types/buildPlanner'
import type { ItemInfo } from '../../../types/gameData'
import ItemInline from '../../Shared/Item/ItemInline.vue'
import GameIcon from '../../Shared/GameIcon.vue'

const store = useBuildPlannerStore()
const gameData = useGameDataStore()

const query = ref('')
const results = ref<ItemInfo[]>([])
const loading = ref(false)
const filterSkill = ref('')
const filterLevelMin = ref<number | undefined>(undefined)
const filterLevelMax = ref<number | undefined>(undefined)

let searchTimeout: ReturnType<typeof setTimeout> | null = null

const currentItem = computed(() => {
  if (!store.selectedSlot) return undefined
  return store.getSlotItem(store.selectedSlot)
})

/** Skills relevant to this build (primary + secondary) for the filter dropdown */
const availableSkills = computed(() => {
  const skills: string[] = []
  if (store.activePreset?.skill_primary) skills.push(store.activePreset.skill_primary)
  if (store.activePreset?.skill_secondary) skills.push(store.activePreset.skill_secondary)
  return skills
})

const hasActiveFilters = computed(() =>
  query.value.length > 0 || filterSkill.value !== '' || filterLevelMin.value != null || filterLevelMax.value != null
)

function getItemArmorType(item: ItemInfo): ArmorType | null {
  if (!item.keywords || item.keywords.length === 0) return null
  return getArmorTypeFromKeywords(item.keywords)
}

function armorTypeBadge(type: string): string {
  switch (type) {
    case 'Cloth': return 'bg-blue-900/30 text-blue-300'
    case 'Leather': return 'bg-amber-900/30 text-amber-300'
    case 'Metal': return 'bg-slate-600/30 text-slate-300'
    case 'Organic': return 'bg-green-900/30 text-green-300'
    default: return 'bg-surface-hover text-text-dim'
  }
}

// Load items when slot changes — auto-browse for the slot
watch(() => store.selectedSlot, () => {
  query.value = ''
  filterSkill.value = ''
  filterLevelMin.value = undefined
  filterLevelMax.value = undefined
  loadItems()
}, { immediate: false })

onMounted(() => {
  if (store.selectedSlot) loadItems()
})

function onFilterChange() {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => loadItems(), 200)
}

function clearFilters() {
  query.value = ''
  filterSkill.value = ''
  filterLevelMin.value = undefined
  filterLevelMax.value = undefined
  loadItems()
}

async function loadItems() {
  if (!store.selectedSlot) return
  loading.value = true
  try {
    let items = await gameData.searchItems(query.value, 100, {
      equipSlot: store.selectedSlot,
      levelMin: filterLevelMin.value,
      levelMax: filterLevelMax.value,
    })

    // Client-side skill filter: keep items that require the selected skill
    if (filterSkill.value && items.length > 0) {
      items = items.filter(item => {
        if (!item.skill_reqs) return false
        return item.skill_reqs[filterSkill.value] != null
      })
    }

    results.value = items
  } catch {
    results.value = []
  } finally {
    loading.value = false
  }
}

async function selectItem(item: ItemInfo) {
  if (!store.selectedSlot) return
  await store.setSlotItem(store.selectedSlot, item.id, item.name)
}
</script>
