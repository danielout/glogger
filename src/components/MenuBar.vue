<template>
  <div class="flex justify-between items-center px-4 py-3 bg-surface-base border-b border-border-default -m-4 mb-4">
    <div class="flex items-center gap-6">
      <span class="text-accent-gold font-bold text-xl">Glogger</span>
      <div class="flex gap-1">
        <button
          v-for="item in navItems"
          :key="item.view"
          class="px-4 py-2 bg-transparent border-none text-text-secondary cursor-pointer font-mono text-[0.95rem] rounded transition-all hover:bg-surface-elevated hover:text-text-primary"
          :class="{ 'bg-surface-elevated! text-accent-gold!': currentView === item.view }"
          @click="emit('navigate', item.view)">
          {{ item.label }}
        </button>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <button
        v-if="activeCharacter"
        class="px-3 py-1.5 bg-transparent border-none text-text-secondary text-sm font-mono cursor-pointer rounded transition-all hover:bg-surface-elevated"
        :class="{ 'bg-surface-elevated! text-accent-gold!': currentView === 'character' }"
        @click="emit('navigate', 'character')"
        title="View character summary">
        <span class="text-text-primary">{{ activeCharacter }}</span>
        <span class="text-text-muted ml-1">{{ activeServer }}</span>
      </button>
      <button
        class="px-3 py-1.5 bg-transparent border-none text-text-secondary cursor-pointer text-xl rounded transition-all leading-none hover:bg-surface-elevated hover:text-text-primary"
        :class="{ 'bg-surface-elevated! text-accent-gold!': currentView === 'settings' }"
        @click="emit('navigate', 'settings')"
        title="Settings">
        ⚙
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, computed } from "vue";
import { useSettingsStore } from "../stores/settingsStore";

export type AppView = "skills" | "surveying" | "character" | "inventory" | "data-browser" | "chat" | "settings";

const settingsStore = useSettingsStore();
const activeCharacter = computed(() => settingsStore.settings.activeCharacterName);
const activeServer = computed(() => settingsStore.settings.activeServerName);

const navItems: { view: AppView; label: string }[] = [
  { view: "skills", label: "Skills" },
  { view: "surveying", label: "Surveying" },
  { view: "inventory", label: "Inventory" },
  { view: "data-browser", label: "Data Browser" },
  { view: "chat", label: "Chat Logs" },
];

defineProps<{
  currentView: AppView;
}>();

const emit = defineEmits<{
  navigate: [view: AppView];
}>();
</script>
