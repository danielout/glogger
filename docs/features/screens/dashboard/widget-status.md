# Widget: Status

**ID:** `context-bar` | **Default size:** Small | **Component:** `ContextBar.vue`

Compact at-a-glance card combining time, environment, and economy info in an organized two-column layout.

## Layout

**Top section** — two columns side by side:
- **Left column: Times** — Game time, Server time, and Local time stacked vertically (each independently toggleable)
- **Right column: Moon phase** — large phase emoji, phase label, days until next phase, and days until full moon (hidden when already full moon)

**Middle section** — horizontal row:
- **Weather** — current zone weather
- **Status** — Combat/mount badges (red "In Combat", blue "Mounted", or "Idle")
- **Active effects count** — shown when > 0

**Bottom section** — currencies listed vertically with label left and gold-accented amount right. Each currency has its own visibility toggle. "Gold" from game data displays as "Councils" (the in-game name). Currencies are displayed in a fixed order: Councils, Vidaria Renown, Statehelm Renown, Glamour Credits, Live Event Credits, Combat Wisdom.

Sections are separated by horizontal dividers that only render when both adjacent sections are visible.

## Data sources

- **Game time** — 12 game days per real day (1 game hour = 5 real minutes), anchored to Eastern midnight, from `gameStateStore.gameTime`
- **Server time** — US Eastern (where servers are hosted), from `gameStateStore.serverTime`
- **Local time** — optional, off by default (useful when player isn't in Eastern)
- **Moon phase** — current phase emoji, label, and days until next phase / full moon (via `useMoonPhase` composable)
- **Weather** — current zone weather from `gameStateStore.world.weather`
- **Combat/mount status** — from `gameStateStore.world.combat` / `gameStateStore.world.mount`
- **Active effects** — count of `gameStateStore.namedEffects`
- **Currencies** — non-zero balances from `gameStateStore.currencies`

Clock refs are updated every second in the store. Times display without seconds in either 24h or 12h format.

## Configurable sections

All toggles are in the gear icon config popover. Preferences persist via `useViewPrefs('widget.context-bar')`.

| Option | Key | Default |
|---|---|---|
| Game Time | `showGameTime` | on |
| Server Time | `showServerTime` | on |
| Local Time | `showLocalTime` | off |
| 24-hour format | `use24h` | on |
| Moon Phase | `showMoon` | on |
| Weather | `showWeather` | on |
| Combat / Mount | `showCombat` | on |
| Councils | `currency_gold` | on |
| Vidaria Renown | `currency_vidaria_renown` | on |
| Statehelm Renown | `currency_statehelm_renown` | on |
| Glamour Credits | `currency_glamour_credits` | on |
| Live Event Credits | `currency_liveeventcredits` | on |
| Combat Wisdom | `currency_combat_wisdom` | off |

Currency display names and sort order are defined in `contextBarPrefs.ts`. Unknown currencies from the character data appear at the end and default to visible.

**Config component:** `ContextBarConfig.vue` — renders section toggles and per-currency toggles in the DashboardCard config popover.

**Shared prefs:** Both `ContextBar.vue` and `ContextBarConfig.vue` share the same reactive pref instance via `useViewPrefs`, so config changes are reflected immediately without a reload.
