<template>
  <Teleport to="body">
    <Transition name="help-overlay">
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center" @keydown.escape="close">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/50" @click="close" />

        <!-- Modal -->
        <div class="relative bg-surface-elevated border border-border-default rounded-lg shadow-xl w-[85vw] max-w-275 h-[80vh] flex flex-col overflow-hidden">
          <!-- Header -->
          <div class="flex items-center justify-between px-6 py-4 border-b border-border-default shrink-0">
            <h2 class="text-accent-gold m-0 text-xl">Help</h2>
            <button
              class="text-text-muted hover:text-text-primary bg-transparent border-none cursor-pointer text-lg leading-none transition-colors"
              @click="close"
              title="Close (Esc)">
              ✕
            </button>
          </div>

          <!-- Body: side tabs + content -->
          <div class="flex flex-1 min-h-0">
            <nav class="flex flex-col gap-1 min-w-40 border-r border-border-default p-3">
              <button
                v-for="tab in tabs"
                :key="tab.id"
                @click="activeTab = tab.id"
                class="px-4 py-2.5 bg-transparent border-none rounded text-text-secondary cursor-pointer font-mono text-sm text-left transition-all whitespace-nowrap hover:text-text-primary hover:bg-surface-base"
                :class="{ 'text-accent-gold! bg-surface-base! border-l-2 border-l-accent-gold pl-3.5': activeTab === tab.id }">
                {{ tab.label }}
              </button>
            </nav>

            <div class="flex-1 min-w-0 overflow-y-auto p-6">
              <div class="max-w-2xl">
                <AboutTab v-if="activeTab === 'about'" />
                <HelpSetupTab v-else-if="activeTab === 'help'" @navigate="$emit('navigate', $event)" />
                <ChangelogTab v-else-if="activeTab === 'changelog'" />
                <KnownIssuesTab v-else-if="activeTab === 'known-issues'" />
                <PgNewsTab v-else-if="activeTab === 'pg-news'" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import AboutTab from './AboutTab.vue'
import HelpSetupTab from './HelpSetupTab.vue'
import ChangelogTab from './ChangelogTab.vue'
import KnownIssuesTab from './KnownIssuesTab.vue'
import PgNewsTab from './PgNewsTab.vue'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  navigate: [view: string]
}>()

type TabId = 'about' | 'help' | 'changelog' | 'known-issues' | 'pg-news'

const tabs: { id: TabId; label: string }[] = [
  { id: 'about', label: 'About' },
  { id: 'help', label: 'Help' },
  { id: 'changelog', label: 'Glogger Changelog' },
  { id: 'known-issues', label: 'Known Issues' },
  { id: 'pg-news', label: 'PG News' },
]

const activeTab = ref<TabId>('about')

function close() {
  emit('update:show', false)
}

// Focus trap: listen for escape globally when open
watch(() => props.show, async (open) => {
  if (open) {
    await nextTick()
    window.addEventListener('keydown', handleKeydown)
  } else {
    window.removeEventListener('keydown', handleKeydown)
  }
})

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    e.stopPropagation()
    close()
  }
}
</script>

<style scoped>
.help-overlay-enter-active,
.help-overlay-leave-active {
  transition: opacity 0.15s ease;
}
.help-overlay-enter-from,
.help-overlay-leave-to {
  opacity: 0;
}
</style>
