<script setup lang="ts">
import { defineProps, defineEmits } from "vue";

export type AppView = "skills" | "surveying" | "data-browser" | "settings";

defineProps<{
  currentView: AppView;
}>();

const emit = defineEmits<{
  navigate: [view: AppView];
}>();
</script>

<template>
  <div class="menu-bar">
    <div class="menu-left">
      <span class="app-title">Glogger</span>
      <div class="menu-buttons">
        <button
          class="menu-button"
          :class="{ active: currentView === 'skills' }"
          @click="emit('navigate', 'skills')">
          Skills
        </button>
        <button
          class="menu-button"
          :class="{ active: currentView === 'surveying' }"
          @click="emit('navigate', 'surveying')">
          Surveying
        </button>
        <button
          class="menu-button"
          :class="{ active: currentView === 'data-browser' }"
          @click="emit('navigate', 'data-browser')">
          Data Browser
        </button>
      </div>
    </div>
    <div class="menu-right">
      <button
        class="settings-button"
        :class="{ active: currentView === 'settings' }"
        @click="emit('navigate', 'settings')"
        title="Settings">
        ⚙
      </button>
    </div>
  </div>
</template>

<style scoped>
.menu-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  background: #1a1a1a;
  border-bottom: 1px solid #333;
  margin: -1rem -1rem 1rem -1rem;
}

.menu-left {
  display: flex;
  align-items: center;
  gap: 1.5rem;
}

.app-title {
  color: #e0c060;
  font-weight: bold;
  font-size: 1.2rem;
}

.menu-buttons {
  display: flex;
  gap: 0.25rem;
}

.menu-button {
  padding: 0.5rem 1rem;
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  font-family: monospace;
  font-size: 0.95rem;
  border-radius: 4px;
  transition: all 0.2s;
}

.menu-button:hover {
  background: #252525;
  color: #aaa;
}

.menu-button.active {
  background: #2a2a2a;
  color: #e0c060;
}

.menu-right {
  display: flex;
  align-items: center;
}

.settings-button {
  padding: 0.4rem 0.75rem;
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  font-size: 1.2rem;
  border-radius: 4px;
  transition: all 0.2s;
  line-height: 1;
}

.settings-button:hover {
  background: #252525;
  color: #aaa;
}

.settings-button.active {
  background: #2a2a2a;
  color: #e0c060;
}
</style>
