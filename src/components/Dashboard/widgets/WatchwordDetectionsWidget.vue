<template>
  <div class="flex flex-col h-full">
    <div v-if="rules.length === 0" class="text-xs text-text-dim italic">
      No watch rules configured. Set up rules in the Chat &gt; Watchwords tab.
    </div>

    <div v-else-if="loading" class="text-xs text-text-dim italic">Loading recent matches...</div>

    <template v-else>
      <div v-if="recentMatches.length === 0" class="text-xs text-text-dim italic">
        No recent matches found.
      </div>

      <div v-else class="flex flex-col gap-0.5 overflow-y-auto max-h-52 pr-1">
        <div
          v-for="match in recentMatches"
          :key="`${match.ruleId}-${match.message.id}`"
          class="flex items-start gap-2 py-1 px-2 rounded text-xs hover:bg-surface-elevated/50 cursor-pointer"
          @click="goToRule(match.ruleId)">
          <!-- Timestamp -->
          <span class="text-text-dim shrink-0">{{ formatTs(match.message.timestamp) }}</span>

          <!-- Rule badge -->
          <span class="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase tracking-wide bg-accent-gold/15 text-accent-gold border border-accent-gold/20 shrink-0 leading-none">
            {{ match.ruleName }}
          </span>

          <!-- Message content -->
          <span class="text-text-primary truncate min-w-0">
            <span v-if="match.message.sender" class="text-text-secondary">{{ match.message.sender }}:</span>
            {{ match.message.message }}
          </span>
        </div>
      </div>

      <!-- Summary -->
      <div v-if="recentMatches.length > 0" class="mt-auto pt-2 border-t border-border-default text-xs text-text-muted">
        {{ recentMatches.length }} recent match{{ recentMatches.length === 1 ? '' : 'es' }} across {{ activeRuleCount }} rule{{ activeRuleCount === 1 ? '' : 's' }}
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../../../stores/settingsStore'
import { formatAnyTimestamp } from '../../../composables/useTimestamp'
import { useViewNavigation, pendingWatchwordRuleId } from '../../../composables/useViewNavigation'
import type { ChatMessage } from '../../../types/database'

const { navigateToView } = useViewNavigation()

const settingsStore = useSettingsStore()
const loading = ref(false)

interface MatchEntry {
  ruleId: number
  ruleName: string
  message: ChatMessage
}

const recentMatches = ref<MatchEntry[]>([])

const rules = computed(() => settingsStore.settings.watchRules)
const activeRuleCount = computed(() => rules.value.filter(r => r.enabled).length)

function goToRule(ruleId: number) {
  pendingWatchwordRuleId.value = ruleId
  navigateToView({ view: 'chat', subTab: 'watchwords' })
}

function formatTs(ts: string): string {
  return formatAnyTimestamp(ts)
}

async function loadMatches() {
  const enabledRules = rules.value.filter(r => r.enabled)
  if (enabledRules.length === 0) return

  loading.value = true
  try {
    const allMatches: MatchEntry[] = []

    await Promise.all(enabledRules.map(async (rule) => {
      const messages = await invoke<ChatMessage[]>('get_watch_rule_messages', {
        ruleId: rule.id,
        limit: 10,
        offset: 0,
      })
      for (const msg of messages) {
        allMatches.push({ ruleId: rule.id, ruleName: rule.name, message: msg })
      }
    }))

    // Sort by timestamp descending, take most recent 15
    allMatches.sort((a, b) => b.message.timestamp.localeCompare(a.message.timestamp))
    recentMatches.value = allMatches.slice(0, 15)
  } catch (e) {
    console.error('[WatchwordDetections] Failed to load matches:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => loadMatches())
</script>
