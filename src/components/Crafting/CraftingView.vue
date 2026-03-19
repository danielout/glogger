<template>
  <div class="flex flex-col gap-4">
    <div class="flex gap-2 border-b-2 border-border-default pb-2">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="[
          'px-5 py-2 bg-[#1a1a2e] border border-border-light border-b-0 rounded-t text-text-secondary cursor-pointer text-sm font-medium transition-all relative hover:bg-[#2a2a3e] hover:text-text-secondary',
          activeTab === tab.id && 'bg-[#2a2a3e]! border-[#7ec8e3]! text-[#7ec8e3]! font-semibold after:content-[\'\'] after:absolute after:-bottom-0.5 after:left-0 after:right-0 after:h-0.5 after:bg-[#7ec8e3]'
        ]"
        @click="activeTab = tab.id">
        {{ tab.label }}
      </button>
    </div>

    <div class="flex-1">
      <QuickCalcTab v-if="activeTab === 'quick-calc'" />
      <ProjectsTab v-else-if="activeTab === 'projects'" />
      <LevelingTab v-else-if="activeTab === 'leveling'" />
      <HistoryTab v-else-if="activeTab === 'history'" />
      <WorkOrdersTab v-else-if="activeTab === 'work-orders'" />
      <CooksHelperTab v-else-if="activeTab === 'cooks-helper'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import QuickCalcTab from "./QuickCalcTab.vue";
import ProjectsTab from "./ProjectsTab.vue";
import LevelingTab from "./LevelingTab.vue";
import HistoryTab from "./HistoryTab.vue";
import WorkOrdersTab from "./WorkOrdersTab.vue";
import CooksHelperTab from "./CooksHelperTab.vue";

type TabId = "quick-calc" | "projects" | "leveling" | "history" | "work-orders" | "cooks-helper";

const tabs: { id: TabId; label: string }[] = [
  { id: "quick-calc", label: "Quick Calc" },
  { id: "projects", label: "Projects" },
  { id: "leveling", label: "Leveling" },
  { id: "history", label: "History" },
  { id: "work-orders", label: "Work Orders" },
  { id: "cooks-helper", label: "Cook's Helper" },
];

const activeTab = ref<TabId>("quick-calc");
</script>
