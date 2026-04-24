# In-App Help System

Implementation plan for per-screen contextual help integrated into PaneLayout.

## Content Storage

Help content lives in `src/help/` as TypeScript files exporting structured objects — one file per top-level screen. Each exports a record keyed by sub-tab ID.

Content interface (`src/help/types.ts`):
- `ScreenHelp`: title, summary (one sentence), sections (heading + body + optional tips), shortcuts
- `HelpSection`: heading, body (plain text), optional tips array

Why TypeScript over markdown: no markdown rendering library in the project, structured data renders with consistent styling, type safety.

## Trigger Mechanism

- **Button**: `?` icon in PaneLayout top-right corner, rendered when `helpKey` prop is provided
- **Keyboard**: `F1` toggles help panel (doesn't conflict with existing shortcuts)
- Opt-in via PaneLayout `helpKey` prop — screens without it get no help button

## Display Pattern

Slide-in right panel (not modal/overlay) — users read help while viewing the screen.
- ~320px fixed width, overlays center content
- Semi-transparent edge, close on Escape or click-outside
- CSS transition slide-in from right
- Component: `src/components/Shared/ScreenHelpPanel.vue`
- State managed by `src/composables/useScreenHelp.ts`

## PaneLayout Integration

Add optional `helpKey?: string` prop. When provided:
- Center pane gets `relative` positioning
- `?` button rendered absolutely at `top-2 right-2`
- `ScreenHelpPanel` mounted inside, controlled by local ref
- For sub-tabs, parent passes computed `helpKey` combining screen + active tab

## Content Template (per screen)

1. **Title** — screen/tab name
2. **Summary** — one sentence purpose
3. **Sections** (2-4): Getting Started, How It Works, Key Features, Tips
4. **Shortcuts** — screen-specific keyboard shortcuts

Target: readable in under 30 seconds per screen.

## Phases

### Phase 1: Infrastructure
1. Create `ScreenHelp` type interface
2. Create help content registry with key-based lookup
3. Build `ScreenHelpPanel.vue` slide-in component
4. Build `useScreenHelp.ts` composable (visibility, F1 shortcut)
5. Add `helpKey` prop to `PaneLayout.vue`

### Phase 2: Validate (Dashboard)
6. Write dashboard help content
7. Add `helpKey="dashboard"` to DashboardView
8. Test full flow

### Phase 3: Core screens
9. Character + 9 sub-tabs
10. Crafting + 9 sub-tabs
11. Economics + 4 sub-tabs
12. Inventory + 3 sub-tabs

### Phase 4: Remaining screens
13. Chat Logs + 9 sub-tabs
14. Search, Settings
15. Data Browser + 11 browser types
16. Surveying + sub-tabs

### Phase 5: Polish
17. "First visit" pulse indicator on `?` button
18. "Show tips on first visit" toggle in Settings

## Scope

- 9 top-level screens + ~45 sub-tabs = ~54 help entries
- Many share templates (all Data Browser tabs, all Chat tabs)
- Realistic: ~25 unique content entries with shared bases

## Key Files

- [PaneLayout.vue](../../src/components/Shared/PaneLayout.vue) — gets `helpKey` prop
- [SidePane.vue](../../src/components/Shared/SidePane.vue) — reference for panel animation
- [HelpOverlay.vue](../../src/components/Help/HelpOverlay.vue) — existing app-level help (distinct)
- [MenuBar.vue](../../src/components/MenuBar.vue) — authoritative screen/tab list
- [useKeyboard.ts](../../src/composables/useKeyboard.ts) — shortcut pattern reference
