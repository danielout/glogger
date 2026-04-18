<template>
  <div class="flex flex-col h-full">
    <!-- Search header -->
    <div class="px-6 py-4 border-b border-border-default bg-surface-base">
      <div class="flex gap-2 items-center">
        <input
          ref="searchInput"
          type="text"
          v-model="rawQuery"
          @input="onSearchInput"
          placeholder="Search messages... (try from:player or in:channel)"
          class="flex-1 px-4 py-2 bg-surface-elevated border border-border-light rounded text-text-primary focus:outline-none focus:border-accent-gold"
        />
        <button
          @click="refresh"
          :disabled="loading"
          class="w-9 h-9 p-0 bg-surface-elevated border border-border-light rounded text-text-primary text-xl cursor-pointer transition-all flex items-center justify-center hover:bg-border-default hover:border-border-hover disabled:opacity-50 disabled:cursor-not-allowed"
          title="Refresh"
        >
          &#10227;
        </button>
      </div>

      <!-- Active filter chips -->
      <div v-if="parsed.sender || parsed.channel || parsed.textWords.length > 0" class="flex gap-2 mt-2 flex-wrap">
        <span
          v-for="word in parsed.textWords"
          :key="'text-' + word"
          class="inline-flex items-center gap-1 px-2.5 py-1 bg-text-secondary/15 text-text-primary text-sm rounded-full"
        >
          {{ word }}
          <button
            @click="removeTextWord(word)"
            class="ml-0.5 w-4 h-4 flex items-center justify-center bg-transparent border-none text-text-muted cursor-pointer hover:text-text-primary text-xs leading-none"
          >&times;</button>
        </span>
        <span
          v-if="parsed.sender"
          class="inline-flex items-center gap-1 px-2.5 py-1 bg-accent-blue/15 text-accent-blue text-sm rounded-full"
        >
          from:{{ parsed.sender }}
          <button
            @click="removeOperator('from')"
            class="ml-0.5 w-4 h-4 flex items-center justify-center bg-transparent border-none text-accent-blue/60 cursor-pointer hover:text-accent-blue text-xs leading-none"
          >&times;</button>
        </span>
        <span
          v-if="parsed.channel"
          class="inline-flex items-center gap-1 px-2.5 py-1 bg-accent-gold/15 text-accent-gold text-sm rounded-full"
        >
          in:{{ parsed.channel }}
          <button
            @click="removeOperator('in')"
            class="ml-0.5 w-4 h-4 flex items-center justify-center bg-transparent border-none text-accent-gold/60 cursor-pointer hover:text-accent-gold text-xs leading-none"
          >&times;</button>
        </span>
      </div>
    </div>

    <!-- Context mode header -->
    <div v-if="contextMessageId" class="px-6 py-2 border-b border-border-default bg-surface-elevated flex items-center gap-3">
      <button
        @click="exitContext"
        class="px-3 py-1 bg-surface-base border border-border-light rounded text-text-secondary text-sm cursor-pointer hover:bg-surface-hover hover:text-text-primary transition-all"
      >
        &larr; Back to results
      </button>
      <span class="text-text-muted text-sm">
        Showing context in
        <span v-if="contextChannel" class="font-semibold text-text-secondary">[{{ contextChannel }}]</span>
      </span>
    </div>

    <!-- Results -->
    <ChatMessageList
      :messages="displayMessages"
      :loading="contextMessageId ? contextLoading : loading"
      :has-more="contextMessageId ? false : hasMore"
      :show-channel="!contextMessageId"
      :sort-order="contextMessageId ? undefined : sortOrder"
      :clickable="!contextMessageId"
      :highlight-id="contextMessageId ?? undefined"
      @load-more="loadMore"
      @toggle-sort="toggleSort"
      @message-click="onMessageClick"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ChatMessage, ChatFilter } from '../../types/database'
import ChatMessageList from './ChatMessageList.vue'
import { parseSearchQuery } from '../../utils/parseSearchQuery'

const rawQuery = ref('')
const messages = ref<ChatMessage[]>([])
const loading = ref(false)
const hasMore = ref(true)
const offset = ref(0)
const sortOrder = ref<'asc' | 'desc'>('desc')
const searchInput = ref<HTMLInputElement>()
const LIMIT = 100

// Context mode state
const contextMessageId = ref<number | null>(null)
const contextMessages = ref<ChatMessage[]>([])
const contextLoading = ref(false)
const contextChannel = ref<string | null>(null)

let searchTimeout: number | null = null

const parsed = computed(() => parseSearchQuery(rawQuery.value))

const displayMessages = computed(() =>
  contextMessageId.value ? contextMessages.value : messages.value
)

async function loadMessages() {
  loading.value = true
  try {
    const p = parsed.value
    const filter: ChatFilter = {
      searchText: p.text || undefined,
      sender: p.sender || undefined,
      channel: p.channel || undefined,
      limit: LIMIT,
      offset: offset.value,
      sortOrder: sortOrder.value,
    }

    console.log('[ChatSearch] filter:', JSON.stringify(filter))
    const newMessages = await invoke<ChatMessage[]>('get_chat_messages', filter)

    if (offset.value === 0) {
      messages.value = newMessages
    } else {
      messages.value = [...messages.value, ...newMessages]
    }

    hasMore.value = newMessages.length === LIMIT
    offset.value += newMessages.length
  } catch (e) {
    console.error('Failed to search messages:', e)
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

function onSearchInput() {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = window.setTimeout(() => {
    exitContext()
    offset.value = 0
    hasMore.value = true
    loadMessages()
  }, 300)
}

function toggleSort() {
  sortOrder.value = sortOrder.value === 'desc' ? 'asc' : 'desc'
  offset.value = 0
  hasMore.value = true
  loadMessages()
}

function removeTextWord(word: string) {
  // Remove the first occurrence of this word (not inside an operator)
  const escaped = word.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  rawQuery.value = rawQuery.value.replace(new RegExp(`\\b${escaped}\\b`, 'i'), '').trim().replace(/\s+/g, ' ')
  offset.value = 0
  hasMore.value = true
  loadMessages()
}

function removeOperator(op: 'from' | 'in') {
  // Remove the operator from the raw query string
  const pattern = op === 'from'
    ? /\bfrom:(?:"[^"]*"|[\S]+)/gi
    : /\bin:(?:"[^"]*"|[\S]+)/gi
  rawQuery.value = rawQuery.value.replace(pattern, '').trim().replace(/\s+/g, ' ')
  offset.value = 0
  hasMore.value = true
  loadMessages()
}

async function onMessageClick(msg: ChatMessage) {
  contextMessageId.value = msg.id
  contextChannel.value = msg.channel ?? null
  contextLoading.value = true
  try {
    const result = await invoke<ChatMessage[]>('get_chat_messages_around', {
      messageId: msg.id,
      contextCount: 25,
    })
    contextMessages.value = result
  } catch (e) {
    console.error('Failed to load message context:', e)
    contextMessageId.value = null
  } finally {
    contextLoading.value = false
  }
}

function exitContext() {
  contextMessageId.value = null
  contextMessages.value = []
  contextChannel.value = null
}

onMounted(() => {
  loadMessages()
  nextTick(() => searchInput.value?.focus())
})
</script>
