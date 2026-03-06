<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'

const logPath = ref('')
const updates = ref<any[]>([])
const error = ref('')
const watching = ref(false)

onMounted(async () => {
  await listen('skill-update', (event) => {
    updates.value.unshift(event.payload)
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
  try {
    await invoke('start_watching', { path: logPath.value })
    watching.value = true
  } catch (e) {
    error.value = String(e)
  }
}
</script>

<template>
  <div style="padding: 1rem; font-family: monospace;">
    <h2>Glogger — Skill Update Test</h2>

    <div style="margin: 1rem 0; display: flex; gap: 0.5rem;">
      <input
        v-model="logPath"
        placeholder="Pick a log file..."
        style="flex: 1; padding: 0.4rem;"
        readonly
      />
      <button @click="pickFile" :disabled="watching">Browse</button>
      <button @click="startWatching" :disabled="watching || !logPath">
        {{ watching ? 'Watching...' : 'Start' }}
      </button>
    </div>

    <div v-if="error" style="color: red; margin-bottom: 1rem;">{{ error }}</div>

    <div v-if="updates.length === 0" style="color: #888;">
      No skill updates seen yet.
    </div>

    <div
      v-for="(u, i) in updates"
      :key="i"
      style="border-bottom: 1px solid #ccc; padding: 0.4rem 0; font-size: 0.85rem;"
    >
      <strong>{{ u.skill_type }}</strong>
      Lv {{ u.level }} — {{ u.xp }} xp ({{ u.tnl }} to next level)
      <div style="color: #888; font-size: 0.75rem;">{{ u.raw_line }}</div>
    </div>
  </div>
</template>