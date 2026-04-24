<template>
  <EntityTooltipWrapper
    border-class="border-entity-area/50"
    :disabled="!resolved"
    entity-type="area"
    :entity-reference="reference"
    :entity-label="displayName"
    @hover="() => {}"
  >
    <span
      class="inline-flex items-center text-entity-area font-medium cursor-pointer hover:underline"
      :class="bordered ? 'bg-entity-area/5 border border-entity-area/20 rounded px-1 py-0.5' : 'border-b border-dotted border-entity-area/30'"
      @click="handleClick"
    >
      {{ displayName }}
    </span>
    <template #tooltip>
      <AreaTooltip
        v-if="resolved"
        :area-key="resolvedKey"
        :area-name="displayName"
        :short-name="shortName"
      />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { useEntityNavigation } from '../../../composables/useEntityNavigation'
import EntityTooltipWrapper from '../EntityTooltipWrapper.vue'
import AreaTooltip from './AreaTooltip.vue'

const props = withDefaults(defineProps<{
  reference: string
  bordered?: boolean
}>(), {
  bordered: false,
})

const gameData = useGameDataStore()
const { navigateToEntity } = useEntityNavigation()

const resolved = ref(false)
const resolvedKey = ref(props.reference)
const friendlyName = ref<string | null>(null)
const shortName = ref<string | null>(null)

const displayName = ref(cleanKey(props.reference))

function cleanKey(key: string): string {
  const stripped = key.startsWith('Area') ? key.slice(4) : key
  return stripped.replace(/([a-z])([A-Z])/g, '$1 $2')
}

async function loadData() {
  try {
    const info = await gameData.resolveArea(props.reference)
    if (!info) return
    friendlyName.value = info.friendly_name
    shortName.value = info.short_friendly_name
    if (info.friendly_name) {
      displayName.value = info.friendly_name
    }
    resolved.value = true
  } catch {
    resolved.value = false
  }
}

watch(
  () => props.reference,
  (newRef) => {
    resolved.value = false
    friendlyName.value = null
    shortName.value = null
    resolvedKey.value = newRef
    displayName.value = cleanKey(newRef)
    loadData()
  },
)

onMounted(loadData)

function handleClick() {
  navigateToEntity({ type: 'area', id: props.reference })
}
</script>
