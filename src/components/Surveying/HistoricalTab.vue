<template>
  <PaneLayout screenKey="surveying-historical">
    <div class="flex flex-col gap-4 p-4 overflow-y-auto h-full">
    <div class="flex justify-between items-center">
      <h3 class="text-lg text-[#7ec8e3] m-0">Historical Survey Sessions</h3>
      <button @click="loadSessions" :disabled="loading"
        class="px-3 py-1.5 text-sm bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all">
        {{ loading ? "Loading..." : "Refresh" }}
      </button>
    </div>

    <div v-if="error" class="text-[#c87e7e] bg-[#2a1a1a] border border-[#5a3a3a] rounded p-3 text-sm">{{ error }}</div>

    <!-- Aggregate Stats -->
    <div v-if="sessions.length > 0" class="grid grid-cols-5 gap-3">
      <div class="bg-surface-card border border-border-default rounded p-3 text-center">
        <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Sessions</div>
        <div class="text-lg font-bold text-text-primary">{{ sessions.length }}</div>
      </div>
      <div class="bg-surface-card border border-border-default rounded p-3 text-center">
        <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Surveys</div>
        <div class="text-lg font-bold text-text-primary">{{ aggTotalSurveys }}</div>
      </div>
      <div class="bg-surface-card border border-border-default rounded p-3 text-center">
        <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Profit</div>
        <div :class="['text-lg font-bold', aggTotalProfit >= 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]']">
          {{ formatGold(aggTotalProfit) }}
        </div>
      </div>
      <div class="bg-surface-card border border-border-default rounded p-3 text-center">
        <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Avg Profit/Survey</div>
        <div :class="['text-lg font-bold', aggAvgProfitPerSurvey >= 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]']">
          {{ formatGold(aggAvgProfitPerSurvey) }}
        </div>
      </div>
      <div class="bg-surface-card border border-border-default rounded p-3 text-center">
        <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Best Session</div>
        <div class="text-lg font-bold text-[#d4af37]">
          {{ bestSessionProfitPerHour !== null ? formatGold(bestSessionProfitPerHour) + '/hr' : '\u2014' }}
        </div>
      </div>
    </div>

    <div v-if="sessions.length === 0 && !loading" class="text-text-dim italic text-center p-8">
      No historical sessions found. Complete a survey session to see it here.
    </div>

    <!-- Session List -->
    <div v-else class="flex flex-col gap-2">
      <div
        v-for="sess in sessions"
        :key="sess.id"
        :class="[
          'bg-[#1a1a2e] border border-border-light rounded-md px-4 py-3 transition-all',
          expandedId === sess.id ? 'border-[#7ec8e3]!' : 'cursor-pointer hover:bg-[#2a2a3e] hover:border-border-hover'
        ]">

        <!-- Session Summary Row (clickable) -->
        <div class="flex justify-between items-center cursor-pointer" @click="toggleExpand(sess.id)">
          <div class="flex items-center gap-3">
            <span class="text-text-dim text-xs">{{ expandedId === sess.id ? '\u25BC' : '\u25B6' }}</span>
            <input
              :value="sess.name"
              @click.stop
              @change="(e) => updateSessionField(sess, 'name', (e.target as HTMLInputElement).value)"
              class="text-sm text-text-primary font-medium bg-transparent border-none outline-none cursor-text hover:bg-white/5 rounded px-1 -mx-1 max-w-48"
            />
            <span class="text-xs text-text-dim">{{ formatDate(sess.start_time) }}</span>
            <span class="text-xs text-text-dim">{{ formatDuration(sess.elapsed_seconds) }}</span>
          </div>
          <div class="flex gap-6 items-center">
            <span v-if="sess.survey_types_used" class="text-xs text-text-dim max-w-64 truncate">{{ sess.survey_types_used }}</span>
            <span class="text-xs text-text-secondary">{{ sess.total_completions }} surveys</span>
            <span :class="['text-xs font-semibold', sessionEconomics(sess).profit >= 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]']">
              {{ formatGold(sessionEconomics(sess).profit) }}
            </span>
            <span class="text-xs text-text-dim">
              {{ formatGold(sessionEconomics(sess).profitPerHour) }}/hr
            </span>
          </div>
        </div>

        <!-- Expanded Detail -->
        <div v-if="expandedId === sess.id" class="mt-4 pt-4 border-t border-border-default" @click.stop>
          <div class="flex gap-4">
            <!-- Left: Stats sidebar (matching session view) -->
            <div class="w-52 shrink-0 flex flex-col gap-3">
              <!-- Stats -->
              <div class="bg-black/20 border border-border-default rounded-lg p-3">
                <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-2 font-bold">Stats</div>
                <div class="flex flex-col gap-1.5">
                  <div class="flex justify-between text-xs">
                    <span class="text-text-muted">Duration</span>
                    <span class="text-text-primary font-bold">{{ formatDuration(sess.elapsed_seconds) }}</span>
                  </div>
                  <div class="flex justify-between text-xs">
                    <span class="text-text-muted">Maps Crafted</span>
                    <span class="text-text-primary font-bold">{{ sess.maps_started }}</span>
                  </div>
                  <div class="flex justify-between text-xs">
                    <span class="text-text-muted">Completed</span>
                    <span class="text-text-primary font-bold">{{ sess.total_completions }}</span>
                  </div>
                </div>
              </div>

              <!-- XP -->
              <div v-if="sess.surveying_xp_gained || sess.mining_xp_gained || sess.geology_xp_gained" class="bg-black/20 border border-border-default rounded-lg p-3">
                <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-2 font-bold">XP Gained</div>
                <div class="flex flex-col gap-1.5">
                  <div v-if="sess.surveying_xp_gained" class="flex justify-between text-xs">
                    <span class="text-[#7ec87e]">Surveying</span>
                    <span class="font-bold text-[#7ec87e]">+{{ sess.surveying_xp_gained?.toLocaleString() }}</span>
                  </div>
                  <div v-if="sess.mining_xp_gained" class="flex justify-between text-xs">
                    <span class="text-[#c87e7e]">Mining</span>
                    <span class="font-bold text-[#c87e7e]">+{{ sess.mining_xp_gained?.toLocaleString() }}</span>
                  </div>
                  <div v-if="sess.geology_xp_gained" class="flex justify-between text-xs">
                    <span class="text-[#c8b47e]">Geology</span>
                    <span class="font-bold text-[#c8b47e]">+{{ sess.geology_xp_gained?.toLocaleString() }}</span>
                  </div>
                </div>
              </div>

              <!-- Economics -->
              <div class="bg-black/20 border border-border-default rounded-lg p-3">
                <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-2 font-bold">Economics</div>
                <div class="flex flex-col gap-1.5">
                  <div class="flex justify-between text-xs">
                    <span class="text-text-muted">Revenue</span>
                    <span class="font-bold text-[#d4af37]">{{ formatGold(sessionEconomics(sess).revenue) }}</span>
                  </div>
                  <div class="flex justify-between text-xs">
                    <span class="text-text-muted">Cost</span>
                    <span class="font-bold text-[#d4af37]">{{ formatGold(sessionEconomics(sess).cost) }}</span>
                  </div>
                  <div class="flex justify-between text-xs border-t border-[#2a2a3e] pt-1.5">
                    <span class="text-text-muted">Profit</span>
                    <span :class="['font-bold', sessionEconomics(sess).profit >= 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]']">
                      {{ sessionEconomics(sess).profit >= 0 ? '+' : '' }}{{ formatGold(sessionEconomics(sess).profit) }}
                    </span>
                  </div>
                  <div class="flex justify-between text-xs">
                    <span class="text-text-muted">Per Hour</span>
                    <span :class="['font-bold', sessionEconomics(sess).profitPerHour >= 0 ? 'text-[#d4af37]' : 'text-[#c87e7e]']">
                      {{ formatGold(sessionEconomics(sess).profitPerHour) }}/hr
                    </span>
                  </div>
                </div>
              </div>

              <!-- Notes -->
              <div class="bg-black/20 border border-border-default rounded-lg p-3">
                <div class="text-[0.65rem] uppercase tracking-widest text-text-dim mb-2 font-bold">Notes</div>
                <textarea
                  :value="sess.notes"
                  @change="(e) => updateSessionField(sess, 'notes', (e.target as HTMLTextAreaElement).value)"
                  class="w-full text-xs text-text-secondary bg-black/20 border border-border-default rounded p-1.5 resize-y min-h-12 outline-none focus:border-border-hover placeholder:text-text-dim"
                  placeholder="Add notes..."
                  rows="3"
                />
              </div>
            </div>

            <!-- Right: Loot breakdown -->
            <div class="flex-1 min-w-0">
              <div v-if="sessionLoot[sess.id]">
                <!-- Primary Loot Table -->
                <div v-if="primaryLoot(sess.id).length > 0" class="mb-4">
                  <div class="text-[0.65rem] uppercase tracking-widest text-text-dim mb-2 font-bold">
                    Survey Rewards
                    <span class="text-text-dim font-normal ml-2">{{ primaryLoot(sess.id).length }} unique items</span>
                  </div>
                  <div class="flex flex-col gap-1">
                    <div class="grid grid-cols-[1fr_60px_60px] gap-3 px-3 py-1 text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
                      <div>Item</div>
                      <div class="text-right">Total</div>
                      <div class="text-right">Drops</div>
                    </div>
                    <div
                      v-for="loot in primaryLoot(sess.id)"
                      :key="'p-' + loot.item_name"
                      class="grid grid-cols-[1fr_60px_60px] gap-3 px-3 py-1.5 text-xs bg-black/20 border border-border-default rounded hover:bg-black/30">
                      <div class="min-w-0"><ItemInline :reference="loot.item_name" /></div>
                      <div class="text-right font-mono text-text-primary">{{ loot.primary_quantity }}</div>
                      <div class="text-right font-mono text-text-secondary">{{ loot.times_received }}</div>
                    </div>
                  </div>
                </div>
                <div v-else class="text-text-dim italic text-xs mb-3">No primary loot recorded.</div>

                <!-- Speed Bonus Loot Table -->
                <div v-if="bonusLoot(sess.id).length > 0">
                  <div class="text-[0.65rem] uppercase tracking-widest text-[#c8b47e] mb-2 font-bold">
                    Speed Bonus
                    <span class="text-text-dim font-normal ml-2">{{ sess.speed_bonus_count }} procs</span>
                  </div>
                  <div class="flex flex-col gap-1">
                    <div class="grid grid-cols-[1fr_60px_60px] gap-3 px-3 py-1 text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
                      <div>Item</div>
                      <div class="text-right">Total</div>
                      <div class="text-right">Drops</div>
                    </div>
                    <div
                      v-for="loot in bonusLoot(sess.id)"
                      :key="'b-' + loot.item_name"
                      class="grid grid-cols-[1fr_60px_60px] gap-3 px-3 py-1.5 text-xs bg-black/20 border border-[#4a3a2a] rounded hover:bg-black/30">
                      <div class="min-w-0"><ItemInline :reference="loot.item_name" /></div>
                      <div class="text-right font-mono text-[#c8b47e]">{{ loot.bonus_quantity }}</div>
                      <div class="text-right font-mono text-text-secondary">{{ loot.times_received }}</div>
                    </div>
                  </div>
                </div>

                <!-- Maps Used -->
                <div v-if="sess.maps_used_summary" class="mt-4 pt-3 border-t border-border-default">
                  <div class="text-[0.65rem] uppercase tracking-widest text-text-dim mb-2 font-bold">Maps Used</div>
                  <div class="flex flex-wrap gap-2">
                    <span v-for="map in sess.maps_used_summary.split(', ')" :key="map"
                      class="px-2 py-1 text-xs bg-black/30 border border-border-default rounded text-text-secondary">
                      {{ map }}
                    </span>
                  </div>
                </div>
              </div>
              <div v-else>
                <div class="text-text-dim italic text-xs">Loading loot data...</div>
              </div>
            </div>
          </div>

          <!-- Delete button -->
          <div class="flex justify-end pt-2 border-t border-border-default">
            <button
              @click.stop="deleteSession(sess.id)"
              class="px-3 py-1 text-xs bg-[#3a2a2a]! border border-[#5a3a3a]! rounded text-[#c87e7e]! cursor-pointer transition-all font-medium hover:bg-[#4a3a3a] hover:border-[#6a4a4a]">
              Delete Session
            </button>
          </div>
        </div>
      </div>
    </div>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { HistoricalSession } from "../../types/database";
import ItemInline from "../Shared/Item/ItemInline.vue";
import PaneLayout from "../Shared/PaneLayout.vue";
import { useMarketStore } from "../../stores/marketStore";
import { formatDateTimeShort, formatDuration } from "../../composables/useTimestamp";

const marketStore = useMarketStore();

interface LootBreakdownEntry {
  item_name: string;
  item_id: number | null;
  total_quantity: number;
  primary_quantity: number;
  bonus_quantity: number;
  times_received: number;
  vendor_value: number;
}

const sessions = ref<HistoricalSession[]>([]);
const loading = ref(false);
const error = ref("");
const expandedId = ref<number | null>(null);
const sessionLoot = ref<Record<number, LootBreakdownEntry[]>>({});

/** Get effective price for a loot entry: market price if set, otherwise vendor value */
function getEffectivePrice(entry: LootBreakdownEntry): number {
  const market = marketStore.valuesByName[entry.item_name];
  if (market) return market.market_value;
  return entry.vendor_value;
}

/** Recompute revenue for a session from its loot data + current market prices */
function sessionRevenue(sessionId: number): number | null {
  const loot = sessionLoot.value[sessionId];
  if (!loot) return null;
  return loot.reduce((sum, entry) => sum + getEffectivePrice(entry) * entry.total_quantity, 0);
}

/** Get economics for a session: recomputed from loot if available, else stored values */
function sessionEconomics(sess: HistoricalSession) {
  const revenue = sessionRevenue(sess.id);
  if (revenue === null) {
    return {
      revenue: sess.total_revenue,
      cost: sess.total_cost,
      profit: sess.total_profit,
      profitPerHour: sess.profit_per_hour,
    };
  }
  const cost = sess.total_cost;
  const profit = revenue - cost;
  const hours = sess.elapsed_seconds / 3600;
  const profitPerHour = hours > 0 ? Math.round(profit / hours) : 0;
  return { revenue, cost, profit, profitPerHour };
}

const aggTotalSurveys = computed(() =>
  sessions.value.reduce((sum, s) => sum + s.total_completions, 0)
);

const aggTotalProfit = computed(() =>
  sessions.value.reduce((sum, s) => sum + sessionEconomics(s).profit, 0)
);

const aggAvgProfitPerSurvey = computed(() => {
  const total = aggTotalSurveys.value;
  if (total === 0) return 0;
  return Math.round(aggTotalProfit.value / total);
});

const bestSessionProfitPerHour = computed(() => {
  if (sessions.value.length === 0) return null;
  return Math.max(...sessions.value.map(s => sessionEconomics(s).profitPerHour));
});

onMounted(() => {
  loadSessions();
});

async function loadSessions() {
  loading.value = true;
  error.value = "";
  try {
    sessions.value = await invoke<HistoricalSession[]>("get_historical_sessions", { limit: 50 });
    // Load loot for all sessions so we can recompute economics with market prices
    await Promise.all(
      sessions.value.map(async (sess) => {
        if (!sessionLoot.value[sess.id]) {
          try {
            sessionLoot.value[sess.id] = await invoke<LootBreakdownEntry[]>("get_loot_breakdown", {
              sessionId: sess.id,
              limit: 200,
            });
          } catch {
            sessionLoot.value[sess.id] = [];
          }
        }
      })
    );
  } catch (e) {
    error.value = `Failed to load sessions: ${e}`;
  } finally {
    loading.value = false;
  }
}

async function toggleExpand(id: number) {
  if (expandedId.value === id) {
    expandedId.value = null;
    return;
  }
  expandedId.value = id;

  if (!sessionLoot.value[id]) {
    try {
      const loot = await invoke<LootBreakdownEntry[]>("get_loot_breakdown", {
        sessionId: id,
        limit: 50,
      });
      sessionLoot.value[id] = loot;
    } catch (e) {
      console.error(`Failed to load loot for session ${id}:`, e);
      sessionLoot.value[id] = [];
    }
  }
}

async function updateSessionField(sess: HistoricalSession, field: 'name' | 'notes', value: string) {
  sess[field] = value;
  try {
    await invoke("update_survey_session", {
      sessionId: sess.id,
      name: sess.name,
      notes: sess.notes,
    });
  } catch (e) {
    console.error(`Failed to update session ${field}:`, e);
  }
}

function primaryLoot(sessionId: number): LootBreakdownEntry[] {
  const loot = sessionLoot.value[sessionId];
  if (!loot) return [];
  return loot
    .filter(l => l.primary_quantity > 0)
    .sort((a, b) => b.primary_quantity - a.primary_quantity);
}

function bonusLoot(sessionId: number): LootBreakdownEntry[] {
  const loot = sessionLoot.value[sessionId];
  if (!loot) return [];
  return loot
    .filter(l => l.bonus_quantity > 0)
    .sort((a, b) => b.bonus_quantity - a.bonus_quantity);
}

function formatGold(amount: number): string {
  const rounded = Math.round(amount);
  if (rounded >= 0) {
    return rounded.toLocaleString() + "g";
  }
  return "-" + Math.abs(rounded).toLocaleString() + "g";
}

async function deleteSession(id: number) {
  try {
    await invoke("delete_survey_session", { sessionId: id });
    sessions.value = sessions.value.filter((s) => s.id !== id);
    if (expandedId.value === id) expandedId.value = null;
    delete sessionLoot.value[id];
  } catch (e) {
    console.error("[surveying] Failed to delete session:", e);
  }
}

function formatDate(dateStr: string): string {
  return formatDateTimeShort(dateStr)
}


</script>
