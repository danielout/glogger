<template>
  <div class="flex flex-col gap-4">
    <div class="flex gap-2 border-b-2 border-border-default pb-2">
      <button
        :class="[
          'px-5 py-2 bg-[#1a1a2e] border border-border-light border-b-0 rounded-t text-text-secondary cursor-pointer text-sm font-medium transition-all relative hover:bg-[#2a2a3e] hover:text-text-secondary',
          activeTab === 'session' && 'bg-[#2a2a3e]! border-[#7ec8e3]! text-[#7ec8e3]! font-semibold after:content-[\'\'] after:absolute after:-bottom-0.5 after:left-0 after:right-0 after:h-0.5 after:bg-[#7ec8e3]'
        ]"
        @click="activeTab = 'session'">
        Session
      </button>
      <button
        :class="[
          'px-5 py-2 bg-[#1a1a2e] border border-border-light border-b-0 rounded-t text-text-secondary cursor-pointer text-sm font-medium transition-all relative hover:bg-[#2a2a3e] hover:text-text-secondary',
          activeTab === 'historical' && 'bg-[#2a2a3e]! border-[#7ec8e3]! text-[#7ec8e3]! font-semibold after:content-[\'\'] after:absolute after:-bottom-0.5 after:left-0 after:right-0 after:h-0.5 after:bg-[#7ec8e3]'
        ]"
        @click="activeTab = 'historical'">
        Historical
      </button>
    </div>

    <div class="flex-1">
      <SessionTab v-if="activeTab === 'session'" />
      <HistoricalTab v-else-if="activeTab === 'historical'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import SessionTab from "./SessionTab.vue";
import HistoricalTab from "./HistoricalTab.vue";

const activeTab = ref<"session" | "historical">("session");
</script>
