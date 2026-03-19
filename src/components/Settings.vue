<template>
  <div class="max-w-225 p-4">
    <h2 class="text-accent-gold mt-0 mb-6 text-2xl">Settings</h2>

    <div class="flex gap-6 min-h-125">
      <nav class="flex flex-col gap-1 min-w-40 border-r border-border-default pr-4">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          class="px-4 py-2.5 bg-transparent border-none rounded text-text-secondary cursor-pointer font-mono text-sm text-left transition-all whitespace-nowrap hover:text-text-primary hover:bg-surface-base"
          :class="{ 'text-accent-gold! bg-surface-base! border-l-2 border-l-accent-gold pl-3.5': activeTab === tab.id }">
          {{ tab.label }}
        </button>
      </nav>

      <div class="flex-1 min-w-0">
        <GeneralSettings
          v-if="activeTab === 'general'" />

        <AppSettingsTab
          v-else-if="activeTab === 'app'" />

        <ChatLogsSettings
          v-else-if="activeTab === 'chat-logs'"
          :parsing="props.parsing"
          :error="props.error" />

        <NotificationsSettings
          v-else-if="activeTab === 'notifications'" />

        <UserDataSettings
          v-else-if="activeTab === 'user-data'" />

        <GameDataSettings
          v-else-if="activeTab === 'game-data'" />

        <AdvancedSettings
          v-else-if="activeTab === 'advanced'"
          :parsing="props.parsing"
          :error="props.error"
          :onParseLog="props.onParseLog" />
      </div>
    </div>

    <div class="settings-section mt-4 text-text-secondary text-sm">
      <p class="m-0">Settings are automatically saved to: <code class="text-accent-gold bg-surface-dark px-1.5 py-0.5 rounded-sm text-xs break-all">{{ settingsStore.settingsFilePath || 'Loading...' }}</code></p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useSettingsStore } from "../stores/settingsStore";
import GeneralSettings from "./Settings/GeneralSettings.vue";
import AppSettingsTab from "./Settings/AppSettingsTab.vue";
import ChatLogsSettings from "./Settings/ChatLogsSettings.vue";
import NotificationsSettings from "./Settings/NotificationsSettings.vue";
import UserDataSettings from "./Settings/UserDataSettings.vue";
import GameDataSettings from "./Settings/GameDataSettings.vue";
import AdvancedSettings from "./Settings/AdvancedSettings.vue";

const settingsStore = useSettingsStore();

const props = defineProps<{
  parsing: boolean;
  error: string;
  onParseLog: () => void;
}>();

type TabId = 'general' | 'app' | 'chat-logs' | 'notifications' | 'user-data' | 'game-data' | 'advanced';

const tabs: { id: TabId; label: string }[] = [
  { id: 'general', label: 'General' },
  { id: 'app', label: 'App Settings' },
  { id: 'chat-logs', label: 'Chat Logs' },
  { id: 'notifications', label: 'Notifications' },
  { id: 'user-data', label: 'User Data' },
  { id: 'game-data', label: 'Game Data' },
  { id: 'advanced', label: 'Advanced' },
];

const activeTab = ref<TabId>('general');
</script>
