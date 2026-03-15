import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useGameDataStore } from "./gameDataStore";

export interface SurveyLogEntry {
  kind: "session-start" | "located" | "completed";
  timestamp: string;
  label: string;
  lootText?: string;
}

export interface LootEntry {
  item: string;
  count: number;
}

export interface SurveyTypeStats {
  count: number;
  completed: number;
  totalValue: number;
  totalCost: number;
}

export interface SessionStats {
  startTime: string;
  endTime: string | null;
  mapsStarted: number;
  surveysLocated: number;
  surveysCompleted: number;
  surveyingXpGained: number;
  miningXpGained: number;
  geologyXpGained: number;
  _surveyingXpBaseline: number;
  _miningXpBaseline: number;
  _geologyXpBaseline: number;
  completionTimestamps: string[];
  lootTotals: Record<string, number>;
  itemValues: Record<string, number>;
  surveyTypeStats: Record<string, SurveyTypeStats>; // Track stats per survey type
  surveyMapCosts: Record<string, number>; // Map name -> crafting cost
  isPaused: boolean;
  pauseStartTime: string | null;
  totalPausedSeconds: number;
  manualMode: boolean; // If true, disable auto start/end detection
  sessionId: number | null; // Database session ID from save_survey_session_stats
}

const SURVEY_SKILLS = ["Surveying", "Mining", "Geology"];

// Helper function to log survey events to database
async function logSurveyEventToDb(input: {
  timestamp: string;
  session_id: number | null;
  event_type: "session_start" | "completed";
  map_type: string | null;
  survey_type: string | null;
  speed_bonus_earned: boolean;
}): Promise<number> {
  try {
    const eventId = await invoke<number>("log_survey_event", { input });
    return eventId;
  } catch (e) {
    console.error("Failed to log survey event to database:", e);
    throw e;
  }
}

// Helper function to log individual loot items to database
async function logLootItemToDb(input: {
  event_id: number;
  item_id: number | null;
  item_name: string;
  quantity: number;
  is_speed_bonus: boolean;
  is_primary: boolean;
}): Promise<void> {
  try {
    await invoke("log_survey_loot_item", { input });
  } catch (e) {
    console.error("Failed to log loot item to database:", e);
    throw e;
  }
}

export const useSurveyStore = defineStore("survey", () => {
  const sessionActive = ref(false);
  const session = ref<SessionStats | null>(null);
  const log = ref<SurveyLogEntry[]>([]);

  const gameDataStore = useGameDataStore();

  // Fetch item value from game data and cache it
  async function fetchItemValue(itemName: string): Promise<number> {
    if (!session.value) return 0;

    // Return cached value if we have it
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

  // Fetch and calculate crafting cost for a survey map
  async function fetchSurveyMapCost(mapName: string): Promise<number> {
    if (!session.value) return 0;

    // Return cached cost if we have it
    if (session.value.surveyMapCosts[mapName] !== undefined) {
      console.log(`Using cached cost for ${mapName}: ${session.value.surveyMapCosts[mapName]}g`);
      return session.value.surveyMapCosts[mapName];
    }

    try {
      // Get the item for this map
      const mapItem = await gameDataStore.getItemByName(mapName);
      if (!mapItem) {
        console.warn(`Map item not found in game data: "${mapName}"`);
        session.value.surveyMapCosts[mapName] = 0;
        return 0;
      }

      console.log(`Found map item: ${mapItem.name} (id: ${mapItem.id})`);

      // Get recipes that produce this map
      const recipes = await gameDataStore.getRecipesForItem(mapItem.id);
      if (!recipes || recipes.length === 0) {
        console.warn(`No recipes found for ${mapName} (id: ${mapItem.id})`);
        session.value.surveyMapCosts[mapName] = 0;
        return 0;
      }

      console.log(`Found ${recipes.length} recipe(s) for ${mapName}`);

      // Use the first recipe (should only be one)
      const recipe = recipes[0];

      // Calculate total ingredient cost
      let totalCost = 0;
      const ingredientIds = recipe.ingredients
        .map((ing) => ing.item_id)
        .filter((id): id is number => id !== null);
      const ingredientItems = await gameDataStore.getItemsBatch(ingredientIds);

      for (const ingredient of recipe.ingredients) {
        if (ingredient.item_id === null) continue;
        const item = ingredientItems[ingredient.item_id];
        if (item) {
          const unitValue = item.value ?? 0;
          const ingredientCost = unitValue * ingredient.stack_size;
          totalCost += ingredientCost;
          console.log(`  - ${item.name} x${ingredient.stack_size}: ${ingredientCost}g`);
        }
      }

      console.log(`Total crafting cost for ${mapName}: ${totalCost}g`);
      session.value.surveyMapCosts[mapName] = totalCost;
      return totalCost;
    } catch (e) {
      console.error(`Failed to fetch cost for ${mapName}:`, e);
      session.value.surveyMapCosts[mapName] = 0;
      return 0;
    }
  }

  async function handleSurveyEvent(payload: {
    kind: string;
    timestamp: string;
    map_name?: string;
    survey_name?: string;
    loot_items?: Array<{
      item_name: string;
      quantity: number;
      is_speed_bonus: boolean;
      is_primary: boolean;
    }>;
    speed_bonus_earned?: boolean;
  }) {
    if (payload.kind === "SessionStart") {
      if (!sessionActive.value) {
        sessionActive.value = true;
        session.value = {
          startTime: payload.timestamp,
          endTime: null,
          mapsStarted: 1,
          surveysLocated: 0,
          surveysCompleted: 0,
          surveyingXpGained: 0,
          miningXpGained: 0,
          geologyXpGained: 0,
          _surveyingXpBaseline: 0,
          _miningXpBaseline: 0,
          _geologyXpBaseline: 0,
          completionTimestamps: [],
          lootTotals: {},
          itemValues: {},
          surveyTypeStats: {},
          surveyMapCosts: {},
          isPaused: false,
          pauseStartTime: null,
          totalPausedSeconds: 0,
          manualMode: false,
          sessionId: null,
        };
      } else if (session.value) {
        session.value.mapsStarted++;
      }

      // Fetch map cost in background if we have the map name
      if (payload.map_name && session.value) {
        // Extract survey type from map name (map names should match survey names pattern)
        const surveyType = extractSurveyType(payload.map_name);

        // Initialize survey type stats if needed
        if (!session.value.surveyTypeStats[surveyType]) {
          session.value.surveyTypeStats[surveyType] = {
            count: 0,
            completed: 0,
            totalValue: 0,
            totalCost: 0,
          };
        }
        session.value.surveyTypeStats[surveyType].count++;

        // Fetch and add cost asynchronously
        fetchSurveyMapCost(payload.map_name).then((cost) => {
          if (session.value && session.value.surveyTypeStats[surveyType]) {
            session.value.surveyTypeStats[surveyType].totalCost += cost;
            console.log(`Added cost ${cost}g for ${payload.map_name} (type: ${surveyType})`);
          }
        }).catch((err) => {
          console.error(`Failed to fetch cost for ${payload.map_name}:`, err);
        });
      }

      log.value.unshift({
        kind: "session-start",
        timestamp: payload.timestamp,
        label: "Survey run started",
      });

      // Log event to database
      logSurveyEventToDb({
        timestamp: payload.timestamp,
        session_id: session.value?.sessionId ?? null,
        event_type: "session_start",
        map_type: payload.map_name ?? null,
        survey_type: null,
        speed_bonus_earned: false,
      }).catch((err: unknown) => {
        console.error("Failed to log session_start event:", err);
      });
    }

    if (payload.kind === "Located" && session.value) {
      session.value.surveysLocated++;
      log.value.unshift({
        kind: "located",
        timestamp: payload.timestamp,
        label: `Located: ${payload.survey_name}`,
      });
      // Note: We no longer log "located" events to the database
    }

    if (payload.kind === "Completed" && session.value) {
      session.value.surveysCompleted++;
      session.value.completionTimestamps.push(payload.timestamp);

      // Track survey type stats
      const surveyType = extractSurveyType(payload.survey_name);
      if (!session.value.surveyTypeStats[surveyType]) {
        session.value.surveyTypeStats[surveyType] = {
          count: 0,
          completed: 0,
          totalValue: 0,
          totalCost: 0,
        };
      }
      session.value.surveyTypeStats[surveyType].completed++;

      // Parse and tally loot from individual items
      let surveyLootValue = 0;
      if (payload.loot_items) {
        for (const lootItem of payload.loot_items) {
          session.value.lootTotals[lootItem.item_name] =
            (session.value.lootTotals[lootItem.item_name] ?? 0) + lootItem.quantity;
          // Fetch item value in background
          const value = await fetchItemValue(lootItem.item_name);
          surveyLootValue += value * lootItem.quantity;
        }
      }

      // Add loot value to survey type
      session.value.surveyTypeStats[surveyType].totalValue += surveyLootValue;

      // Build loot text for display
      const lootText = payload.loot_items
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

      // Log event to database and get event ID
      try {
        const eventId = await logSurveyEventToDb({
          timestamp: payload.timestamp,
          session_id: session.value.sessionId,
          event_type: "completed",
          map_type: null,
          survey_type: surveyType,
          speed_bonus_earned: payload.speed_bonus_earned ?? false,
        });

        // Log each loot item to database
        if (payload.loot_items) {
          for (const lootItem of payload.loot_items) {
            // Try to get item_id from game data
            let itemId: number | null = null;
            try {
              const item = await gameDataStore.getItemByName(lootItem.item_name);
              itemId = item?.id ?? null;
            } catch (e) {
              console.warn(`Could not find item ID for ${lootItem.item_name}:`, e);
            }

            await logLootItemToDb({
              event_id: eventId,
              item_id: itemId,
              item_name: lootItem.item_name,
              quantity: lootItem.quantity,
              is_speed_bonus: lootItem.is_speed_bonus,
              is_primary: lootItem.is_primary,
            });
          }
        }
      } catch (err: unknown) {
        console.error("Failed to log completed event or loot items:", err);
      }

      // Auto-end detection: if we've completed as many surveys as we started maps for
      if (!session.value.manualMode && !session.value.endTime) {
        if (session.value.surveysCompleted >= session.value.mapsStarted) {
          session.value.endTime = payload.timestamp;
          // Save session to database when auto-ending
          saveSession().catch((err) => {
            console.error("Failed to auto-save session:", err);
          });
        }
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
      // Resuming
      if (session.value.pauseStartTime) {
        const pauseStart = tsToSeconds(session.value.pauseStartTime);
        const now = tsToSeconds(getCurrentTimestamp());
        const pauseDuration = now - pauseStart;
        session.value.totalPausedSeconds += pauseDuration;
        session.value.pauseStartTime = null;
      }
      session.value.isPaused = false;
    } else {
      // Pausing
      session.value.isPaused = true;
      session.value.pauseStartTime = getCurrentTimestamp();
    }
  }

  function manualStart() {
    if (sessionActive.value) return;

    sessionActive.value = true;
    session.value = {
      startTime: getCurrentTimestamp(),
      endTime: null,
      mapsStarted: 0,
      surveysLocated: 0,
      surveysCompleted: 0,
      surveyingXpGained: 0,
      miningXpGained: 0,
      geologyXpGained: 0,
      _surveyingXpBaseline: 0,
      _miningXpBaseline: 0,
      _geologyXpBaseline: 0,
      completionTimestamps: [],
      lootTotals: {},
      itemValues: {},
      surveyTypeStats: {},
      surveyMapCosts: {},
      isPaused: false,
      pauseStartTime: null,
      totalPausedSeconds: 0,
      manualMode: true,
      sessionId: null,
    };

    log.value.unshift({
      kind: "session-start",
      timestamp: getCurrentTimestamp(),
      label: "Survey session started (manual)",
    });
  }

  async function manualEnd() {
    if (!session.value) return;
    session.value.endTime = getCurrentTimestamp();
    session.value.manualMode = true;
    await saveSession();
  }

  async function saveSession() {
    if (!session.value) return;

    // Convert HH:MM:SS to ISO datetime string
    const startDate = timestampToDate(session.value.startTime);
    const endDate = session.value.endTime ? timestampToDate(session.value.endTime) : null;

    // Calculate elapsed seconds
    let elapsedSeconds = 0;
    if (session.value.endTime) {
      const start = tsToSeconds(session.value.startTime);
      const end = tsToSeconds(session.value.endTime);
      elapsedSeconds = Math.max(0, end - start - session.value.totalPausedSeconds);
    }

    try {
      const sessionId = await invoke<number>("save_survey_session_stats", {
        input: {
          start_time: startDate.toISOString(),
          end_time: endDate?.toISOString() || null,
          maps_started: session.value.mapsStarted,
          surveys_located: session.value.surveysLocated,
          surveys_completed: session.value.surveysCompleted,
          surveying_xp_gained: session.value.surveyingXpGained,
          mining_xp_gained: session.value.miningXpGained,
          geology_xp_gained: session.value.geologyXpGained,
          total_revenue: totalValue.value,
          total_cost: totalCost.value,
          total_profit: totalProfit.value,
          profit_per_hour: profitPerHour.value,
          elapsed_seconds: elapsedSeconds,
          is_manual: session.value.manualMode,
        },
      });
      session.value.sessionId = sessionId;
      console.log("Survey session saved to database with ID:", sessionId);
    } catch (e) {
      console.error("Failed to save survey session:", e);
    }
  }

  function toggleManualMode(enabled: boolean) {
    if (session.value) {
      session.value.manualMode = enabled;
    }
  }

  const elapsed = computed(() => {
    if (!session.value) return "—";
    const start = tsToSeconds(session.value.startTime);

    // Determine end time: use endTime if set, otherwise use latest activity or current time
    let endSeconds: number;
    if (session.value.endTime) {
      endSeconds = tsToSeconds(session.value.endTime);
    } else if (session.value.isPaused && session.value.pauseStartTime) {
      // If paused, use the pause start time as the end
      endSeconds = tsToSeconds(session.value.pauseStartTime);
    } else {
      const lastTs = log.value[0]?.timestamp;
      endSeconds = lastTs ? tsToSeconds(lastTs) : tsToSeconds(getCurrentTimestamp());
    }

    const totalSeconds = Math.max(0, endSeconds - start);
    const activeSeconds = totalSeconds - session.value.totalPausedSeconds;
    const m = Math.floor(activeSeconds / 60);
    const s = activeSeconds % 60;
    return `${m}m ${s}s`;
  });

  // Average seconds between survey completions
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

  // Loot sorted by count descending, with percentage
  const lootSummary = computed((): Array<LootEntry & { pct: number }> => {
    if (!session.value) return [];
    const totals = session.value.lootTotals;
    const grandTotal = Object.values(totals).reduce((a, b) => a + b, 0);
    if (grandTotal === 0) return [];
    return Object.entries(totals)
      .map(([item, count]) => ({
        item,
        count,
        pct: Math.round((count / grandTotal) * 100),
      }))
      .sort((a, b) => b.count - a.count);
  });

  // Survey type breakdown with profit calculations
  const surveyTypeBreakdown = computed(() => {
    if (!session.value) return [];
    return Object.entries(session.value.surveyTypeStats)
      .filter(([type, stats]) => {
        // Filter out "Unknown" types with no completions
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
      }))
      .sort((a, b) => b.completed - a.completed);
  });

  // Total value of all loot collected (in gold) - this is revenue
  const totalValue = computed(() => {
    if (!session.value) return 0;
    const totals = session.value.lootTotals;
    const values = session.value.itemValues;
    return Object.entries(totals).reduce((sum, [item, count]) => {
      const unitValue = values[item] ?? 0;
      return sum + unitValue * count;
    }, 0);
  });

  // Total cost of all survey maps crafted
  const totalCost = computed(() => {
    if (!session.value) return 0;
    return Object.values(session.value.surveyTypeStats).reduce(
      (sum, stats) => sum + stats.totalCost,
      0
    );
  });

  // Total profit (revenue - cost)
  const totalProfit = computed(() => {
    return totalValue.value - totalCost.value;
  });

  // Revenue per completed survey
  const valuePerSurvey = computed(() => {
    if (!session.value || session.value.surveysCompleted === 0) return 0;
    return Math.round(totalValue.value / session.value.surveysCompleted);
  });

  // Cost per completed survey
  const costPerSurvey = computed(() => {
    if (!session.value || session.value.surveysCompleted === 0) return 0;
    return Math.round(totalCost.value / session.value.surveysCompleted);
  });

  // Profit per completed survey
  const profitPerSurvey = computed(() => {
    return valuePerSurvey.value - costPerSurvey.value;
  });

  // Revenue per hour (based on elapsed time, excluding paused time)
  const valuePerHour = computed(() => {
    if (!session.value) return 0;
    const start = tsToSeconds(session.value.startTime);

    // Determine end time
    let endSeconds: number;
    if (session.value.endTime) {
      endSeconds = tsToSeconds(session.value.endTime);
    } else if (session.value.isPaused && session.value.pauseStartTime) {
      endSeconds = tsToSeconds(session.value.pauseStartTime);
    } else {
      const lastTs = log.value[0]?.timestamp;
      endSeconds = lastTs ? tsToSeconds(lastTs) : tsToSeconds(getCurrentTimestamp());
    }

    const totalSeconds = Math.max(1, endSeconds - start);
    const activeSeconds = Math.max(1, totalSeconds - session.value.totalPausedSeconds);
    const activeHours = activeSeconds / 3600;
    return Math.round(totalValue.value / activeHours);
  });

  // Profit per hour
  const profitPerHour = computed(() => {
    if (!session.value) return 0;
    const start = tsToSeconds(session.value.startTime);

    let endSeconds: number;
    if (session.value.endTime) {
      endSeconds = tsToSeconds(session.value.endTime);
    } else if (session.value.isPaused && session.value.pauseStartTime) {
      endSeconds = tsToSeconds(session.value.pauseStartTime);
    } else {
      const lastTs = log.value[0]?.timestamp;
      endSeconds = lastTs ? tsToSeconds(lastTs) : tsToSeconds(getCurrentTimestamp());
    }

    const totalSeconds = Math.max(1, endSeconds - start);
    const activeSeconds = Math.max(1, totalSeconds - session.value.totalPausedSeconds);
    const activeHours = activeSeconds / 3600;
    return Math.round(totalProfit.value / activeHours);
  });

  return {
    sessionActive,
    session,
    log,
    elapsed,
    avgSurveyTime,
    lootSummary,
    surveyTypeBreakdown,
    totalValue,
    totalCost,
    totalProfit,
    valuePerSurvey,
    costPerSurvey,
    profitPerSurvey,
    valuePerHour,
    profitPerHour,
    fetchItemValue,
    handleSurveyEvent,
    handleSkillUpdate,
    reset,
    togglePause,
    manualStart,
    manualEnd,
    toggleManualMode,
  };
});

// --- helpers ---

function tsToSeconds(ts: string): number {
  const [h, m, s] = ts.split(":").map(Number);
  return h * 3600 + m * 60 + s;
}

// Extract survey type from survey name
// e.g., "Eitibule Green Mineral Survey" -> "Eitibule Green Mineral"
// Returns the full survey type (zone + color + category) or "Unknown" if pattern doesn't match
function extractSurveyType(surveyName: string | undefined): string {
  if (!surveyName) return "Unknown";

  // Patterns: "Eitibule Green Mineral Survey", "Some Zone Color Geology Survey", etc.
  // Extract everything before " Survey" at the end
  const match = surveyName.match(/^(.+)\s+Survey$/i);
  if (match) {
    return match[1]; // Returns "Eitibule Green Mineral", etc.
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

function timestampToDate(ts: string): Date {
  const [h, m, s] = ts.split(":").map(Number);
  const now = new Date();
  now.setHours(h, m, s, 0);
  return now;
}

function xpDelta(newXp: number, baseline: number): number {
  if (baseline === 0) return 0;
  if (newXp >= baseline) return newXp - baseline;
  return newXp; // level-up rollover
}
