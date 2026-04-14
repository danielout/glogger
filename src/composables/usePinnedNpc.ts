import { ref, watch } from 'vue'

const MAX_PINNED = 5
const STORAGE_KEY = 'pinnedNpcs'

// Module-level singleton state
const pinnedNpcKeys = ref<string[]>(loadFromStorage())

function loadFromStorage(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

// Persist on change
watch(pinnedNpcKeys, (keys) => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(keys))
}, { deep: true })

export function usePinnedNpc() {
  function pinNpc(npcKey: string) {
    if (pinnedNpcKeys.value.includes(npcKey)) return
    if (pinnedNpcKeys.value.length >= MAX_PINNED) {
      // Remove oldest (first) to make room
      pinnedNpcKeys.value.shift()
    }
    pinnedNpcKeys.value.push(npcKey)
  }

  function unpinNpc(npcKey: string) {
    const idx = pinnedNpcKeys.value.indexOf(npcKey)
    if (idx !== -1) {
      pinnedNpcKeys.value.splice(idx, 1)
    }
  }

  function isPinned(npcKey: string): boolean {
    return pinnedNpcKeys.value.includes(npcKey)
  }

  function togglePin(npcKey: string) {
    if (isPinned(npcKey)) {
      unpinNpc(npcKey)
    } else {
      pinNpc(npcKey)
    }
  }

  return {
    pinnedNpcKeys,
    pinNpc,
    unpinNpc,
    isPinned,
    togglePin,
  }
}
