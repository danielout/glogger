<template>
  <div
    ref="pickerEl"
    class="bg-surface-base border border-surface-elevated rounded shadow-lg p-3 w-72 max-h-64 overflow-y-auto">
    <div class="flex items-center justify-between mb-2">
      <h4 class="text-text-secondary text-xs font-semibold m-0">
        Matching Items
        <span class="text-text-muted font-normal">({{ itemKeys.join(', ') }})</span>
      </h4>
      <button
        class="text-text-muted text-xs cursor-pointer bg-transparent border-none hover:text-text-primary"
        @click="$emit('close')">
        ✕
      </button>
    </div>

    <input
      v-if="matchingItems.length > 10"
      v-model="filterQuery"
      class="input w-full text-xs mb-2"
      placeholder="Filter items..." />

    <div v-if="loading" class="text-text-dim text-xs italic py-2">Loading...</div>

    <div v-else-if="filteredItems.length === 0" class="text-text-dim text-xs italic py-2">
      No matching items found
    </div>

    <ul v-else class="list-none m-0 p-0">
      <li
        v-for="item in filteredItems.slice(0, 50)"
        :key="item.id"
        class="flex items-center gap-2 py-1 border-b border-surface-dark/50 text-xs">
        <ItemInline :name="item.name" />
      </li>
      <li v-if="filteredItems.length > 50" class="text-text-dim text-[0.65rem] italic py-1">
        ...and {{ filteredItems.length - 50 }} more
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import type { ItemInfo } from "../../types/gameData/items";
import ItemInline from "../Shared/Item/ItemInline.vue";

const props = defineProps<{
  itemKeys: string[]
}>();

defineEmits<{
  close: []
}>();

const gameData = useGameDataStore();

const loading = ref(true);
const matchingItems = ref<ItemInfo[]>([]);
const filterQuery = ref("");
const pickerEl = ref<HTMLElement | null>(null);

const filteredItems = computed(() => {
  if (!filterQuery.value.trim()) return matchingItems.value;
  const q = filterQuery.value.toLowerCase();
  return matchingItems.value.filter((item) => item.name.toLowerCase().includes(q));
});

onMounted(async () => {
  try {
    // Fetch items for all keywords and deduplicate
    const seen = new Set<number>();
    const results: ItemInfo[] = [];
    for (const key of props.itemKeys) {
      const items = await gameData.getItemsByKeyword(key);
      for (const item of items) {
        if (!seen.has(item.id)) {
          seen.add(item.id);
          results.push(item);
        }
      }
    }
    results.sort((a, b) => a.name.localeCompare(b.name));
    matchingItems.value = results;
  } catch (e) {
    console.error("[crafting] Failed to load keyword items:", e);
  } finally {
    loading.value = false;
  }

  document.addEventListener("click", handleClickOutside);
});

onBeforeUnmount(() => {
  document.removeEventListener("click", handleClickOutside);
});

function handleClickOutside(e: MouseEvent) {
  if (pickerEl.value && !pickerEl.value.contains(e.target as Node)) {
    // Small delay so the toggle click doesn't immediately re-open
    setTimeout(() => {
      // Emit close - parent handles visibility
    }, 0);
  }
}
</script>
