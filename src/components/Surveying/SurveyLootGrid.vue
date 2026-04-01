<template>
  <div v-if="items.length > 0">
    <div class="text-[0.65rem] uppercase tracking-widest mb-2 font-bold" :class="titleClass">
      {{ title }}
    </div>
    <div class="flex flex-wrap gap-2">
      <div
        v-for="entry in items"
        :key="entry.item"
        class="relative"
      >
        <ItemMinicard :reference="entry.item" />
        <div class="flex items-center gap-2 px-2 pb-1 text-[0.6rem]">
          <span class="text-text-secondary font-semibold">&times;{{ entry.count }}</span>
          <span class="text-entity-item">{{ entry.pct }}%</span>
          <span v-if="entry.rate > 0" class="text-text-muted">{{ entry.rate }}/hr</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { LootSummaryEntry } from "../../stores/surveyStore";
import ItemMinicard from "../Shared/Item/ItemMinicard.vue";

defineProps<{
  items: LootSummaryEntry[];
  title: string;
  titleClass?: string;
}>();
</script>
