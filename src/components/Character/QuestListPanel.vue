<template>
  <div class="flex flex-col gap-2 overflow-hidden h-full">
    <!-- Filters -->
    <div class="flex items-center gap-2 flex-wrap">
      <input
        v-model="filter"
        type="text"
        placeholder="Filter quests..."
        class="px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 w-40" />

      <select
        v-model="groupBy"
        class="px-2 py-1.5 bg-surface-base border border-border-default rounded text-xs text-text-primary cursor-pointer">
        <option value="category">By Type</option>
        <option value="area">By Area</option>
        <option value="npc">By NPC</option>
        <option value="level">By Level</option>
        <option value="keyword">By Keyword</option>
        <option value="none">No Grouping</option>
      </select>

      <select
        v-model="sortBy"
        class="px-2 py-1.5 bg-surface-base border border-border-default rounded text-xs text-text-primary cursor-pointer">
        <option value="name">Name</option>
        <option value="level">Level</option>
      </select>

      <span class="text-xs text-text-dim ml-auto">{{ totalVisible }} quests</span>
    </div>

    <!-- Quest list -->
    <div class="overflow-y-auto flex-1 border border-surface-elevated">
      <template v-for="group in groupedQuests" :key="group.label">
        <div
          v-if="group.label"
          class="sticky top-0 z-10 bg-surface-base px-2 py-1 text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-elevated cursor-pointer flex items-center gap-1"
          @click="toggleGroup(group.label)">
          <span class="text-[0.6rem]">{{ collapsedGroups.has(group.label) ? '\u25B6' : '\u25BC' }}</span>
          {{ group.label }} ({{ group.quests.length }})
        </div>

        <template v-if="!group.label || !collapsedGroups.has(group.label)">
          <div
            v-for="row in group.quests"
            :key="row.key"
            class="flex items-center gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-surface-row-hover"
            :class="{
              'bg-surface-card border-l-2 border-l-accent-gold': selectedQuestKey === row.key,
            }"
            @click="$emit('select', row.key)">
            <span class="flex-1 text-text-primary/75 truncate">{{ row.display_name }}</span>

            <!-- Level badge -->
            <span
              v-if="row.level"
              class="text-[0.6rem] px-1 py-0.5 rounded-sm bg-[#2a2a1a] text-text-secondary shrink-0">
              Lv {{ row.level }}
            </span>

            <!-- Area -->
            <span v-if="row.area" class="text-text-dim text-[0.6rem] truncate max-w-20 shrink-0">
              {{ row.area }}
            </span>

            <!-- Category badge -->
            <span
              class="text-[0.65rem] px-1.5 py-0.5 rounded border shrink-0 text-center"
              :class="categoryBadge(row.category)">
              {{ categoryLabel(row.category) }}
            </span>
          </div>
        </template>
      </template>

      <div v-if="totalVisible === 0" class="text-text-dim text-xs italic p-4 text-center">
        No quests match your filters.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { QuestInfo } from '../../types/gameData'
import type { GameStateSkill, GameStateFavor } from '../../types/gameState'
import { getQuestDisplayName, getQuestLevel, getQuestArea, extractNpcDisplayFromFavorPath } from '../../utils/questDisplay'

interface QuestRow {
  key: string
  display_name: string
  level: number | null
  area: string | null
  favor_npc_display: string | null
  first_keyword: string | null
  category: string
  is_work_order: boolean
}

interface QuestGroup {
  label: string
  quests: QuestRow[]
}

const props = defineProps<{
  quests: QuestInfo[]
  skillsByName: Record<string, GameStateSkill>
  favorByNpc: Record<string, GameStateFavor>
  selectedQuestKey: string | null
  questCategories?: Map<string, string>
}>()

defineEmits<{
  select: [questKey: string]
}>()

const filter = ref('')
const groupBy = ref<'category' | 'area' | 'npc' | 'level' | 'keyword' | 'none'>('category')
const sortBy = ref<'name' | 'level'>('name')
const collapsedGroups = ref(new Set<string>())

function toggleGroup(label: string) {
  const next = new Set(collapsedGroups.value)
  if (next.has(label)) next.delete(label)
  else next.add(label)
  collapsedGroups.value = next
}

function categoryLabel(category: string): string {
  switch (category) {
    case 'active': return 'Active'
    case 'work_order': return 'Work Order'
    case 'completed_work_order': return 'Completed'
    default: return category
  }
}

function categoryBadge(category: string): string {
  switch (category) {
    case 'active': return 'bg-accent-gold/20 border-accent-gold/40 text-accent-gold'
    case 'work_order': return 'bg-entity-area/20 border-entity-area/40 text-entity-area'
    case 'completed_work_order': return 'bg-green-400/20 border-green-400/40 text-green-300'
    default: return 'bg-surface-elevated border-border-default text-text-muted'
  }
}

const CATEGORY_ORDER = ['active', 'work_order', 'completed_work_order']

// Build rows
const allRows = computed<QuestRow[]>(() => {
  return props.quests.map(quest => {
    const favorNpc = quest.raw?.FavorNpc
    const category = props.questCategories?.get(quest.internal_name) ?? 'active'
    return {
      key: quest.internal_name,
      display_name: getQuestDisplayName(quest),
      level: getQuestLevel(quest),
      area: getQuestArea(quest),
      favor_npc_display: favorNpc ? extractNpcDisplayFromFavorPath(favorNpc) : null,
      first_keyword: quest.raw?.Keywords?.[0] ?? null,
      category,
      is_work_order: category === 'work_order' || !!quest.raw?.WorkOrderSkill,
    }
  })
})

// Filter
const filteredRows = computed(() => {
  let rows = allRows.value

  const f = filter.value.toLowerCase()
  if (f) {
    rows = rows.filter(r =>
      r.display_name.toLowerCase().includes(f)
      || (r.area?.toLowerCase().includes(f) ?? false)
      || (r.favor_npc_display?.toLowerCase().includes(f) ?? false)
      || (r.first_keyword?.toLowerCase().includes(f) ?? false)
      || r.key.toLowerCase().includes(f)
      || categoryLabel(r.category).toLowerCase().includes(f)
    )
  }

  return rows
})

// Sort
const sortedRows = computed(() => {
  const rows = [...filteredRows.value]
  switch (sortBy.value) {
    case 'name':
      rows.sort((a, b) => a.display_name.localeCompare(b.display_name))
      break
    case 'level':
      rows.sort((a, b) => (a.level ?? 999) - (b.level ?? 999) || a.display_name.localeCompare(b.display_name))
      break
  }
  return rows
})

// Group
function levelBucket(level: number | null): string {
  if (level == null) return 'Unknown Level'
  if (level <= 10) return '1-10'
  if (level <= 20) return '11-20'
  if (level <= 30) return '21-30'
  if (level <= 40) return '31-40'
  if (level <= 50) return '41-50'
  if (level <= 60) return '51-60'
  if (level <= 70) return '61-70'
  if (level <= 80) return '71-80'
  return '80+'
}

const groupedQuests = computed<QuestGroup[]>(() => {
  const rows = sortedRows.value

  if (groupBy.value === 'none') {
    return [{ label: '', quests: rows }]
  }

  if (groupBy.value === 'category') {
    return CATEGORY_ORDER
      .map(cat => ({
        label: categoryLabel(cat),
        quests: rows.filter(r => r.category === cat),
      }))
      .filter(g => g.quests.length > 0)
  }

  // Generic grouping
  const groupMap = new Map<string, QuestRow[]>()
  for (const row of rows) {
    let key: string
    switch (groupBy.value) {
      case 'area':
        key = row.area ?? 'Unknown Area'
        break
      case 'npc':
        key = row.favor_npc_display ?? 'No NPC'
        break
      case 'level':
        key = levelBucket(row.level)
        break
      case 'keyword':
        key = row.first_keyword ?? 'No Keywords'
        break
      default:
        key = ''
    }
    if (!groupMap.has(key)) groupMap.set(key, [])
    groupMap.get(key)!.push(row)
  }

  if (groupBy.value === 'level') {
    return Array.from(groupMap.entries())
      .sort(([a], [b]) => {
        const aNum = parseInt(a) || 999
        const bNum = parseInt(b) || 999
        return aNum - bNum
      })
      .map(([label, quests]) => ({ label, quests }))
  }

  return Array.from(groupMap.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, quests]) => ({ label, quests }))
})

const totalVisible = computed(() => filteredRows.value.length)
</script>
