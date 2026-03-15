<template>
  <div class="flex flex-col h-full bg-surface-base">
    <div v-if="loading" class="flex flex-col items-center justify-center h-full text-text-muted">
      <div class="w-10 h-10 border-3 border-border-default border-t-accent-gold rounded-full animate-spin mb-4"></div>
      <p>Loading messages...</p>
    </div>
    <div v-else-if="messages.length === 0" class="flex flex-col items-center justify-center h-full text-text-muted">
      <p class="my-1">No messages found</p>
      <p class="text-sm text-text-dim">Try importing chat logs from the Management tab</p>
    </div>
    <div v-else class="flex-1 overflow-y-auto p-4" ref="messagesContainer">
      <!-- Tell conversation layout -->
      <template v-if="isTellView">
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="flex mb-1"
          :class="msg.from_player ? 'justify-end' : 'justify-start'"
        >
          <div
            class="max-w-[75%] rounded-lg px-3 py-2 leading-relaxed"
            :class="msg.from_player
              ? 'bg-accent-blue/15 rounded-br-sm'
              : 'bg-surface-elevated rounded-bl-sm'"
          >
            <div class="flex items-baseline gap-2 mb-0.5">
              <span class="text-xs font-semibold" :class="msg.from_player ? 'text-accent-blue' : 'text-accent-gold'">
                {{ msg.from_player ? 'You' : msg.sender || 'Unknown' }}
              </span>
              <span class="text-text-dim text-xs font-mono">{{ formatTime(msg.timestamp) }}</span>
            </div>
            <span class="text-text-primary text-sm break-words">
              <MessageWithItemLinks v-if="msg.item_links && msg.item_links.length > 0" :message="msg.message" :item-links="msg.item_links" />
              <template v-else>{{ msg.message }}</template>
            </span>
          </div>
        </div>
      </template>
      <!-- Standard channel layout -->
      <template v-else>
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="flex gap-2 p-2 mb-1 rounded leading-relaxed transition-colors hover:bg-surface-card"
          :class="{
            'opacity-70': msg.is_system,
            'bg-accent-blue/5 hover:bg-accent-blue/10': msg.from_player === true
          }"
        >
          <span class="shrink-0 w-15 text-text-muted text-sm font-mono">{{ formatTime(msg.timestamp) }}</span>
          <span
            v-if="showChannel && msg.channel"
            class="shrink-0 font-semibold text-[0.9rem]"
            :class="channelColorClass(msg.channel)"
          >
            [{{ msg.channel }}]
          </span>
          <span v-if="msg.sender" class="shrink-0 text-sender font-medium">{{ formatSender(msg) }}:</span>
          <span class="flex-1 text-text-primary break-words" :class="{ 'text-text-system italic': msg.is_system }">
            <MessageWithItemLinks v-if="msg.item_links && msg.item_links.length > 0" :message="msg.message" :item-links="msg.item_links" />
            <template v-else>{{ msg.message }}</template>
          </span>
        </div>
      </template>
    </div>
    <div v-if="messages.length > 0 && hasMore" class="p-4 text-center border-t border-border-default">
      <button
        @click="$emit('load-more')"
        :disabled="loading"
        class="px-6 py-2 bg-surface-elevated border border-border-light text-text-primary rounded cursor-pointer transition-all hover:bg-border-default hover:border-border-hover disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Load More Messages
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import type { ChatMessage } from '../../types/database'
import MessageWithItemLinks from './MessageWithItemLinks.vue'

const props = defineProps<{
  messages: ChatMessage[]
  loading: boolean
  showChannel?: boolean
  hasMore?: boolean
  autoScroll?: boolean
}>()

const isTellView = computed(() => {
  return props.messages.length > 0
    && props.messages[0].channel === 'Tell'
    && !props.showChannel
})

defineEmits<{
  'load-more': []
}>()

const messagesContainer = ref<HTMLElement>()

function channelColorClass(channel: string): string {
  const map: Record<string, string> = {
    global: 'text-channel-global',
    trade: 'text-channel-trade',
    help: 'text-channel-help',
    guild: 'text-channel-guild',
    nearby: 'text-channel-nearby',
    status: 'text-channel-status',
    combat: 'text-channel-combat',
    lfg: 'text-channel-lfg',
    party: 'text-channel-party',
  }
  return map[channel.toLowerCase()] || 'text-text-secondary'
}

function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const isToday = date.toDateString() === now.toDateString()

  if (isToday) {
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false })
  } else {
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    })
  }
}

function formatSender(msg: ChatMessage): string {
  if (msg.channel === 'Tell' && msg.from_player !== null && msg.from_player !== undefined) {
    return msg.from_player ? 'YOU' : msg.sender || 'Unknown'
  }
  return msg.sender || 'Unknown'
}

watch(() => props.messages.length, async () => {
  if (props.autoScroll && messagesContainer.value) {
    await nextTick()
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
})
</script>
