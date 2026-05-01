import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  FarmingSession,
  FarmingLogEntry,
  FarmingLogKind,
  SaveFarmingSessionInput,
} from "../types/farming";
import type { PlayerEvent } from "../types/playerEvents";
import { useGameDataStore } from "./gameDataStore";
import { formatTimeFull, formatDuration } from "../composables/useTimestamp";

export const useFarmingStore = defineStore("farming", () => {
  const sessionActive = ref(false);
  const session = ref<FarmingSession | null>(null);
  const log = ref<FarmingLogEntry[]>([]);

  // Live timer tick — increments every second to drive reactive elapsed display
  const timerTick = ref(0);
  let timerInterval: ReturnType<typeof setInterval> | null = null;

  function startTimer() {
    stopTimer();
    timerInterval = setInterval(() => { timerTick.value++; }, 1000);
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
  }

  // Buffer of recent item gain events for correlating with LootPickedUp
  const recentItemEvents: Array<{ instanceId: number; itemName: string; quantity: number }> = [];

  // ── Event Handlers ──────────────────────────────────────────────────────

  function handlePlayerEvent(event: PlayerEvent) {
    if (!sessionActive.value || !session.value) return;
    if (session.value.isPaused) return;

    const s = session.value;

    switch (event.kind) {
      case "ItemStackChanged": {
        if (event.delta === 0) break;
        const name = event.item_name ?? `item#${event.item_type_id}`;
        s.itemDeltas[name] = (s.itemDeltas[name] ?? 0) + event.delta;

        // Buffer positive gains for LootPickedUp correlation
        if (event.delta > 0) {
          recentItemEvents.push({ instanceId: event.instance_id, itemName: name, quantity: event.delta });
          if (recentItemEvents.length > 100) recentItemEvents.splice(0, recentItemEvents.length - 50);
        }

        const kind: FarmingLogKind = event.delta > 0 ? "item-gained" : "item-lost";
        const sign = event.delta > 0 ? "+" : "";
        pushLog(kind, event.timestamp, `${name} ${sign}${event.delta}`);
        break;
      }

      case "ItemAdded": {
        // Buffer for LootPickedUp correlation (ItemAdded carries instance_id + item_name)
        recentItemEvents.push({ instanceId: event.instance_id, itemName: event.item_name, quantity: 1 });
        if (recentItemEvents.length > 100) recentItemEvents.splice(0, recentItemEvents.length - 50);
        break;
      }

      case "LootPickedUp": {
        // Ground truth: this item was picked up from a corpse loot window
        // (skinning/butchering items do NOT produce LootPickedUp)
        if (!event.corpse_name) break;
        const enemyName = event.corpse_name;
        if (!s.kills[enemyName]) break;

        // Find matching item by instance_id (search from end for most recent)
        let match: { instanceId: number; itemName: string; quantity: number } | undefined;
        for (let i = recentItemEvents.length - 1; i >= 0; i--) {
          if (recentItemEvents[i].instanceId === event.instance_id) {
            match = recentItemEvents[i];
            break;
          }
        }
        if (match) {
          s.kills[enemyName].loot[match.itemName] = (s.kills[enemyName].loot[match.itemName] ?? 0) + match.quantity;
        }
        // If no match (stacking case with orphaned id), the item was already
        // tracked in itemDeltas via ItemStackChanged — we just can't attribute it
        // to a specific enemy type on the frontend. The backend coordinator handles
        // this case with item_type_id resolution.
        break;
      }

      case "ItemDeleted": {
        // Only count consumed/unknown as farming losses
        // StorageTransfer and VendorSale are intentional moves
        if (event.context === "Consumed" || event.context === "Unknown") {
          const name = event.item_name ?? "Unknown Item";
          s.itemDeltas[name] = (s.itemDeltas[name] ?? 0) - 1;
          pushLog("item-lost", event.timestamp, `${name} consumed`);
        }
        break;
      }

      case "FavorChanged": {
        const existing = s.favorDeltas[event.npc_name];
        if (existing) {
          existing.delta += event.delta;
        } else {
          s.favorDeltas[event.npc_name] = {
            delta: event.delta,
          };
        }
        const sign = event.delta > 0 ? "+" : "";
        pushLog(
          "favor-change",
          event.timestamp,
          `${event.npc_name} favor ${sign}${event.delta}`
        );
        break;
      }

      case "VendorSold": {
        s.vendorGold += event.price;
        pushLog(
          "vendor-sale",
          event.timestamp,
          `Sold ${event.item_name} for ${event.price}g`
        );
        break;
      }
    }
  }

  function handleSkillUpdate(payload: {
    skill_type: string;
    xp: number;
    level: number;
    tnl: number;
    timestamp: string;
  }) {
    if (!sessionActive.value || !session.value) return;

    const s = session.value;
    const key = payload.skill_type;

    if (!s.skillXp[key]) {
      // First event for this skill during session — set baseline
      s.skillXp[key] = {
        baseline: payload.xp,
        baselineTnl: payload.tnl,
        gained: 0,
        level: payload.level,
        tnl: payload.tnl,
        levelsGained: 0,
      };
      return;
    }

    const entry = s.skillXp[key];
    const prevLevel = entry.level;

    entry.level = payload.level;
    entry.tnl = payload.tnl;

    if (payload.level > prevLevel) {
      // Level-up: add remaining XP in old level + current XP in new level
      const xpToFinishOldLevel = entry.baselineTnl - entry.baseline;
      entry.gained += xpToFinishOldLevel + payload.xp;
      entry.levelsGained += payload.level - prevLevel;
      entry.baseline = payload.xp;
      entry.baselineTnl = payload.tnl;

      pushLog("level-up", payload.timestamp, `${key} leveled up to ${payload.level}!`);
    } else if (payload.xp >= entry.baseline) {
      entry.gained += payload.xp - entry.baseline;
      entry.baseline = payload.xp;
      entry.baselineTnl = payload.tnl;
    }

    if (entry.gained > 0) {
      // Update the last xp-gain log entry for this skill or add new one
      pushLog("xp-gain", payload.timestamp, `${key} +${entry.gained.toLocaleString()} XP`);
    }
  }

  function handleEnemyKilled(payload: {
    enemy_name: string;
    enemy_entity_id: string;
    killing_ability: string;
    health_damage: number;
    armor_damage: number;
    timestamp: string;
  }) {
    if (!sessionActive.value || !session.value) return;
    if (session.value.isPaused) return;

    const s = session.value;
    const name = payload.enemy_name;

    if (!s.kills[name]) {
      s.kills[name] = { count: 0, loot: {} };
    }
    s.kills[name].count++;

    pushLog("enemy-killed", payload.timestamp, `Killed ${name}`);
  }

  // ── Session Controls ────────────────────────────────────────────────────

  function startSession(name?: string) {
    if (sessionActive.value) return;

    const ts = getCurrentTimestamp();
    sessionActive.value = true;
    session.value = {
      name: name ?? "Farming Session",
      notes: "",
      startTime: ts,
      endTime: null,
      isPaused: false,
      pauseStartTime: null,
      totalPausedSeconds: 0,
      skillXp: {},
      itemDeltas: {},
      ignoredItems: new Set(),
      favorDeltas: {},
      kills: {},
      vendorGold: 0,
    };
    log.value = [];

    pushLog("session-start", ts, "Farming session started");
    startTimer();
  }

  async function endSession() {
    if (!session.value) return;

    const ts = getCurrentTimestamp();
    session.value.endTime = ts;
    pushLog("session-end", ts, "Farming session ended");
    stopTimer();

    // Persist to database
    try {
      const s = session.value;
      const input: SaveFarmingSessionInput = {
        name: s.name,
        notes: s.notes,
        start_time: s.startTime,
        end_time: s.endTime,
        elapsed_seconds: getActiveSeconds(),
        total_paused_seconds: s.totalPausedSeconds,
        vendor_gold: s.vendorGold,
        skills: await Promise.all(
          Object.entries(s.skillXp)
            .filter(([, v]) => v.gained > 0 || v.levelsGained > 0)
            .map(async ([skillType, v]) => {
              const gameData = useGameDataStore();
              const resolved = await gameData.resolveSkill(skillType);
              return {
                skill_id: resolved?.id ?? 0,
                skill_name: resolved?.name ?? skillType,
                xp_gained: v.gained,
                levels_gained: v.levelsGained,
              };
            })
        ),
        items: Object.entries(s.itemDeltas)
          .filter(([name, qty]) => qty !== 0 && !s.ignoredItems.has(name))
          .map(([item_name, net_quantity]) => ({ item_name, net_quantity })),
        favors: await Promise.all(
          Object.entries(s.favorDeltas)
            .filter(([, v]) => v.delta !== 0)
            .map(async ([npcName, v]) => {
              const gameData = useGameDataStore();
              const resolved = await gameData.resolveNpc(npcName);
              return {
                npc_key: resolved?.key ?? npcName,
                npc_name: resolved?.name ?? npcName,
                delta: v.delta,
              };
            })
        ),
        kills: Object.entries(s.kills)
          .filter(([, v]) => v.count > 0)
          .map(([enemy_name, v]) => ({ enemy_name, kill_count: v.count })),
      };

      await invoke("save_farming_session", { input });
      console.log("[farming] Session saved to database");
    } catch (e) {
      console.error("[farming] Failed to save session:", e);
    }
  }

  function togglePause() {
    if (!session.value) return;

    if (session.value.isPaused) {
      if (session.value.pauseStartTime) {
        const pauseStart = tsToSeconds(session.value.pauseStartTime);
        const now = tsToSeconds(getCurrentTimestamp());
        const pauseDiff = now - pauseStart;
        session.value.totalPausedSeconds += pauseDiff >= 0 ? pauseDiff : pauseDiff + 86400;
        session.value.pauseStartTime = null;
      }
      session.value.isPaused = false;
      startTimer();
    } else {
      session.value.isPaused = true;
      session.value.pauseStartTime = getCurrentTimestamp();
      stopTimer();
    }
  }

  function updateName(name: string) {
    if (session.value) session.value.name = name;
  }

  function updateNotes(notes: string) {
    if (session.value) session.value.notes = notes;
  }

  function reset() {
    sessionActive.value = false;
    session.value = null;
    log.value = [];
    stopTimer();
  }

  // ── Computed ────────────────────────────────────────────────────────────

  function getActiveSeconds(): number {
    if (!session.value) return 0;
    const start = tsToSeconds(session.value.startTime);

    let endSeconds: number;
    if (session.value.endTime) {
      endSeconds = tsToSeconds(session.value.endTime);
    } else if (session.value.isPaused && session.value.pauseStartTime) {
      endSeconds = tsToSeconds(session.value.pauseStartTime);
    } else {
      endSeconds = tsToSeconds(getCurrentTimestamp());
    }

    // Handle midnight rollover: if end < start, session crossed midnight
    const rawDiff = endSeconds - start;
    const totalSeconds = rawDiff >= 0 ? rawDiff : rawDiff + 86400;
    return Math.max(0, totalSeconds - session.value.totalPausedSeconds);
  }

  const elapsed = computed(() => {
    // Depend on timerTick so this recomputes every second
    void timerTick.value;
    if (!session.value) return "—";
    return formatDuration(getActiveSeconds(), { alwaysShowSeconds: true });
  });

  const totalXpGained = computed(() => {
    if (!session.value) return 0;
    return Object.values(session.value.skillXp).reduce((sum, s) => sum + s.gained, 0);
  });

  const totalItemsGained = computed(() => {
    if (!session.value) return 0;
    const ignored = session.value.ignoredItems;
    return Object.entries(session.value.itemDeltas)
      .filter(([name]) => !ignored.has(name))
      .reduce((sum, [, qty]) => sum + Math.max(0, qty), 0);
  });

  const totalItemsLost = computed(() => {
    if (!session.value) return 0;
    const ignored = session.value.ignoredItems;
    return Object.entries(session.value.itemDeltas)
      .filter(([name]) => !ignored.has(name))
      .reduce((sum, [, qty]) => sum + Math.abs(Math.min(0, qty)), 0);
  });

  const totalKills = computed(() => {
    if (!session.value) return 0;
    return Object.values(session.value.kills).reduce((sum, k) => sum + k.count, 0);
  });

  const killSummary = computed(() => {
    void timerTick.value;
    if (!session.value) return [];
    const activeHours = Math.max(1, getActiveSeconds()) / 3600;
    return Object.entries(session.value.kills)
      .filter(([, v]) => v.count > 0)
      .map(([name, v]) => ({
        name,
        count: v.count,
        perHour: Math.round(v.count / activeHours),
        loot: Object.entries(v.loot)
          .filter(([, qty]) => qty > 0)
          .map(([itemName, qty]) => ({ name: itemName, quantity: qty }))
          .sort((a, b) => b.quantity - a.quantity),
      }))
      .sort((a, b) => b.count - a.count);
  });

  const totalFavorGained = computed(() => {
    if (!session.value) return 0;
    return Object.values(session.value.favorDeltas).reduce((sum, v) => sum + v.delta, 0);
  });

  const skillSummary = computed(() => {
    // Depend on timerTick for per-hour rate updates
    void timerTick.value;
    if (!session.value) return [];
    const activeHours = Math.max(1, getActiveSeconds()) / 3600;
    return Object.entries(session.value.skillXp)
      .filter(([, v]) => v.gained > 0 || v.levelsGained > 0)
      .map(([name, v]) => ({
        name,
        gained: v.gained,
        levelsGained: v.levelsGained,
        level: v.level,
        tnl: v.tnl,
        currentXp: v.baseline,
        xpProgress: v.tnl > 0 ? (v.baseline / v.tnl) * 100 : 0,
        perHour: Math.round(v.gained / activeHours),
      }))
      .sort((a, b) => b.gained - a.gained);
  });

  const itemSummary = computed(() => {
    void timerTick.value;
    if (!session.value) return [];
    const activeHours = Math.max(1, getActiveSeconds()) / 3600;
    const ignored = session.value.ignoredItems;
    return Object.entries(session.value.itemDeltas)
      .filter(([, qty]) => qty !== 0)
      .map(([name, qty]) => ({
        name,
        netQuantity: qty,
        perHour: Math.round(Math.abs(qty) / activeHours),
        isIgnored: ignored.has(name),
      }))
      .sort((a, b) => {
        // Ignored items go to the bottom
        if (a.isIgnored !== b.isIgnored) return a.isIgnored ? 1 : -1;
        return b.netQuantity - a.netQuantity;
      });
  });

  function toggleIgnoreItem(name: string) {
    if (!session.value) return;
    if (session.value.ignoredItems.has(name)) {
      session.value.ignoredItems.delete(name);
    } else {
      session.value.ignoredItems.add(name);
    }
    // Trigger reactivity by reassigning the Set
    session.value.ignoredItems = new Set(session.value.ignoredItems);
  }

  const favorSummary = computed(() => {
    if (!session.value) return [];
    return Object.entries(session.value.favorDeltas)
      .filter(([, v]) => v.delta !== 0)
      .map(([name, v]) => ({
        name,
        delta: v.delta,
      }))
      .sort((a, b) => b.delta - a.delta);
  });

  function xpPerHour(skillName: string): number {
    if (!session.value?.skillXp[skillName]) return 0;
    const activeHours = Math.max(1, getActiveSeconds()) / 3600;
    return Math.round(session.value.skillXp[skillName].gained / activeHours);
  }

  // ── Helpers ─────────────────────────────────────────────────────────────

  function pushLog(kind: FarmingLogKind, timestamp: string, label: string, detail?: string) {
    log.value.unshift({ kind, timestamp, label, detail });
    // Cap log size
    if (log.value.length > 500) log.value.length = 500;
  }

  return {
    sessionActive,
    session,
    log,
    elapsed,
    totalXpGained,
    totalItemsGained,
    totalItemsLost,
    totalKills,
    totalFavorGained,
    skillSummary,
    itemSummary,
    killSummary,
    favorSummary,
    xpPerHour,
    getActiveSeconds,
    handlePlayerEvent,
    handleSkillUpdate,
    handleEnemyKilled,
    startSession,
    endSession,
    togglePause,
    toggleIgnoreItem,
    updateName,
    updateNotes,
    reset,
  };
});

// ── Module helpers ────────────────────────────────────────────────────────

function tsToSeconds(ts: string): number {
  const [h, m, s] = ts.split(":").map(Number);
  return h * 3600 + m * 60 + s;
}

function getCurrentTimestamp(): string {
  return formatTimeFull(new Date().toISOString());
}
