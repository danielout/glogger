<template>
  <div class="flex gap-2 items-start mb-2">
    <img
      v-if="iconSrc"
      :src="iconSrc"
      :alt="item.name"
      class="w-8 h-8 object-contain bg-black/30 border border-border-light rounded shrink-0" />
    <div class="flex-1">
      <div class="font-bold text-entity-item text-sm mb-0.5">{{ item.name }}</div>
      <div v-if="item.value" class="text-accent-gold text-xs">
        Value: {{ item.value }}g
      </div>
    </div>
  </div>

  <div v-if="item.description" class="text-text-secondary text-xs leading-relaxed mb-2 italic">
    {{ item.description }}
  </div>

  <div v-if="item.keywords?.length" class="flex flex-wrap gap-1 mb-2">
    <span
      v-for="keyword in item.keywords"
      :key="keyword"
      class="bg-entity-item/10 text-entity-item px-1.5 py-0.5 rounded-sm text-[0.65rem] uppercase tracking-wide"
    >
      {{ keyword }}
    </span>
  </div>

  <div v-if="resolvedEffects.length" class="mb-2">
    <div
      v-for="(effect, i) in resolvedEffects"
      :key="i"
      class="text-accent-green text-xs leading-relaxed pl-2 relative before:content-['•'] before:absolute before:left-0"
    >
      {{ effect.formatted }}
    </div>
  </div>

  <div v-if="item.max_stack_size || ownedCount > 0" class="text-text-muted text-[0.7rem] mt-2 pt-2 border-t border-[#2a2a3e] flex justify-between">
    <span v-if="item.max_stack_size">Max Stack: {{ item.max_stack_size }}</span>
    <span v-if="ownedCount > 0" class="text-accent-gold">Owned: {{ ownedCount.toLocaleString() }}</span>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useCharacterStore } from "../../../stores/characterStore";
import type { ItemInfo } from "../../../types/gameData";

interface ResolvedEffect {
  label: string
  value: string
  display_type: string
  formatted: string
  icon_id: number | null
}

const props = defineProps<{
  item: ItemInfo;
  iconSrc: string | null;
}>();

const characterStore = useCharacterStore();
const ownedCount = computed(() => characterStore.ownedItemCounts[props.item.name] ?? 0);

const resolvedEffects = ref<ResolvedEffect[]>([]);

async function resolveEffects() {
  if (!props.item.effect_descs?.length) {
    resolvedEffects.value = [];
    return;
  }
  try {
    resolvedEffects.value = await invoke<ResolvedEffect[]>('resolve_effect_descs', {
      descs: props.item.effect_descs,
    });
  } catch {
    // Fallback to raw strings
    resolvedEffects.value = props.item.effect_descs.map(d => ({
      label: d, value: '', display_type: '', formatted: d, icon_id: null,
    }));
  }
}

watch(() => props.item, resolveEffects, { immediate: true });
</script>
