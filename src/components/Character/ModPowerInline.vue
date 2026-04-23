<template>
  <EntityTooltipWrapper
    border-class="border-purple-500/50"
    @hover="loadData"
  >
    <span class="text-xs text-purple-300 cursor-help hover:text-purple-200">
      {{ displayName }}
    </span>
    <template #tooltip>
      <div v-if="powerInfo" class="min-w-48">
        <div class="font-bold text-purple-300 text-sm mb-1">{{ powerInfo.internal_name }}</div>
        <div v-if="powerInfo.skill" class="text-text-muted text-xs mb-1">
          Skill: <span class="text-text-secondary">{{ powerInfo.skill }}</span>
        </div>
        <div v-if="powerInfo.prefix || powerInfo.suffix" class="text-text-muted text-xs mb-2">
          <span v-if="powerInfo.prefix">Prefix: <span class="text-text-secondary">{{ powerInfo.prefix }}</span></span>
          <span v-if="powerInfo.prefix && powerInfo.suffix"> · </span>
          <span v-if="powerInfo.suffix">Suffix: <span class="text-text-secondary">{{ powerInfo.suffix }}</span></span>
        </div>
        <div v-if="powerInfo.tier_effects.length" class="border-t border-purple-500/30 pt-1.5 mt-1.5">
          <div class="text-text-muted text-[10px] uppercase tracking-wide mb-1">Tier {{ tier }} Effects</div>
          <div
            v-for="(effect, i) in powerInfo.tier_effects"
            :key="i"
            class="text-accent-green text-xs leading-relaxed"
          >
            {{ effect }}
          </div>
        </div>
      </div>
      <div v-else class="text-text-muted text-xs">Loading...</div>
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import EntityTooltipWrapper from "../Shared/EntityTooltipWrapper.vue";

interface TsysPowerInfo {
  internal_name: string
  skill: string | null
  prefix: string | null
  suffix: string | null
  slots: string[]
  tier_effects: string[]
}

const props = defineProps<{
  power: string
  tier: number
}>();

const powerInfo = ref<TsysPowerInfo | null>(null);

const displayName = props.power
  .replace(/([a-z])([A-Z])/g, '$1 $2')
  .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2');

async function loadData() {
  if (powerInfo.value) return;
  try {
    powerInfo.value = await invoke<TsysPowerInfo | null>('get_tsys_power_info', {
      powerName: props.power,
      tier: props.tier,
    });
  } catch (e) {
    console.warn(`Failed to load TSys power: ${props.power}`, e);
  }
}
</script>
