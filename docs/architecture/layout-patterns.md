# Layout Patterns

Standard layout components and patterns for building screens. All patterns follow the UX standards defined in [ux-standards.md](../plans/ux-standards.md) and are modeled after the Data Browser.

## Screen Navigation (v-show with Lazy Mount)

**File:** [src/App.vue](../../src/App.vue)

Screens use a `v-if` + `v-show` hybrid for navigation:
- `v-if="visited.has(viewName)"` controls initial mount (lazy — only mounts when first navigated to)
- `v-show="currentView === viewName"` controls visibility after first mount

This preserves component state (scroll position, selections, form input) when navigating between screens. A reactive `Set<AppView>` tracks which views have been visited.

## TabBar

**File:** [src/components/Shared/TabBar.vue](../../src/components/Shared/TabBar.vue)

Standardized horizontal tab bar using the `.tab` / `.tab-active` component classes.

**Props:**
- `tabs: Array<{ id: string, label: string }>` — tab definitions
- `modelValue: string` — the active tab ID (v-model)

**Emits:** `update:modelValue`

Used on: DataBrowser, CraftingView, ChatView, CharacterView. Pair with `useKeyboard({ tabCycling })` for Q/E keyboard cycling.

## EmptyState

**File:** [src/components/Shared/EmptyState.vue](../../src/components/Shared/EmptyState.vue)

Consistent component for displaying empty/no-data states.

**Props:**
- `primary: string` — main message (always shown)
- `secondary?: string` — supporting detail or call to action
- `variant?: "panel" | "compact"` — panel centers vertically for main content areas, compact is inline for side panes and widgets

**Guidelines:**
- Always tell the user *why* the area is empty and *what they can do* about it
- Be specific ("No skill updates yet") not generic ("Nothing to show")
- Panel variant for main content areas, compact variant for widgets and side panes

## PaneLayout + SidePane

**Files:**
- [src/components/Shared/PaneLayout.vue](../../src/components/Shared/PaneLayout.vue)
- [src/components/Shared/SidePane.vue](../../src/components/Shared/SidePane.vue)
- [src/composables/usePaneResize.ts](../../src/composables/usePaneResize.ts)

Standardized multi-pane layout system. Screens use `PaneLayout` with optional `#left` and `#right` slots for side panes, and a default slot for the center content.

**PaneLayout Props:**
- `screenKey: string` — unique prefix for persisting pane preferences
- `leftPane?: { title, defaultWidth?, minWidth?, maxWidth? }` — left side pane config (enables `#left` slot)
- `rightPane?: { title, defaultWidth?, minWidth?, maxWidth? }` — right side pane config (enables `#right` slot)

**SidePane features:**
- **Collapsible** — collapses to a 28px vertical strip showing the pane title rotated 90° (`writing-mode: vertical-lr`). Click the strip to expand.
- **Resizable** — drag the interior-edge handle to resize. Double-click to reset to default width.
- **Persistent** — collapsed state and width are saved per-screen via `useViewPrefs`.
- **Smooth transitions** — uses `v-show` + CSS `transition-[width]` for animated collapse/expand.

**Usage example:**
```vue
<PaneLayout screen-key="npcs" :left-pane="{ title: 'NPC List', defaultWidth: 320 }">
  <template #left>
    <NpcListPanel ... />
  </template>
  <NpcDetailPanel ... />
</PaneLayout>
```

**Layout rules:**
- Center pane is always `flex-1` (main content, fills remaining space)
- Left pane is for navigation/selection (search, filters, lists). Prefer left + center for two-pane layouts.
- Right pane is for contextual detail or secondary info
- Each pane scrolls independently (`overflow-y-auto`). No page-level scrollbar.
- Panes fill the full available height.

## ToastContainer

**File:** [src/components/Shared/ToastContainer.vue](../../src/components/Shared/ToastContainer.vue)

Mounted once in App.vue. Renders toast notifications via Teleport to body. See [toast-system.md](toast-system.md) for details.
