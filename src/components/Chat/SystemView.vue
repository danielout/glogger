<template>
  <div class="flex flex-col h-full">
    <div class="px-6 py-4 border-b border-border-default flex justify-between items-center bg-surface-base">
      <h2 class="screen-title m-0">System Messages</h2>
      <div class="flex gap-2">
        <button @click="refresh" :disabled="loading" class="w-9 h-9 p-0 bg-surface-elevated border border-border-light rounded text-text-primary text-xl cursor-pointer transition-all flex items-center justify-center hover:bg-border-default hover:border-border-hover disabled:opacity-50 disabled:cursor-not-allowed" title="Refresh">⟳</button>
      </div>
    </div>
    <ChatMessageList
      :messages="messages"
      :loading="loading"
      :has-more="hasMore"
      :sort-order="sortOrder"
      @load-more="loadMore"
      @toggle-sort="toggleSort"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ChatMessage, ChatFilter } from '../../types/database'
import ChatMessageList from './ChatMessageList.vue'

const messages = ref<ChatMessage[]>([])
const loading = ref(false)
const hasMore = ref(true)
const offset = ref(0)
const sortOrder = ref<'asc' | 'desc'>('desc')
const LIMIT = 100

async function loadMessages() {
  loading.value = true
  try {
    const filter: ChatFilter = {
      channel: 'Status',
      limit: LIMIT,
      offset: offset.value,
      sortOrder: sortOrder.value,
    }

    const newMessages = await invoke<ChatMessage[]>('get_chat_messages', filter)

    if (offset.value === 0) {
      messages.value = newMessages
    } else {
      messages.value = [...messages.value, ...newMessages]
    }

    hasMore.value = newMessages.length === LIMIT
    offset.value += newMessages.length
  } catch (e) {
    console.error('Failed to load messages:', e)
  } finally {
    loading.value = false
  }
}

function loadMore() {
  if (loading.value) return
  loadMessages()
}

function refresh() {
  offset.value = 0
  hasMore.value = true
  loadMessages()
}

function toggleSort() {
  sortOrder.value = sortOrder.value === 'desc' ? 'asc' : 'desc'
  offset.value = 0
  hasMore.value = true
  loadMessages()
}

onMounted(() => {
  loadMessages()
})
</script>
