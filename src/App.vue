<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { useSkillStore } from './stores/skillStore'
import SkillGrid from './components/SkillGrid.vue'

const skillStore = useSkillStore()

const logPath = ref('')
const error = ref('')
const watching = ref(false)
const parsing = ref(false)

onMounted(async () => {
  await listen('skill-update', (event: any) => {
    skillStore.handleUpdate(event.payload)
  })
})

async function pickFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Log Files', extensions: ['log', 'txt'] }]
  })
  if (selected) logPath.value = selected
}

async function startWatching() {
  error.value = ''
  skillStore.reset()
  try {
    await invoke('start_watching', { path: logPath.value })
    watching.value = true
  } catch (e) {
    error.value = String(e)
  }
}

async function parseLog() {
  error.value = ''
  skillStore.reset()
  parsing.value = true
  try {
    await invoke('parse_log', { path: logPath.value })
  } catch (e) {
    error.value = String(e)
  } finally {
    parsing.value = false
  }
}
</script>

<template>
  <div style="padding: 1rem; font-family: monospace; background: #111; min-height: 100vh; color: #ccc;">
    <h2 style="color: #e0c060; margin-bottom: 1rem;">Glogger</h2>

    <div style="display: flex; gap: 0.5rem; margin-bottom: 0.5rem;">
      <input
        v-model="logPath"
        placeholder="Pick a log file..."
        style="flex: 1; padding: 0.4rem; background: #222; color: #ccc; border: 1px solid #444;"
        readonly
      />
      <button @click="pickFile" :disabled="watching || parsing">Browse</button>
      <button @click="startWatching" :disabled="watching || parsing || !logPath">
        {{ watching ? 'Watching...' : 'Start Watching' }}
      </button>
      <button @click="parseLog" :disabled="watching || parsing || !logPath">
        {{ parsing ? 'Parsing...' : 'Parse Log' }}
      </button>
    </div>

    <div v-if="error" style="color: #f66; margin-bottom: 1rem;">{{ error }}</div>

    <SkillGrid />
  </div>
</template>