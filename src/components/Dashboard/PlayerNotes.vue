<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- Add note input -->
    <div class="flex gap-1 mb-2">
      <input
        v-model="newNote"
        type="text"
        placeholder="Add a note..."
        class="flex-1 bg-surface-elevated border border-border-default rounded px-2 py-1 text-xs text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-gold/50"
        @keydown.enter="addNote" />
      <button
        class="px-2 py-1 text-xs rounded bg-accent-gold/20 text-accent-gold border border-accent-gold/30 hover:bg-accent-gold/30 transition-colors disabled:opacity-30"
        :disabled="!newNote.trim()"
        @click="addNote">
        +
      </button>
    </div>

    <!-- Notes list -->
    <div v-if="notes.length === 0" class="text-xs text-text-dim italic">No notes yet.</div>

    <div v-else class="flex flex-col gap-0.5 flex-1 overflow-y-auto min-h-0 pr-1">
      <div
        v-for="note in notes"
        :key="note.id"
        class="flex items-center gap-2 py-1 px-2 rounded text-xs group hover:bg-surface-elevated/50">
        <!-- Checkbox -->
        <input
          type="checkbox"
          :checked="note.done"
          class="accent-accent-gold shrink-0 cursor-pointer"
          @change="toggleNote(note.id)" />

        <!-- Text -->
        <span
          class="flex-1 min-w-0 truncate"
          :class="note.done ? 'text-text-dim line-through' : 'text-text-primary'">
          {{ note.text }}
        </span>

        <!-- Delete button -->
        <button
          class="text-text-dim hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
          title="Remove"
          @click="removeNote(note.id)">
          ×
        </button>
      </div>
    </div>

    <!-- Clear completed -->
    <button
      v-if="completedCount > 0"
      class="mt-2 text-[0.65rem] text-text-dim hover:text-text-secondary transition-colors self-start"
      @click="clearCompleted">
      Clear {{ completedCount }} completed
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

interface Note {
  id: number
  text: string
  done: boolean
}

const STORAGE_KEY = 'glogger-player-notes'

function loadNotes(): Note[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? JSON.parse(raw) : []
  } catch {
    return []
  }
}

function saveNotes(items: Note[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items))
}

const notes = ref<Note[]>(loadNotes())
const newNote = ref('')
let nextId = notes.value.reduce((max, n) => Math.max(max, n.id), 0) + 1

const completedCount = computed(() => notes.value.filter(n => n.done).length)

watch(notes, (val) => saveNotes(val), { deep: true })

function addNote() {
  const text = newNote.value.trim()
  if (!text) return
  notes.value.unshift({ id: nextId++, text, done: false })
  newNote.value = ''
}

function toggleNote(id: number) {
  const note = notes.value.find(n => n.id === id)
  if (note) note.done = !note.done
}

function removeNote(id: number) {
  notes.value = notes.value.filter(n => n.id !== id)
}

function clearCompleted() {
  notes.value = notes.value.filter(n => !n.done)
}
</script>
