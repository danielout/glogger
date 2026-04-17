<template>
  <div v-if="!store.sessionActive" class="py-4 flex flex-col items-center gap-4">
    <div class="text-text-dim italic">
      No active survey session. Start watching a log or parse a file.
    </div>
    <button @click="store.manualStart" class="px-4! py-2! text-sm! bg-[#2a3a2a]! border border-[#4a5a4a]! text-[#8ec88e]! rounded cursor-pointer transition-all font-medium hover:bg-[#3a4a3a] hover:border-[#5a7a5a] hover:text-[#aedaae]">
      Start Manual Session
    </button>
  </div>

  <div v-else>
    <!-- Warning: backend session not created (only for auto-started sessions that received events) -->
    <div v-if="store.sessionActive && !store.backendSessionId && !store.session?.manualMode && (store.session?.mapsStarted ?? 0) > 0" class="mb-3 flex items-center gap-3 bg-[#2a1a1a] border border-[#5a3a3a] rounded-lg px-4 py-2">
      <span class="text-xs text-[#c87e7e]">Warning: Session data is not being saved to disk. Try re-importing the Player.log from Settings > Advanced after your session.</span>
    </div>

    <!-- Session ended banner with New Session button -->
    <div v-if="store.sessionEnded" class="mb-3 flex items-center gap-3 bg-[#1a2a1a] border border-[#3a5a3a] rounded-lg px-4 py-2">
      <span class="text-xs text-text-dim">Session ended.</span>
      <button @click="store.newSession" class="px-3 py-1.5 text-xs bg-[#2a3a2a] border border-[#4a5a4a] text-[#8ec88e] rounded cursor-pointer transition-all font-medium hover:bg-[#3a4a3a] hover:border-[#5a7a5a] hover:text-[#aedaae]">
        New Session
      </button>
    </div>

    <div class="flex gap-4">
      <!-- Left Sidebar: Stats / Economics / XP -->
      <SessionSidebar />

      <!-- Center: Type Breakdown + Speed Bonus -->
      <div class="flex-1 min-w-0 flex flex-col gap-4">
        <SurveyTypeAccordion />

        <!-- Speed Bonus (global, not per-type) -->
        <div v-if="store.speedBonusLootSummary.length > 0" class="bg-[#1a1a2e] border border-border-light rounded-lg p-4">
          <SurveyLootGrid
            :items="store.speedBonusLootSummary"
            title="Speed Bonus"
            title-class="text-[#c8b47e]"
          />
        </div>

        <!-- Crafting Materials -->
        <div v-if="hasCraftingMaterials" class="bg-[#1a1a2e] border border-border-light rounded-lg p-4">
          <div class="text-[0.65rem] uppercase tracking-widest text-text-dim mb-2 font-bold">Crafting Materials Consumed</div>
          <div class="flex flex-wrap gap-2">
            <div
              v-for="(qty, name) in store.session!.craftingMaterials"
              :key="name"
              class="bg-black/20 border border-border-default rounded px-2.5 py-1.5 text-xs"
            >
              <ItemInline :reference="String(name)" :show-icon="false" />
              <span class="text-text-secondary ml-1">&times;{{ qty }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Sidebar: Collapsible Activity Log -->
      <div class="shrink-0" :class="logExpanded ? 'w-72' : 'w-8'">
        <div class="bg-surface-dark border border-border-default rounded-lg overflow-hidden h-full">
          <button
            @click="logExpanded = !logExpanded"
            class="w-full flex items-center gap-2 px-2 py-2 text-[0.7rem] uppercase tracking-widest text-text-dim cursor-pointer hover:text-text-secondary transition-colors"
            :class="!logExpanded && 'justify-center'"
          >
            <span v-if="logExpanded" class="flex-1 text-left">Activity Log</span>
            <span class="text-xs">{{ logExpanded ? '▶' : '◀' }}</span>
          </button>
          <div v-if="logExpanded" class="px-3 pb-3 max-h-[calc(100vh-12rem)] overflow-y-auto">
            <SurveyLog :embedded="true" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useSurveyStore } from "../../stores/surveyStore";
import SessionSidebar from "./SessionSidebar.vue";
import SurveyTypeAccordion from "./SurveyTypeAccordion.vue";
import SurveyLootGrid from "./SurveyLootGrid.vue";
import SurveyLog from "./SurveyLog.vue";
import ItemInline from "../Shared/Item/ItemInline.vue";

const store = useSurveyStore();
const logExpanded = ref(true);

const hasCraftingMaterials = computed(() => {
  if (!store.session) return false;
  return Object.keys(store.session.craftingMaterials).length > 0;
});
</script>
