# Widget Sizing Audit

Audit of the dashboard widget sizing system, covering the registry, grid layout, DashboardCard wrapper, and per-widget height behavior.

## Current System

### Widget Registry

**File:** [src/components/Dashboard/dashboardWidgets.ts](../../src/components/Dashboard/dashboardWidgets.ts)

Each widget has a `defaultSize` of `"small"`, `"medium"`, or `"large"`. These map to CSS grid column spans via `SIZE_CLASSES`:

- **small** = `""` (1 column, no class)
- **medium** = `"col-span-2"` (2 columns)
- **large** = `"col-span-4"` (4 columns)

The registry defines no height information. Size is purely about width (column span).

### Grid System

**File:** [src/components/Dashboard/DashboardView.vue](../../src/components/Dashboard/DashboardView.vue)

The grid uses `grid-template-columns: repeat(auto-fill, minmax(280px, 1fr))` as an inline style on the `<draggable>` element. This means:

- Column count is dynamic based on viewport width
- At 1120px+ viewport: 4 columns of ~280px
- At 840px: 3 columns
- At 560px: 2 columns
- Below 560px: 1 column

**Problem:** `col-span-4` (large widgets) assumes 4 columns exist. If the viewport only provides 3 columns, the widget overflows or the grid breaks. There is no `@media` or container query logic to adapt spans.

### DashboardCard Wrapper

**File:** [src/components/Dashboard/DashboardCard.vue](../../src/components/Dashboard/DashboardCard.vue)

DashboardCard provides:
- A title bar (drag handle)
- A content area with `class="p-4 flex-1 min-h-0 overflow-visible"`

Key observations:
- **`overflow-visible`** on the content area means child content can visually escape the card boundaries. No clipping or scrolling is enforced at the card level.
- **No max-height** is set on DashboardCard itself. Cards grow unbounded unless the widget component sets its own constraints.
- **`min-h-0`** is correct for flex children but without a max-height constraint, it only prevents the card from being forced taller than its content.

## Per-Widget Inventory

| Widget | ID | Registered Size | Height Constraint | Scrollable | Content Behavior |
|---|---|---|---|---|---|
| Status | `context-bar` | small | None | No | Content-sized, compact. Grows slightly based on toggled sections. |
| Live Skill Tracking | `skill-tracking` | large | None | No | `flex-wrap` layout of SkillCards. Grows unbounded with more skills. |
| Zone NPCs | `zone-npcs` | medium | `max-h-80` (320px) on NPC list | Yes | Scrollable NPC list, capped at 320px. |
| Items Incoming | `items-incoming` | medium | `max-h-52` (208px) via ActivityFeed | Yes | ActivityFeed sets max-h-52 on entry list. |
| Items Outgoing | `items-outgoing` | medium | `max-h-52` (208px) via ActivityFeed | Yes | ActivityFeed sets max-h-52 on entry list. |
| Councils | `councils` | medium | `max-h-52` (208px) via ActivityFeed | Yes | ActivityFeed sets max-h-52 on entry list. |
| Favor Changes | `favor-changes` | medium | `max-h-52` (208px) via ActivityFeed | Yes | ActivityFeed sets max-h-52 on entry list. |
| Notes | `player-notes` | medium | `max-h-64` (256px) on notes list | Yes | Input + scrollable list. Capped at 256px. |
| Critical Resources | `critical-resources` | small | None | No | Static list of 6 items. Small, fixed content. |
| Statehelm Gifting | `statehelm-summary` | medium | None | No | Progress bar + up to 5 NPCs. Grows with NPC count (capped at 5 in code). |
| Watchword Alerts | `watchword-detections` | medium | `max-h-52` (208px) on matches list | Yes | Scrollable matches list, capped at 208px. |
| Death Tracker | `death-tracker` | medium | None | No | Shows up to 5 recent deaths + up to 5 rezzers + up to 5 rezzed. Grows with content. |
| Recipe Items | `recipe-items` | medium | `max-h-40` (160px) per section (x3) | Yes | Three independently scrollable sections, each capped at 160px. Total can reach ~480px+ with headers. |
| Gift Watcher | `gift-watcher` | medium | None (dropdown has `max-h-48`) | No | NPC match list grows unbounded. Only the search dropdown is height-capped. |
| Teleport Codes | `teleport-codes` | medium | `max-h-80` (320px) on code list | Yes | Search + zone filter + scrollable code list. |
| Milking Timers | `milking-timers` | small | None (uses `flex-1 overflow-y-auto`) | Yes | Tab content uses `flex-1 overflow-y-auto min-h-0` -- relies on parent constraining height, but parent (DashboardCard) does not. Effectively unbounded. |
| Mushroom Farming | `mushroom-farming` | large | `max-h-80` (320px) on table area | Yes | Filter controls + scrollable table. |
| Stat Tracker | `stat-tracker` | small | None | No | Simple stat list. Grows with tracked stats (configurable). |
| Words of Power | `words-of-power` | medium | None (uses `flex-1 overflow-y-auto`) | Yes | Like Milking Timers: uses flex-1 pattern that needs a constrained parent. Effectively unbounded. |
| Trip Planner | `trip-planner` | medium | `max-h-64` (256px) on route display | Yes | Form inputs + scrollable route results. |

## Inconsistencies

### 1. No standardized height per size category

There is no relationship between a widget's registered size (small/medium/large) and its height. A "small" widget and a "large" widget can be the same height or wildly different. The size system only controls width.

### 2. Inconsistent max-height values across widgets

Widgets that do cap height use at least five different values:
- `max-h-40` (160px) -- RecipeItems sections
- `max-h-52` (208px) -- ActivityFeed (Items Incoming/Outgoing, Councils, Favor Changes, Watchword Alerts)
- `max-h-64` (256px) -- PlayerNotes, TripPlanner route
- `max-h-80` (320px) -- ZoneNpcs, TeleportCodes, MushroomFarming

### 3. Several widgets grow unbounded

These widgets have no height cap at all and grow with content:
- **Skill Tracking** (large) -- wrapping flex of SkillCards, can get very tall with many skills
- **Death Tracker** (medium) -- up to ~15 items across 3 sections + summary
- **Gift Watcher** (medium) -- match list for N watched NPCs with no limit
- **Statehelm Gifting** (medium) -- capped at 5 NPCs in code, but no CSS height limit

### 4. Flex-1 pattern broken without constrained parent

**Milking Timers** and **Words of Power** both use `h-full min-h-0` on their root and `flex-1 overflow-y-auto min-h-0` on their scrollable area. This pattern only works when the parent provides a height constraint. Since DashboardCard has `overflow-visible` and no max-height, these widgets grow to full content height -- the scroll never activates.

### 5. DashboardCard overflow-visible defeats containment

The content slot in DashboardCard uses `overflow-visible`, which means tooltips and popovers can escape the card (intentional), but it also means no scrolling or clipping happens at the card level. Height containment is entirely delegated to individual widgets, with no safety net.

### 6. Large widget col-span-4 breaks on narrow viewports

The auto-fill grid creates fewer than 4 columns at viewport widths below ~1120px. When this happens, `col-span-4` causes the widget to overflow the grid. There is no responsive fallback (no `@media` queries or `min()` clamping on the span).

### 7. Medium widget col-span-2 at single-column viewports

At very narrow widths (below ~560px), the grid has only 1 column. `col-span-2` widgets will also overflow. This may be acceptable for a desktop Tauri app, but it is a latent issue.

## Proposed Standards

### Uniform height, variable width

**Size controls width only.** All widgets must be the same height regardless of their size category (small/medium/large). This ensures every row of the dashboard grid stays visually aligned no matter how the user reorders widgets. A small 1-column widget and a large 4-column widget sitting in the same row should be the same height.

One uniform max-height should be chosen and enforced by DashboardCard — widgets fill up to that height and scroll internally if their content exceeds it. Widgets with less content simply leave empty space at the bottom (or center their content vertically where appropriate).

The exact height value needs to be determined empirically — try `max-h-80` (320px) or `max-h-96` (384px) and see what feels right with the actual widget content.

### DashboardCard should enforce the uniform height

Rather than relying on each widget to set its own max-height, DashboardCard should:
1. Apply the single uniform max-height (and a matching fixed height) to the card wrapper
2. Change the content area from `overflow-visible` to `overflow-hidden` or `overflow-auto`

Tooltips and popovers should use portals (teleport to body) rather than relying on `overflow-visible` to escape card boundaries.

### Widget internal scrolling pattern

Widgets with variable-length content should use the flex-1 scroll pattern:
```
root: flex flex-col h-full min-h-0
scrollable area: flex-1 overflow-y-auto min-h-0
```
This pattern works correctly when the parent (DashboardCard) provides the uniform height constraint.

Widgets should remove their own `max-h-*` classes once DashboardCard enforces height, since the card-level constraint makes per-widget caps redundant.

### Grid column derivation

Column count should be derived from available panel width divided by the small widget width: `columns = floor(panel_width / small_widget_width)`. The current `auto-fill, minmax(280px, 1fr)` approach is close to this already — the problem is only that `col-span-4` can exceed the available columns.

Since medium = 2 columns and large = 4 columns, these are exact multiples of small. As long as the panel is wide enough for 4 columns (which it will be at any reasonable app window size), the spans just work. No breakpoint logic needed — the grid math handles it naturally.

The minimum panel width should be at least `4 × small_widget_width` to guarantee large widgets fit. Below that threshold the layout degrades, but no one should have their app window that small anyway.

## Migration Plan

### Phase 1: DashboardCard uniform height enforcement

1. Choose a single uniform height value (start with `max-h-80` or `max-h-96`, test visually)
2. Apply both `h-[value]` and `max-h-[value]` to DashboardCard's root `div` so all cards are the same height
3. Change the content area overflow from `overflow-visible` to `overflow-hidden`
4. Move tooltip/popover rendering to use `<Teleport to="body">` if any are clipped

### Phase 2: Remove redundant per-widget max-heights

Once DashboardCard enforces heights, update each widget:

| Widget | Change |
|---|---|
| ActivityFeed | Remove `max-h-52`, use flex-1 scroll pattern |
| PlayerNotes | Remove `max-h-64`, use flex-1 scroll pattern |
| ZoneNpcsWidget | Remove `max-h-80`, use flex-1 scroll pattern |
| TeleportCodesWidget | Remove `max-h-80`, use flex-1 scroll pattern |
| MushroomFarmingWidget | Remove `max-h-80` on table, use flex-1 scroll pattern |
| RecipeItemsWidget | Remove three `max-h-40` values, restructure sections with shared scroll |
| TripPlannerWidget | Remove `max-h-64` on route display, use flex-1 scroll pattern |
| WatchwordDetectionsWidget | Already inside ActivityFeed -- remove `max-h-52` pattern |

### Phase 3: Fix unbounded widgets

| Widget | Change |
|---|---|
| SkillTrackingWidget | Add `overflow-y-auto` to the flex-wrap container (card max-h will contain it) |
| DeathTrackerWidget | Wrap in flex-col with overflow-y-auto |
| GiftWatcherWidget | Wrap match list in scrollable container |
| MilkingTimersWidget | Already uses flex-1 pattern; will work once card constrains height |
| WordsOfPowerWidget | Already uses flex-1 pattern; will work once card constrains height |

### Phase 4: Grid column derivation

1. Keep the `auto-fill` grid approach but ensure the `minmax` base width is chosen so that 4 columns fit at any reasonable panel width
2. Validate that `col-span-2` and `col-span-4` always have enough columns available (if the panel can fit 4 small widgets, large widgets work by definition)
3. No breakpoint overrides needed — the grid math handles responsive behavior naturally
