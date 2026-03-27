<template>
  <div v-if="s" class="w-56 shrink-0 flex flex-col gap-3">
    <!-- Status + Controls -->
    <div class="bg-[#1a1a2e] border border-border-light rounded-lg p-3">
      <div class="flex items-center gap-2 mb-2">
        <span
          :class="[
            'inline-block w-2 h-2 rounded-full',
            s.endTime ? 'bg-text-dim' : s.isPaused ? 'bg-[#c8b47e] animate-pulse' : 'bg-[#7ec87e] animate-pulse'
          ]" />
        <input
          :value="s.name"
          @change="(e) => store.updateName((e.target as HTMLInputElement).value)"
          class="text-sm font-bold text-[#7ec8e3] bg-transparent border-none outline-none w-full cursor-text hover:bg-white/5 rounded px-1 -mx-1"
          placeholder="Session name..."
        />
        <span v-if="s.endTime" class="text-[0.6rem] text-text-dim uppercase shrink-0">(Ended)</span>
      </div>
      <div class="text-[0.65rem] text-text-muted mb-2">
        Started {{ s.startTime }}
        <span v-if="s.endTime"> · Ended {{ s.endTime }}</span>
        <br />{{ store.elapsed }} elapsed
        <span v-if="s.isPaused" class="text-[#c8b47e] font-bold ml-1">(PAUSED)</span>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-if="!s.endTime"
          @click="store.togglePause"
          :class="[
            'px-2 py-1 text-[0.65rem] bg-[#2a2a3e] border border-border-light rounded text-text-secondary cursor-pointer transition-all font-medium hover:bg-[#3a3a4e] hover:text-text-primary',
            s.isPaused && 'bg-[#3a4a2a]! border-[#5a7a3a]! text-[#8ec88e]!'
          ]">
          {{ s.isPaused ? "Resume" : "Pause" }}
        </button>
        <button
          v-if="!s.endTime"
          @click="store.manualEnd"
          class="px-2 py-1 text-[0.65rem] bg-[#3a2a2a]! border border-[#5a3a3a]! rounded text-[#c87e7e]! cursor-pointer transition-all font-medium hover:bg-[#4a3a3a]">
          End
        </button>
        <button @click="store.reset" class="px-2 py-1 text-[0.65rem] bg-[#2a2a3a]! border border-[#4a4a5a]! rounded text-text-secondary cursor-pointer transition-all font-medium hover:bg-[#3a3a4e] hover:text-text-primary">
          Reset
        </button>
      </div>
      <div v-if="s.manualMode" class="text-[0.6rem] text-[#7ec8e3] uppercase tracking-wide mt-1">Manual Mode</div>
      <textarea
        :value="s.notes"
        @change="(e) => store.updateNotes((e.target as HTMLTextAreaElement).value)"
        class="mt-2 w-full text-xs text-text-secondary bg-black/20 border border-border-default rounded p-1.5 resize-y min-h-12 outline-none focus:border-border-hover placeholder:text-text-dim"
        placeholder="Session notes..."
        rows="2"
      />
    </div>

    <!-- Stats -->
    <div class="bg-[#1a1a2e] border border-border-light rounded-lg p-3">
      <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-2 font-bold">Stats</div>
      <div class="flex flex-col gap-2">
        <div class="flex justify-between text-xs">
          <span class="text-text-muted">Maps Crafted</span>
          <span class="text-text-primary font-bold">{{ s.mapsStarted }}</span>
        </div>
        <div class="flex justify-between text-xs">
          <span class="text-text-muted">Completed</span>
          <span class="text-text-primary font-bold">{{ s.surveysCompleted }}</span>
        </div>
        <div class="flex justify-between text-xs">
          <span class="text-text-muted">Avg Time</span>
          <span class="text-text-primary font-bold">{{ store.avgSurveyTime }}</span>
        </div>
      </div>
    </div>

    <!-- XP -->
    <div class="bg-[#1a1a2e] border border-border-light rounded-lg p-3">
      <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-2 font-bold">XP Gained</div>
      <div class="flex flex-col gap-1.5">
        <div class="flex flex-col gap-0.5">
          <div class="flex justify-between text-xs">
            <span class="text-[#7ec87e]">Surveying</span>
            <span class="font-bold text-[#7ec87e]">+{{ s.surveyingXpGained.toLocaleString() }}</span>
          </div>
          <div v-if="store.surveysToLevelSurveying != null" class="text-[0.6rem] text-text-dim text-right">
            ~{{ store.surveysToLevelSurveying }} crafts to level
          </div>
        </div>
        <div class="flex flex-col gap-0.5">
          <div class="flex justify-between text-xs">
            <span class="text-[#c87e7e]">Mining</span>
            <span class="font-bold text-[#c87e7e]">+{{ s.miningXpGained.toLocaleString() }}</span>
          </div>
          <div v-if="store.surveysToLevelMining != null" class="text-[0.6rem] text-text-dim text-right">
            ~{{ store.surveysToLevelMining }} completions to level
          </div>
        </div>
        <div class="flex flex-col gap-0.5">
          <div class="flex justify-between text-xs">
            <span class="text-[#c8b47e]">Geology</span>
            <span class="font-bold text-[#c8b47e]">+{{ s.geologyXpGained.toLocaleString() }}</span>
          </div>
          <div v-if="store.surveysToLevelGeology != null" class="text-[0.6rem] text-text-dim text-right">
            ~{{ store.surveysToLevelGeology }} completions to level
          </div>
        </div>
      </div>
    </div>

    <!-- Economics -->
    <div v-if="store.totalValue > 0 || store.totalCost > 0" class="bg-[#1a1a2e] border border-border-light rounded-lg p-3">
      <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-2 font-bold">Economics</div>
      <div class="flex flex-col gap-1.5">
        <div class="flex justify-between text-xs">
          <span class="text-text-muted">Revenue</span>
          <span class="font-bold text-[#d4af37]">{{ store.totalValue.toLocaleString() }}g</span>
        </div>
        <div class="flex justify-between text-xs">
          <span class="text-text-muted">Cost</span>
          <span class="font-bold text-[#d4af37]">{{ store.totalCost.toLocaleString() }}g</span>
        </div>
        <div class="flex justify-between text-xs border-t border-[#2a2a3e] pt-1.5">
          <span class="text-text-muted">Profit</span>
          <span :class="['font-bold', store.totalProfit < 0 ? 'text-[#c87e7e]' : 'text-[#7ec87e]']">
            {{ store.totalProfit >= 0 ? '+' : '' }}{{ store.totalProfit.toLocaleString() }}g
          </span>
        </div>
        <div class="flex justify-between text-xs">
          <span class="text-text-muted">Per Survey</span>
          <span :class="['font-bold', store.profitPerSurvey < 0 ? 'text-[#c87e7e]' : 'text-[#d4af37]']">
            {{ store.profitPerSurvey >= 0 ? '+' : '' }}{{ store.profitPerSurvey.toLocaleString() }}g
          </span>
        </div>
        <div class="flex justify-between text-xs">
          <span class="text-text-muted">Per Hour</span>
          <span :class="['font-bold', store.profitPerHour < 0 ? 'text-[#c87e7e]' : 'text-[#d4af37]']">
            {{ store.profitPerHour >= 0 ? '+' : '' }}{{ store.profitPerHour.toLocaleString() }}g
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useSurveyStore } from "../../stores/surveyStore";

const store = useSurveyStore();
const s = computed(() => store.session);
</script>
