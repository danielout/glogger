<template>
  <div class="flex flex-col gap-2 h-full min-h-0">
    <!-- Current moon phase -->
    <div v-if="phase" class="flex items-center gap-2 text-sm">
      <span class="text-lg">{{ phase.emoji }}</span>
      <span class="text-text-secondary">{{ phase.label }}</span>
      <span v-if="nextPhase" class="text-text-dim text-xs ml-auto">
        {{ nextPhase.label }} in {{ nextPhase.days }}d
      </span>
    </div>

    <!-- Filter controls -->
    <div class="flex items-center gap-2">
      <div class="flex rounded border border-border overflow-hidden text-xs">
        <button
          v-for="opt in filterOptions"
          :key="opt.value"
          class="px-2.5 py-1 transition-colors"
          :class="
            filterMode === opt.value
              ? 'bg-accent-blue text-white'
              : 'bg-surface-2 text-text-secondary hover:bg-surface-3'
          "
          @click="filterMode = opt.value">
          {{ opt.label }}
        </button>
      </div>
      <input
        v-model="search"
        type="text"
        placeholder="Search..."
        class="flex-1 px-2 py-1 rounded bg-surface-2 border border-border text-sm text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-blue" />
    </div>

    <!-- Mushroom table -->
    <div class="flex-1 overflow-y-auto min-h-0">
      <table class="w-full text-sm">
        <thead>
          <tr class="text-text-dim text-xs border-b border-border">
            <th class="text-left py-1 pr-2 cursor-pointer hover:text-text-secondary" @click="toggleSort('name')">
              Mushroom {{ sortIndicator('name') }}
            </th>
            <th class="text-center py-1 px-2 cursor-pointer hover:text-text-secondary" @click="toggleSort('level')">
              Lvl {{ sortIndicator('level') }}
            </th>
            <th class="text-center py-1 px-2 cursor-pointer hover:text-text-secondary" @click="toggleSort('growTime')">
              Time {{ sortIndicator('growTime') }}
            </th>
            <th class="text-left py-1 px-2">Substrate</th>
            <th class="text-left py-1 px-2">Moon</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="m in filteredMushrooms"
            :key="m.name"
            class="border-b border-border/50 hover:bg-surface-2"
            :class="rowClass(m)">
            <td class="py-1.5 pr-2">
              <ItemInline :reference="m.name" />
            </td>
            <td class="text-center py-1.5 px-2 text-text-dim font-mono">{{ m.level }}</td>
            <td class="text-center py-1.5 px-2 text-text-dim font-mono">{{ m.growTime }}h</td>
            <td class="py-1.5 px-2">
              <span class="text-accent-gold">{{ m.optimalSubstrate }}</span>
              <span class="text-text-dim"> / {{ m.adequateSubstrate }}</span>
            </td>
            <td class="py-1.5 px-2">
              <span
                v-for="p in m.robustPhases"
                :key="p"
                class="mr-1"
                :class="phase?.name === p ? 'text-green-400' : 'text-text-dim'"
                :title="phaseLabel(p) + ' (robust)'">
                {{ phaseEmoji(p) }}
              </span>
              <span class="text-text-dim mx-0.5">|</span>
              <span
                v-for="p in m.poorPhases"
                :key="p"
                class="mr-1"
                :class="phase?.name === p ? 'text-red-400' : 'text-text-dim'"
                :title="phaseLabel(p) + ' (poor)'">
                {{ phaseEmoji(p) }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="filteredMushrooms.length === 0" class="text-xs text-text-dim italic">
      No mushrooms match your filter.
    </div>

    <div class="text-xs text-text-dim flex items-center gap-1">
      <span class="text-green-400">&#9679;</span> extra yield
      <span class="text-red-400 ml-2">&#9679;</span> reduced yield
      <span class="ml-2">Substrate: <span class="text-accent-gold">optimal</span> / adequate</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMoonPhase, ALL_PHASES, type MoonPhaseName } from '../../../composables/useMoonPhase'
import ItemInline from '../../Shared/Item/ItemInline.vue'

const { phase, daysUntil } = useMoonPhase()

const search = ref('')
const filterMode = ref<'all' | 'robust' | 'poor'>('robust')
const sortKey = ref<'name' | 'level' | 'growTime'>('level')

const filterOptions = [
  { value: 'all' as const, label: 'All' },
  { value: 'robust' as const, label: 'Extra Yield' },
  { value: 'poor' as const, label: 'Reduced Yield' },
]
const sortAsc = ref(true)

interface MushroomInfo {
  name: string
  level: number
  growTime: number
  adequateSubstrate: string
  optimalSubstrate: string
  robustPhases: MoonPhaseName[]
  poorPhases: MoonPhaseName[]
}

const MUSHROOMS: MushroomInfo[] = [
  { name: 'Parasol Mushroom', level: 0, growTime: 2, adequateSubstrate: 'Dirt', optimalSubstrate: 'Organs', robustPhases: ['FullMoon', 'WaningCrescentMoon'], poorPhases: ['NewMoon', 'WaxingGibbousMoon'] },
  { name: 'Mycena Mushroom', level: 5, growTime: 3, adequateSubstrate: 'Bone', optimalSubstrate: 'Limbs', robustPhases: ['WaxingCrescentMoon', 'QuarterMoon'], poorPhases: ['WaningGibbousMoon', 'LastQuarterMoon'] },
  { name: 'Boletus Mushroom', level: 10, growTime: 4, adequateSubstrate: 'Meat', optimalSubstrate: 'Exotic', robustPhases: ['NewMoon', 'WaningGibbousMoon'], poorPhases: ['WaxingCrescentMoon', 'FullMoon'] },
  { name: 'Goblin Puffball', level: 12, growTime: 5, adequateSubstrate: 'Dirt', optimalSubstrate: 'Exotic', robustPhases: ['NewMoon', 'WaxingGibbousMoon'], poorPhases: ['FullMoon', 'WaningCrescentMoon'] },
  { name: 'Field Mushroom', level: 15, growTime: 5, adequateSubstrate: 'Bone', optimalSubstrate: 'Organs', robustPhases: ['WaxingGibbousMoon', 'LastQuarterMoon'], poorPhases: ['QuarterMoon', 'WaningCrescentMoon'] },
  { name: 'Blusher Mushroom', level: 20, growTime: 6, adequateSubstrate: 'Meat', optimalSubstrate: 'Exotic', robustPhases: ['NewMoon', 'WaningGibbousMoon'], poorPhases: ['WaxingCrescentMoon', 'FullMoon'] },
  { name: 'Milk Cap Mushroom', level: 25, growTime: 7, adequateSubstrate: 'Dirt', optimalSubstrate: 'Organs', robustPhases: ['FullMoon', 'WaningCrescentMoon'], poorPhases: ['NewMoon', 'WaxingGibbousMoon'] },
  { name: 'Blood Mushroom', level: 30, growTime: 8, adequateSubstrate: 'Dirt', optimalSubstrate: 'Limbs', robustPhases: ['WaxingCrescentMoon', 'LastQuarterMoon'], poorPhases: ['QuarterMoon', 'WaningGibbousMoon'] },
  { name: 'Blastcap Mushroom', level: 33, growTime: 8, adequateSubstrate: 'Meat', optimalSubstrate: 'Organs', robustPhases: ['FullMoon', 'WaningGibbousMoon'], poorPhases: ['NewMoon', 'WaxingCrescentMoon'] },
  { name: 'Coral Mushroom', level: 40, growTime: 9, adequateSubstrate: 'Meat', optimalSubstrate: 'Limbs', robustPhases: ['QuarterMoon', 'WaxingGibbousMoon'], poorPhases: ['LastQuarterMoon', 'WaningCrescentMoon'] },
  { name: 'Iocaine Mushroom', level: 40, growTime: 10, adequateSubstrate: 'Bone', optimalSubstrate: 'Limbs', robustPhases: ['WaxingCrescentMoon', 'QuarterMoon'], poorPhases: ['WaningGibbousMoon', 'LastQuarterMoon'] },
  { name: 'Groxmax Mushroom', level: 47, growTime: 11, adequateSubstrate: 'Bone', optimalSubstrate: 'Organs', robustPhases: ['WaxingGibbousMoon', 'LastQuarterMoon'], poorPhases: ['QuarterMoon', 'WaningCrescentMoon'] },
  { name: 'Porcini Mushroom', level: 55, growTime: 12, adequateSubstrate: 'Meat', optimalSubstrate: 'Exotic', robustPhases: ['FullMoon', 'WaningGibbousMoon'], poorPhases: ['NewMoon', 'WaxingCrescentMoon'] },
  { name: 'False Agaric', level: 57, growTime: 12, adequateSubstrate: 'Bone', optimalSubstrate: 'Limbs', robustPhases: ['WaningCrescentMoon', 'LastQuarterMoon'], poorPhases: ['QuarterMoon', 'WaxingGibbousMoon'] },
  { name: 'Black Foot Morel', level: 64, growTime: 13, adequateSubstrate: 'Dirt', optimalSubstrate: 'Exotic', robustPhases: ['NewMoon', 'WaningCrescentMoon'], poorPhases: ['FullMoon', 'WaxingGibbousMoon'] },
  { name: "Pixie's Parasol", level: 70, growTime: 14, adequateSubstrate: 'Meat', optimalSubstrate: 'Organs', robustPhases: ['QuarterMoon', 'WaxingGibbousMoon'], poorPhases: ['LastQuarterMoon', 'WaningCrescentMoon'] },
  { name: 'Fly Amanita', level: 77, growTime: 15, adequateSubstrate: 'Bone', optimalSubstrate: 'Organs', robustPhases: ['WaxingCrescentMoon', 'FullMoon'], poorPhases: ['NewMoon', 'WaningGibbousMoon'] },
  { name: "Wizard's Mushroom", level: 75, growTime: 16, adequateSubstrate: 'Bone', optimalSubstrate: 'Organs', robustPhases: ['WaxingCrescentMoon', 'QuarterMoon'], poorPhases: ['WaxingGibbousMoon', 'LastQuarterMoon'] },
  { name: 'Charged Mycelium', level: 80, growTime: 16, adequateSubstrate: 'Meat', optimalSubstrate: 'Exotic', robustPhases: ['NewMoon', 'WaxingGibbousMoon'], poorPhases: ['WaningCrescentMoon', 'FullMoon'] },
  { name: 'Granamurch Mushroom', level: 85, growTime: 17, adequateSubstrate: 'Dirt', optimalSubstrate: 'Limbs', robustPhases: ['FullMoon', 'NewMoon'], poorPhases: ['WaningGibbousMoon', 'LastQuarterMoon', 'WaningCrescentMoon', 'QuarterMoon'] },
  { name: 'Ghostshroom', level: 90, growTime: 18, adequateSubstrate: 'Exotic', optimalSubstrate: 'Meat', robustPhases: ['FullMoon', 'NewMoon'], poorPhases: ['LastQuarterMoon', 'WaxingCrescentMoon', 'QuarterMoon', 'WaxingGibbousMoon'] },
  { name: 'Mortaferus Mushroom', level: 95, growTime: 19, adequateSubstrate: 'Unknown', optimalSubstrate: 'Organs', robustPhases: ['WaningGibbousMoon', 'LastQuarterMoon'], poorPhases: ['WaxingCrescentMoon'] },
]

const PHASE_EMOJI: Record<MoonPhaseName, string> = Object.fromEntries(
  ALL_PHASES.map(p => [p.name, p.emoji])
) as Record<MoonPhaseName, string>

const PHASE_LABELS: Record<MoonPhaseName, string> = Object.fromEntries(
  ALL_PHASES.map(p => [p.name, p.label])
) as Record<MoonPhaseName, string>

function phaseEmoji(name: MoonPhaseName): string {
  return PHASE_EMOJI[name] ?? '?'
}

function phaseLabel(name: MoonPhaseName): string {
  return PHASE_LABELS[name] ?? name
}

const nextPhase = computed(() => daysUntil.value.length > 0 ? daysUntil.value[0] : null)

function isRobustNow(m: MushroomInfo): boolean {
  return !!phase.value && m.robustPhases.includes(phase.value.name)
}

function isPoorNow(m: MushroomInfo): boolean {
  return !!phase.value && m.poorPhases.includes(phase.value.name)
}

function rowClass(m: MushroomInfo): string {
  if (isRobustNow(m)) return 'bg-green-500/5'
  if (isPoorNow(m)) return 'bg-red-500/5'
  return ''
}

function toggleSort(key: 'name' | 'level' | 'growTime') {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortKey.value = key
    sortAsc.value = true
  }
}

function sortIndicator(key: string): string {
  if (sortKey.value !== key) return ''
  return sortAsc.value ? '\u25B2' : '\u25BC'
}

const filteredMushrooms = computed(() => {
  const q = search.value.toLowerCase().trim()
  let result = MUSHROOMS.filter(m => {
    if (q && !m.name.toLowerCase().includes(q)) return false
    if (filterMode.value === 'robust' && !isRobustNow(m)) return false
    if (filterMode.value === 'poor' && !isPoorNow(m)) return false
    return true
  })

  result.sort((a, b) => {
    let cmp = 0
    if (sortKey.value === 'name') cmp = a.name.localeCompare(b.name)
    else if (sortKey.value === 'level') cmp = a.level - b.level
    else if (sortKey.value === 'growTime') cmp = a.growTime - b.growTime
    return sortAsc.value ? cmp : -cmp
  })

  return result
})
</script>
