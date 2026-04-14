<template>
  <div class="flex flex-col gap-4">
    <!-- Loading -->
    <div v-if="loading" class="text-sm text-text-muted py-8 text-center">
      Loading release notes...
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
