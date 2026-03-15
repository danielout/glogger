<template>
  <div class="flex flex-col h-full bg-surface-base rounded-lg overflow-hidden">
    <!-- Top Navigation Tabs -->
    <div class="flex bg-surface-dark border-b-2 border-border-default px-4 pt-2 gap-1">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="px-6 py-3 bg-transparent border-none border-b-3 border-transparent cursor-pointer text-[0.95rem] font-medium transition-all font-sans"
        :class="currentTab === tab.id
          ? 'text-accent-gold border-b-accent-gold! bg-surface-base'
          : 'text-text-secondary hover:text-text-primary/70 hover:bg-surface-base'"
        @click="currentTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 overflow-hidden flex flex-col">
      <ChannelView v-if="currentTab === 'channels'" />
      <TellsView v-else-if="currentTab === 'tells'" />
      <PartyView v-else-if="currentTab === 'party'" />
      <NearbyView v-else-if="currentTab === 'nearby'" />
      <GuildView v-else-if="currentTab === 'guild'" />
      <SystemView v-else-if="currentTab === 'system'" />
      <AllMessagesView v-else-if="currentTab === 'all'" />
      <WatchwordsView v-else-if="currentTab === 'watchwords'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ChannelView from './ChannelView.vue'
import TellsView from './TellsView.vue'
import PartyView from './PartyView.vue'
import NearbyView from './NearbyView.vue'
import GuildView from './GuildView.vue'
import SystemView from './SystemView.vue'
import AllMessagesView from './AllMessagesView.vue'
import WatchwordsView from './WatchwordsView.vue'

const currentTab = ref('channels')

const tabs = [
  { id: 'channels', label: 'Channels' },
  { id: 'tells', label: 'Tells' },
  { id: 'party', label: 'Party' },
  { id: 'nearby', label: 'Nearby' },
  { id: 'guild', label: 'Guild' },
  { id: 'system', label: 'System' },
  { id: 'all', label: 'All Messages' },
  { id: 'watchwords', label: 'Watchwords' },
]
</script>
