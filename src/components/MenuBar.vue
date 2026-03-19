<template>
  <div class="fixed top-0 left-0 right-0 z-50 flex justify-between items-center px-4 py-3 bg-surface-base border-b border-border-default">
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
      <div class="flex items-center gap-1.5">
        <span
          class="w-2 h-2 rounded-full"
          :class="isPlayerLogTailing ? 'bg-green-500' : 'bg-red-500'"
          :title="isPlayerLogTailing ? 'Player.log: tailing' : 'Player.log: not tailing'"
        />
        <span
          class="w-2 h-2 rounded-full"
          :class="isChatLogTailing ? 'bg-green-500' : 'bg-red-500'"
          :title="isChatLogTailing ? 'Chat log: tailing' : 'Chat log: not tailing'"
        />
      </div>
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
import { useCoordinatorStore } from "../stores/coordinatorStore";

export type AppView = "dashboard" | "skills" | "surveying" | "farming" | "crafting" | "character" | "inventory" | "data-browser" | "chat" | "gourmand" | "settings";

const settingsStore = useSettingsStore();
const coordinatorStore = useCoordinatorStore();
const activeCharacter = computed(() => settingsStore.settings.activeCharacterName);
const activeServer = computed(() => settingsStore.settings.activeServerName);
const isPlayerLogTailing = computed(() => coordinatorStore.isPlayerLogTailing);
const isChatLogTailing = computed(() => coordinatorStore.isChatLogTailing);

const navItems: { view: AppView; label: string }[] = [
  { view: "dashboard", label: "Dashboard" },
  { view: "skills", label: "Skills" },
  { view: "surveying", label: "Surveying" },
  { view: "farming", label: "Farming" },
  { view: "crafting", label: "Crafting" },
  { view: "inventory", label: "Inventory" },
  { view: "data-browser", label: "Data Browser" },
  { view: "chat", label: "Chat Logs" },
  { view: "gourmand", label: "Gourmand" },
];

defineProps<{
  currentView: AppView;
}>();

const emit = defineEmits<{
  navigate: [view: AppView];
}>();
</script>
