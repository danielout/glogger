<template>
  <PaneLayout screen-key="dashboard">
  <div class="pt-4 flex flex-col gap-4 h-full overflow-y-auto">
    <!-- View toggle -->
    <div class="flex items-center gap-2">
      <button
        class="px-3 py-1.5 text-sm rounded border transition-colors"
        :class="viewMode === 'active' ? 'bg-surface-elevated border-accent-gold text-accent-gold' : 'bg-transparent border-border-default text-text-muted hover:text-text-primary'"
        @click="viewMode = 'active'">
        Active Character
      </button>
      <button
        class="px-3 py-1.5 text-sm rounded border transition-colors"
        :class="viewMode === 'aggregate' ? 'bg-surface-elevated border-accent-gold text-accent-gold' : 'bg-transparent border-border-default text-text-muted hover:text-text-primary'"
        @click="viewMode = 'aggregate'">
        All Characters on Server
      </button>
    </div>

    <!-- Active character view -->
    <template v-if="viewMode === 'active'">
      <!-- Context Bar: weather, combat, currencies -->
      <ContextBar />

      <!-- Live Skill Tracking -->
      <div>
        <h2 class="text-sm font-bold text-text-secondary uppercase tracking-wide mb-3">Live Skill Tracking</h2>
        <EmptyState
          v-if="store.sessionSkillList.length === 0"
          variant="compact"
          primary="No skill updates yet."
          secondary="Start playing to see XP gains here." />
        <div v-else class="flex flex-wrap gap-4">
          <SkillCard v-for="skill in store.sessionSkillList" :key="skill.skillType" :skill="skill" />
        </div>
      </div>

      <!-- Activity Feeds -->
      <div class="grid grid-cols-3 gap-4">
        <!-- Items Incoming -->
        <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
          <ActivityFeed
            title="Items Incoming"
            :entries="store.itemsIncoming"
            dot-color="bg-green-500"
            empty-text="No items received."
            empty-hint="Loot, crafting output, and summoned items appear here."
            show-item-links
            unit="items"
            :warning-tooltip="ACCURACY_WARNING" />
        </div>

        <!-- Items Outgoing -->
        <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
          <ActivityFeed
            title="Items Outgoing"
            :entries="store.itemsOutgoing"
            dot-color="bg-red-500"
            empty-text="No items lost."
            empty-hint="Sold, stored, and consumed items appear here."
            show-item-links
            unit="items"
            quantity-prefix
            :warning-tooltip="ACCURACY_WARNING" />
        </div>

        <!-- Councils (Gold) -->
        <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
          <ActivityFeed
            title="Councils"
            :entries="store.councilChanges"
            dot-color="bg-yellow-500"
            empty-text="No council changes."
            empty-hint="Vendor sales, loot, and council transactions appear here."
            unit="councils"
            signed-total
            :warning-tooltip="ACCURACY_WARNING" />
        </div>
      </div>

      <!-- Second row: Current Zone + Favor + Moon Phase -->
      <div class="grid grid-cols-3 gap-4">
        <!-- Current Zone -->
        <CurrentZone />

        <!-- Favor Changes -->
        <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
          <ActivityFeed
            title="Favor Changes"
            :entries="store.favorChanges"
            dot-color="bg-purple-500"
            empty-text="No favor changes."
            empty-hint="NPC favor gains and losses appear here."
            show-npc-links
            unit="favor"
            signed-total />
        </div>

        <!-- Player Notes -->
        <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
          <PlayerNotes />
        </div>
      </div>

      <!-- Third row: Moon Phase + future cards -->
      <div class="grid grid-cols-3 gap-4">
        <MoonPhaseCard />
      </div>
    </template>

    <!-- Aggregate view -->
    <AggregateView v-else />
  </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import PaneLayout from '../Shared/PaneLayout.vue'
import EmptyState from '../Shared/EmptyState.vue'
import SkillCard from '../Shared/SkillCard.vue'
import ContextBar from './ContextBar.vue'
import ActivityFeed from './ActivityFeed.vue'
import PlayerNotes from './PlayerNotes.vue'
import AggregateView from './AggregateView.vue'
import CurrentZone from './CurrentZone.vue'
import MoonPhaseCard from './MoonPhaseCard.vue'

const store = useGameStateStore()
const viewMode = ref<'active' | 'aggregate'>('active')

const ACCURACY_WARNING = 'Right now Glogger is doing the best it can to try to infer and figure out quantities, stack sizes, etc. However, due to limitations in the log files that is not a straightforward task. Do not be surprised if this is wrong! The best way to ensure Glogger has an accurate picture of your inventory is always your VIP Inventory JSON export.'
</script>
