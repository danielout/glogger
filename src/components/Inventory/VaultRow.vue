<template>
  <div class="bg-bg-tertiary border border-border-secondary rounded px-3 py-1.5">
    <div class="flex items-center justify-between gap-2">
      <!-- Left: expand toggle + name + restriction -->
      <div class="flex items-center gap-2 min-w-0">
        <button
          v-if="items.length > 0"
          class="bg-transparent border-none text-xs text-text-muted cursor-pointer p-0 shrink-0"
          @click="expanded = !expanded"
        >
          {{ expanded ? 'v' : '>' }}
        </button>
        <!-- NpcInline self-resolves from the preloaded NPC map -->
        <NpcInline
          v-if="isNpcVault"
          :reference="detail.key"
        />
        <span v-else class="text-sm text-text-primary">{{ displayName }}</span>
        <span v-if="detail.requirement_description" class="text-xs text-text-muted italic truncate">
          ({{ detail.requirement_description }})
        </span>
      </div>

      <!-- Right: favor + capacity -->
      <div class="flex items-center gap-3 shrink-0">
        <!-- Favor tier badge (NPC vaults only) -->
        <span v-if="favorTier" class="text-[10px] px-1.5 py-0.5 rounded bg-bg-primary text-text-secondary">
          {{ favorTier }}
        </span>

        <!-- Capacity display -->
        <div v-if="maxPossibleSlots != null && maxPossibleSlots > 0" class="flex items-center gap-2">
          <div class="w-20 h-1.5 bg-bg-primary rounded-full overflow-hidden">
            <div
              class="h-full rounded-full transition-all"
              :class="capacityColor"
              :style="{ width: capacityPercent + '%' }"
            />
          </div>
          <span class="text-xs text-text-secondary whitespace-nowrap tabular-nums">
            <template v-if="unlockedSlots != null">
              {{ items.length }}/{{ unlockedSlots }}
            </template>
            <template v-else>
              {{ items.length }}/?
            </template>
            <span class="text-text-muted">({{ maxPossibleSlots }} max)</span>
          </span>
        </div>
        <span v-else-if="items.length > 0" class="text-xs text-text-secondary">
          {{ items.length }} items
        </span>
        <span v-else class="text-xs text-text-muted">—</span>
      </div>
    </div>

    <!-- Expanded item list -->
    <div v-if="expanded && items.length > 0" class="mt-2 flex flex-col gap-0.5">
      <div
        v-for="item in sortedItems"
        :key="item.instance_id"
        class="flex items-center justify-between py-0.5 px-1 text-xs hover:bg-bg-secondary rounded"
      >
        <ItemInline :reference="item.item_name" />
        <span v-if="item.stack_size > 1" class="text-text-secondary ml-2">
          x{{ item.stack_size }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { StorageVaultDetail, GameStateStorageItem } from "../../types/gameState";
import ItemInline from "../Shared/Item/ItemInline.vue";
import NpcInline from "../Shared/NPC/NpcInline.vue";

const props = defineProps<{
  detail: StorageVaultDetail;
  items: GameStateStorageItem[];
  maxPossibleSlots: number | null;
  unlockedSlots: number | null;
  favorTier: string | null;
}>();

const expanded = ref(props.items.length > 0 && props.items.length <= 10);

const isNpcVault = computed(() => props.detail.key.startsWith("NPC_"));

const displayName = computed(() =>
  props.detail.npc_friendly_name ?? props.detail.key
);

const sortedItems = computed(() =>
  [...props.items].sort((a, b) => a.item_name.localeCompare(b.item_name))
);

const capacityPercent = computed(() => {
  const cap = props.unlockedSlots ?? props.maxPossibleSlots;
  if (!cap) return 0;
  return Math.min(100, (props.items.length / cap) * 100);
});

const capacityColor = computed(() => {
  const pct = capacityPercent.value;
  if (pct >= 90) return "bg-red-500";
  if (pct >= 70) return "bg-yellow-500";
  return "bg-accent-primary";
});
</script>
