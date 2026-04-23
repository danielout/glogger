<template>
  <div v-if="services.length" class="flex flex-col gap-3">
    <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Services
    </div>

    <!-- Store (Vendor) -->
    <div v-for="store in storeServices" :key="'store'" class="flex flex-col gap-1.5">
      <div class="text-xs font-semibold text-text-primary flex items-center gap-1.5">
        <span class="text-accent-gold">$</span> Vendor
        <span class="text-text-dim text-[10px] font-normal">
          ({{ tierDisplayName(store.favor) }}+)
        </span>
      </div>
      <div v-if="store.capIncreases.length" class="flex flex-col gap-0.5">
        <div
          v-for="cap in store.capIncreases"
          :key="cap.tier"
          class="flex items-center gap-2 px-2 py-0.5 text-xs rounded"
          :class="capRowClasses(cap.tier)">
          <span class="min-w-24 shrink-0" :class="favorColor(cap.tier)">
            {{ tierDisplayName(cap.tier) }}
          </span>
          <span class="text-accent-gold font-bold min-w-16 text-right">
            {{ cap.maxGold.toLocaleString() }}
          </span>
          <span class="text-text-dim text-[10px]">councils</span>
          <span class="text-text-secondary text-[10px] ml-auto">
            {{ cap.itemTypes.join(', ') }}
          </span>
        </div>
      </div>
      <div v-else class="text-xs text-text-dim italic px-2">No cap increase data</div>
    </div>

    <!-- Training -->
    <div v-for="(training, i) in trainingServices" :key="'train-' + i" class="flex flex-col gap-1.5">
      <div class="text-xs font-semibold text-text-primary flex items-center gap-1.5">
        <span class="text-entity-skill">&#x2726;</span> Training
        <span class="text-text-dim text-[10px] font-normal">
          ({{ tierDisplayName(training.favor) }}+)
        </span>
      </div>
      <div class="flex flex-wrap gap-1 px-2">
        <SkillInline
          v-for="skill in training.skills"
          :key="skill"
          :reference="skill" />
      </div>
      <div v-if="training.unlocks.length" class="text-[10px] text-text-dim px-2">
        Additional training unlocks at:
        <span v-for="(tier, j) in training.unlocks" :key="tier">
          <span :class="favorColor(tier)">{{ tierDisplayName(tier) }}</span><span v-if="j < training.unlocks.length - 1">, </span>
        </span>
      </div>
    </div>

    <!-- Barter -->
    <div v-for="(barter, i) in barterServices" :key="'barter-' + i" class="flex flex-col gap-1">
      <div class="text-xs font-semibold text-text-primary flex items-center gap-1.5">
        <span class="text-yellow-500">&#x21C4;</span> Barter
        <span class="text-text-dim text-[10px] font-normal">
          ({{ tierDisplayName(barter.favor) }}+)
        </span>
      </div>
      <div v-if="barter.additionalUnlocks.length" class="text-[10px] text-text-dim px-2">
        Additional barter unlocks at:
        <span v-for="(tier, j) in barter.additionalUnlocks" :key="tier">
          <span :class="favorColor(tier)">{{ tierDisplayName(tier) }}</span><span v-if="j < barter.additionalUnlocks.length - 1">, </span>
        </span>
      </div>
    </div>

    <!-- Consignment -->
    <div v-for="(consign, i) in consignmentServices" :key="'consign-' + i" class="flex flex-col gap-1">
      <div class="text-xs font-semibold text-text-primary flex items-center gap-1.5">
        <span class="text-blue-400">&#x2692;</span> Consignment
        <span class="text-text-dim text-[10px] font-normal">
          ({{ tierDisplayName(consign.favor) }}+)
        </span>
      </div>
      <div class="text-xs text-text-secondary px-2">
        Accepts: {{ consign.itemTypes.join(', ') }}
      </div>
      <div v-if="consign.unlocks.length" class="text-[10px] text-text-dim px-2">
        Expanded at:
        <span v-for="(tier, j) in consign.unlocks" :key="tier">
          <span :class="favorColor(tier)">{{ tierDisplayName(tier) }}</span><span v-if="j < consign.unlocks.length - 1">, </span>
        </span>
      </div>
    </div>

    <!-- Storage -->
    <div v-for="(storage, i) in storageServices" :key="'storage-' + i" class="flex flex-col gap-1">
      <div class="text-xs font-semibold text-text-primary flex items-center gap-1.5">
        <span class="text-cyan-400">&#x25A3;</span> Storage
        <span class="text-text-dim text-[10px] font-normal">
          ({{ tierDisplayName(storage.favor) }}+)
        </span>
      </div>
      <div v-if="storage.spaceIncreases.length" class="text-[10px] text-text-dim px-2">
        More space at:
        <span v-for="(tier, j) in storage.spaceIncreases" :key="tier">
          <span :class="favorColor(tier)">{{ tierDisplayName(tier) }}</span><span v-if="j < storage.spaceIncreases.length - 1">, </span>
        </span>
      </div>
    </div>

    <!-- Generic / Other -->
    <div v-for="(svc, i) in genericServices" :key="'generic-' + i" class="flex flex-col gap-1">
      <div class="text-xs font-semibold text-text-primary flex items-center gap-1.5">
        <span class="text-text-muted">&#x2022;</span> {{ svc.type }}
        <span class="text-text-dim text-[10px] font-normal">
          ({{ tierDisplayName(svc.favor) }}+)
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { NpcInfo } from '../../../types/gameData'
import type {
  StoreService,
  TrainingService,
  BarterService,
  ConsignmentService,
  StorageService,
  GenericService,
} from '../../../types/npcServices'
import { getServices } from '../../../composables/useNpcServices'
import { favorColor, tierDisplayName, isTierAtOrAbove } from '../../../composables/useFavorTiers'
import SkillInline from '../../Shared/Skill/SkillInline.vue'

const props = defineProps<{
  npc: NpcInfo
  playerTier: string
}>()

const services = computed(() => getServices(props.npc))

const storeServices = computed(() =>
  services.value.filter((s): s is StoreService => s.type === 'Store')
)
const trainingServices = computed(() =>
  services.value.filter((s): s is TrainingService => s.type === 'Training')
)
const barterServices = computed(() =>
  services.value.filter((s): s is BarterService => s.type === 'Barter')
)
const consignmentServices = computed(() =>
  services.value.filter((s): s is ConsignmentService => s.type === 'Consignment')
)
const storageServices = computed(() =>
  services.value.filter((s): s is StorageService => s.type === 'Storage')
)
const genericServices = computed(() =>
  services.value.filter((s): s is GenericService =>
    !['Store', 'Training', 'Barter', 'Consignment', 'Storage'].includes(s.type)
  )
)

function capRowClasses(tier: string): string {
  if (isTierAtOrAbove(props.playerTier, tier)) {
    return 'bg-[#151515]'
  }
  return 'bg-[#151515] opacity-40'
}
</script>
