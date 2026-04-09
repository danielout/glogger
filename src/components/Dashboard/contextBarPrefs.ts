/**
 * Shared constants for the Status (context-bar) widget preferences.
 * Used by both ContextBar.vue (display) and ContextBarConfig.vue (config popover).
 */

/** Ordered list of known currency keys (lowercase) for display sorting. */
export const CURRENCY_DISPLAY_ORDER = [
  'gold',                // Councils
  'vidaria_renown',
  'statehelm_renown',
  'glamour_credits',
  'liveeventcredits',
  'combat_wisdom',
]

/** Display name overrides for currency keys from the game data. */
export const CURRENCY_DISPLAY_NAMES: Record<string, string> = {
  gold: 'Councils',
  liveeventcredits: 'Live Event Credits',
  glamour_credits: 'Glamour Credits',
  combat_wisdom: 'Combat Wisdom',
  vidaria_renown: 'Vidaria Renown',
  statehelm_renown: 'Statehelm Renown',
}

/** Known currency pref keys and their default visibility. */
export const CURRENCY_DEFAULTS: Record<string, boolean> = {
  currency_gold: true,
  currency_vidaria_renown: true,
  currency_statehelm_renown: true,
  currency_glamour_credits: true,
  currency_liveeventcredits: true,
  currency_combat_wisdom: false,
}

export interface ContextBarPrefs extends Record<string, unknown> {
  showGameTime: boolean
  showServerTime: boolean
  showLocalTime: boolean
  showMoon: boolean
  showWeather: boolean
  showCombat: boolean
  use24h: boolean
  // Per-currency toggles are dynamic: `currency_<lowercase_name>: boolean`
  [key: string]: unknown
}

export const CONTEXT_BAR_DEFAULTS: ContextBarPrefs = {
  showGameTime: true,
  showServerTime: true,
  showLocalTime: false,
  showMoon: true,
  showWeather: true,
  showCombat: true,
  use24h: true,
  ...CURRENCY_DEFAULTS,
}
