import { defineStore } from "pinia";
import { ref } from "vue";

export interface SkillStat {
  skillType: string;
  currentLevel: number;
  tnl: number;
  xpGained: number;
  levelsGained: number;
  firstTimestamp: string; // "HH:MM:SS" of first seen event
  lastTimestamp: string; // "HH:MM:SS" of most recent event
  firstXp: number; // xp value at first event (for delta calc)
}

function timestampToSeconds(ts: string): number {
  const [h, m, s] = ts.split(":").map(Number);
  return h * 3600 + m * 60 + s;
}

export const useSkillStore = defineStore("skills", () => {
  const skills = ref<Record<string, SkillStat>>({});

  function handleUpdate(payload: {
    skill_type: string;
    level: number;
    xp: number;
    tnl: number;
    timestamp: string;
  }) {
    const key = payload.skill_type;

    if (!skills.value[key]) {
      // First time we see this skill
      skills.value[key] = {
        skillType: payload.skill_type,
        currentLevel: payload.level,
        tnl: payload.tnl,
        xpGained: 0,
        levelsGained: 0,
        firstTimestamp: payload.timestamp,
        lastTimestamp: payload.timestamp,
        firstXp: payload.xp,
      };
    } else {
      const s = skills.value[key];
      const prevLevel = s.currentLevel;

      s.currentLevel = payload.level;
      s.tnl = payload.tnl;
      s.lastTimestamp = payload.timestamp;

      // XP gained = difference between current xp and the first snapshot
      // The log reports cumulative xp-toward-level, so we track net gain
      // by accumulating the delta on each update
      // We use the running xp field: each update gives us current xp in level
      // so total gained = (levels * avg_xp_per_level) + current_xp - first_xp
      // Simpler: just add the positive xp deltas each tick
      if (payload.xp >= s.firstXp || payload.level > prevLevel) {
        if (payload.level > prevLevel) {
          // Level-up: add whatever was left to gain in the old level, then current xp
          s.xpGained += s.tnl - s.firstXp + payload.xp;
          s.levelsGained += payload.level - prevLevel;
          s.firstXp = payload.xp;
        } else {
          s.xpGained += payload.xp - s.firstXp;
          s.firstXp = payload.xp;
        }
      }
    }
  }

  function reset() {
    skills.value = {};
  }

  function xpPerHour(skill: SkillStat): number {
    const startSec = timestampToSeconds(skill.firstTimestamp);
    const endSec = timestampToSeconds(skill.lastTimestamp);
    const elapsedHours = (endSec - startSec) / 3600;
    if (elapsedHours <= 0) return 0;
    return Math.round(skill.xpGained / elapsedHours);
  }

  function timeToNextLevel(skill: SkillStat): string {
    const rate = xpPerHour(skill);
    if (rate <= 0) return "—";
    const hoursLeft = skill.tnl / rate;
    const totalMinutes = Math.round(hoursLeft * 60);
    if (totalMinutes < 1) return "< 1 min";
    if (totalMinutes < 60) return `~${totalMinutes} min`;
    const h = Math.floor(totalMinutes / 60);
    const m = totalMinutes % 60;
    return m > 0 ? `~${h}h ${m}m` : `~${h}h`;
  }

  return { skills, handleUpdate, reset, xpPerHour, timeToNextLevel };
});
