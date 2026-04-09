# Widget: Status

**ID:** `context-bar` | **Default size:** Small | **Component:** `ContextBar.vue`

Compact at-a-glance card combining time, environment, and economy info:

- **Server time** — US Eastern (where servers are hosted), from `gameStateStore.serverTime`
- **Game time** — 12 game days per real day (1 game hour = 5 real minutes), anchored to Eastern midnight, from `gameStateStore.gameTime`
- **Local time** — optional, off by default (useful when player isn't in Eastern)
- **Moon phase** — current phase emoji, label, and days until next phase (via `useMoonPhase` composable)
- **Weather** — current zone weather
- **Combat/mount status** — red/blue badges for in-combat/mounted, "Idle" otherwise
- **Active effects count** — shown when > 0
- **Currencies** — non-zero balances, gold-accented amounts

**Configurable sections:** Each row can be toggled on/off via the gear icon. Preferences persist via `useViewPrefs('widget.context-bar')`.

| Option | Key | Default |
|---|---|---|
| Server / Game Time | `showTime` | on |
| Local Time | `showLocalTime` | off |
| Moon Phase | `showMoon` | on |
| Weather | `showWeather` | on |
| Combat / Mount | `showCombat` | on |
| Currencies | `showCurrencies` | on |

**Data sources:** `gameStateStore` (serverTime, gameTime, world, namedEffects, currencies), `useMoonPhase()`. Clock refs are updated every second in the store so any component can read them.

**Config component:** `ContextBarConfig.vue` — renders toggle checkboxes in the DashboardCard config popover.
