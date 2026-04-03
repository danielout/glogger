# TODO

Small tasks and notes that don't belong in a dedicated plan.

*Last reviewed: 2026-04-03*

---

## Completed

- [x] Create Python script for CDN field schema extraction — `scripts/extract_cdn_schemas.py` is fully functional
- [x] Timestamp standardization — unified UTC storage, display-time conversion, `Timestamp.vue` component, v2 migration
- [x] Layout standardization — all screens use `PaneLayout`, documented in `docs/architecture/layout-patterns.md`
- [x] Death tracking system — `DeathsView.vue`, combat parser, `character_deaths` + `death_damage_sources` tables
- [x] Search system — quick search overlay (Ctrl+F) + dedicated search page
- [x] Crafting skill reports — Skills tab with analytics, charts, recipe tables
- [x] Surveying analytics — zone analytics, item cost calculator, speed bonus charts, cross-zone comparison
- [x] Shared inline components — full set of entity inline/tooltip components for Items, NPCs, Skills, Quests, Recipes, Abilities, Areas, Enemies
- [x] Delete unused dashboard widgets — removed `ActiveSkillsWidget.vue` and `SessionWidget.vue`
- [x] Audit Rust build warnings — zero warnings on `cargo check`, cleaned up unused imports/dead code/unused variants
- [x] Death screen vital colors — `DeathsView.vue` now uses `text-vital-health` / `text-vital-armor` instead of hardcoded classes
- [x] Rapid crafting DB batching — `process_events_batch()` wraps all player events in a single SQLite transaction; coordinator flushes batched writes instead of per-event
- [x] Live crafting tracker improvements — primary detection now uses `RecipeUpdated` events (authoritative completion count with baseline tracking), with `ItemStackChanged` as fallback; added manual +/- adjustment buttons for missed/extra detections
- [x] Cook's helper food-first approach — new `get_recipes_producing_items` backend command replaces hardcoded skill names; auto-discovers all food-producing recipes (Cooking, Sushi Preparation, Cheesemaking, Mycology, etc.); dynamic skill filter pills
- [x] Vendor-purchasable segregation — new `get_vendor_purchasable_item_ids` command uses CDN sources data (Barter/Vendor entries); shopping list now only marks items as vendor-buyable when confirmed by sources data
- [x] NPC vendor inventory in data browser — `vendor_items_by_npc` index built at CDN load time; `get_npc_vendor_items` command returns resolved item data per NPC; NPC browser detail panel shows "Sells Items" section with `ItemInline` links and estimated vendor prices. No manual entry needed — CDN `sources_items.json` already has all Vendor/Barter data (~3k entries across ~440 NPCs).

---


## Medium Effort, High Value

- [ ] Update data browser to have better, more readable layouts
  - Design/UX task. All tabs already use `PaneLayout` with consistent two-pane search/detail pattern. Improvements are incremental polish: better section headers, consistent key-value grids, spacing per `docs/architecture/ux-standards.md`. Could break into sub-tasks per browser tab.
  - **Effort: Medium (iterative) | Impact: Medium (polish)**

---

## Larger Efforts / Research Needed

- [ ] Shop/stall tracking — track what you put in, what sells, trends (would require manual entry or future log support) (reported by Reyetta)
  - Big feature. Needs: investigation of what log events exist for stalls, schema for tracking stock/sales, analytics UI. Manual entry fallback would work but adoption is questionable (Reyetta herself said manual entry is "too much effort"). The player event parser handles ~24 of ~60 known event types — player vendor events are in the "low priority / future" bucket per the parser docs.
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

- [ ] Gear transmutation reference tool (reported by Reyetta)
  - No transmutation data in CDN. Would need manual data entry for transmutation rules, costs, and outcomes. Code is straightforward once data exists — the data is the bottleneck.
  - **Effort: Large (data acquisition) | Impact: Low-Medium (niche use case)**
