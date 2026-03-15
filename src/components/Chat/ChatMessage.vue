<template>
  <div
    class="flex gap-2 px-2 py-1 text-sm hover:bg-gray-800 rounded"
    :class="messageClass"
  >
    <span class="text-gray-500 flex-shrink-0 font-mono text-xs">
      {{ formatTime(message.timestamp) }}
    </span>
    <span
      v-if="message.channel"
      class="flex-shrink-0 font-semibold"
      :class="channelClass"
    >
      [{{ message.channel }}]
    </span>
    <span v-if="message.sender" class="flex-shrink-0 font-semibold text-blue-400">
      {{ message.sender }}:
    </span>
    <span class="flex-1 break-words" :class="messageTextClass">
      <template v-if="message.message && message.item_links && message.item_links.length > 0">
        <MessageWithItemLinks :message="message.message" :item-links="message.item_links" />
      </template>
      <template v-else-if="message.message">
        {{ message.message }}
      </template>
      <template v-else>
        <span class="text-gray-500 italic">(empty message)</span>
      </template>
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ChatMessage } from '../../types/database'
import MessageWithItemLinks from './MessageWithItemLinks.vue'

const props = defineProps<{
  message: ChatMessage
}>()

function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('en-US', { hour12: false })
}

const messageClass = computed(() => {
  if (props.message.is_system) {
    return 'opacity-75'
  }
  return ''
})

const messageTextClass = computed(() => {
  if (props.message.is_system) {
    return 'text-gray-300 italic'
  }
  return 'text-gray-100'
})

const channelClass = computed(() => {
  const channel = props.message.channel?.toLowerCase()

  const channelColors: Record<string, string> = {
    'global': 'text-yellow-400',
    'trade': 'text-green-400',
    'help': 'text-blue-400',
    'guild': 'text-purple-400',
    'nearby': 'text-orange-400',
    'status': 'text-cyan-400',
    'combat': 'text-red-400',
    'lfg': 'text-pink-400',
  }

  return channelColors[channel || ''] || 'text-gray-400'
})
</script>
