<template>
  <div class="card overflow-hidden flex flex-col" ref="cardRef">
    <!-- Title bar — drag handle -->
    <div class="dashboard-card-handle flex items-center gap-2 px-3 py-1 border-b border-border-default cursor-grab active:cursor-grabbing bg-surface-base/30 select-none">
      <span class="text-xs font-bold text-text-secondary uppercase tracking-wide truncate">{{ title }}</span>
      <div v-if="hasConfig" class="ml-auto relative">
        <button
          class="p-0.5 text-text-dim hover:text-text-secondary transition-colors"
          title="Widget options"
          @click.stop="configOpen = !configOpen">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-3.5 h-3.5">
            <path fill-rule="evenodd" d="M7.84 1.804A1 1 0 0 1 8.82 1h2.36a1 1 0 0 1 .98.804l.331 1.652a6.993 6.993 0 0 1 1.929 1.115l1.598-.54a1 1 0 0 1 1.186.447l1.18 2.044a1 1 0 0 1-.205 1.251l-1.267 1.113a7.047 7.047 0 0 1 0 2.228l1.267 1.113a1 1 0 0 1 .206 1.25l-1.18 2.045a1 1 0 0 1-1.187.447l-1.598-.54a6.993 6.993 0 0 1-1.929 1.115l-.33 1.652a1 1 0 0 1-.98.804H8.82a1 1 0 0 1-.98-.804l-.331-1.652a6.993 6.993 0 0 1-1.929-1.115l-1.598.54a1 1 0 0 1-1.186-.447l-1.18-2.044a1 1 0 0 1 .205-1.251l1.267-1.114a7.05 7.05 0 0 1 0-2.227L1.821 7.773a1 1 0 0 1-.206-1.25l1.18-2.045a1 1 0 0 1 1.187-.447l1.598.54A6.992 6.992 0 0 1 7.51 3.456l.33-1.652ZM10 13a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z" clip-rule="evenodd" />
          </svg>
        </button>
        <!-- Config popover -->
        <div
          v-if="configOpen"
          class="absolute right-0 top-full mt-1 z-50 min-w-48 bg-surface-elevated border border-border-default rounded-lg shadow-lg p-3 text-xs text-text-secondary">
          <slot name="config" />
        </div>
      </div>
    </div>

    <!-- Card content -->
    <div class="p-4 flex-1 min-h-0 overflow-y-auto max-h-96">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, useSlots } from 'vue'

defineProps<{
  title: string
  cardId?: string
}>()

const slots = useSlots()
const hasConfig = computed(() => !!slots.config)
const configOpen = ref(false)
const cardRef = ref<HTMLElement | null>(null)

function handleClickOutside(e: MouseEvent) {
  if (configOpen.value && cardRef.value && !cardRef.value.contains(e.target as Node)) {
    configOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>
