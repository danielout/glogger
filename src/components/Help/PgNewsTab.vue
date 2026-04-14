<template>
  <div class="flex flex-col gap-4">
    <!-- Loading -->
    <div v-if="loading" class="text-sm text-text-muted py-8 text-center">
      Loading PG news...
    </div>

    <!-- Error -->
    <div v-else-if="error" class="text-sm text-text-muted py-8 text-center">
      <p>{{ error }}</p>
      <button class="btn btn-primary mt-3" @click="loadNews">Retry</button>
    </div>

    <!-- News entries -->
    <template v-else>
      <div
        v-for="(entry, i) in newsEntries"
        :key="i"
        class="bg-surface-base rounded border border-border-default overflow-hidden">
        <!-- Entry header -->
        <div class="px-4 py-2.5 border-b border-border-default bg-surface-dark">
          <span class="text-accent-gold font-bold text-sm">{{ entry.title }}</span>
        </div>
        <!-- Entry body -->
        <div class="px-4 py-3 text-sm text-text-secondary leading-relaxed news-body" v-html="entry.html" />
      </div>

      <div v-if="newsEntries.length === 0" class="text-sm text-text-muted py-8 text-center">
        No news entries found.
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface NewsEntry {
  title: string
  html: string
}

const newsEntries = ref<NewsEntry[]>([])
const loading = ref(true)
const error = ref('')

async function loadNews() {
  loading.value = true
  error.value = ''
  try {
    const raw = await invoke<string>('fetch_pg_news')
    newsEntries.value = parseNews(raw)
  } catch (e) {
    error.value = `Could not load PG news: ${e}`
  } finally {
    loading.value = false
  }
}

function parseNews(raw: string): NewsEntry[] {
  // Split on the title pattern: <size=38><color=#ffcc00>Title</color></size>
  const sections = raw.split(/(?=<size=38>)/).filter(s => s.trim())
  return sections.map(section => {
    // Extract title from first line
    const titleMatch = section.match(/<size=\d+><color=[^>]+>(.+?)<\/color><\/size>/)
    const title = titleMatch ? titleMatch[1] : 'Update'

    // Remove the title line from body
    let body = section.replace(/<size=\d+><color=[^>]+>.+?<\/color><\/size>\s*/, '')

    // Convert Unity rich text to HTML
    body = convertUnityRichText(body)

    return { title, html: body }
  })
}

function convertUnityRichText(text: string): string {
  return text
    // Color tags
    .replace(/<color=#ff0000>(.*?)<\/color>/g, '<span class="text-accent-red font-medium">$1</span>')
    .replace(/<color=#ffcc00>(.*?)<\/color>/g, '<span class="text-accent-gold font-bold">$1</span>')
    .replace(/<color=#([0-9a-fA-F]{6})>(.*?)<\/color>/g, '<span style="color: #$1">$2</span>')
    // Size tags — map to reasonable classes
    .replace(/<size=\d+>(.*?)<\/size>/g, '<span class="font-bold">$1</span>')
    // Bold/italic
    .replace(/<b>(.*?)<\/b>/g, '<strong>$1</strong>')
    .replace(/<i>(.*?)<\/i>/g, '<em>$1</em>')
    // Strip any remaining unknown tags
    .replace(/<\/?[a-z]+=?[^>]*>/gi, '')
    // Bullet points (lines starting with -)
    .replace(/^- (.+)$/gm, '<li class="ml-4 list-disc">$1</li>')
    .replace(/((?:<li[^>]*>.*<\/li>\n?)+)/g, '<ul class="my-1.5">$1</ul>')
    // Paragraphs
    .replace(/\n\n+/g, '</p><p class="mt-2">')
    .replace(/\n/g, '<br/>')
    // Wrap in paragraph
    .replace(/^(.+)$/, '<p>$1</p>')
}

onMounted(loadNews)
</script>

<style scoped>
.news-body :deep(ul) {
  list-style: disc;
  padding-left: 1.25rem;
}

.news-body :deep(li) {
  margin: 0.15rem 0;
}
</style>
