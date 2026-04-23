<template>
  <div class="flex flex-col gap-3 h-full min-h-0 px-1">
    <div class="flex items-center gap-2">
      <h3 class="text-sm font-semibold text-text-primary">Search All Mods</h3>
      <input
        v-model="query"
        type="text"
        placeholder="Search across all equipped mods..."
        class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-xs text-text-primary flex-1" />
    </div>

    <div v-if="!query" class="text-xs text-text-muted py-4 text-center">
      Type to search across all mods in your build (e.g., "Thunderstrike", "damage", "armor").
    </div>

    <div v-else-if="groupedResults.length === 0" class="text-xs text-text-dim py-4 text-center">
      No mods match "{{ query }}"
    </div>

    <div v-else class="flex-1 overflow-y-auto space-y-3">
      <div v-for="group in groupedResults" :key="group.slotId">
        <h4 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-1">
          {{ group.slotLabel }} ({{ group.mods.length }})
        </h4>
        <div class="space-y-1">
          <div
            v-for="mod in group.mods"
            :key="mod.id"
            class="flex items-start gap-2 px-2 py-1.5 rounded text-sm bg-surface-elevated border border-border-default">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="font-medium text-text-primary">{{ mod.power_name }}</span>
                <span v-if="mod.is_augment" class="text-[10px] font-semibold text-mod-augment uppercase">AUG</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="text-[10px] text-text-dim text-center pt-2">
        {{ totalMatches }} mod{{ totalMatches !== 1 ? 's' : '' }} across {{ groupedResults.length }} slot{{ groupedResults.length !== 1 ? 's' : '' }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS } from '../../../types/buildPlanner'

const store = useBuildPlannerStore()
const query = ref('')

interface ModGroup {
  slotId: string
  slotLabel: string
  mods: typeof store.presetMods
}

const groupedResults = computed((): ModGroup[] => {
  if (!query.value.trim()) return []
  const q = query.value.toLowerCase()

  const groups: ModGroup[] = []
  for (const slot of EQUIPMENT_SLOTS) {
    const slotMods = store.presetMods.filter(m =>
      m.equip_slot === slot.id && m.power_name.toLowerCase().includes(q)
    )
    if (slotMods.length > 0) {
      groups.push({
        slotId: slot.id,
        slotLabel: slot.label,
        mods: slotMods,
      })
    }
  }
  return groups
})

const totalMatches = computed(() =>
  groupedResults.value.reduce((sum, g) => sum + g.mods.length, 0)
)
</script>
