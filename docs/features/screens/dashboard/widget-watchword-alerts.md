# Widget: Watchword Alerts

**ID:** `watchword-detections` | **Default size:** Medium | **Component:** `widgets/WatchwordDetectionsWidget.vue`

Compact feed of recent watchword matches across all enabled rules:
- Each entry shows: timestamp, rule name badge (gold tag), sender, and message text (truncated)
- Fetches up to 10 matches per enabled rule, merges all, sorts by timestamp, displays most recent 15
- Summary footer with total match count and active rule count
- Empty states for "no rules configured" and "no matches found"
- Clicking a match row navigates to Chat > Watchwords with that rule selected

Matches are fetched on mount via `invoke('get_watch_rule_messages', { ruleId, limit, offset })` for each enabled rule in `settingsStore.settings.watchRules`.

**Data source:** `settingsStore.settings.watchRules` (rules), `get_watch_rule_messages` Tauri command (matches from database).
