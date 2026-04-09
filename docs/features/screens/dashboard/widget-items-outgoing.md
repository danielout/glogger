# Widget: Items Outgoing

**ID:** `items-outgoing` | **Default size:** Medium | **Component:** `widgets/ItemsOutgoingWidget.vue`

Activity feed (red dot) showing items lost during the session:
- Player.log only: `ItemDeleted` (with context: sold, stored, consumed) and `ItemStackChanged` (negative delta)
- Chat status has no item removal events, so Player.log is the sole source
- Item names rendered via `ItemInline`
- Uses quantity prefix format (x5 instead of +5)

**Data source:** `gameStateStore.itemsOutgoing`. Session-only, in-memory.
