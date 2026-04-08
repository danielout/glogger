import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/** The 8 moon phase names used by the game, in cycle order starting from New Moon. */
export type MoonPhaseName =
  | 'NewMoon'
  | 'WaxingCrescentMoon'
  | 'QuarterMoon'
  | 'WaxingGibbousMoon'
  | 'FullMoon'
  | 'WaningGibbousMoon'
  | 'LastQuarterMoon'
  | 'WaningCrescentMoon'

export interface MoonPhaseInfo {
  name: MoonPhaseName
  label: string
  emoji: string
}

interface DaysUntilPhase {
  game_phase: string
  label: string
  days: number
}

interface MoonPhaseResult {
  game_phase: string
  label: string
  phase_index: number
  days_until: DaysUntilPhase[]
}

const PHASE_EMOJI: Record<MoonPhaseName, string> = {
  NewMoon: '🌑',
  WaxingCrescentMoon: '🌒',
  QuarterMoon: '🌓',
  WaxingGibbousMoon: '🌔',
  FullMoon: '🌕',
  WaningGibbousMoon: '🌖',
  LastQuarterMoon: '🌗',
  WaningCrescentMoon: '🌘',
}

export const ALL_PHASES: MoonPhaseInfo[] = [
  { name: 'NewMoon', label: 'New Moon', emoji: '🌑' },
  { name: 'WaxingCrescentMoon', label: 'Waxing Crescent', emoji: '🌒' },
  { name: 'QuarterMoon', label: 'First Quarter', emoji: '🌓' },
  { name: 'WaxingGibbousMoon', label: 'Waxing Gibbous', emoji: '🌔' },
  { name: 'FullMoon', label: 'Full Moon', emoji: '🌕' },
  { name: 'WaningGibbousMoon', label: 'Waning Gibbous', emoji: '🌖' },
  { name: 'LastQuarterMoon', label: 'Last Quarter', emoji: '🌗' },
  { name: 'WaningCrescentMoon', label: 'Waning Crescent', emoji: '🌘' },
]

/** Reactive composable that tracks the current moon phase via the Rust backend. */
export function useMoonPhase() {
  const phase = ref<MoonPhaseInfo | null>(null)
  const daysUntil = ref<DaysUntilPhase[]>([])
  const loading = ref(true)

  async function refresh() {
    try {
      const result = await invoke<MoonPhaseResult>('get_current_moon_phase')
      const name = result.game_phase as MoonPhaseName
      phase.value = {
        name,
        label: result.label,
        emoji: PHASE_EMOJI[name] ?? '🌙',
      }
      daysUntil.value = result.days_until
    } catch (e) {
      console.error('[useMoonPhase] Failed to get moon phase:', e)
    } finally {
      loading.value = false
    }
  }

  let interval: ReturnType<typeof setInterval> | null = null

  onMounted(() => {
    refresh()
    // Recheck every 10 minutes — phase only changes at midnight Eastern
    interval = setInterval(refresh, 10 * 60_000)
  })

  onUnmounted(() => {
    if (interval) clearInterval(interval)
  })

  return {
    phase,
    daysUntil,
    loading,
  }
}
