<template>
  <span
    class="inline-flex items-center text-entity-area font-medium border-b border-dotted border-entity-area/30 cursor-default"
    :title="reference"
  >
    {{ displayName }}
  </span>
</template>

<script setup lang="ts">
// Inline area display — resolves an area key (e.g. "AreaSerbule") to its
// friendly name (e.g. "Serbule") via the backend resolve_area command.
// Falls back to the raw reference if resolution fails or hasn't completed.
import { ref, onMounted, watch } from 'vue'
import { useGameDataStore } from '../../../stores/gameDataStore'

const props = defineProps<{
  reference: string
}>()

const gameData = useGameDataStore()
const friendlyName = ref<string | null>(null)

async function loadName() {
  try {
    const info = await gameData.resolveArea(props.reference)
    friendlyName.value = info?.friendly_name ?? null
  } catch {
    friendlyName.value = null
  }
}

// Display: prefer friendly name, fall back to a cleaned-up version of the
// raw key (strip "Area" prefix and insert spaces before capitals) so even
// unresolved keys are semi-readable.
const displayName = ref(cleanKey(props.reference))

function cleanKey(key: string): string {
  // "AreaKurMountains" → "Kur Mountains"
  const stripped = key.startsWith('Area') ? key.slice(4) : key
  return stripped.replace(/([a-z])([A-Z])/g, '$1 $2')
}

watch(
  () => friendlyName.value,
  (name) => {
    if (name) displayName.value = name
  },
)

watch(
  () => props.reference,
  (newRef) => {
    friendlyName.value = null
    displayName.value = cleanKey(newRef)
    loadName()
  },
)

onMounted(loadName)
</script>
