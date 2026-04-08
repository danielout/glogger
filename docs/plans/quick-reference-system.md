# Quick Reference System

## Problem

Players use glogger as a companion while actively playing. When they need to look something up — "what do I need for this recipe?", "where does this NPC live?", "what level do I need for that skill?" — they currently have two options:

1. **Hover for a tooltip** (500ms) — fast, but disappears when the mouse moves. Can't keep it visible while acting on the info.
2. **Click to open Data Browser** — full context switch. Leaves the current screen entirely. Useful for deep exploration, but overkill for a quick reference check.

The gap: there's no way to **keep a piece of info visible** while continuing to work on the current screen. This forces players to memorize tooltip contents, repeatedly re-hover, or abandon their current workflow to visit the Data Browser.

Three TODO items all attack this same gap from different angles:
- **Pinned tooltips in a bottom tray** — let users "stick" a tooltip so it stays visible
- **UX for checking recipes/data without losing context** — access data browser info without leaving the current screen
- **Quick reference favorites/bookmarking** — save frequently-checked entities for fast access

These should be designed as one unified feature rather than three separate ones.

## Design Principles

1. **Don't interrupt the primary workflow.** The player is playing a game. Glogger should surface info without demanding focus.
2. **Progressive disclosure.** Hover = peek. Pin = keep visible. Open = deep dive. Each level adds detail without forcing it.
3. **Minimal chrome.** No new full screens, no modal dialogs, no complex management UI. The feature should feel like a natural extension of the existing tooltip system.
4. **Leverage what exists.** The tooltip components already render rich entity summaries. The Data Browser already has full detail views. Don't rebuild either — connect them.

## Proposed Design: Reference Shelf

A collapsible tray at the bottom of the app window that holds pinned entity references. Think of it like browser tabs, but for game entities.

### User Flow

#### Pinning an entity

```
User hovers an ItemInline/SkillInline/etc. anywhere in the app
    |
Tooltip appears (existing behavior, unchanged)
    |
Tooltip now includes a small pin icon in the top-right corner
    |
User clicks the pin icon
    |
Entity appears as a chip in the Reference Shelf at the bottom of the screen
    |
Tooltip dismisses normally. User stays on their current screen.
```

#### Using pinned references

```
Reference Shelf shows a row of entity chips at the bottom:
  [Sword icon] Amazing Longsword  |  [NPC icon] Tadion  |  [Skill icon] Carpentry  | ...
    |
Hover a chip -> full tooltip appears (same tooltip as inline hover, anchored above the shelf)
    |
Click a chip -> navigates to Data Browser (existing behavior, for deep dives)
    |
Right-click or X button on chip -> unpins it
```

#### Shelf behavior

- **Collapsed by default** when empty. No visual footprint until first pin.
- **Auto-shows** when the first entity is pinned. Appears as a slim bar (roughly 32-36px tall) at the bottom of the app.
- **Collapsible** via a small toggle. When collapsed with pins, shows a subtle indicator (dot or count badge) so users know pins exist.
- **Wraps to multiple rows** when pins don't fit on one line. No horizontal scrolling — chips flow naturally like text. If a user pins enough entities to eat half their screen, that's their choice and their natural incentive to clean up. The shelf's growing height is its own soft pressure to keep pins under control.
- **Persists across screens.** Pinned entities stay visible regardless of which screen/tab the user is on (shelf lives in App.vue, above or outside PaneLayout).
- **Persists across sessions.** Pinned entity list saved to settings so it survives app restart.

### Visual Design

```
+------------------------------------------------------------------+
|  [Main App Content - Dashboard/Crafting/Chat/etc.]               |
|                                                                  |
|                                                                  |
+------------------------------------------------------------------+
| [pin icon] Amazing Longsword | Tadion | Carpentry | ...    [v]   |
+------------------------------------------------------------------+
```

- Chips use the same color coding as inline components (item color, skill color, NPC color, etc.).
- Entity icons shown on chips where available (items have icons already).
- Shelf background matches the app's dark surface color, with a subtle top border to separate from content.
- The `[v]` toggle collapses the shelf to just a thin line or hides it entirely.

### Why Not a Sidebar / Floating Panel?

- **Sidebar** competes with PaneLayout's left/right panes. Many screens already use both. Adding a third persistent pane would be cramped.
- **Floating panel** would occlude game content and require drag/position management — complex UI for a simple need.
- **Bottom tray** uses the one edge of the screen that PaneLayout doesn't manage. It's always visible, never overlaps content, and mirrors the mental model of "notes pinned to the bottom of your desk."

### Why Not a Full Favorites/Bookmarking System?

A bookmarking system implies categories, folders, management UI, persistence strategies — a lot of overhead for "I want to remember this recipe while I play." The shelf is intentionally ephemeral-feeling:

- **No categories.** Just a flat list of chips.
- **No management screen.** Pin and unpin inline. That's it.
- **Low commitment.** Pinning is one click. Unpinning is one click. No "are you sure?" dialogs.
- **Session-oriented by default.** While pins persist across restarts, the UX encourages treating them as temporary working references, not a permanent collection.

If users want permanent organization later, that's a separate feature that could build on this foundation (e.g., named pin groups or a dedicated favorites view). But start simple.

## Implementation Approach

### Data Model

Pinned entities are stored as an array:

```ts
interface PinnedEntity {
  type: EntityType  // 'item' | 'skill' | 'npc' | 'quest' | 'recipe' | 'ability' | 'enemy' | 'area'
  reference: string // same reference string used by inline components
  label: string     // display name, resolved at pin time
}
```

Stored in the settings store (same persistence as pane widths). No database table needed — this is UI state, not player data.

### Component Structure

- **`ReferenceShelf.vue`** — the bottom tray. Lives in App.vue, outside/below the main screen area. Renders the chip row + collapse toggle.
- **`ShelfChip.vue`** — individual pinned entity chip. Renders icon + name, handles hover (show tooltip) and click (navigate). Uses `EntityTooltipWrapper` for tooltip display.
- **Pin icon addition to `EntityTooltipWrapper.vue`** — add a small pin/unpin button to the tooltip chrome. Uses a composable or store method to add/remove from the pinned list.

### Integration Points

1. **EntityTooltipWrapper.vue** — Add pin button to tooltip. Needs access to entity type + reference from parent inline component (new props or provide/inject).
2. **App.vue** — Mount `ReferenceShelf` below the main screen container. Shelf must not interfere with PaneLayout height calculations.
3. **Settings store** — Add `pinnedEntities` array to persisted settings.
4. **useEntityNavigation** — ShelfChip clicks use the same navigation system as inline component clicks.

### Shelf Positioning

The shelf sits between the main content area and the app's bottom edge. PaneLayout fills its container with `h-full`, so the shelf needs to be a sibling that takes its natural height, with the main content area flexing to fill the remainder:

```
App.vue flex column:
  [MenuBar]           <- fixed height
  [Screen Container]  <- flex-1, min-h-0 (PaneLayout fills this)
  [ReferenceShelf]    <- auto height, shrink-0 (grows with content)
```

The key layout contract: the screen container is `flex-1 min-h-0` so it yields space as the shelf grows, and the shelf is `flex-shrink-0` so it always renders at its natural height. As users pin/unpin entities, the shelf wraps to more or fewer rows, and the screen container shrinks or grows accordingly. No changes to PaneLayout itself — just the App.vue flex structure.

Because the shelf height is dynamic (one row for a few pins, multiple rows for many), the content area above must handle resize gracefully. PaneLayout already uses `h-full` + `overflow-y-auto` on its panes, so this should work naturally — the panes will simply have less vertical space and scroll more. Worth testing with screens that have tight layouts (e.g., three-pane views with short content).

## Scope and Phases

### Phase 1: Core shelf + pinning (the MVP)

- ReferenceShelf component with chip display
- Pin button on tooltips
- Hover chips to see tooltips, click to navigate
- Unpin via X button on chip
- Persist pins in settings
- Shelf collapse/expand toggle
- Main content area resizes dynamically as shelf grows/shrinks

This covers all three TODO items at a basic level:
- Pinned tooltips -> pin button + hover-to-peek on chips
- Data without losing context -> hover shelf chips for info, stay on current screen
- Quick reference -> shelf persists across screens and sessions

### Phase 2: Polish (based on user feedback)

- Drag to reorder chips
- Keyboard shortcut to toggle shelf visibility
- "Pin" action in Data Browser detail pane (not just tooltips)
- Chip grouping by entity type (visual separators, not categories)
- Limit max pins with a gentle nudge ("shelf is getting full, unpin something?")

## Open Questions

- **Pin from Data Browser?** The detail pane in the browser could also have a pin button. This would let users browse, pin interesting things, then go back to their workflow. Worth including in phase 1 or deferring?
- **Tooltip positioning above shelf.** Tooltips currently render below the trigger. Shelf chips are at the bottom of the screen, so tooltips need to render *above* them. The tooltip positioning logic in `useTooltip.ts` may need a "prefer above" option.
