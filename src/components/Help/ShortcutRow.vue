<template>
  <div class="flex items-center gap-4 px-3 py-2 rounded-lg hover:bg-surface-base/50 transition-colors group">
    <span class="shrink-0 min-w-28 flex items-center gap-1">
      <kbd
        v-for="(key, i) in keyParts"
        :key="i"
        class="kbd-key">
        {{ key }}
      </kbd>
    </span>
    <span class="text-sm text-text-muted group-hover:text-text-secondary transition-colors">{{ description }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  keys: string
  description: string
}>()

const keyParts = computed(() => props.keys.split('+').map(k => k.trim()))
</script>

<style scoped>
.kbd-key {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.75rem;
  padding: 0.2rem 0.5rem;
  font-size: 0.7rem;
  font-family: inherit;
  font-weight: 600;
  color: var(--color-accent-gold);
  background: var(--color-surface-dark);
  border: 1px solid var(--color-border-default);
  border-bottom-width: 2px;
  border-bottom-color: var(--color-border-light);
  border-radius: 0.375rem;
  box-shadow: 0 1px 0 var(--color-border-default);
  line-height: 1;
}

.kbd-key + .kbd-key::before {
  content: '+';
  position: absolute;
  margin-left: -0.6rem;
  color: var(--color-text-dim);
  font-weight: 400;
}

.kbd-key + .kbd-key {
  position: relative;
  margin-left: 0.75rem;
}
</style>
