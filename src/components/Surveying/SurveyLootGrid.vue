<template>
  <div v-if="items.length > 0">
    <div class="text-[0.65rem] uppercase tracking-widest mb-2 font-bold" :class="titleClass">
      {{ title }}
    </div>
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
      <div
        v-for="entry in items"
        :key="entry.item"
        class="bg-black/20 border border-border-default rounded-md p-2.5 transition-all duration-200 hover:bg-black/35 hover:border-border-hover"
      >
        <div class="flex items-center gap-2">
          <div class="flex-1 min-w-0">
            <div class="text-text-primary text-xs font-medium truncate">
              <ItemInline :name="entry.item" />
            </div>
            <div class="flex items-center gap-2 mt-0.5">
              <span class="text-text-secondary text-xs font-semibold">&times;{{ entry.count }}</span>
              <span class="text-entity-item text-[0.65rem]">{{ entry.pct }}%</span>
              <span v-if="entry.rate > 0" class="text-text-muted text-[0.6rem]">{{ entry.rate }}/hr</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { LootSummaryEntry } from "../../stores/surveyStore";
import ItemInline from "../Shared/Item/ItemInline.vue";

defineProps<{
  items: LootSummaryEntry[];
  title: string;
  titleClass?: string;
}>();
</script>
