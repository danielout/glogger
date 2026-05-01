<template>
  <div class="flex flex-col gap-4">
    <div v-if="loading" class="space-y-3">
      <div class="flex gap-6">
        <SkeletonLoader v-for="i in 4" :key="i" variant="rect" width="w-20" height="h-12" />
      </div>
      <SkeletonLoader variant="text" :lines="4" />
    </div>
    <div v-else-if="error" class="text-[#c87e7e] text-sm">{{ error }}</div>
    <EmptyState v-else-if="sessions.length === 0" variant="panel" primary="No saved farming sessions" secondary="Complete a farming session to see history here." />

    <template v-else>
      <!-- Aggregate stats -->
      <div class="flex gap-6 flex-wrap text-center">
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Sessions</div>
          <div class="text-lg font-bold text-text-primary">{{ sessions.length }}</div>
        </div>
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Time</div>
          <div class="text-lg font-bold text-text-primary">{{ formatDuration(totalElapsed) }}</div>
        </div>
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total XP</div>
          <div class="text-lg font-bold text-[#7ec87e]">{{ totalXp.toLocaleString() }}</div>
        </div>
        <div v-if="totalGold > 0">
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Vendor Gold</div>
          <div class="text-lg font-bold text-[#d4af37]">{{ totalGold.toLocaleString() }}g</div>
        </div>
      </div>

      <!-- Session list -->
      <div class="flex flex-col gap-2">
        <div
          v-for="session in sessions"
          :key="session.id"
          class="bg-[#1a1a2e] border border-border-light rounded-lg overflow-hidden">
          <!-- Summary row -->
          <div
            class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-[#2a2a3e] transition-colors"
            @click="toggleExpanded(session.id)">
            <div class="flex items-center gap-3">
              <span class="text-xs text-text-dim">{{ formatDate(session.created_at) }}</span>
              <span class="text-sm font-bold text-entity-item">{{ session.name }}</span>
              <span class="text-xs text-text-muted">{{ formatDuration(session.elapsed_seconds) }}</span>
            </div>
            <div class="flex items-center gap-4 text-xs">
              <span v-if="sessionTotalXp(session) > 0" class="text-[#7ec87e]">
                +{{ sessionTotalXp(session).toLocaleString() }} XP
              </span>
              <span v-if="sessionTotalXp(session) > 0" class="text-text-dim">
                {{ xpPerHour(session).toLocaleString() }}/hr
              </span>
              <span v-if="session.items.length > 0" class="text-text-secondary">
                {{ session.items.length }} item{{ session.items.length !== 1 ? 's' : '' }}
              </span>
              <span v-if="sessionTotalKills(session) > 0" class="text-[#e87e7e]">
                {{ sessionTotalKills(session) }} kill{{ sessionTotalKills(session) !== 1 ? 's' : '' }}
              </span>
              <span v-if="session.vendor_gold > 0" class="text-[#d4af37]">
                {{ session.vendor_gold.toLocaleString() }}g
              </span>
              <span class="text-text-dim">{{ expanded.has(session.id) ? '\u25B2' : '\u25BC' }}</span>
            </div>
          </div>

          <!-- Expanded detail -->
          <div v-if="expanded.has(session.id)" class="border-t border-border-default px-4 py-3">
            <!-- Editable name + notes -->
            <div class="flex items-start gap-3 mb-3">
              <div class="flex flex-col gap-1 flex-1">
                <input
                  :value="session.name"
                  @change="updateSession(session, 'name', ($event.target as HTMLInputElement).value)"
                  class="text-sm font-bold text-entity-item bg-transparent border-none outline-none w-full hover:bg-[#2a2a3e] focus:bg-[#2a2a3e] rounded px-1 -mx-1"
                />
                <textarea
                  :value="session.notes"
                  @change="updateSession(session, 'notes', ($event.target as HTMLTextAreaElement).value)"
                  placeholder="Add notes..."
                  rows="2"
                  class="w-full px-2 py-1 text-xs bg-[#12122a] border border-border-default rounded text-text-secondary placeholder-text-dim outline-none resize-y focus:border-entity-item"
                />
              </div>
            </div>

            <!-- Skills -->
            <div v-if="session.skills.length > 0" class="mb-3">
              <div class="text-[0.6rem] uppercase tracking-widest text-entity-item mb-1 font-bold">Skills</div>
              <div class="flex gap-2 flex-wrap">
                <div
                  v-for="skill in session.skills"
                  :key="skill.skill_name"
                  class="flex items-center gap-2 px-3 py-1.5 rounded text-xs bg-[#1a2e1a] border border-[#3a5a3a]">
                  <SkillInline :reference="skill.skill_name" />
                  <span class="text-[#7ec87e] font-bold">+{{ skill.xp_gained.toLocaleString() }}</span>
                  <span class="text-text-dim text-[0.6rem]">{{ skillXpPerHour(skill.xp_gained, session.elapsed_seconds).toLocaleString() }}/hr</span>
                  <span v-if="skill.levels_gained > 0" class="text-[#c8b47e] font-bold">(+{{ skill.levels_gained }} lvl)</span>
                </div>
              </div>
            </div>

            <!-- Items -->
            <div v-if="session.items.length > 0" class="mb-3">
              <div class="text-[0.6rem] uppercase tracking-widest text-text-dim mb-1 font-bold">Items</div>
              <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-1">
                <div
                  v-for="item in session.items"
                  :key="item.item_name"
                  class="flex items-center justify-between px-2 py-1 rounded text-xs bg-black/20 border border-border-default">
                  <ItemInline :reference="item.item_name" />
                  <div class="flex items-center gap-2">
                    <span
                      :class="[
                        'font-mono font-bold',
                        item.net_quantity > 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]'
                      ]">
                      {{ item.net_quantity > 0 ? '+' : '' }}{{ item.net_quantity }}
                    </span>
                    <span class="text-text-dim text-[0.6rem]">{{ itemPerHour(item.net_quantity, session.elapsed_seconds) }}/hr</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Favors -->
            <div v-if="session.favors.length > 0" class="mb-3">
              <div class="text-[0.6rem] uppercase tracking-widest text-text-dim mb-1 font-bold">Favor</div>
              <div class="flex gap-2 flex-wrap">
                <div
                  v-for="fav in session.favors"
                  :key="fav.npc_name"
                  class="px-3 py-1.5 rounded text-xs bg-black/20 border border-border-default">
                  <NpcInline :reference="fav.npc_name" />
                  <span
                    :class="[
                      'font-mono font-bold ml-1',
                      fav.delta > 0 ? 'text-[#c8b47e]' : 'text-[#c87e7e]'
                    ]">
                    {{ fav.delta > 0 ? '+' : '' }}{{ fav.delta.toFixed(1) }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Kills -->
            <div v-if="session.kills && session.kills.length > 0" class="mb-3">
              <div class="text-[0.6rem] uppercase tracking-widest text-[#e87e7e] mb-1 font-bold">Kills</div>
              <div class="flex gap-2 flex-wrap">
                <div
                  v-for="kill in session.kills"
                  :key="kill.enemy_name"
                  class="flex items-center gap-2 px-3 py-1.5 rounded text-xs bg-black/20 border border-border-default">
                  <EnemyInline :reference="kill.enemy_name" />
                  <span class="text-[#e87e7e] font-bold">x{{ kill.kill_count }}</span>
                  <span class="text-text-dim text-[0.6rem]">{{ killsPerHour(kill.kill_count, session.elapsed_seconds) }}/hr</span>
                </div>
              </div>
            </div>

            <!-- Delete button -->
            <div class="flex justify-end pt-2 border-t border-border-default">
              <button
                @click.stop="deleteSession(session.id)"
                class="px-3 py-1 text-xs bg-[#3a2a2a]! border border-[#5a3a3a]! rounded text-[#c87e7e]! cursor-pointer transition-all font-medium hover:bg-[#4a3a3a] hover:border-[#6a4a4a]">
                Delete Session
              </button>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { HistoricalFarmingSession } from "../../types/farming";
import EmptyState from "../Shared/EmptyState.vue";
import SkeletonLoader from "../Shared/SkeletonLoader.vue";
import ItemInline from "../Shared/Item/ItemInline.vue";
import { formatDateTimeShort, formatDuration } from "../../composables/useTimestamp";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import NpcInline from "../Shared/NPC/NpcInline.vue";
import EnemyInline from "../Shared/Enemy/EnemyInline.vue";

const sessions = ref<HistoricalFarmingSession[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const expanded = ref<Set<number>>(new Set());

async function loadSessions() {
  loading.value = true;
  error.value = null;
  try {
    sessions.value = await invoke<HistoricalFarmingSession[]>("get_farming_sessions", { limit: 50 });
  } catch (e: any) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function toggleExpanded(id: number) {
  if (expanded.value.has(id)) {
    expanded.value.delete(id);
  } else {
    expanded.value.add(id);
  }
  expanded.value = new Set(expanded.value);
}

async function updateSession(
  session: HistoricalFarmingSession,
  field: 'name' | 'notes',
  value: string,
) {
  const updated = { ...session, [field]: value };
  try {
    await invoke("update_farming_session", {
      sessionId: session.id,
      name: updated.name,
      notes: updated.notes,
    });
    // Update local state
    const idx = sessions.value.findIndex((s) => s.id === session.id);
    if (idx >= 0) {
      sessions.value[idx] = { ...sessions.value[idx], [field]: value };
    }
  } catch (e) {
    console.error("[farming] Failed to update session:", e);
  }
}

async function deleteSession(id: number) {
  try {
    await invoke("delete_farming_session", { sessionId: id });
    sessions.value = sessions.value.filter((s) => s.id !== id);
  } catch (e) {
    console.error("[farming] Failed to delete session:", e);
  }
}

function sessionTotalXp(session: HistoricalFarmingSession): number {
  return session.skills.reduce((sum, s) => sum + s.xp_gained, 0);
}

function xpPerHour(session: HistoricalFarmingSession): number {
  const hours = Math.max(1, session.elapsed_seconds) / 3600;
  return Math.round(sessionTotalXp(session) / hours);
}

function skillXpPerHour(xpGained: number, elapsedSeconds: number): number {
  const hours = Math.max(1, elapsedSeconds) / 3600;
  return Math.round(xpGained / hours);
}

function itemPerHour(netQuantity: number, elapsedSeconds: number): number {
  const hours = Math.max(1, elapsedSeconds) / 3600;
  return Math.round(Math.abs(netQuantity) / hours);
}

function sessionTotalKills(session: HistoricalFarmingSession): number {
  return (session.kills ?? []).reduce((sum, k) => sum + k.kill_count, 0);
}

function killsPerHour(killCount: number, elapsedSeconds: number): number {
  const hours = Math.max(1, elapsedSeconds) / 3600;
  return Math.round(killCount / hours);
}

const totalElapsed = computed(() =>
  sessions.value.reduce((sum, s) => sum + s.elapsed_seconds, 0)
);

const totalXp = computed(() =>
  sessions.value.reduce((sum, s) => sum + sessionTotalXp(s), 0)
);

const totalGold = computed(() =>
  sessions.value.reduce((sum, s) => sum + s.vendor_gold, 0)
);

function formatDate(isoStr: string): string {
  return formatDateTimeShort(isoStr)
}

onMounted(loadSessions);
</script>
