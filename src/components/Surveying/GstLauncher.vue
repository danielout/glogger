<template>
  <div class="flex flex-col text-[0.7rem]">
    <!-- Accordion header -->
    <button
      class="w-full flex items-center gap-1.5 text-left text-text-secondary hover:text-text-primary transition-colors"
      @click="toggle"
    >
      <span class="text-[0.6rem] text-text-dim transition-transform" :class="expanded ? 'rotate-90' : ''">&#9654;</span>
      <span>GorgonSurveyTracker</span>
      <span v-if="status?.installed_version" class="text-text-dim">
        {{ status.installed_version }}
      </span>
    </button>

    <!-- Accordion body -->
    <div v-if="expanded" class="flex flex-col gap-1.5 pt-1.5 pl-3">
      <!-- Windows: download + launch -->
      <template v-if="status && isWindows">
        <!-- Not installed -->
        <template v-if="!status.installed">
          <button
            class="w-full text-xs px-3 py-1.5 rounded border transition-colors border-accent-gold/60 text-accent-gold hover:bg-accent-gold/10"
            :disabled="busy"
            @click="download"
          >
            {{ busy ? 'Downloading...' : 'Download GST' }}
          </button>
        </template>

        <!-- Installed -->
        <template v-else>
          <div class="flex gap-1.5">
            <button
              class="flex-1 text-xs px-3 py-1.5 rounded border transition-colors border-accent-gold/60 text-accent-gold hover:bg-accent-gold/10"
              :disabled="busy"
              @click="launch"
            >
              Launch GST
            </button>
            <button
              v-if="status.update_available"
              class="text-xs px-2 py-1.5 rounded border transition-colors border-blue-500/60 text-blue-400 hover:bg-blue-500/10"
              :disabled="busy"
              :title="`Update to ${status.latest_version}`"
              @click="download"
            >
              {{ busy ? '...' : 'Update' }}
            </button>
          </div>
          <div v-if="status.update_available" class="text-text-dim text-[0.65rem]">
            {{ status.latest_version }} available
          </div>
        </template>
      </template>

      <!-- Mac/Linux: setup instructions hint -->
      <template v-else-if="status && !isWindows">
        <div class="text-text-dim text-[0.65rem]">
          Requires Python 3.8+ — see the GST page for
          {{ status.platform === 'macos' ? 'macOS' : 'Linux' }} setup steps.
        </div>
      </template>

      <!-- Loading / checking -->
      <template v-else>
        <span class="text-text-dim italic">Checking...</span>
      </template>

      <button
        class="text-[0.65rem] text-text-dim hover:text-accent-gold transition-colors text-left"
        @click="openGstPage"
      >
        GST Homepage &amp; Instructions &#8599;
      </button>

      <div v-if="error" class="text-red-400 text-[0.65rem]">{{ error }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useViewPrefs } from '../../composables/useViewPrefs'

const GST_HOMEPAGE = 'https://github.com/kaeus/GorgonSurveyTracker'

interface GstStatus {
  installed: boolean
  installed_version: string | null
  latest_version: string | null
  update_available: boolean
  platform: string
}

const { prefs, update } = useViewPrefs('gst-launcher', {
  expanded: true,
})
const expanded = computed(() => prefs.value.expanded)

function toggle() {
  update({ expanded: !prefs.value.expanded })
}

const status = ref<GstStatus | null>(null)
const busy = ref(false)
const error = ref<string | null>(null)

const isWindows = computed(() => status.value?.platform === 'windows')

async function checkStatus() {
  try {
    status.value = await invoke<GstStatus>('gst_check_status')
  } catch (e) {
    console.error('[GstLauncher] status check failed:', e)
    status.value = null
  }
}

async function download() {
  busy.value = true
  error.value = null
  try {
    status.value = await invoke<GstStatus>('gst_download')
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message ?? 'Download failed'
  } finally {
    busy.value = false
  }
}

async function launch() {
  error.value = null
  try {
    await invoke('gst_launch')
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message ?? 'Launch failed'
  }
}

function openGstPage() {
  openUrl(GST_HOMEPAGE)
}

onMounted(() => {
  void checkStatus()
})
</script>
