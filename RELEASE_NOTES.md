## What's Changed since v0.5.1

### Features
- feat(StallTracker): Phase 12 — polish (`0c2d435`)
- feat(StallTracker): Phase 11 — store-level race protection (`8eb7291`)
- feat(StallTracker): Phase 10 — Import / Export / Clear (`cf53f1a`)
- feat(StallTracker): custom DatePicker, replaces native date inputs (`e5ca778`)
- feat(StallTracker): Phase 9 — Shop Log modal (`c068e85`)
- feat(StallTracker): Phase 8 — Inventory tab with last-known-price (`9c8de45`)
- feat(StallTracker): Phase 7 — Revenue pivot tab (`bed2c23`)
- feat(StallTracker): Phase 6 — Sales tab (`33715fb`)
- feat(StallTracker): Phase 5 — Pinia store + parent view (`2d87d05`)
- feat(StallTracker): Phase 4 — Revenue + Inventory aggregations (`0e6900a`)
- feat(StallTracker): Phase 3 — Tauri CRUD commands (`df960e7`)
- feat(StallTracker): Phase 2 — live ingest via coordinator (`6a289e6`)
- feat(StallTracker): Phase 1 — parser, year resolver, migration (`eae0dd8`)
- feat: dev tools screen for testing (`c89e34f`)
- feat: lorebook databrowser (`c12bd40`)

### Fixes
- fix(DatePicker): sync visible month when modelValue changes externally (`65ecb60`)
- fix(StallTracker): clear filterTimer on tab unmount (`8a680dd`)
- fix(StallTracker): clear stale error on no-owner reload across all tabs (`5f1eae6`)
- fix(StallTracker): reset actionInProgress on parent view unmount (`ffc2099`)
- fix(StallTracker): wrap handleImport body in try/finally (`c9218a1`)
- fix(DataBrowser): replace nested buttons in sidebar list items (`f5e2ddc`)
- fix(StallTracker): drop unused backfill_year helper (`d2b6599`)
- fix(StallTracker): make Recently Sold Out table sortable (`9f72fec`)
- fix(StallTracker): render Revenue pivot items via ItemInline (`a60af64`)
- fix(StallTracker): make Shop Log table sortable (`29ec79a`)
- fix(StallTracker): make Inventory In Stock table sortable (`f987968`)
- fix(StallTracker): reverse Revenue period columns to newest-first (`2fafb42`)
- fix(StallTracker): render items via ItemInline for tooltips and navigation (`ba0b03b`)
- fix(StallTracker): extract xN quantity from stacked bought listings (`5c111ad`)
- fix(StallTracker): blur date inputs on outside click (`9c9c33c`)
- fix: fixed multi output recipes moving from level planner to projects fix: slightly improved item tracking feat: known issues listin ? menu (`635fe04`)
- fix: statehelm gift tracker doesn't require you to be on the page fix: better item name matching for gear fix: some tooltip clipping issues (`7f2bad4`)

---
*42 commits since v0.5.1*
