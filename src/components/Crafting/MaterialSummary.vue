<template>
  <div class="flex flex-col gap-3">
    <div v-if="!bare" class="flex items-center justify-between">
      <h4 class="text-text-secondary text-xs font-semibold uppercase tracking-wide m-0">
        Material Availability
      </h4>
      <div class="flex gap-3 text-[0.65rem] text-text-muted">
        <span>
          <span class="text-green-400">{{ coveredCount }}</span> covered
        </span>
        <span>
          <span class="text-yellow-400">{{ partialCount }}</span> partial
        </span>
        <span>
          <span class="text-accent-red">{{ missingCount }}</span> missing
        </span>
      </div>
    </div>

    <table class="w-full text-xs">
      <thead>
        <tr class="text-text-dim border-b border-border-light">
          <th class="text-left py-1 font-medium">Item</th>
          <th class="text-right py-1 font-medium w-16">Need</th>
          <th class="text-right py-1 font-medium w-16">Inv</th>
          <th class="text-right py-1 font-medium w-16">Storage</th>
          <th class="text-right py-1 font-medium w-20">Shortfall</th>
        </tr>
      </thead>
      <tbody>
        <template v-for="mat in needs" :key="mat.is_dynamic ? `dyn-${mat.item_name}` : mat.item_id">
          <tr
            class="border-b border-surface-dark"
            :class="[rowClass(mat), mat.is_dynamic && mat.dynamic_breakdown?.length ? 'cursor-pointer' : '']"
            @click="mat.is_dynamic && mat.dynamic_breakdown?.length ? toggleExpanded(mat.item_name) : undefined">
            <td class="py-1">
              <template v-if="mat.is_dynamic">
                <span
                  v-if="mat.dynamic_breakdown?.length"
                  class="text-text-muted text-[0.6rem] mr-1 inline-block transition-transform"
                  :class="expandedKeys.has(mat.item_name) ? 'rotate-90' : ''">▸</span>
                <span class="text-accent-gold/60 text-[0.65rem] mr-1">&#9670;</span>
                <span class="text-text-secondary">{{ mat.item_name }}</span>
              </template>
              <ItemInline v-else :reference="mat.item_name" />
            </td>
            <td class="text-right py-1 font-mono text-text-primary">{{ mat.quantity_needed }}</td>
            <td class="text-right py-1 font-mono" :class="mat.inventory_have > 0 ? 'text-green-400' : 'text-text-muted'">
              {{ mat.inventory_have }}
            </td>
            <td class="text-right py-1 font-mono" :class="mat.storage_have > 0 ? 'text-green-400' : 'text-text-muted'">
              {{ mat.storage_have }}
            </td>
            <td class="text-right py-1 font-mono font-semibold" :class="mat.shortfall > 0 ? 'text-accent-red' : 'text-green-400'">
              {{ mat.shortfall > 0 ? mat.shortfall : '✓' }}
            </td>
          </tr>
          <!-- Expandable breakdown for dynamic items -->
          <tr
            v-for="sub in (expandedKeys.has(mat.item_name) ? mat.dynamic_breakdown ?? [] : [])"
            :key="`${mat.item_name}-${sub.item_id}`"
            class="border-b border-surface-dark/50 bg-surface-dark/20">
            <td class="py-0.5 pl-6">
              <ItemInline :reference="String(sub.item_id)" />
            </td>
            <td class="text-right py-0.5"></td>
            <td class="text-right py-0.5 font-mono text-[0.65rem]" :class="sub.inventory_qty > 0 ? 'text-green-400/70' : 'text-text-dim'">
              {{ sub.inventory_qty }}
            </td>
            <td class="text-right py-0.5 font-mono text-[0.65rem]" :class="sub.storage_qty > 0 ? 'text-green-400/70' : 'text-text-dim'">
              {{ sub.storage_qty }}
            </td>
            <td class="text-right py-0.5"></td>
          </tr>
        </template>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { MaterialNeed } from "../../types/crafting";
import ItemInline from "../Shared/Item/ItemInline.vue";

const props = withDefaults(defineProps<{
  needs: MaterialNeed[]
  bare?: boolean
}>(), {
  bare: false,
});

const expandedKeys = ref(new Set<string>());

function toggleExpanded(key: string) {
  const next = new Set(expandedKeys.value);
  if (next.has(key)) {
    next.delete(key);
  } else {
    next.add(key);
  }
  expandedKeys.value = next;
}

const coveredCount = computed(() => props.needs.filter((m) => m.shortfall === 0).length);
const partialCount = computed(() =>
  props.needs.filter((m) => m.shortfall > 0 && (m.inventory_have > 0 || m.storage_have > 0)).length
);
const missingCount = computed(() =>
  props.needs.filter((m) => m.shortfall > 0 && m.inventory_have === 0 && m.storage_have === 0).length
);

function rowClass(mat: MaterialNeed): string {
  if (mat.shortfall === 0) return "opacity-60";
  return "";
}
</script>
