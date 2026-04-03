# TODO

Small tasks and notes that don't belong in a dedicated plan.

---

## Quick Wins

- [ ] Delete unused dashboard widget files: `ActiveSkillsWidget.vue`, `SessionWidget.vue` (superseded by the dashboard rewrite — ContextBar, TransactionLog, PlayerNotes)
- [ ] Audit Rust build warnings (`cargo check` in `src-tauri/`) — clean up unused imports, dead fields, and stale enum variants
- [ ] Gourmand XP values shown in app appear wrong — verify which CDN data field is being used (reported by Reyetta)
  - Investigation found no XP values are displayed anywhere in food/gourmand UI. `FoodItem` has `food_level` and `gourmand_req` but no XP field. Actual XP lives in Recipe data (`reward_skill_xp`, `reward_skill_xp_first_time`). Need to clarify what Reyetta was seeing — may be a misleading label or missing feature rather than wrong data.
  - **Effort: Small | Impact: Medium (data correctness)**

- [ ] Create Python script for CDN field schema extraction (currently only exists as a one-off)
  - Tooling task. No user-facing impact, just makes our life easier when CDN data changes.
  - **Effort: Small | Impact: Low (developer QoL)**

---

## Medium Effort, High Value

- [ ] Rapid crafting (spam-crafting 25-100 items) causes game bugs — likely too many inventory update events firing at once. Investigate adding a rate limiter or debounce on log processing (reported by Reyetta)
  - Currently zero debouncing — every event triggers an immediate DB write via GameStateManager and a frontend emit. Spam-crafting 100 items = 200+ rapid-fire events (ItemAdded + ItemStackChanged per craft). Fix: batch inventory domain DB writes in GameStateManager — accumulate over a ~300-500ms window, flush in single transaction. Still emit to frontend immediately so UI feels responsive. The coordinator processes events synchronously so the batching goes in GameStateManager.
  - **Effort: Medium | Impact: High (reported bug, affects gameplay)**

- [ ] Live crafting tracker misses events sometimes — add manual checkbox fallback so users can tick off completed crafts (reported by Reyetta)
  - Current detection relies on item events which can miss lines. Better approach: `RecipeUpdated` events fire on craft completion with `recipe_id` + `completion_count` — this is a much more reliable signal than watching for item adds. Could switch detection to use RecipeUpdated (compare completion_count delta) AND add manual checkbox as fallback. Small schema addition for manual overrides.
  - **Effort: Small-Medium | Impact: Medium (reliability improvement)**

- [ ] Update data browser to have better, more readable layouts
  - Design/UX task. The data is all there, it's about presentation. Should follow the patterns in ux-standards.md (consistent section headers, key-value grids, spacing). Could break into sub-tasks per browser tab.
  - **Effort: Medium (iterative) | Impact: Medium (polish)**

---

## Larger Efforts / Research Needed

- [ ] Add barter tables and vendor tables to the data browser (not in CDN data, requires manual entry) (reported by Reyetta)
  - we should double check this isn't in the CDN data anywhere. i am pretty sure it isn't, but still.
  - If truly manual: need schema for barter_tables, a data entry UI or import mechanism, and a browser view. The data acquisition is the hard part, not the code.
  - **Effort: Large (data acquisition dominates) | Impact: Medium**

- [ ] Shop/stall tracking — track what you put in, what sells, trends (would require manual entry or future log support) (reported by Reyetta)
  - we have seen some actions come through in the player log, but we'd need to have a much more thorough investigation here i think
  - Big feature. Needs: investigation of what log events exist for stalls, schema for tracking stock/sales, analytics UI. Manual entry fallback would work but adoption is questionable (Reyetta herself said manual entry is "too much effort"). The player event parser handles ~24 of ~60 known event types — player vendor events are in the "low priority / future" bucket per the parser docs.
  - **Effort: Large | Impact: High (if automated), Low (if manual-only)**

- [ ] Gear transmutation reference tool (reported by Reyetta)
  - No transmutation data in CDN. Would need manual data entry for transmutation rules, costs, and outcomes. Code is straightforward once data exists — the data is the bottleneck.
  - **Effort: Large (data acquisition) | Impact: Low-Medium (niche use case)**

---

## Completed

- [x] Skill level display now accounts for synergy bonuses — `SkillCard.vue` shows `level + bonus_levels` with breakdown
- [x] Export crafting material list to shareable txt — button in ProjectMaterialsPanel, uses Tauri save dialog
- [x] `check_material_availability()` now queries `game_state_inventory` (log-driven) for player inventory, not just snapshot data
- [x] Interactive tooltips — `EntityTooltipWrapper` supports `interactive` prop; ItemInline tooltips stay open when hovered so users can click market value editing
- [x] Shopping list split into "Buy from Vendors" (with cost) and "Source Elsewhere" sections
- [x] Craftable ingredient tagging — `MaterialNeed.is_craftable` and `FlattenedMaterial.is_craftable` flow through from recipe resolution; "craftable" label shown in Source Elsewhere section
- [x] Cook's helper "Start Fresh" mode — can now start without importing a skill report (treats all foods as uneaten)
- [x] Vault materials grouped by zone — PickupList already groups by vault name (which maps to zone-specific storage NPCs)
- [x] Cook's helper shows owned food count — each recipe row shows "have X" if the dish is already in inventory/storage
- [x] Entity nav deep-linking — clicking inline entities (items, skills, quests, recipes, NPCs) navigates to DataBrowser AND auto-selects the entity in the detail pane
- [x] Scoped CSS fully converted to Tailwind — `ToastContainer.vue` uses TransitionGroup class props, `InventoryItemPanel.vue` uses arbitrary variant selectors. Zero scoped CSS remaining.



## TO BE SORTED

- Death screen: we've got the health/armor split on abilities, and we color health red (good) but we should color armor yellow to align with the game colors
- cook's helper: are we checking sushi and mycology recipes too? cheesemaking? should make sure we're checking all the craftable items that could grant gourmand
- suggestion from rey: when calculating the materials needed for a crafting project, segregate or distinguish between materials you can buy unlimited from a vendor and stuff you have to farm/find
  - the databrowser work has started to build cross references of stuff like this, so we should be able to find this list. we don't have vendor _prices_ anywhere i know of, but we at least know what is/isn't sold