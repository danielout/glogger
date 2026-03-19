import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useGameDataStore } from "./gameDataStore";

export interface SurveyLogEntry {
  kind: "map-crafted" | "survey-used" | "completed";
  timestamp: string;
  label: string;
  lootText?: string;
}

export interface LootEntry {
  item: string;
  count: number;
}

export interface LootSummaryEntry {
  item: string;
  count: number;
  pct: number;
  rate: number;
}

export interface SurveyTypeStats {
  count: number;
  completed: number;
  totalValue: number;
  totalCost: number;
  primaryLootTotals: Record<string, number>;
  speedBonusLootTotals: Record<string, number>;
}

export interface ConsumedIngredient {
  item_name: string;
  quantity: number;
}

export interface SessionStats {
  startTime: string;
  endTime: string | null;
  mapsStarted: number;
  surveysCompleted: number;
  surveyingXpGained: number;
  miningXpGained: number;
  geologyXpGained: number;
  _surveyingXpBaseline: number;
  _miningXpBaseline: number;
  _geologyXpBaseline: number;
  completionTimestamps: string[];
  primaryLootTotals: Record<string, number>;
  speedBonusLootTotals: Record<string, number>;
  itemValues: Record<string, number>;
  surveyTypeStats: Record<string, SurveyTypeStats>;
  surveyMapCosts: Record<string, number>;
  craftingMaterials: Record<string, number>;
  isPaused: boolean;
  pauseStartTime: string | null;
  totalPausedSeconds: number;
  manualMode: boolean;
  name: string;
  notes: string;
}

const SURVEY_SKILLS = ["Surveying", "Mining", "Geology"];

export const useSurveyStore = defineStore("survey", () => {
  const sessionActive = ref(false);
  const session = ref<SessionStats | null>(null);
  const log = ref<SurveyLogEntry[]>([]);
  const backendSessionId = ref<number | null>(null);

  const gameDataStore = useGameDataStore();

  // Cache of survey type name → crafting_cost from the survey_types table
  const surveyTypeCosts = ref<Record<string, number>>({});
  let surveyTypeCostsLoaded = false;

  async function loadSurveyTypeCosts() {
    if (surveyTypeCostsLoaded) return;
    try {
      const types = await invoke<Array<{ name: string; crafting_cost: number | null }>>("get_all_survey_types");
      for (const t of types) {
        surveyTypeCosts.value[t.name] = t.crafting_cost ?? 0;
      }
      surveyTypeCostsLoaded = true;
    } catch (e) {
      console.warn("[survey] Failed to load survey type costs:", e);
    }
  }

  function getSurveyMapCost(mapName: string): number {
    return surveyTypeCosts.value[mapName] ?? 0;
  }

  // Fetch item value from game data and cache it
  async function fetchItemValue(itemName: string): Promise<number> {
    if (!session.value) return 0;

    if (session.value.itemValues[itemName] !== undefined) {
      return session.value.itemValues[itemName];
    }

    try {
      const item = await gameDataStore.getItemByName(itemName);
      const value = item?.value ?? 0;
      session.value.itemValues[itemName] = value;
      return value;
    } catch (e) {
      console.warn(`Failed to fetch value for ${itemName}:`, e);
      session.value.itemValues[itemName] = 0;
      return 0;
    }
  }


  function createSession(timestamp: string, manual: boolean): SessionStats {
    return {
      startTime: timestamp,
      endTime: null,
      mapsStarted: 0,
      surveysCompleted: 0,
      surveyingXpGained: 0,
      miningXpGained: 0,
      geologyXpGained: 0,
      _surveyingXpBaseline: 0,
      _miningXpBaseline: 0,
      _geologyXpBaseline: 0,
      completionTimestamps: [],
      primaryLootTotals: {},
      speedBonusLootTotals: {},
      itemValues: {},
      surveyTypeStats: {},
      surveyMapCosts: {},
      craftingMaterials: {},
      isPaused: false,
      pauseStartTime: null,
      totalPausedSeconds: 0,
      manualMode: manual,
      name: "Survey Session",
      notes: "",
    };
  }

  // Handle survey events from the backend.
  // DB persistence is handled entirely in Rust — this only updates display state.
  async function handleSurveyEvent(payload: {
    kind: string;
    timestamp: string;
    // MapCrafted fields
    map_name?: string;
    internal_name?: string;
    ingredients_consumed?: ConsumedIngredient[];
    // SurveyUsed fields
    survey_name?: string;
    // Completed fields
    loot_items?: Array<{
      item_name: string;
      quantity: number;
      is_speed_bonus: boolean;
      is_primary: boolean;
    }>;
    speed_bonus_earned?: boolean;
  }) {
    if (payload.kind === "MapCrafted") {
      // Ensure survey type costs are loaded before we need them
      await loadSurveyTypeCosts();

      if (!sessionActive.value) {
        sessionActive.value = true;
        session.value = createSession(payload.timestamp, false);
      }

      if (session.value) {
        session.value.mapsStarted++;

        // Track crafting ingredients
        if (payload.ingredients_consumed) {
          for (const ing of payload.ingredients_consumed) {
            session.value.craftingMaterials[ing.item_name] =
              (session.value.craftingMaterials[ing.item_name] ?? 0) +
              ing.quantity;
          }
        }

        // Look up pre-computed crafting cost from survey_types table
        if (payload.map_name) {
          const surveyType = extractSurveyType(payload.map_name);

          if (!session.value.surveyTypeStats[surveyType]) {
            session.value.surveyTypeStats[surveyType] = {
              count: 0,
              completed: 0,
              totalValue: 0,
              totalCost: 0,
              primaryLootTotals: {},
              speedBonusLootTotals: {},
            };
          }
          session.value.surveyTypeStats[surveyType].count++;

          const cost = getSurveyMapCost(payload.map_name);
          session.value.surveyTypeStats[surveyType].totalCost += cost;
          session.value.surveyMapCosts[payload.map_name] = cost;
        }

        log.value.unshift({
          kind: "map-crafted",
          timestamp: payload.timestamp,
          label: payload.map_name
            ? `Map crafted: ${payload.map_name}`
            : "Survey map crafted",
        });
      }
    }

    if (payload.kind === "SurveyUsed") {
      log.value.unshift({
        kind: "survey-used",
        timestamp: payload.timestamp,
        label: payload.survey_name
          ? `Survey used: ${payload.survey_name}`
          : "Survey used",
      });
    }

    if (payload.kind === "Completed" && session.value) {
      session.value.surveysCompleted++;
      session.value.completionTimestamps.push(payload.timestamp);

      const surveyType = extractSurveyType(payload.survey_name);
      if (!session.value.surveyTypeStats[surveyType]) {
        session.value.surveyTypeStats[surveyType] = {
          count: 0,
          completed: 0,
          totalValue: 0,
          totalCost: 0,
          primaryLootTotals: {},
          speedBonusLootTotals: {},
        };
      }
      session.value.surveyTypeStats[surveyType].completed++;

      // Tally loot into primary and speed bonus buckets (global + per-type), fetch values
      const typeStats = session.value.surveyTypeStats[surveyType];
      let surveyLootValue = 0;
      if (payload.loot_items) {
        for (const lootItem of payload.loot_items) {
          if (lootItem.is_speed_bonus) {
            session.value.speedBonusLootTotals[lootItem.item_name] =
              (session.value.speedBonusLootTotals[lootItem.item_name] ?? 0) +
              lootItem.quantity;
            typeStats.speedBonusLootTotals[lootItem.item_name] =
              (typeStats.speedBonusLootTotals[lootItem.item_name] ?? 0) +
              lootItem.quantity;
          } else {
            session.value.primaryLootTotals[lootItem.item_name] =
              (session.value.primaryLootTotals[lootItem.item_name] ?? 0) +
              lootItem.quantity;
            typeStats.primaryLootTotals[lootItem.item_name] =
              (typeStats.primaryLootTotals[lootItem.item_name] ?? 0) +
              lootItem.quantity;
          }
          const value = await fetchItemValue(lootItem.item_name);
          surveyLootValue += value * lootItem.quantity;
        }
      }

      session.value.surveyTypeStats[surveyType].totalValue += surveyLootValue;

      // Build loot text for log display
      const lootText =
        payload.loot_items
          ?.map((item) =>
            item.quantity > 1
              ? `${item.item_name} x${item.quantity}${item.is_speed_bonus ? " (speed bonus)" : ""}`
              : `${item.item_name}${item.is_speed_bonus ? " (speed bonus)" : ""}`
          )
          .join(", ") ?? "";

      log.value.unshift({
        kind: "completed",
        timestamp: payload.timestamp,
        label: payload.survey_name ?? "Survey",
        lootText: lootText,
      });
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
    if (!SURVEY_SKILLS.includes(payload.skill_type)) return;

    const s = session.value;
    if (payload.skill_type === "Surveying") {
      s.surveyingXpGained += xpDelta(payload.xp, s._surveyingXpBaseline);
      s._surveyingXpBaseline = payload.xp;
    } else if (payload.skill_type === "Mining") {
      s.miningXpGained += xpDelta(payload.xp, s._miningXpBaseline);
      s._miningXpBaseline = payload.xp;
    } else if (payload.skill_type === "Geology") {
      s.geologyXpGained += xpDelta(payload.xp, s._geologyXpBaseline);
      s._geologyXpBaseline = payload.xp;
    }
  }

  function reset() {
    sessionActive.value = false;
    session.value = null;
    log.value = [];
  }

  function togglePause() {
    if (!session.value) return;

    if (session.value.isPaused) {
      if (session.value.pauseStartTime) {
        const pauseStart = tsToSeconds(session.value.pauseStartTime);
        const now = tsToSeconds(getCurrentTimestamp());
        const pauseDuration = now - pauseStart;
        session.value.totalPausedSeconds += pauseDuration;
        session.value.pauseStartTime = null;
      }
      session.value.isPaused = false;
    } else {
      session.value.isPaused = true;
      session.value.pauseStartTime = getCurrentTimestamp();
    }
  }

  function manualStart() {
    if (sessionActive.value) return;

    const ts = getCurrentTimestamp();
    sessionActive.value = true;
    session.value = createSession(ts, true);

    log.value.unshift({
      kind: "map-crafted",
      timestamp: ts,
      label: "Survey session started (manual)",
    });
  }

  function manualEnd() {
    if (!session.value) return;
    session.value.endTime = getCurrentTimestamp();
    session.value.manualMode = true;
    if (backendSessionId.value) {
      patchSessionToBackend(backendSessionId.value);
    }
  }

  /// Called when the backend emits survey-session-ended (auto-end).
  /// The backend already finalized revenue/cost/profit — we patch in elapsed/XP.
  function handleSessionEnded(sessionId: number) {
    if (session.value && !session.value.endTime) {
      session.value.endTime = getCurrentTimestamp();
    }
    patchSessionToBackend(sessionId);
  }

  /// Patch frontend-only data (elapsed with pause accounting, XP, manual flag) to the DB.
  async function patchSessionToBackend(sessionId: number) {
    if (!session.value) return;

    const s = session.value;
    const start = tsToSeconds(s.startTime);
    const end = s.endTime
      ? tsToSeconds(s.endTime)
      : tsToSeconds(getCurrentTimestamp());
    const totalSeconds = Math.max(0, end - start);
    const elapsedSeconds = Math.max(1, totalSeconds - s.totalPausedSeconds);

    try {
      await invoke("patch_survey_session", {
        sessionId,
        input: {
          elapsed_seconds: elapsedSeconds,
          surveying_xp_gained: s.surveyingXpGained,
          mining_xp_gained: s.miningXpGained,
          geology_xp_gained: s.geologyXpGained,
          is_manual: s.manualMode,
        },
      });
    } catch (e) {
      console.error("[survey] Failed to patch session:", e);
    }
  }

  function toggleManualMode(enabled: boolean) {
    if (session.value) {
      session.value.manualMode = enabled;
    }
  }

  // --- Computed: time ---

  const activeSeconds = computed(() => {
    if (!session.value) return 0;
    const start = tsToSeconds(session.value.startTime);

    let endSeconds: number;
    if (session.value.endTime) {
      endSeconds = tsToSeconds(session.value.endTime);
    } else if (session.value.isPaused && session.value.pauseStartTime) {
      endSeconds = tsToSeconds(session.value.pauseStartTime);
    } else {
      const lastTs = log.value[0]?.timestamp;
      endSeconds = lastTs
        ? tsToSeconds(lastTs)
        : tsToSeconds(getCurrentTimestamp());
    }

    const totalSeconds = Math.max(0, endSeconds - start);
    return Math.max(1, totalSeconds - session.value.totalPausedSeconds);
  });

  const activeHours = computed(() => activeSeconds.value / 3600);

  const elapsed = computed(() => {
    if (!session.value) return "—";
    const s = activeSeconds.value;
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m}m ${sec}s`;
  });

  const avgSurveySeconds = computed(() => {
    const ts = session.value?.completionTimestamps;
    if (!ts || ts.length < 2) return null;
    const total = tsToSeconds(ts[ts.length - 1]) - tsToSeconds(ts[0]);
    return Math.round(total / (ts.length - 1));
  });

  const avgSurveyTime = computed(() => {
    const s = avgSurveySeconds.value;
    if (s === null) return "—";
    if (s < 60) return `${s}s`;
    return `${Math.floor(s / 60)}m ${s % 60}s`;
  });

  // --- Computed: loot summaries ---

  function buildLootSummary(
    totals: Record<string, number>
  ): LootSummaryEntry[] {
    const grandTotal = Object.values(totals).reduce((a, b) => a + b, 0);
    if (grandTotal === 0) return [];
    const hours = activeHours.value;
    return Object.entries(totals)
      .map(([item, count]) => ({
        item,
        count,
        pct: Math.round((count / grandTotal) * 100),
        rate: hours > 0 ? Math.round(count / hours) : 0,
      }))
      .sort((a, b) => b.count - a.count);
  }

  const primaryLootSummary = computed((): LootSummaryEntry[] => {
    if (!session.value) return [];
    return buildLootSummary(session.value.primaryLootTotals);
  });

  const speedBonusLootSummary = computed((): LootSummaryEntry[] => {
    if (!session.value) return [];
    return buildLootSummary(session.value.speedBonusLootTotals);
  });

  // --- Computed: economics ---

  const surveyTypeBreakdown = computed(() => {
    if (!session.value) return [];
    return Object.entries(session.value.surveyTypeStats)
      .filter(([type, stats]) => {
        if (type === "Unknown" && stats.completed === 0) return false;
        return true;
      })
      .map(([type, stats]) => ({
        type,
        count: stats.count,
        completed: stats.completed,
        revenue: stats.totalValue,
        cost: stats.totalCost,
        profit: stats.totalValue - stats.totalCost,
        profitPerSurvey:
          stats.completed > 0
            ? Math.round((stats.totalValue - stats.totalCost) / stats.completed)
            : 0,
        primaryLoot: buildLootSummary(stats.primaryLootTotals),
        speedBonusLoot: buildLootSummary(stats.speedBonusLootTotals),
      }))
      .sort((a, b) => b.completed - a.completed);
  });

  const totalValue = computed(() => {
    if (!session.value) return 0;
    const allTotals = {
      ...session.value.primaryLootTotals,
    };
    // Also include speed bonus in total value
    for (const [item, count] of Object.entries(
      session.value.speedBonusLootTotals
    )) {
      allTotals[item] = (allTotals[item] ?? 0) + count;
    }
    const values = session.value.itemValues;
    return Object.entries(allTotals).reduce((sum, [item, count]) => {
      const unitValue = values[item] ?? 0;
      return sum + unitValue * count;
    }, 0);
  });

  const totalCost = computed(() => {
    if (!session.value) return 0;
    return Object.values(session.value.surveyTypeStats).reduce(
      (sum, stats) => sum + stats.totalCost,
      0
    );
  });

  const totalProfit = computed(() => {
    return totalValue.value - totalCost.value;
  });

  const profitPerSurvey = computed(() => {
    if (!session.value || session.value.surveysCompleted === 0) return 0;
    return Math.round(totalProfit.value / session.value.surveysCompleted);
  });

  const profitPerHour = computed(() => {
    if (!session.value) return 0;
    const hours = activeHours.value;
    return hours > 0 ? Math.round(totalProfit.value / hours) : 0;
  });

  return {
    sessionActive,
    session,
    log,
    elapsed,
    avgSurveyTime,
    primaryLootSummary,
    speedBonusLootSummary,
    surveyTypeBreakdown,
    totalValue,
    totalCost,
    totalProfit,
    profitPerSurvey,
    profitPerHour,
    fetchItemValue,
    handleSurveyEvent,
    handleSessionEnded,
    handleSkillUpdate,
    reset,
    togglePause,
    manualStart,
    manualEnd,
    toggleManualMode,
    updateName,
    updateNotes,
  };

  function updateName(name: string) {
    if (session.value) {
      session.value.name = name;
      if (backendSessionId.value) {
        updateSessionNameNotes(backendSessionId.value, name, session.value.notes);
      }
    }
  }

  function updateNotes(notes: string) {
    if (session.value) {
      session.value.notes = notes;
      if (backendSessionId.value) {
        updateSessionNameNotes(backendSessionId.value, session.value.name, notes);
      }
    }
  }
});

function updateSessionNameNotes(sessionId: number, name: string, notes: string) {
  invoke("update_survey_session", { sessionId, name, notes }).catch((e) =>
    console.error("[survey] Failed to update session name/notes:", e)
  );
}

// --- helpers ---

function tsToSeconds(ts: string): number {
  const [h, m, s] = ts.split(":").map(Number);
  return h * 3600 + m * 60 + s;
}

function extractSurveyType(surveyName: string | undefined): string {
  if (!surveyName) return "Unknown";
  const match = surveyName.match(/^(.+)\s+(?:Survey|Map)$/i);
  if (match) {
    return match[1];
  }
  return "Unknown";
}

function getCurrentTimestamp(): string {
  const now = new Date();
  const h = String(now.getHours()).padStart(2, "0");
  const m = String(now.getMinutes()).padStart(2, "0");
  const s = String(now.getSeconds()).padStart(2, "0");
  return `${h}:${m}:${s}`;
}

function xpDelta(newXp: number, baseline: number): number {
  if (baseline === 0) return 0;
  if (newXp >= baseline) return newXp - baseline;
  return newXp;
}
