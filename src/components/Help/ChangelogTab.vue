<template>
  <div class="flex flex-col gap-4">
    <!-- Update banner (when update available) -->
    <div
      v-if="updateStore.updateAvailable"
      class="bg-accent-blue/10 border border-accent-blue/30 rounded-lg p-4 flex items-center justify-between gap-4">
      <div>
        <div class="text-sm font-semibold text-text-primary">
          Glogger v{{ updateStore.latestVersion }} is available
        </div>
        <div class="text-xs text-text-secondary mt-0.5">
          Review the changes below, then update when you're ready.
        </div>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <template v-if="updateStore.installing">
          <span class="text-xs text-accent-blue">Updating... {{ updateStore.downloadProgress }}%</span>
        </template>
        <template v-else>
          <button
            class="px-4 py-2 text-sm font-medium rounded cursor-pointer bg-accent-blue/20 border border-accent-blue/40 text-accent-blue hover:bg-accent-blue/30 transition-colors"
            @click="updateStore.downloadAndInstall()">
            Update Glogger
          </button>
        </template>
      </div>
      <div v-if="updateStore.installError" class="text-xs text-accent-red mt-1 w-full">
        {{ updateStore.installError }}
      </div>
    </div>

    <!-- Check for updates (when no update detected yet) -->
    <div v-else class="flex items-center gap-3">
      <button
        class="px-3 py-1.5 text-xs font-medium rounded cursor-pointer bg-surface-base border border-border-default text-text-secondary hover:bg-surface-elevated hover:text-text-primary transition-colors"
        :disabled="checking"
        @click="manualCheck">
        {{ checking ? 'Checking...' : 'Check for Updates' }}
      </button>
      <span v-if="checkResult" class="text-xs text-accent-green">{{ checkResult }}</span>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="py-4 space-y-4">
      <SkeletonLoader v-for="i in 3" :key="i" variant="rect" height="h-24" />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="text-sm text-text-muted py-8 text-center">
      <p>{{ error }}</p>
      <button class="btn btn-primary mt-3" @click="loadReleases">Retry</button>
    </div>

    <!-- Releases -->
    <template v-else>
      <div
        v-for="release in releases"
        :key="release.tag_name"
        class="bg-surface-base rounded border border-border-default overflow-hidden">
        <!-- Release header -->
        <div class="flex items-center justify-between px-4 py-2.5 border-b border-border-default bg-surface-dark">
          <div class="flex items-center gap-2">
            <span class="text-accent-gold font-bold text-sm">{{ release.tag_name }}</span>
            <span v-if="release.name !== release.tag_name" class="text-text-secondary text-sm">
              &mdash; {{ release.name }}
            </span>
          </div>
          <span class="text-text-dim text-xs">{{ formatDate(release.published_at) }}</span>
        </div>
        <!-- Release body -->
        <div class="px-4 py-3 text-sm text-text-secondary leading-relaxed changelog-body" v-html="renderMarkdown(release.body)" />
      </div>

      <div v-if="releases.length === 0" class="text-sm text-text-muted py-8 text-center">
        No releases found.
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useUpdateStore } from '../../stores/updateStore'
import SkeletonLoader from '../Shared/SkeletonLoader.vue'

const updateStore = useUpdateStore()

const checking = ref(false)
const checkResult = ref<string | null>(null)

async function manualCheck() {
  checking.value = true
  checkResult.value = null
  await updateStore.checkForUpdate(false)
  checking.value = false
  if (updateStore.updateAvailable) {
    checkResult.value = null // banner will show instead
  } else {
    checkResult.value = 'You\'re on the latest version.'
  }
}

interface ReleaseInfo {
  tag_name: string
  name: string
  published_at: string
  html_url: string
  body: string
}

const releases = ref<ReleaseInfo[]>([])
const loading = ref(true)
const error = ref('')

async function loadReleases() {
  loading.value = true
  error.value = ''
  try {
    releases.value = await invoke<ReleaseInfo[]>('fetch_github_releases')
  } catch (e) {
    error.value = `Could not load changelog: ${e}`
  } finally {
    loading.value = false
  }
}

function formatDate(iso: string): string {
  if (!iso) return ''
  const d = new Date(iso)
  return d.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })
}

function renderMarkdown(body: string): string {
  if (!body) return ''
  return body
    // Headers
    .replace(/^### (.+)$/gm, '<h4 class="text-text-primary font-bold mt-3 mb-1 text-sm">$1</h4>')
    .replace(/^## (.+)$/gm, '<h3 class="text-accent-gold font-bold mt-4 mb-1.5 text-sm">$1</h3>')
    // Bold
    .replace(/\*\*(.+?)\*\*/g, '<strong class="text-text-primary">$1</strong>')
    // Inline code
    .replace(/`([^`]+)`/g, '<code class="text-accent-gold bg-surface-dark px-1 py-0.5 rounded-sm text-xs">$1</code>')
    // List items
    .replace(/^[-*] (.+)$/gm, '<li class="ml-4 list-disc">$1</li>')
    // Wrap consecutive <li> in <ul>
    .replace(/((?:<li[^>]*>.*<\/li>\n?)+)/g, '<ul class="my-1.5">$1</ul>')
    // Line breaks for remaining text
    .replace(/\n\n/g, '<br/><br/>')
    .replace(/\n/g, '<br/>')
}

onMounted(loadReleases)
</script>

<style scoped>
.changelog-body :deep(ul) {
  list-style: disc;
  padding-left: 1.25rem;
}

.changelog-body :deep(li) {
  margin: 0.15rem 0;
}
</style>
