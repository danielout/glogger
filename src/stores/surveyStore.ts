import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useGameDataStore } from "./gameDataStore";
import { useMarketStore } from "./marketStore";
import { formatTimeFull, formatDuration } from "../composables/useTimestamp";

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
  startTime: number;
  endTime: number | null;
  mapsStarted: number;
  surveysCompleted: number;
  surveyingXpGained: number;
  miningXpGained: number;
  geologyXpGained: number;
  _surveyingXpBaseline: number;
  _miningXpBaseline: number;
  _geologyXpBaseline: number;
  completionTimestamps: number[];
  primaryLootTotals: Record<string, number>;
  speedBonusLootTotals: Record<string, number>;
  itemValues: Record<string, number>;
  surveyTypeStats: Record<string, SurveyTypeStats>;
  surveyMapCosts: Record<string, number>;
  craftingMaterials: Record<string, number>;
  isPaused: boolean;
  pauseStartTime: number | null;
  totalPausedSeconds: number;
  manualMode: boolean;
  name: string;
  notes: string;
  // TNL tracking for surveys-to-level estimates
  surveyingTnl: number;
  miningTnl: number;
  geologyTnl: number;
  // Track XP events count per skill for averaging
  _miningXpEvents: number;
  _geologyXpEvents: number;
}

const SURVEY_SKILLS = ["Surveying", "Mining", "Geology"];

export const useSurveyStore = defineStore("survey", () => {
  const sessionActive = ref(false);
  const session = ref<SessionStats | null>(null);
  const log = ref<SurveyLogEntry[]>([]);
  const backendSessionId = ref<number | null>(null);

  /** Incremented each time a session is fully ended and patched to the backend.
   *  Other components (HistoricalTab, AnalyticsTab) can watch this to know when to reload. */
  const sessionFinalizedCounter = ref(0);

  const gameDataStore = useGameDataStore();
  const marketStore = useMarketStore();

  // Cache of survey type name → crafting_cost and survey_xp from the survey_types table
  const surveyTypeCosts = ref<Record<string, number>>({});
  const surveyTypeXp = ref<Record<string, number>>({});
  let surveyTypeCostsLoaded = false;

  async function loadSurveyTypeCosts() {
    if (surveyTypeCostsLoaded) return;
    try {
      const types = await invoke<Array<{ name: string; crafting_cost: number | null; survey_xp: number | null }>>("get_all_survey_types");
      for (const t of types) {
        surveyTypeCosts.value[t.name] = t.crafting_cost ?? 0;
        if (t.survey_xp) surveyTypeXp.value[t.name] = t.survey_xp;
      }
      surveyTypeCostsLoaded = true;
    } catch (e) {
      console.warn("[survey] Failed to load survey type costs:", e);
    }
  }

  function getSurveyMapCost(mapName: string): number {
    return surveyTypeCosts.value[mapName] ?? 0;
  }

  // Fetch vendor value from game data and cache it (used as fallback when no market price set)
  async function fetchItemValue(itemName: string): Promise<number> {
    if (!session.value) return 0;

    if (session.value.itemValues[itemName] !== undefined) {
      return session.value.itemValues[itemName];
    }

    try {
      const item = await gameDataStore.resolveItem(itemName);
      const value = item?.value ?? 0;
      session.value.itemValues[itemName] = value;
      return value;
    } catch (e) {
      console.warn(`Failed to fetch value for ${itemName}:`, e);
      session.value.itemValues[itemName] = 0;
      return 0;
    }
  }

  /**
   * Get effective price for an item: market price if set, otherwise cached vendor value.
   * This is reactive — when marketStore.valuesByName changes, computeds that call this recompute.
   */
  function getEffectivePrice(itemName: string): number {
    const market = marketStore.valuesByName[itemName];
    if (market) return market.market_value;
    return session.value?.itemValues[itemName] ?? 0;
  }


  function createSession(_timestamp: string, manual: boolean): SessionStats {
    return {
      startTime: Date.now(),
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
      surveyingTnl: 0,
      miningTnl: 0,
      geologyTnl: 0,
      _miningXpEvents: 0,
      _geologyXpEvents: 0,
    };
  }

  // Handle survey events from the backend.
  // DB persistence is handled entirely in Rust — this only updates display state.
  async function handleSurveyEvent(payload: {
    kind: string;
    timestamp: string;
    session_id?: number;
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
    // Track the backend session ID for patching on end
    if (payload.session_id != null) {
      backendSessionId.value = payload.session_id;
    } else if (payload.kind === "MapCrafted") {
      console.error(
        "[survey] WARNING: MapCrafted event received without session_id — " +
        "backend may have failed to create the session. Survey data will NOT be persisted!"
      );
    }
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
      session.value.completionTimestamps.push(Date.now());

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

    if (payload.kind === "MotherlodeCompleted" && session.value) {
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
      session.value.surveyTypeStats[surveyType].completed++;

      const typeStats = session.value.surveyTypeStats[surveyType];
      let motherLootValue = 0;
      if (payload.loot_items) {
        for (const lootItem of payload.loot_items) {
          session.value.primaryLootTotals[lootItem.item_name] =
            (session.value.primaryLootTotals[lootItem.item_name] ?? 0) +
            lootItem.quantity;
          typeStats.primaryLootTotals[lootItem.item_name] =
            (typeStats.primaryLootTotals[lootItem.item_name] ?? 0) +
            lootItem.quantity;
          const value = await fetchItemValue(lootItem.item_name);
          motherLootValue += value * lootItem.quantity;
        }
      }

      session.value.surveyTypeStats[surveyType].totalValue += motherLootValue;

      const lootText =
        payload.loot_items
          ?.map((item) =>
            item.quantity > 1
              ? `${item.item_name} x${item.quantity}`
              : `${item.item_name}`
          )
          .join(", ") ?? "";

      log.value.unshift({
        kind: "completed",
        timestamp: payload.timestamp,
        label: `Motherlode: ${payload.map_name ?? "Unknown"}`,
        lootText: lootText,
      });
    }
  }

  function handleLootCorrection(payload: {
    item_name: string;
    old_quantity: number;
    new_quantity: number;
    delta: number;
  }) {
    if (!session.value) return;

    // Correct the running totals by adding the delta
    const { item_name, delta } = payload;

    if (session.value.primaryLootTotals[item_name] !== undefined) {
      session.value.primaryLootTotals[item_name] += delta;
    }

    // Also correct per-survey-type stats
    for (const stats of Object.values(session.value.surveyTypeStats)) {
      if (stats.primaryLootTotals[item_name] !== undefined) {
        stats.primaryLootTotals[item_name] += delta;
        // Correct the revenue by the delta * item value
        const value = session.value!.itemValues[item_name] ?? 0;
        stats.totalValue += delta * value;
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
    if (!SURVEY_SKILLS.includes(payload.skill_type)) return;

    const s = session.value;
    if (payload.skill_type === "Surveying") {
      s.surveyingXpGained += xpDelta(payload.xp, s._surveyingXpBaseline);
      s._surveyingXpBaseline = payload.xp;
      s.surveyingTnl = payload.tnl;
    } else if (payload.skill_type === "Mining") {
      const delta = xpDelta(payload.xp, s._miningXpBaseline);
      s.miningXpGained += delta;
      s._miningXpBaseline = payload.xp;
      s.miningTnl = payload.tnl;
      if (delta > 0) s._miningXpEvents++;
    } else if (payload.skill_type === "Geology") {
      const delta = xpDelta(payload.xp, s._geologyXpBaseline);
      s.geologyXpGained += delta;
      s._geologyXpBaseline = payload.xp;
      s.geologyTnl = payload.tnl;
      if (delta > 0) s._geologyXpEvents++;
    }
  }

  function reset() {
    stopTick();
    sessionActive.value = false;
    session.value = null;
    log.value = [];
    backendSessionId.value = null;
  }

  function togglePause() {
    if (!session.value) return;

    if (session.value.isPaused) {
      if (session.value.pauseStartTime) {
        const pauseDuration = Math.round((Date.now() - session.value.pauseStartTime) / 1000);
        session.value.totalPausedSeconds += pauseDuration;
        session.value.pauseStartTime = null;
      }
      session.value.isPaused = false;
    } else {
      session.value.isPaused = true;
      session.value.pauseStartTime = Date.now();
    }
  }

  function manualStart() {
    if (sessionActive.value) return;

    sessionActive.value = true;
    session.value = createSession("", true);

    log.value.unshift({
      kind: "map-crafted",
      timestamp: formatTimeFull(new Date().toISOString()),
      label: "Survey session started (manual)",
    });
  }

  function manualEnd() {
    if (!session.value) return;
    session.value.endTime = Date.now();
    session.value.manualMode = true;
    if (backendSessionId.value) {
      patchSessionToBackend(backendSessionId.value);
    }
  }

  /// Called when the backend emits survey-session-ended (auto-end).
  /// The backend already finalized revenue/cost/profit — we patch in elapsed/XP.
  function handleSessionEnded(sessionId: number) {
    if (session.value && !session.value.endTime) {
      session.value.endTime = Date.now();
    }
    patchSessionToBackend(sessionId);
  }

  /// Patch frontend-only data (elapsed with pause accounting, XP, manual flag) to the DB.
  async function patchSessionToBackend(sessionId: number) {
    if (!session.value) return;

    const s = session.value;
    const end = s.endTime ?? Date.now();
    const totalSeconds = Math.max(0, Math.round((end - s.startTime) / 1000));
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
      sessionFinalizedCounter.value++;
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

  // Tick ref that updates every second while a session is active and not ended/paused.
  // Forces reactivity in activeSeconds so the elapsed display updates in real time.
  const _tick = ref(0);
  let _tickInterval: ReturnType<typeof setInterval> | null = null;

  function startTick() {
    if (_tickInterval) return;
    _tickInterval = setInterval(() => { _tick.value++; }, 1000);
  }

  function stopTick() {
    if (_tickInterval) {
      clearInterval(_tickInterval);
      _tickInterval = null;
    }
  }

  watch(
    () => ({ active: sessionActive.value, ended: session.value?.endTime, paused: session.value?.isPaused }),
    ({ active, ended, paused }) => {
      if (active && !ended && !paused) {
        startTick();
      } else {
        stopTick();
      }
    },
    { immediate: true }
  );

  const activeSeconds = computed(() => {
    // Reference _tick so this recomputes every second
    void _tick.value;

    if (!session.value) return 0;

    let endMs: number;
    if (session.value.endTime) {
      endMs = session.value.endTime;
    } else if (session.value.isPaused && session.value.pauseStartTime) {
      endMs = session.value.pauseStartTime;
    } else {
      endMs = Date.now();
    }

    const totalSeconds = Math.max(0, Math.round((endMs - session.value.startTime) / 1000));
    return Math.max(1, totalSeconds - session.value.totalPausedSeconds);
  });

  const activeHours = computed(() => activeSeconds.value / 3600);

  const elapsed = computed(() => {
    if (!session.value) return "—";
    return formatDuration(activeSeconds.value, { alwaysShowSeconds: true });
  });

  const avgSurveySeconds = computed(() => {
    const ts = session.value?.completionTimestamps;
    if (!ts || ts.length < 2) return null;
    const totalMs = ts[ts.length - 1] - ts[0];
    return Math.round(totalMs / 1000 / (ts.length - 1));
  });

  const avgSurveyTime = computed(() => {
    const s = avgSurveySeconds.value;
    if (s === null) return "—";
    return formatDuration(s, { alwaysShowSeconds: true });
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
      .map(([type, stats]) => {
        // Compute revenue reactively from current market/vendor prices
        const allLoot = { ...stats.primaryLootTotals };
        for (const [item, count] of Object.entries(stats.speedBonusLootTotals)) {
          allLoot[item] = (allLoot[item] ?? 0) + count;
        }
        const revenue = Object.entries(allLoot).reduce(
          (sum, [item, count]) => sum + getEffectivePrice(item) * count,
          0
        );
        return {
          type,
          count: stats.count,
          completed: stats.completed,
          revenue,
          cost: stats.totalCost,
          profit: revenue - stats.totalCost,
          profitPerSurvey:
            stats.completed > 0
              ? Math.round((revenue - stats.totalCost) / stats.completed)
              : 0,
          primaryLoot: buildLootSummary(stats.primaryLootTotals),
          speedBonusLoot: buildLootSummary(stats.speedBonusLootTotals),
        };
      })
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
    return Object.entries(allTotals).reduce((sum, [item, count]) => {
      return sum + getEffectivePrice(item) * count;
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

  // --- Computed: XP-to-level estimates ---

  /** Estimated surveys to craft until Surveying levels up */
  const surveysToLevelSurveying = computed((): number | null => {
    if (!session.value || session.value.surveyingTnl <= 0) return null;
    // Find the avg XP per craft from the survey types used this session
    const craftedTypes = Object.keys(session.value.surveyTypeStats);
    if (craftedTypes.length === 0) return null;
    let totalXp = 0;
    let count = 0;
    for (const type of craftedTypes) {
      // Survey type names in stats don't have "Survey"/"Map" suffix — find matching XP
      for (const [name, xp] of Object.entries(surveyTypeXp.value)) {
        if (name.startsWith(type)) {
          totalXp += xp;
          count++;
          break;
        }
      }
    }
    if (count === 0 || totalXp === 0) return null;
    const avgXp = totalXp / count;
    return Math.ceil(session.value.surveyingTnl / avgXp);
  });

  /** Estimated survey completions until Mining levels up */
  const surveysToLevelMining = computed((): number | null => {
    if (!session.value || session.value.miningTnl <= 0) return null;
    if (session.value._miningXpEvents <= 0 || session.value.miningXpGained <= 0) return null;
    const avgXpPerCompletion = session.value.miningXpGained / session.value._miningXpEvents;
    return Math.ceil(session.value.miningTnl / avgXpPerCompletion);
  });

  /** Estimated survey completions until Geology levels up */
  const surveysToLevelGeology = computed((): number | null => {
    if (!session.value || session.value.geologyTnl <= 0) return null;
    if (session.value._geologyXpEvents <= 0 || session.value.geologyXpGained <= 0) return null;
    const avgXpPerCompletion = session.value.geologyXpGained / session.value._geologyXpEvents;
    return Math.ceil(session.value.geologyTnl / avgXpPerCompletion);
  });

  const sessionEnded = computed(() => {
    return sessionActive.value && session.value?.endTime != null;
  });

  function newSession() {
    reset();
  }

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
    sessionEnded,
    sessionFinalizedCounter,
    backendSessionId,
    newSession,
    surveysToLevelSurveying,
    surveysToLevelMining,
    surveysToLevelGeology,
    fetchItemValue,
    handleSurveyEvent,
    handleSessionEnded,
    handleSkillUpdate,
    handleLootCorrection,
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

function extractSurveyType(surveyName: string | undefined): string {
  if (!surveyName) return "Unknown";
  const match = surveyName.match(/^(.+)\s+(?:Survey|Map)$/i);
  if (match) {
    return match[1];
  }
  return "Unknown";
}

function xpDelta(newXp: number, baseline: number): number {
  if (baseline === 0) return 0;
  if (newXp >= baseline) return newXp - baseline;
  return newXp;
}
