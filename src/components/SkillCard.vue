<script setup lang="ts">
import { computed } from 'vue'
import { useSkillStore, type SkillStat } from '../stores/skillStore'

const props = defineProps<{ skill: SkillStat }>()
const store = useSkillStore()

const xphr = computed(() => store.xpPerHour(props.skill).toLocaleString())
const ttl  = computed(() => store.timeToNextLevel(props.skill))

const tnlPercent = computed(() => {
  const total = props.skill.xpGained + props.skill.tnl
  if (total <= 0) return 0
  return Math.min(100, Math.round((props.skill.xpGained / total) * 100))
})
</script>

<template>
  <div class="skill-card">
    <div class="skill-name">{{ skill.skillType }}</div>
    <div class="skill-level">Lv <span class="highlight">{{ skill.currentLevel }}</span></div>

    <div class="stat-grid">
      <div class="stat">
        <div class="stat-label">XP Gained</div>
        <div class="stat-value">{{ skill.xpGained.toLocaleString() }}</div>
      </div>
      <div class="stat">
        <div class="stat-label">XP / Hour</div>
        <div class="stat-value">{{ xphr }}</div>
      </div>
      <div class="stat">
        <div class="stat-label">Levels Gained</div>
        <div class="stat-value">{{ skill.levelsGained }}</div>
      </div>
      <div class="stat">
        <div class="stat-label">Next Level</div>
        <div class="stat-value">{{ ttl }}</div>
      </div>
    </div>

    <div class="tnl-bar-wrap">
      <div class="tnl-bar" :style="{ width: tnlPercent + '%' }"></div>
    </div>
    <div class="tnl-label">{{ skill.tnl.toLocaleString() }} XP to next level</div>
  </div>
</template>

<style scoped>
.skill-card {
  background: #1a1a2e;
  border: 1px solid #333;
  border-radius: 8px;
  padding: 1rem;
  min-width: 200px;
}
.skill-name { font-size: 1rem; font-weight: bold; color: #e0c060; margin-bottom: 0.25rem; }
.skill-level { font-size: 0.8rem; color: #888; margin-bottom: 0.75rem; }
.highlight { color: #fff; font-weight: bold; }
.stat-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; margin-bottom: 0.75rem; }
.stat-label { font-size: 0.65rem; color: #666; text-transform: uppercase; letter-spacing: 0.05em; }
.stat-value { font-size: 0.9rem; color: #ccc; font-weight: bold; }
.tnl-bar-wrap { height: 4px; background: #333; border-radius: 2px; overflow: hidden; margin-bottom: 0.25rem; }
.tnl-bar { height: 100%; background: #e0c060; border-radius: 2px; transition: width 0.3s ease; }
.tnl-label { font-size: 0.65rem; color: #555; }
</style>