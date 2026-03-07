import { defineStore } from "pinia";
import { ref, computed } from "vue";

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

export interface SessionStats {
  startTime: string;
  mapsStarted: number;
  surveysLocated: number;
  surveysCompleted: number;
  surveyingXpGained: number;
  miningXpGained: number;
  geologyXpGained: number;
  _surveyingXpBaseline: number;
  _miningXpBaseline: number;
  _geologyXpBaseline: number;
  completionTimestamps: string[]; // NEW: timestamp of each completed survey
  lootTotals: Record<string, number>; // NEW: item name -> total count
}

const SURVEY_SKILLS = ["Surveying", "Mining", "Geology"];

export const useSurveyStore = defineStore("survey", () => {
  const sessionActive = ref(false);
  const session = ref<SessionStats | null>(null);
  const log = ref<SurveyLogEntry[]>([]);

  function handleSurveyEvent(payload: {
    kind: string;
    timestamp: string;
    map_name?: string;
    survey_name?: string;
    direction_hint?: string;
    loot_text?: string;
  }) {
    if (payload.kind === "SessionStart") {
      if (!sessionActive.value) {
        sessionActive.value = true;
        session.value = {
          startTime: payload.timestamp,
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
        };
      } else if (session.value) {
        session.value.mapsStarted++;
      }
      log.value.unshift({
        kind: "session-start",
        timestamp: payload.timestamp,
        label: "Survey run started",
      });
    }

    if (payload.kind === "Located" && session.value) {
      session.value.surveysLocated++;
      log.value.unshift({
        kind: "located",
        timestamp: payload.timestamp,
        label: `Located: ${payload.survey_name}`,
      });
    }

    if (payload.kind === "Completed" && session.value) {
      session.value.surveysCompleted++;
      session.value.completionTimestamps.push(payload.timestamp);

      // Parse and tally loot
      if (payload.loot_text) {
        const items = parseLootText(payload.loot_text);
        for (const { item, count } of items) {
          session.value.lootTotals[item] =
            (session.value.lootTotals[item] ?? 0) + count;
        }
      }

      log.value.unshift({
        kind: "completed",
        timestamp: payload.timestamp,
        label: payload.survey_name ?? "Survey",
        lootText: payload.loot_text,
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

  const elapsed = computed(() => {
    if (!session.value) return "—";
    const start = tsToSeconds(session.value.startTime);
    const lastTs = log.value[0]?.timestamp;
    if (!lastTs) return "—";
    const diff = Math.max(0, tsToSeconds(lastTs) - start);
    const m = Math.floor(diff / 60);
    const s = diff % 60;
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

  return {
    sessionActive,
    session,
    log,
    elapsed,
    avgSurveyTime,
    lootSummary,
    handleSurveyEvent,
    handleSkillUpdate,
    reset,
  };
});

// --- helpers ---

function tsToSeconds(ts: string): number {
  const [h, m, s] = ts.split(":").map(Number);
  return h * 3600 + m * 60 + s;
}

function xpDelta(newXp: number, baseline: number): number {
  if (baseline === 0) return 0;
  if (newXp >= baseline) return newXp - baseline;
  return newXp; // level-up rollover
}

// Parses "Tsavorite collected! Also found Moss Agate x2 (speed bonus!)"
// Returns array of { item, count }
function parseLootText(text: string): LootEntry[] {
  const results: LootEntry[] = [];

  // Primary item: everything before " collected!"
  const collectedIdx = text.indexOf(" collected!");
  if (collectedIdx > 0) {
    results.push({ item: text.slice(0, collectedIdx).trim(), count: 1 });
  }

  // Secondary items: after "Also found "
  const alsoIdx = text.indexOf("Also found ");
  if (alsoIdx >= 0) {
    // Strip trailing "(speed bonus!)" or similar parentheticals
    let secondary = text.slice(alsoIdx + "Also found ".length);
    secondary = secondary.replace(/\s*\(.*?\)/g, "").trim();

    // Split on commas
    for (const part of secondary.split(",")) {
      const piece = part.trim();
      if (!piece) continue;

      // Match "Item Name x3" or just "Item Name"
      const xMatch = piece.match(/^(.+?)\s+x(\d+)$/);
      if (xMatch) {
        results.push({ item: xMatch[1].trim(), count: parseInt(xMatch[2]) });
      } else {
        results.push({ item: piece, count: 1 });
      }
    }
  }

  return results;
}
