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

      <!-- Bottom row: Transactions + Notes -->
      <div class="grid grid-cols-2 gap-4">
        <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
          <TransactionLog />
        </div>
        <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
          <PlayerNotes />
        </div>
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
import TransactionLog from './TransactionLog.vue'
import PlayerNotes from './PlayerNotes.vue'
import AggregateView from './AggregateView.vue'

const store = useGameStateStore()
const viewMode = ref<'active' | 'aggregate'>('active')
</script>
