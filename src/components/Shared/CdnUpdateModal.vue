<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center">
        <!-- Backdrop (no click-to-dismiss — this is intentionally blocking) -->
        <div class="absolute inset-0 bg-black/60" />

        <!-- Dialog -->
        <div class="relative bg-surface-base border border-accent-red/40 rounded-lg shadow-xl w-[28rem] max-w-[90vw]">
          <!-- Header -->
          <div class="px-5 pt-5 pb-2 flex items-center gap-2">
            <span class="text-accent-red text-lg">!</span>
            <h3 class="text-sm font-semibold text-text-primary">Game Data Update Available</h3>
          </div>

          <!-- Content -->
          <div class="px-5 pb-4">
            <p class="text-sm text-text-secondary leading-relaxed">
              Game data has been updated
              <span class="text-text-primary font-medium">
                (v{{ currentVersion }} &#8594; v{{ remoteVersion }})
              </span>.
              Glogger needs to restart to load the new data.
            </p>
            <p class="mt-2 text-xs text-accent-red/80 leading-relaxed">
              Running with outdated game data may cause incorrect item names,
              broken recipes, or missing information.
            </p>
          </div>

          <!-- Footer -->
          <div class="flex items-center justify-end gap-2 px-5 pb-5">
            <button
              class="px-3 py-1.5 text-xs font-medium rounded cursor-pointer bg-surface-elevated border border-border-default text-text-secondary hover:bg-surface-hover"
              :disabled="restarting"
              @click="$emit('dismiss')">
              Remind Me Later
            </button>
            <button
              class="px-3 py-1.5 text-xs font-medium rounded cursor-pointer bg-accent-red/20 border border-accent-red/40 text-accent-red hover:bg-accent-red/30"
              :disabled="restarting"
              @click="$emit('restart')">
              {{ restarting ? 'Restarting...' : 'Restart Now' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
defineProps<{
  show: boolean
  currentVersion: number
  remoteVersion: number
  restarting: boolean
}>()

defineEmits<{
  dismiss: []
  restart: []
}>()
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.15s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
