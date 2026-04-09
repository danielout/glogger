# Widget: Councils

**ID:** `councils` | **Default size:** Medium | **Component:** `widgets/CouncilsWidget.vue`

Activity feed (yellow dot) showing gold/council currency changes:
- Chat status: `CouncilsChanged` (received/spent) and `CoinsLooted` (corpse search)
- Player.log: `VendorSold` events (sale price)
- Shows signed running total (+/- format)
- Accuracy warning tooltip in the summary footer

**Data source:** `gameStateStore.councilChanges`. Session-only, in-memory.
