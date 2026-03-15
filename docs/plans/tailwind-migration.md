# Tailwind Migration Plan

## Current State

We have Tailwind v4.2 installed via `@tailwindcss/vite` and a `@theme` block in `src/assets/main.css` defining our design tokens. However, only the **Chat** components actually use Tailwind utility classes. Everything else uses scoped `<style>` blocks or inline CSS with raw hex values.

### What's already on Tailwind
- `Chat/ChannelView.vue` — fully Tailwind
- `Chat/ChatMessageList.vue` — fully Tailwind
- `Chat/ChatMessage.vue` — Tailwind
- `Chat/MessageWithItemLinks.vue` — Tailwind
- `Chat/PartyView.vue`, `Chat/TellsView.vue` — partial Tailwind

### What still uses traditional CSS
- `App.vue` — global `<style>` block (reset, layout)
- `MenuBar.vue` — scoped styles
- `Settings.vue` — scoped styles
- `Settings/*` (7 subcomponents) — import `settings-shared.css`
- `DataBrowser/DataBrowser.vue` — scoped styles
- `Shared/SkillCard.vue`, `Shared/SkillGrid.vue` — scoped styles
- `Shared/ItemCard.vue`, `Shared/ItemTooltip.vue` — scoped styles
- `Surveying/SurveyView.vue`, `Surveying/SurveyLog.vue`, `Surveying/SessionTab.vue` — scoped styles
- `Chat/ChatLogsSettings.vue` — inline `style=""` attributes

### Standalone CSS files
- `src/components/Settings/settings-shared.css` — form controls, buttons, layout for settings tabs

---

## Goals

1. **Every component uses Tailwind utilities** — no scoped `<style>` blocks, no inline `style=""` for things Tailwind can handle.
2. **Centralized style system** — a `src/assets/css/` directory with organized CSS files for shared patterns.
3. **Consistent look** — all components draw from the same theme tokens.
4. **Dynamic styles are the only exception** — `:style` bindings for truly dynamic values (computed widths, dynamic colors) are fine.

---

## Phase 1: CSS Architecture Setup

### 1.1 Create the style directory structure

```
src/assets/css/
├── main.css          ← entry point (moved from src/assets/main.css)
├── theme.css         ← @theme block (design tokens)
├── base.css          ← base/reset styles, body defaults, scrollbar, selection
├── components.css    ← @apply-based component classes (@layer components)
└── utilities.css     ← custom utilities if needed (@layer utilities)
```

### 1.2 `main.css` — entry point

```css
@import "tailwindcss";
@import "./theme.css";
@import "./base.css";
@import "./components.css";
@import "./utilities.css";
```

### 1.3 `theme.css` — design tokens

Move the existing `@theme` block here. Also expand it with:
- **Spacing scale** if we need non-default values
- **Font sizes** — define `--font-*` tokens if the app needs custom sizes
- **Border radius** tokens if we settle on consistent rounding
- **Transition durations** — e.g. `--duration-fast: 150ms`

### 1.4 `base.css` — global resets

Pull the current `App.vue` global styles into `@layer base {}`:
- `*, *::before, *::after` box-sizing
- `body` background, color, font
- Scrollbar styling (currently in App.vue)
- Selection color
- `@keyframes spin` (already exists)

### 1.5 `components.css` — reusable component classes

This is where `settings-shared.css` patterns and any other repeated patterns get defined using `@apply` inside `@layer components`. Examples:

```css
@layer components {
  .btn { @apply px-3 py-1.5 rounded text-sm font-medium transition-colors cursor-pointer; }
  .btn-primary { @apply bg-accent-gold text-surface-dark hover:bg-accent-gold/80; }
  .btn-secondary { @apply bg-surface-elevated text-text-secondary border border-border-default hover:bg-surface-card; }
  .card { @apply bg-surface-card border border-border-default rounded-lg; }
  .input { @apply bg-surface-dark border border-border-default rounded px-3 py-1.5 text-text-primary; }
  .tab { @apply px-4 py-2 text-text-secondary cursor-pointer border-b-2 border-transparent; }
  .tab-active { @apply text-accent-gold border-accent-gold; }
}
```

### 1.6 Delete `settings-shared.css`

All its patterns are absorbed into `components.css` as Tailwind component classes.

---

## Phase 2: Migrate Components (by area)

Work area-by-area. For each component: remove the `<style>` block, replace class names and inline styles with Tailwind utilities (or component classes from `components.css`), and verify visually.

### 2.1 App.vue & MenuBar.vue (app shell)
- `App.vue`: Move global styles to `base.css`. Template layout → Tailwind flex utilities.
- `MenuBar.vue`: Replace scoped styles with Tailwind utilities for the top bar layout and button styling.

### 2.2 Settings area
- `Settings.vue`: Replace layout/nav styles with Tailwind utilities.
- All `Settings/*` tabs: Replace `@import './settings-shared.css'` usage with the new component classes from `components.css` (`.btn`, `.input`, `.card`, etc.) plus Tailwind utilities.

### 2.3 DataBrowser area
- `DataBrowser/DataBrowser.vue`: Tab navigation and content container → Tailwind utilities + `.tab` / `.tab-active` component classes.

### 2.4 Shared components
- `SkillGrid.vue`: Simple flex → Tailwind `flex flex-wrap gap-*`.
- `SkillCard.vue`: Card layout + TNL bar → Tailwind. The `:style` for dynamic width stays.
- `ItemCard.vue` / `ItemTooltip.vue`: Card and tooltip styling → Tailwind.

### 2.5 Surveying area
- `SurveyView.vue`: Tab navigation → reuse `.tab` component classes.
- `SurveyLog.vue`: Log entry layout → Tailwind utilities. The `:style` for dynamic border color stays.
- `SessionTab.vue`: Simple flex → Tailwind.

### 2.6 Chat area (cleanup pass)
- Already mostly Tailwind. Do a consistency pass:
  - `ChatLogsSettings.vue`: Replace inline `style=""` attributes with Tailwind classes.
  - `ChatView.vue`: Remove remaining `<style>` block if it still has scoped CSS.
  - Ensure all Chat components reference theme tokens (e.g. `bg-surface-dark`) not raw hex values.

---

## Phase 3: Cleanup & Verification

1. **Search for orphaned styles** — grep for `<style` in all `.vue` files. The only acceptable `<style>` blocks should be empty or contain only truly component-specific dynamic CSS that can't be expressed in Tailwind.
2. **Search for inline styles** — grep for `style="` in templates. Only `:style` with dynamic computed values should remain.
3. **Search for raw hex colors** — grep for `#[0-9a-fA-F]{3,8}` in `.vue` files. Everything should reference theme tokens.
4. **Visual review** — go through each view in the app and confirm nothing is broken.
5. **Delete any empty `<style>` blocks** left behind.

---

## Styling Guide

### Directory structure

```
src/assets/css/
├── main.css          ← only imports, nothing else
├── theme.css         ← all design tokens (@theme block)
├── base.css          ← html/body resets, scrollbar, keyframes
├── components.css    ← reusable component classes via @apply
└── utilities.css     ← custom one-off utilities
```

### Where to put styles

| What you're styling | Where it goes |
|---|---|
| Design tokens (colors, spacing, radii) | `theme.css` inside `@theme {}` |
| HTML element defaults (body, scrollbar, selection) | `base.css` inside `@layer base {}` |
| Reusable patterns (buttons, cards, inputs, tabs, badges) | `components.css` inside `@layer components {}` |
| Custom utilities (one-off helpers) | `utilities.css` inside `@layer utilities {}` |
| Component-specific layout | Tailwind utility classes directly in the template |
| Truly dynamic values (computed width, runtime color) | `:style` binding in the template |

### Rules

1. **No `<style>` blocks in `.vue` files.** All styling is either Tailwind utilities in the template or shared classes in `src/assets/css/`.
2. **No inline `style=""` attributes.** Use Tailwind classes. The only exception is `:style` with dynamic/computed values.
3. **No raw color values in templates.** Always use theme tokens: `text-text-primary`, `bg-surface-card`, `border-border-default`, etc.
4. **Prefer utilities over component classes.** Only create a component class in `components.css` when the same combination of utilities appears 3+ times across different components.
5. **Name component classes semantically.** `.btn-primary`, `.card`, `.input` — not `.blue-rounded-thing`.
6. **Keep `components.css` flat.** No nesting, no BEM. Each class is a single `@apply` line. If a pattern needs variants, use separate classes (`.btn`, `.btn-primary`, `.btn-secondary`).
7. **Theme tokens are the source of truth.** If you need a new color, add it to `theme.css` first, then reference it. Never hardcode hex values.
8. **Dynamic styles stay in the template.** If a value comes from a computed property or prop (progress bar width, channel-specific color), use `:style`. This is the one acceptable place for inline styles.

### Common patterns

**Buttons:**
```html
<button class="btn btn-primary">Save</button>
<button class="btn btn-secondary">Cancel</button>
```

**Cards:**
```html
<div class="card p-4">
  <h3 class="text-text-primary text-sm font-medium">Title</h3>
  <p class="text-text-secondary text-xs mt-1">Description</p>
</div>
```

**Tab navigation:**
```html
<div class="flex border-b border-border-default">
  <button class="tab tab-active">Active Tab</button>
  <button class="tab">Other Tab</button>
</div>
```

**Form inputs:**
```html
<input class="input" placeholder="Enter value..." />
<div class="flex gap-2">
  <input class="input flex-1" />
  <button class="btn btn-secondary">Browse</button>
</div>
```

**Dynamic width (acceptable `:style`):**
```html
<div class="h-1 bg-accent-gold rounded transition-all" :style="{ width: percent + '%' }"></div>
```
