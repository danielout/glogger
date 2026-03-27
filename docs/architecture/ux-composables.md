# UX Composables

Composables that implement the UX standards defined in [ux-standards.md](../plans/ux-standards.md). Each is opt-in — screens import only what they need.

## useKeyboard

**File:** [src/composables/useKeyboard.ts](../../src/composables/useKeyboard.ts)

Registers a single `keydown` listener scoped to the component lifecycle. Screens opt in to the features they need via the options object.

### Options

- **listNavigation** — Up/Down (or W/S) to move a selected index through a list. Enter to confirm. Automatically scrolls the selected item into view.
  - `items`: reactive array of list items
  - `selectedIndex`: writable ref tracking the current selection
  - `onConfirm(index)`: callback when Enter is pressed
  - `scrollContainerRef`: optional ref to the scrollable container element

- **paneNavigation** — Left/Right (or A/D) to move focus between named panes.
  - `panes`: static array of pane IDs
  - `activePane`: writable ref

- **tabCycling** — Shift+Left/Shift+Right (or Q/E) to cycle through tabs. Wraps around at boundaries.
  - `tabs`: array of tab IDs (static or reactive)
  - `activeTab`: writable ref

- **onEscape** — Called when Escape is pressed. Falls back to dismissing the top toast if no handler is provided.

### Input Suppression

When the active element is an `<input>`, `<textarea>`, or `<select>`, letter-based nav keys (W/S/A/D/Q/E) are suppressed so typing works normally. Arrow Up/Down still work for list navigation (so users can type a search query and immediately arrow through results). Arrow Left/Right for pane/tab cycling are also suppressed in inputs.

### Usage

See any tabbed screen for a minimal example — e.g., [CraftingView.vue](../../src/components/Crafting/CraftingView.vue) wires tab cycling with three lines.

---

## useToast

**File:** [src/composables/useToast.ts](../../src/composables/useToast.ts)

Thin wrapper around the toast store. Returns four methods:

- `success(message)` — green, auto-dismisses
- `info(message)` — blue, auto-dismisses
- `warn(message)` — gold/warning, auto-dismisses
- `error(message)` — red, persists until manually dismissed

See [toast-system.md](toast-system.md) for the full toast architecture.

---

## useViewPrefs

**File:** [src/composables/useViewPrefs.ts](../../src/composables/useViewPrefs.ts)

Persists per-screen UI preferences (sort order, filter values, collapsed pane state) across app restarts.

### API

```
useViewPrefs<T>(screenKey: string, defaults: T) → { prefs: Ref<T>, update(partial: Partial<T>) }
```

- `screenKey`: unique string identifying the screen (e.g., `"data-browser"`, `"crafting"`)
- `defaults`: object with default values for all preference keys
- `prefs`: reactive ref merged from stored values + defaults
- `update(partial)`: merges the partial into prefs and saves to the settings file with a 500ms debounce

### Storage

Preferences are stored as opaque JSON in `settings.json` under the `view_preferences` key, managed by the Rust backend. The frontend treats this as a `Record<string, Record<string, unknown>>` keyed by screen name.

### What to Persist

- Sort order, group-by mode, column visibility
- Collapsed/expanded state of panes and sections
- Filter selections that the user would expect to stick

### What NOT to Persist

- Transient selections (which list item is highlighted)
- Scroll positions
- Toast/notification state
