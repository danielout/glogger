<template>
  <div class="flex flex-col gap-4">
    <div class="flex justify-between items-center mb-2">
      <h3 class="text-lg text-[#7ec8e3] m-0">Historical Survey Sessions</h3>
      <button @click="loadSessions" :disabled="loading" class="btn btn-secondary">
        {{ loading ? "Loading..." : "Refresh" }}
      </button>
    </div>

    <div v-if="error" class="text-[#c87e7e] bg-[#2a1a1a] border border-[#5a3a3a] rounded p-3 text-sm">{{ error }}</div>

    <div v-if="sessions.length === 0 && !loading" class="text-text-dim italic text-center p-8">
      No historical sessions found. Complete a survey session to see it here.
    </div>

    <div v-else class="flex flex-col gap-2">
      <div
        v-for="session in sessions"
        :key="session.id"
        :class="[
          'bg-[#1a1a2e] border border-border-light rounded-md px-4 py-3 cursor-pointer transition-all hover:bg-[#2a2a3e] hover:border-border-hover',
          expandedId === session.id && 'border-[#7ec8e3]!'
        ]"
        @click="toggleExpand(session.id)">
        <div class="flex justify-between items-center">
          <div class="text-sm text-text-primary font-medium">
            {{ formatDate(session.start_time) }}
          </div>
          <div class="flex gap-6">
            <span class="text-xs text-text-secondary">{{ session.surveys_completed }} surveys</span>
            <span class="text-xs text-text-secondary">{{ session.total_profit }}g profit</span>
          </div>
        </div>

        <div v-if="expandedId === session.id" class="mt-4 pt-4 border-t border-border-default">
          <div class="grid grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-3">
            <div class="flex flex-col gap-1">
              <span class="text-[0.65rem] text-text-muted uppercase tracking-wide">Duration:</span>
              <span class="text-sm text-text-primary font-medium">{{ formatDuration(session.start_time, session.end_time) }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-[0.65rem] text-text-muted uppercase tracking-wide">Maps Started:</span>
              <span class="text-sm text-text-primary font-medium">{{ session.maps_started }}</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-[0.65rem] text-text-muted uppercase tracking-wide">Revenue:</span>
              <span class="text-sm text-text-primary font-medium text-[#8ec88e]!">{{ session.total_revenue }}g</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-[0.65rem] text-text-muted uppercase tracking-wide">Cost:</span>
              <span class="text-sm text-text-primary font-medium text-[#c87e7e]!">{{ session.total_cost }}g</span>
            </div>
            <div class="flex flex-col gap-1">
              <span class="text-[0.65rem] text-text-muted uppercase tracking-wide">Profit/Hour:</span>
              <span class="text-sm text-text-primary font-medium">{{ session.profit_per_hour }}g/hr</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { HistoricalSession } from "../../types/database";

const sessions = ref<HistoricalSession[]>([]);
const loading = ref(false);
const error = ref("");
const expandedId = ref<number | null>(null);

onMounted(() => {
  loadSessions();
});

async function loadSessions() {
  loading.value = true;
  error.value = "";
  try {
    sessions.value = await invoke<HistoricalSession[]>("get_historical_sessions");
  } catch (e) {
    error.value = `Failed to load sessions: ${e}`;
  } finally {
    loading.value = false;
  }
}

function toggleExpand(id: number) {
  expandedId.value = expandedId.value === id ? null : id;
}

function formatDate(dateStr: string): string {
  // Expected format: "YYYY-MM-DD HH:MM:SS"
  const date = new Date(dateStr);
  return date.toLocaleDateString() + " " + date.toLocaleTimeString();
}

function formatDuration(start: string, end: string | null): string {
  if (!end) return "\u2014";
  const startDate = new Date(start);
  const endDate = new Date(end);
  const diffMs = endDate.getTime() - startDate.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const hours = Math.floor(diffMins / 60);
  const mins = diffMins % 60;
  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins}m`;
}
</script>
