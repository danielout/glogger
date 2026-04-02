<template>
  <PaneLayout screen-key="inventory">
    <div class="flex flex-col gap-4 h-full">
      <div class="flex-1 min-h-0">
        <LiveInventoryTab v-if="activeTab === 'live'" />
        <InventoryView v-else-if="activeTab === 'storage'" />
        <VaultDatabaseTab v-else-if="activeTab === 'vaults'" />
      </div>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from '../Shared/PaneLayout.vue'
import LiveInventoryTab from './LiveInventoryTab.vue'
import InventoryView from '../Character/InventoryView.vue'
import VaultDatabaseTab from './VaultDatabaseTab.vue'

defineProps<{
  activeTab: string;
}>();
</script>
