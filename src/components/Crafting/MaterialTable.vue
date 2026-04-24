<template>
  <table class="w-full text-xs">
    <thead>
      <tr class="text-text-dim border-b border-border-light">
        <th class="text-left py-1 font-medium">Item</th>
        <th class="text-right py-1 font-medium w-16">Need</th>
        <th v-if="pricingMode" class="text-right py-1 font-medium w-20">They Give</th>
        <th class="text-right py-1 font-medium w-16">Inv</th>
        <th class="text-right py-1 font-medium w-16">Storage</th>
        <th class="text-right py-1 font-medium w-20">Shortfall</th>
      </tr>
    </thead>
    <tbody>
      <template v-for="mat in items" :key="mat.is_dynamic ? `dyn-${mat.item_name}` : mat.item_id">
        <tr
          class="border-b border-surface-dark"
          :class="[rowClass(mat), mat.is_dynamic && mat.dynamic_breakdown?.length ? 'cursor-pointer' : '']"
          @click="mat.is_dynamic && mat.dynamic_breakdown?.length ? $emit('toggle-expanded', mat.item_name) : undefined">
          <td class="py-1">
            <template v-if="mat.is_dynamic">
              <span
                v-if="mat.dynamic_breakdown?.length"
                class="text-text-muted text-[0.6rem] mr-1 inline-block transition-transform"
                :class="expandedKeys.has(mat.item_name) ? 'rotate-90' : ''">&#9656;</span>
              <span class="text-accent-gold/60 text-[0.65rem] mr-1">&#9670;</span>
              <span class="text-text-secondary">{{ mat.item_name }}</span>
            </template>
            <ItemInline v-else :reference="mat.item_name" />
          </td>
          <td class="text-right py-1 font-mono text-text-primary">{{ mat.quantity_needed }}</td>
          <td v-if="pricingMode" class="text-right py-1">
            <input
              :value="getCustomerProvides(mat)"
              type="number"
              min="0"
              :max="Math.ceil(mat.quantity_needed)"
              class="input w-14 text-xs text-right py-0"
              @change="onCustomerProvidesChange(getMaterialKey(mat), mat.quantity_needed, $event)" />
          </td>
          <td class="text-right py-1 font-mono" :class="mat.inventory_have > 0 ? 'text-green-400' : 'text-text-muted'">
            {{ mat.inventory_have }}
          </td>
          <td class="text-right py-1 font-mono" :class="mat.storage_have > 0 ? 'text-green-400' : 'text-text-muted'">
            {{ mat.storage_have }}
          </td>
          <td class="text-right py-1 font-mono font-semibold" :class="mat.shortfall > 0 ? 'text-accent-red' : 'text-green-400'">
            {{ mat.shortfall > 0 ? mat.shortfall : '&#10003;' }}
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
</template>

<script setup lang="ts">
import type { MaterialNeed } from "../../types/crafting";
import ItemInline from "../Shared/Item/ItemInline.vue";

const props = defineProps<{
  items: MaterialNeed[]
  pricingMode?: boolean
  customerProvides?: Record<string, number>
  expandedKeys: Set<string>
}>();

const emit = defineEmits<{
  'toggle-expanded': [key: string]
  'update-customer-provides': [key: string, quantity: number]
}>();

function getCustomerProvides(mat: MaterialNeed): number {
  if (!props.customerProvides) return 0;
  const key = mat.is_dynamic ? `kw:${mat.item_name}` : String(mat.item_id);
  return props.customerProvides[key] ?? 0;
}

function getMaterialKey(mat: MaterialNeed): string {
  return mat.is_dynamic ? `kw:${mat.item_name}` : String(mat.item_id);
}

function onCustomerProvidesChange(key: string, maxQty: number, event: Event) {
  const target = event.target as HTMLInputElement;
  const qty = Math.max(0, Math.min(Math.ceil(maxQty), parseFloat(target.value) || 0));
  emit('update-customer-provides', key, qty);
}

function rowClass(mat: MaterialNeed): string {
  if (mat.shortfall === 0) return "opacity-60";
  return "";
}
</script>
