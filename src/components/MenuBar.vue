<template>
  <div class="fixed top-0 left-0 right-0 z-50">
    <!-- Primary nav row -->
    <div class="grid grid-cols-[auto_1fr_auto] items-center px-4 py-2 bg-surface-base border-b border-border-default">
      <!-- Left: Identity block + Data Browser -->
      <div class="flex items-center gap-4">
        <div class="flex flex-col leading-tight">
          <div class="flex items-center gap-2">
            <component
              :is="settingsStore.settings.devModeEnabled ? 'button' : 'span'"
              class="text-accent-gold font-bold text-lg"
              :class="settingsStore.settings.devModeEnabled ? 'cursor-pointer hover:text-accent-gold-bright bg-transparent border-none p-0 font-bold transition-colors' : ''"
              :title="settingsStore.settings.devModeEnabled ? 'Open Dev Panel' : undefined"
              @click="settingsStore.settings.devModeEnabled && openDevPanel()"
            >glogger</component>
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
          </div>
          <CharacterPicker :isActive="currentView === 'character'" />
        </div>
        <div class="border-l border-border-default h-8" />
        <button
          class="px-3 py-1.5 bg-transparent border-none text-text-secondary cursor-pointer font-mono text-sm rounded transition-all hover:bg-surface-elevated hover:text-text-primary"
          :class="{ 'bg-surface-elevated! text-accent-gold!': dataBrowserStore.isOpen }"
          @click="dataBrowserStore.toggle()"
          title="Data Browser (Ctrl+D)">
          Data Browser
        </button>
      </div>

      <!-- Center: Navigation links -->
      <div class="flex justify-center gap-1">
        <button
          v-for="item in navItems"
          :key="item.view"
          class="px-3 py-1.5 bg-transparent border-none text-text-secondary cursor-pointer font-mono text-sm rounded transition-all hover:bg-surface-elevated hover:text-text-primary"
          :class="{ 'bg-surface-elevated! text-accent-gold!': currentView === item.view }"
          @click="emit('navigate', item.view)">
          {{ item.label }}
        </button>
      </div>

      <!-- Right: Search, Settings, Help -->
      <div class="flex items-center justify-end gap-1">
        <button
          class="px-3 py-1.5 bg-transparent border-none text-text-secondary cursor-pointer text-sm rounded transition-all leading-none hover:bg-surface-elevated hover:text-text-primary font-mono flex items-center gap-1.5"
          :class="{ 'bg-surface-elevated! text-accent-gold!': currentView === 'search' }"
          @click="emit('navigate', 'search')"
          title="Search (Ctrl+F for quick search)">
          Search
          <kbd class="text-[0.55rem] text-text-muted bg-surface-elevated border border-border-default rounded px-1 py-0.5 leading-none">Ctrl+F</kbd>
        </button>
        <button
          class="px-3 py-1.5 bg-transparent border-none text-text-secondary cursor-pointer text-xl rounded transition-all leading-none hover:bg-surface-elevated hover:text-text-primary"
          :class="{ 'bg-surface-elevated! text-accent-gold!': currentView === 'settings' }"
          @click="emit('navigate', 'settings')"
          title="Settings">
          ⚙
        </button>
        <button
          class="relative px-3 py-1.5 bg-transparent border-none text-text-secondary cursor-pointer text-xl rounded transition-all leading-none hover:bg-surface-elevated hover:text-text-primary"
          :class="{ 'bg-surface-elevated! text-accent-gold!': helpOpen }"
          @click="emit('toggleHelp')"
          :title="updateStore.updateAvailable ? `Update available: v${updateStore.latestVersion}` : 'Help'">
          ?
          <span
            v-if="updateStore.updateAvailable"
            class="absolute top-0.5 right-0.5 w-2 h-2 rounded-full bg-accent-blue animate-pulse"
          />
        </button>
      </div>
    </div>

    <!-- Sub-tab flyout row -->
    <div
      class="overflow-hidden bg-surface-base border-b border-border-default transition-all duration-250 ease-out"
      :class="currentTabs.length ? 'subtab-open' : 'subtab-closed'"
    >
      <div class="flex justify-center gap-1 px-4 py-1.5">
        <button
          v-for="tab in currentTabs"
          :key="tab.id"
          class="px-3 py-1 bg-transparent border-none text-text-secondary cursor-pointer font-mono text-xs rounded transition-all hover:bg-surface-elevated hover:text-text-primary"
          :class="{ 'text-accent-gold! bg-surface-elevated!': activeSubTabs[currentView] === tab.id }"
          @click="selectSubTab(tab.id)">
          {{ tab.label }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useCoordinatorStore } from "../stores/coordinatorStore";
import { useDataBrowserStore } from "../stores/dataBrowserStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useUpdateStore } from "../stores/updateStore";
import { useKeyboard } from "../composables/useKeyboard";
import { useDevPanel } from "../composables/useDevPanel";
import CharacterPicker from "./CharacterPicker.vue";

const settingsStore = useSettingsStore();
const updateStore = useUpdateStore();
const { openDevPanel } = useDevPanel();

export type AppView = "dashboard" | "character" | "inventory" | "crafting" | "economics" | "chat" | "data-browser" | "search" | "settings";

interface SubTab {
  id: string;
  label: string;
}

// All sub-tab definitions, keyed by view
const viewTabs: Partial<Record<AppView, SubTab[]>> = {
  character: [
    { id: "skills", label: "Skills" },
    { id: "stats", label: "Stats" },
    { id: "npcs", label: "NPCs" },
    { id: "quests", label: "Quests" },
    { id: "deaths", label: "Deaths" },
    { id: "gourmand", label: "Gourmand" },
    { id: "statehelm", label: "Statehelm" },
    { id: "build-planner", label: "Build Planner" },
    { id: "account", label: "Account" },
  ],
  inventory: [
    { id: "live", label: "Inventory" },
    { id: "storage", label: "Storage" },
    { id: "vaults", label: "Vaults" },
  ],
  crafting: [
    { id: "quick-calc", label: "Quick Calc" },
    { id: "projects", label: "Projects" },
    { id: "leveling", label: "Leveling" },
    { id: "history", label: "History" },
    { id: "work-orders", label: "Work Orders" },
    { id: "cooks-helper", label: "Cook's Helper" },
    { id: "brewery", label: "Brewery" },
    { id: "skills", label: "Skills" },
    { id: "dynamic-items", label: "Dynamic Items" },
  ],
  economics: [
    { id: "market", label: "Market Prices" },
    { id: "farming", label: "Farming" },
    { id: "surveying", label: "Surveying" },
    { id: "stall-tracker", label: "Stall Tracker" },
  ],
  chat: [
    { id: "search", label: "Search" },
    { id: "channels", label: "Channels" },
    { id: "tells", label: "Tells" },
    { id: "party", label: "Party" },
    { id: "nearby", label: "Nearby" },
    { id: "guild", label: "Guild" },
    { id: "system", label: "System" },
    { id: "all", label: "All Messages" },
    { id: "watchwords", label: "Watchwords" },
  ],
};

// Default sub-tab per view
const defaultSubTabs: Partial<Record<AppView, string>> = {
  character: "skills",
  inventory: "live",
  crafting: "quick-calc",
  economics: "market",
  chat: "search",
};

const coordinatorStore = useCoordinatorStore();
const dataBrowserStore = useDataBrowserStore();
const isPlayerLogTailing = computed(() => coordinatorStore.isPlayerLogTailing);
const isChatLogTailing = computed(() => coordinatorStore.isChatLogTailing);

const navItems: { view: AppView; label: string }[] = [
  { view: "dashboard", label: "Dashboard" },
  { view: "character", label: "Character" },
  { view: "inventory", label: "Inventory" },
  { view: "crafting", label: "Crafting" },
  { view: "economics", label: "Economics" },
  { view: "chat", label: "Chat Logs" },
];

const props = defineProps<{
  currentView: AppView;
  helpOpen?: boolean;
}>();

const emit = defineEmits<{
  navigate: [view: AppView];
  "update:subTab": [tab: string];
  toggleHelp: [];
}>();

// Track active sub-tab per view
const activeSubTabs = reactive<Record<string, string>>({});

// Initialize defaults
for (const [view, defaultTab] of Object.entries(defaultSubTabs)) {
  activeSubTabs[view] = defaultTab;
}

const currentTabs = computed(() => viewTabs[props.currentView] ?? []);

const hasTabs = computed(() => currentTabs.value.length > 0);

function selectSubTab(tabId: string) {
  activeSubTabs[props.currentView] = tabId;
  emit("update:subTab", tabId);
}

// Emit current sub-tab whenever the view changes (so the child view gets the right tab)
watch(() => props.currentView, (view) => {
  const tabs = viewTabs[view];
  if (tabs) {
    emit("update:subTab", activeSubTabs[view] ?? tabs[0].id);
  }
}, { immediate: true });

// Keyboard tab cycling for the sub-tab row
const tabIds = computed(() => currentTabs.value.map(t => t.id));
const activeTabRef = computed({
  get: () => activeSubTabs[props.currentView] ?? "",
  set: (val: string) => {
    activeSubTabs[props.currentView] = val;
    emit("update:subTab", val);
  },
});

useKeyboard({
  tabCycling: {
    tabs: tabIds,
    activeTab: activeTabRef,
  },
});

// Expose for parent to read current sub-tab
defineExpose({
  activeSubTabs,
  viewTabs,
  hasTabs,
});
</script>

<style scoped>
.subtab-open {
  max-height: 3rem;
  opacity: 1;
}

.subtab-closed {
  max-height: 0;
  opacity: 0;
  border-bottom-color: transparent;
}
</style>
