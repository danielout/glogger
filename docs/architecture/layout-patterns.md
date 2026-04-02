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

## PaneLayout + SidePane (Required for All Screens)

**Files:**
- [src/components/Shared/PaneLayout.vue](../../src/components/Shared/PaneLayout.vue)
- [src/components/Shared/SidePane.vue](../../src/components/Shared/SidePane.vue)
- [src/composables/usePaneResize.ts](../../src/composables/usePaneResize.ts)

**Every screen and tab component must use `PaneLayout` as its root layout.** This ensures consistent height management, scroll behavior, and the ability to add side panes later without restructuring. Even screens that only need a single content area should wrap their content in a center-only PaneLayout.

**PaneLayout Props:**
- `screenKey: string` — unique prefix for persisting pane preferences
- `leftPane?: { title, defaultWidth?, minWidth?, maxWidth? }` — left side pane config (enables `#left` slot)
- `rightPane?: { title, defaultWidth?, minWidth?, maxWidth? }` — right side pane config (enables `#right` slot)

**SidePane features:**
- **Collapsible** — collapses to a 28px vertical strip showing the pane title rotated 90° (`writing-mode: vertical-lr`). Click the strip to expand.
- **Resizable** — drag the interior-edge handle to resize. Double-click to reset to default width.
- **Persistent** — collapsed state and width are saved per-screen via `useViewPrefs`.
- **Smooth transitions** — uses `v-show` + CSS `transition-[width]` for animated collapse/expand.

**Usage patterns:**

Center-only (simplest — for dashboard-style screens, tab containers, etc.):
```vue
<PaneLayout screen-key="dashboard">
  <div class="h-full overflow-y-auto">
    <!-- your content -->
  </div>
</PaneLayout>
```

Left pane + center (for list/detail screens — browsers, search, etc.):
```vue
<PaneLayout screen-key="db-items" :left-pane="{ title: 'Items', defaultWidth: 360, minWidth: 280, maxWidth: 500 }">
  <template #left>
    <!-- search, filters, result list -->
  </template>
  <!-- detail/content panel -->
</PaneLayout>
```

Three-pane (left navigation + center content + right detail):
```vue
<PaneLayout screen-key="npcs"
  :left-pane="{ title: 'NPC List', defaultWidth: 320 }"
  :right-pane="{ title: 'Details', defaultWidth: 300 }">
  <template #left>...</template>
  <!-- center content -->
  <template #right>...</template>
</PaneLayout>
```

**Layout rules:**
- **All new screens must use PaneLayout.** No ad-hoc flex wrappers or calc-based heights.
- Center pane is always `flex-1` (main content, fills remaining space).
- Left pane is for navigation/selection (search, filters, lists). Prefer left + center for two-pane layouts.
- Right pane is for contextual detail or secondary info.
- Each pane scrolls independently (`overflow-y-auto`). No page-level scrollbar.
- Panes fill the full available height — never use `h-[calc(...)]` for screen height.
- Screen keys must be unique. Use the pattern `"view-name"` for top-level views and `"view-name-tab"` for sub-tabs (e.g., `"db-items"`, `"crafting-skills"`).

## ToastContainer

**File:** [src/components/Shared/ToastContainer.vue](../../src/components/Shared/ToastContainer.vue)

Mounted once in App.vue. Renders toast notifications via Teleport to body. See [toast-system.md](toast-system.md) for details.
