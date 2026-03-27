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

## CollapsiblePane

**File:** [src/components/Shared/CollapsiblePane.vue](../../src/components/Shared/CollapsiblePane.vue)

Wrapper for side panes that can be collapsed to a narrow toggle strip.

**Props:**
- `side: "left" | "right"` — which side the pane is on (determines toggle button position and arrow direction)
- `width?: string` — Tailwind width class when expanded (default: `"w-80"`)
- `screenKey: string` — unique key for persisting collapsed state via `useViewPrefs`

Collapsed state persists across app restarts. Content is hidden with `v-show` (not unmounted) so state is preserved.

## Pane Layout

Screens use a 1-, 2-, or 3-pane layout:

- **Center pane** is always `flex-1` (main content)
- **Left pane** is for navigation/selection (search, filters, lists). Prefer left + center for two-pane layouts.
- **Right pane** is for contextual detail or secondary info

Each pane manages its own scroll (`overflow-y-auto`). No page-level scrollbar.

Content area fills remaining viewport: `h-[calc(100vh-<offset>px)]` where offset accounts for menu bar + tabs.

## ToastContainer

**File:** [src/components/Shared/ToastContainer.vue](../../src/components/Shared/ToastContainer.vue)

Mounted once in App.vue. Renders toast notifications via Teleport to body. See [toast-system.md](toast-system.md) for details.
