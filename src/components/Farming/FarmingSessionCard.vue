<template>
  <!-- No active session -->
  <div v-if="!store.sessionActive" class="py-4 flex flex-col items-center gap-4">
    <EmptyState variant="compact" primary="No active farming session" secondary="Start one to track XP, items, favor, and more." />
    <div class="flex items-center gap-3">
      <input
        v-model="sessionName"
        type="text"
        placeholder="Session name (optional)"
        class="px-3 py-2 text-sm bg-[#1a1a2e] border border-border-light rounded text-text-primary placeholder-text-dim outline-none focus:border-entity-item"
      />
      <button
        @click="store.startSession(sessionName || undefined)"
        class="px-4! py-2! text-sm! bg-[#2a3a2a]! border border-[#4a5a4a]! text-[#8ec88e]! rounded cursor-pointer transition-all font-medium hover:bg-[#3a4a3a] hover:border-[#5a7a5a] hover:text-[#aedaae]">
        Start Session
      </button>
    </div>
  </div>

  <!-- Active session -->
  <template v-else-if="s">
    <!-- Session header bar -->
    <div class="bg-[#1a1a2e] border border-border-light rounded-lg p-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span
            :class="[
              'inline-block w-2 h-2 rounded-full',
              s.endTime ? 'bg-text-dim' : s.isPaused ? 'bg-[#c8b47e] animate-pulse' : 'bg-[#7ec87e] animate-pulse'
            ]" />
          <input
            :value="s.name"
            @change="store.updateName(($event.target as HTMLInputElement).value)"
            class="text-base font-bold text-entity-item bg-transparent border-none outline-none w-64 hover:bg-[#2a2a3e] focus:bg-[#2a2a3e] rounded px-1 -mx-1"
          />
          <span v-if="s.endTime" class="text-xs text-text-dim uppercase tracking-wide">(Ended)</span>
          <span v-if="s.isPaused" class="text-xs text-[#c8b47e] font-bold uppercase tracking-wide">(Paused)</span>
        </div>

        <div class="flex items-center gap-3">
          <!-- Live timer -->
          <span class="text-lg font-mono font-bold text-text-primary">{{ store.elapsed }}</span>

          <button
            v-if="!s.endTime"
            @click="store.togglePause"
            :class="[
              'px-3 py-1.5 text-xs border rounded cursor-pointer transition-all font-medium',
              s.isPaused
                ? 'bg-[#3a4a2a]! border-[#5a7a3a]! text-[#8ec88e]!'
                : 'bg-[#2a2a3e] border-border-light text-text-secondary hover:bg-[#3a3a4e] hover:text-text-primary'
            ]">
            {{ s.isPaused ? "Resume" : "Pause" }}
          </button>
          <button
            v-if="!s.endTime"
            @click="store.endSession"
            class="px-3 py-1.5 text-xs bg-[#3a2a2a]! border border-[#5a3a3a]! rounded text-[#c87e7e]! cursor-pointer transition-all font-medium hover:bg-[#4a3a3a] hover:border-[#6a4a4a]">
            End Session
          </button>
          <button
            @click="store.reset"
            class="px-3 py-1.5 text-xs bg-[#2a2a3a]! border border-[#4a4a5a]! rounded text-text-secondary cursor-pointer transition-all font-medium hover:bg-[#3a3a4e] hover:border-border-hover hover:text-text-primary">
            Reset
          </button>
        </div>
      </div>

      <!-- Timing + notes row -->
      <div class="flex items-start gap-4 mt-2">
        <span class="text-xs text-text-muted shrink-0 pt-1">
          Started {{ s.startTime }}
          <span v-if="s.endTime"> · Ended {{ s.endTime }}</span>
        </span>
        <textarea
          :value="s.notes"
          @change="store.updateNotes(($event.target as HTMLTextAreaElement).value)"
          placeholder="Session notes..."
          rows="1"
          class="flex-1 px-2 py-1 text-xs bg-[#12122a] border border-border-default rounded text-text-secondary placeholder-text-dim outline-none resize-y focus:border-entity-item"
        />
      </div>

      <!-- Quick stats -->
      <div class="flex gap-6 mt-2 flex-wrap">
        <div class="text-center">
          <span class="text-[0.6rem] text-text-muted uppercase tracking-wide">Total XP</span>
          <span class="text-sm font-bold text-text-primary ml-1">{{ store.totalXpGained.toLocaleString() }}</span>
        </div>
        <div class="text-center">
          <span class="text-[0.6rem] text-text-muted uppercase tracking-wide">Items +</span>
          <span class="text-sm font-bold text-[#7ec87e] ml-1">{{ store.totalItemsGained }}</span>
        </div>
        <div v-if="store.totalItemsLost > 0" class="text-center">
          <span class="text-[0.6rem] text-text-muted uppercase tracking-wide">Items -</span>
          <span class="text-sm font-bold text-[#c87e7e] ml-1">{{ store.totalItemsLost }}</span>
        </div>
        <div v-if="store.totalFavorGained !== 0" class="text-center">
          <span class="text-[0.6rem] text-text-muted uppercase tracking-wide">Favor</span>
          <span :class="['text-sm font-bold ml-1', store.totalFavorGained > 0 ? 'text-[#c8b47e]' : 'text-[#c87e7e]']">
            {{ store.totalFavorGained > 0 ? '+' : '' }}{{ store.totalFavorGained.toFixed(0) }}
          </span>
        </div>
        <div v-if="s.vendorGold > 0" class="text-center">
          <span class="text-[0.6rem] text-text-muted uppercase tracking-wide">Gold</span>
          <span class="text-sm font-bold text-[#d4af37] ml-1">{{ s.vendorGold.toLocaleString() }}g</span>
        </div>
      </div>
    </div>

    <!-- 3-column layout: Skills | Items | Activity Log -->
    <div class="grid grid-cols-[240px_1fr_280px] gap-3 flex-1 min-h-0">
      <!-- LEFT: Skills Panel -->
      <div class="bg-surface-dark border border-border-default rounded-lg p-3 overflow-y-auto">
        <div class="text-[0.65rem] uppercase tracking-widest text-entity-item mb-2 font-bold">Skills</div>
        <EmptyState v-if="store.skillSummary.length === 0" variant="compact" primary="No skill gains yet" />
        <div class="flex flex-col gap-1">
          <div
            v-for="skill in store.skillSummary"
            :key="skill.name"
            class="relative rounded overflow-hidden bg-black/30 border border-border-default">
            <!-- XP progress fill background -->
            <div
              class="absolute inset-0 bg-[#2a4a2a] transition-[width] duration-300"
              :style="{ width: skill.xpProgress + '%' }" />
            <!-- Content -->
            <div class="relative flex items-center justify-between px-2 py-1.5 z-10">
              <div class="flex items-center gap-1.5 min-w-0">
                <SkillInline :reference="skill.name" :show-icon="true" class="text-xs" />
                <span v-if="skill.levelsGained > 0" class="text-[0.6rem] text-[#c8b47e] font-bold">
                  +{{ skill.levelsGained }}lvl
                </span>
              </div>
              <div class="flex flex-col items-end shrink-0">
                <span class="text-xs font-bold text-[#7ec87e]">+{{ skill.gained.toLocaleString() }}</span>
                <span class="text-[0.55rem] text-text-dim">{{ skill.perHour.toLocaleString() }}/hr</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Favor section -->
        <template v-if="store.favorSummary.length > 0">
          <div class="text-[0.65rem] uppercase tracking-widest text-text-dim mt-3 mb-2 font-bold">Favor</div>
          <div class="flex flex-col gap-1">
            <div
              v-for="fav in store.favorSummary"
              :key="fav.name"
              class="flex items-center justify-between px-2 py-1.5 rounded text-xs bg-black/20 border border-border-default">
              <NpcInline :reference="fav.name" />
              <span
                :class="[
                  'font-mono font-bold',
                  fav.delta > 0 ? 'text-[#c8b47e]' : 'text-[#c87e7e]'
                ]">
                {{ fav.delta > 0 ? '+' : '' }}{{ fav.delta.toFixed(1) }}
              </span>
            </div>
          </div>
        </template>
      </div>

      <!-- CENTER: Items Panel -->
      <div class="bg-surface-dark border border-border-default rounded-lg p-3 overflow-y-auto">
        <div class="flex items-center justify-between mb-2">
          <div class="text-[0.65rem] uppercase tracking-widest text-text-dim font-bold">Items</div>
          <button
            v-if="hasIgnoredItems"
            @click="showIgnored = !showIgnored"
            class="text-[0.6rem] text-text-dim hover:text-text-secondary cursor-pointer transition-colors">
            {{ showIgnored ? 'Hide' : 'Show' }} ignored ({{ ignoredCount }})
          </button>
        </div>
        <EmptyState v-if="store.itemSummary.length === 0" variant="compact" primary="No item changes yet" />
        <div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-1.5">
          <div
            v-for="item in visibleItems"
            :key="item.name"
            :class="[
              'group flex items-center justify-between px-2 py-1.5 rounded text-xs border transition-opacity',
              item.isIgnored
                ? 'bg-black/10 border-border-default opacity-40'
                : 'bg-black/20 border-border-default'
            ]">
            <ItemInline :reference="item.name" />
            <div class="flex items-center gap-2">
              <span
                :class="[
                  'font-mono font-bold',
                  item.netQuantity > 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]'
                ]">
                {{ item.netQuantity > 0 ? '+' : '' }}{{ item.netQuantity }}
              </span>
              <span class="text-text-dim text-[0.6rem]">{{ item.perHour }}/hr</span>
              <button
                @click="store.toggleIgnoreItem(item.name)"
                :class="[
                  'opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer text-[0.65rem] px-1 rounded',
                  item.isIgnored
                    ? 'text-[#8ec88e] hover:text-[#aedaae]'
                    : 'text-text-dim hover:text-[#c87e7e]'
                ]"
                :title="item.isIgnored ? 'Show this item' : 'Hide this item'">
                {{ item.isIgnored ? '👁' : '✕' }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- RIGHT: Activity Log -->
      <FarmingLog />
    </div>
  </template>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useFarmingStore } from "../../stores/farmingStore";
import EmptyState from "../Shared/EmptyState.vue";
import ItemInline from "../Shared/Item/ItemInline.vue";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import NpcInline from "../Shared/NPC/NpcInline.vue";
import FarmingLog from "./FarmingLog.vue";

const store = useFarmingStore();
const s = computed(() => store.session);
const sessionName = ref("");
const showIgnored = ref(false);

const hasIgnoredItems = computed(() =>
  store.itemSummary.some((i) => i.isIgnored)
);

const ignoredCount = computed(() =>
  store.itemSummary.filter((i) => i.isIgnored).length
);

const visibleItems = computed(() =>
  showIgnored.value
    ? store.itemSummary
    : store.itemSummary.filter((i) => !i.isIgnored)
);
</script>
