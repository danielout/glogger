<template>
  <ActivityFeed
    :entries="store.councilChanges"
    dot-color="bg-status-warning"
    empty-text="No council changes."
    empty-hint="Vendor sales, loot, and council transactions appear here."
    unit="councils"
    signed-total
    :warning-tooltip="ACCURACY_WARNING" />
</template>

<script setup lang="ts">
import { useGameStateStore } from '../../../stores/gameStateStore'
import ActivityFeed from '../ActivityFeed.vue'

const store = useGameStateStore()

const ACCURACY_WARNING = 'Right now Glogger is doing the best it can to try to infer and figure out quantities, stack sizes, etc. However, due to limitations in the log files that is not a straightforward task. Do not be surprised if this is wrong! The best way to ensure Glogger has an accurate picture of your inventory is always your VIP Inventory JSON export.'
</script>
