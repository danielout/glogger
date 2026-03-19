<template>
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <h4 class="text-text-secondary text-xs font-semibold uppercase tracking-wide m-0">
        Pickup List
      </h4>
      <span class="text-text-muted text-[0.65rem]">{{ vaults.length }} vault{{ vaults.length !== 1 ? 's' : '' }} to visit</span>
    </div>

    <div v-if="vaults.length === 0" class="text-text-dim text-xs italic">
      No items to pick up from storage vaults.
    </div>

    <div v-for="vault in vaults" :key="vault.name" class="bg-surface-base border border-surface-elevated rounded p-3">
      <h5 class="text-accent-gold text-xs font-semibold m-0 mb-2">{{ vault.name }}</h5>
      <ul class="list-none m-0 p-0 space-y-1">
        <li
          v-for="item in vault.items"
          :key="item.item_id"
          class="flex items-center gap-2 text-xs"
          :class="{ 'opacity-40 line-through': item.checked }"
          @click="item.checked = !item.checked">
          <input
            type="checkbox"
            :checked="item.checked"
            class="accent-accent-gold cursor-pointer"
            @click.stop="item.checked = !item.checked" />
          <ItemInline :name="item.item_name" />
          <span class="text-text-primary font-mono ml-auto">
            ×{{ item.pickup_quantity }}
          </span>
          <span v-if="item.pickup_quantity < item.available" class="text-text-muted text-[0.65rem]">
            ({{ item.available }} avail)
          </span>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive } from "vue";
import type { MaterialNeed } from "../../types/crafting";
import ItemInline from "../Shared/Item/ItemInline.vue";

interface PickupItem {
  item_id: number
  item_name: string
  pickup_quantity: number
  available: number
  checked: boolean
}

interface VaultPickup {
  name: string
  items: PickupItem[]
}

const props = defineProps<{
  needs: MaterialNeed[]
}>();

const vaults = computed(() => {
  const vaultMap = new Map<string, PickupItem[]>();

  for (const mat of props.needs) {
    // Only include items that exist in storage and are actually needed
    if (mat.storage_have === 0) continue;

    for (const vs of mat.vault_breakdown) {
      if (vs.quantity === 0) continue;

      // How much to pick up from this vault: min of what's there and what we still need
      const pickupQty = Math.min(vs.quantity, mat.quantity_needed);

      if (!vaultMap.has(vs.vault_name)) {
        vaultMap.set(vs.vault_name, []);
      }
      vaultMap.get(vs.vault_name)!.push(reactive({
        item_id: mat.item_id,
        item_name: mat.item_name,
        pickup_quantity: pickupQty,
        available: vs.quantity,
        checked: false,
      }));
    }
  }

  const result: VaultPickup[] = [];
  for (const [name, items] of vaultMap) {
    items.sort((a, b) => a.item_name.localeCompare(b.item_name));
    result.push({ name, items });
  }
  result.sort((a, b) => a.name.localeCompare(b.name));
  return result;
});
</script>
