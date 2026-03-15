<template>
  <div class="fixed inset-0 bg-surface-dark flex items-center justify-center">
    <div class="w-full max-w-md px-8">
      <h1 class="text-2xl font-bold text-accent-gold mb-2 text-center tracking-wide">glogger</h1>
      <p class="text-text-muted text-sm mb-8 text-center">Starting up...</p>

      <div class="space-y-3">
        <div
          v-for="(task, i) in tasks"
          :key="i"
          class="flex items-center gap-3 text-sm">
          <!-- Status icon -->
          <div class="w-5 h-5 flex items-center justify-center flex-shrink-0">
            <div v-if="task.status === 'pending'" class="w-2 h-2 rounded-full bg-border-default" />
            <div v-else-if="task.status === 'running'" class="w-4 h-4 border-2 border-accent-gold/30 border-t-accent-gold rounded-full animate-spin" />
            <span v-else-if="task.status === 'done'" class="text-accent-green">&#10003;</span>
            <span v-else-if="task.status === 'error'" class="text-accent-red">&#10007;</span>
          </div>

          <span :class="task.status === 'pending' ? 'text-text-muted' : 'text-text-primary'">
            {{ task.label }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { StartupTask } from "../../stores/startupStore";

defineProps<{
  tasks: StartupTask[];
}>();
</script>
