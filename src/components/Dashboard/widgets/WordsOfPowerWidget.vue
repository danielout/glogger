<template>
  <div class="flex flex-col gap-2 text-sm h-full min-h-0">
    <!-- Search -->
    <input
      v-model="search"
      type="text"
      placeholder="Search words..."
      class="w-full px-2 py-1 rounded bg-surface-2 border border-border text-sm text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-blue shrink-0" />

    <!-- Word list grouped by power name -->
    <div class="flex-1 overflow-y-auto min-h-0 flex flex-col gap-2">
      <div
        v-for="group in filteredGroups"
        :key="group.powerName"
        class="rounded bg-surface-2/50 px-2 py-1.5">
        <div class="text-xs font-semibold text-accent-gold mb-1">{{ group.powerName }}</div>
        <div class="flex flex-col gap-1">
          <div
            v-for="word in group.words"
            :key="word.id"
            class="flex items-center gap-2 py-0.5 group">
            <span
              class="font-mono text-xs px-1.5 py-0.5 rounded cursor-pointer transition-colors"
              :class="wordAgeClass(word)"
              :title="'Click to copy — Discovered ' + formatWordAge(word)"
              @click="copyWord(word.word)">
              {{ word.word }}
            </span>
            <span class="text-[11px] text-text-dim flex-1 truncate" :title="word.description ?? ''">
              {{ formatWordAge(word) }}
            </span>
            <button
              class="text-text-dim hover:text-value-negative opacity-0 group-hover:opacity-100 transition-opacity text-xs shrink-0 cursor-pointer"
              title="Remove word"
              @click="removeWord(word.id)">
              &times;
            </button>
          </div>
        </div>
      </div>

      <div v-if="filteredGroups.length === 0 && words.length > 0" class="text-xs text-text-dim italic py-2">
        No words match your search.
      </div>
      <div v-if="words.length === 0" class="text-xs text-text-dim italic py-2">
        No words of power recorded yet. Craft a word in-game to auto-capture it, or add one manually below.
      </div>
    </div>

    <!-- Copied toast -->
    <div
      v-if="copiedWord"
      class="text-xs text-value-positive text-center shrink-0 transition-opacity">
      Copied "{{ copiedWord }}" to clipboard
    </div>

    <!-- Add manually -->
    <div class="shrink-0 border-t border-border pt-2">
      <button
        v-if="!showAddForm"
        class="text-xs text-text-dim hover:text-text-primary cursor-pointer"
        @click="showAddForm = true">
        + Add word manually
      </button>
      <div v-else class="flex flex-col gap-1.5">
        <input
          v-model="newWord"
          type="text"
          placeholder="Word (e.g. TOAEOACHROF)"
          class="w-full px-2 py-1 rounded bg-surface-2 border border-border text-xs text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-blue font-mono" />
        <input
          v-model="newPowerName"
          type="text"
          placeholder="Power name (e.g. Unnatural Gravity)"
          class="w-full px-2 py-1 rounded bg-surface-2 border border-border text-xs text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-blue" />
        <div class="flex gap-1.5">
          <button
            class="px-2 py-1 text-xs rounded bg-accent-blue/20 text-accent-blue hover:bg-accent-blue/30 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="!newWord.trim() || !newPowerName.trim()"
            @click="addManualWord">
            Add
          </button>
          <button
            class="px-2 py-1 text-xs rounded text-text-dim hover:text-text-primary transition-colors cursor-pointer"
            @click="cancelAdd">
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from '../../../stores/settingsStore'
import { parseUtc } from '../../../composables/useTimestamp'

interface WordOfPower {
  id: number
  character_name: string
  server_name: string
  word: string
  power_name: string
  description: string | null
  discovered_at: string
  source: string
}

interface WordGroup {
  powerName: string
  words: WordOfPower[]
}

const settings = useSettingsStore()
const words = ref<WordOfPower[]>([])
const search = ref('')
const showAddForm = ref(false)
const newWord = ref('')
const newPowerName = ref('')
const copiedWord = ref('')
const now = ref(Date.now())

let refreshInterval: ReturnType<typeof setInterval> | null = null
let copiedTimeout: ReturnType<typeof setTimeout> | null = null
let unlisten: UnlistenFn | null = null

const filteredGroups = computed<WordGroup[]>(() => {
  const q = search.value.toLowerCase().trim()
  const filtered = q
    ? words.value.filter(
        w =>
          w.word.toLowerCase().includes(q) ||
          w.power_name.toLowerCase().includes(q),
      )
    : words.value

  const groups = new Map<string, WordOfPower[]>()
  for (const w of filtered) {
    const list = groups.get(w.power_name) || []
    list.push(w)
    groups.set(w.power_name, list)
  }

  return [...groups.entries()]
    .map(([powerName, groupWords]) => ({
      powerName,
      words: groupWords.sort(
        (a, b) => parseUtc(b.discovered_at).getTime() - parseUtc(a.discovered_at).getTime(),
      ),
    }))
    .sort((a, b) => {
      const aNewest = parseUtc(a.words[0].discovered_at).getTime()
      const bNewest = parseUtc(b.words[0].discovered_at).getTime()
      return bNewest - aNewest
    })
})

function wordAgeMs(word: WordOfPower): number {
  const discovered = parseUtc(word.discovered_at).getTime()
  if (isNaN(discovered)) return 0
  return now.value - discovered
}

function formatWordAge(word: WordOfPower): string {
  const ms = wordAgeMs(word)
  if (ms < 0 || ms < 60_000) return '<1 minute ago'

  const totalMinutes = Math.floor(ms / 60_000)
  const totalHours = Math.floor(ms / 3_600_000)
  const totalDays = Math.floor(ms / 86_400_000)

  const parts: string[] = []
  if (totalDays > 0) parts.push(`${totalDays} day${totalDays !== 1 ? 's' : ''}`)
  if (totalHours % 24 > 0) parts.push(`${totalHours % 24} hour${totalHours % 24 !== 1 ? 's' : ''}`)
  if (totalDays === 0 && totalMinutes % 60 > 0) parts.push(`${totalMinutes % 60} minute${totalMinutes % 60 !== 1 ? 's' : ''}`)

  return parts.join(', ') + ' ago'
}

function wordAgeClass(word: WordOfPower): string {
  const ms = wordAgeMs(word)
  const hours = ms / 3_600_000

  if (hours < 1) return 'bg-green-500/20 text-value-positive'
  if (hours < 6) return 'bg-accent-gold/20 text-accent-gold'
  if (hours < 24) return 'bg-orange-500/20 text-orange-400'
  return 'bg-red-500/20 text-value-negative'
}

async function copyWord(word: string) {
  try {
    await navigator.clipboard.writeText(word)
    copiedWord.value = word
    if (copiedTimeout) clearTimeout(copiedTimeout)
    copiedTimeout = setTimeout(() => {
      copiedWord.value = ''
    }, 2000)
  } catch {
    // Clipboard may not be available
  }
}

async function loadWords() {
  const char = settings.settings.activeCharacterName
  const server = settings.settings.activeServerName
  if (!char || !server) return

  try {
    words.value = await invoke<WordOfPower[]>('get_words_of_power', {
      characterName: char,
      serverName: server,
    })
  } catch (e) {
    console.error('Failed to load words of power:', e)
  }
}

async function addManualWord() {
  const char = settings.settings.activeCharacterName
  const server = settings.settings.activeServerName
  if (!char || !server) return

  try {
    await invoke('add_word_of_power', {
      characterName: char,
      serverName: server,
      input: {
        word: newWord.value.trim(),
        power_name: newPowerName.value.trim(),
        description: null,
      },
    })
    cancelAdd()
    await loadWords()
  } catch (e) {
    console.error('Failed to add word of power:', e)
  }
}

async function removeWord(id: number) {
  try {
    await invoke('delete_word_of_power', { id })
    words.value = words.value.filter(w => w.id !== id)
  } catch (e) {
    console.error('Failed to delete word of power:', e)
  }
}

function cancelAdd() {
  showAddForm.value = false
  newWord.value = ''
  newPowerName.value = ''
}

onMounted(async () => {
  await loadWords()

  // Refresh age counters every 30s
  refreshInterval = setInterval(() => {
    now.value = Date.now()
  }, 30_000)

  unlisten = await listen<string[]>('game-state-updated', (event) => {
    if (event.payload.includes('words_of_power')) {
      loadWords()
    }
  })
})

onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval)
  if (copiedTimeout) clearTimeout(copiedTimeout)
  if (unlisten) unlisten()
})
</script>
