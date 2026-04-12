<template>
  <div class="flex flex-col h-full">
    <div class="px-6 py-4 border-b border-border-default flex justify-between items-center bg-surface-base">
      <h2 class="m-0 text-text-primary text-xl font-medium">All Messages</h2>
      <div class="flex gap-2 items-center">
        <input
          type="text"
          v-model="searchText"
          @input="onSearchInput"
          placeholder="Search messages..."
          class="px-4 py-2 bg-surface-elevated border border-border-light rounded text-text-primary w-75 focus:outline-none focus:border-accent-gold"
        />
        <input
          type="text"
          v-model="itemNameFilter"
          @input="onSearchInput"
          placeholder="Item name..."
          class="px-4 py-2 bg-surface-elevated border border-border-light rounded text-text-primary w-50 focus:outline-none focus:border-accent-gold"
        />
        <label class="flex items-center gap-1.5 text-text-secondary text-sm cursor-pointer">
          <input
            type="checkbox"
            v-model="hasItemLinksFilter"
            @change="onFilterChange"
            class="w-4 h-4 cursor-pointer"
          />
          Item links only
        </label>
        <button
          @click="refresh"
          :disabled="loading"
          class="w-9 h-9 p-0 bg-surface-elevated border border-border-light rounded text-text-primary text-xl cursor-pointer transition-all flex items-center justify-center hover:bg-border-default hover:border-border-hover disabled:opacity-50 disabled:cursor-not-allowed"
          title="Refresh"
        >
          ⟳
        </button>
      </div>
    </div>
    <ChatMessageList
      :messages="messages"
      :loading="loading"
      :has-more="hasMore"
      :show-channel="true"
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
const searchText = ref('')
const itemNameFilter = ref('')
const hasItemLinksFilter = ref(false)
const sortOrder = ref<'asc' | 'desc'>('desc')
const LIMIT = 100

let searchTimeout: number | null = null

async function loadMessages() {
  loading.value = true
  try {
    const filter: ChatFilter = {
      searchText: searchText.value || undefined,
      itemName: itemNameFilter.value || undefined,
      hasItemLinks: hasItemLinksFilter.value || undefined,
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
  } catch (e) {
    console.error('Failed to load messages:', e)
  } finally {
    loading.value = false
  }
}

function loadMore() {
  offset.value += LIMIT
  loadMessages()
}

function refresh() {
  offset.value = 0
  hasMore.value = true
  loadMessages()
}

function onSearchInput() {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = window.setTimeout(() => {
    offset.value = 0
    hasMore.value = true
    loadMessages()
  }, 300)
}

function onFilterChange() {
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
