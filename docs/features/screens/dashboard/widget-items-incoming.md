# Widget: Items Incoming

**ID:** `items-incoming` | **Default size:** Medium | **Component:** `widgets/ItemsIncomingWidget.vue`

Activity feed (green dot) showing items gained during the session:
- Chat status only: `ItemGained` and `Summoned` events (Player.log item events excluded to avoid double-counting)
- Item names rendered via `ItemInline` for tooltips and navigation
- Up to 30 entries with summary footer
- Accuracy warning tooltip in the summary footer

**Data source:** `gameStateStore.itemsIncoming`. Session-only, in-memory.
