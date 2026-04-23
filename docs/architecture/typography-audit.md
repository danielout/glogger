# Typography Audit

Audit of font and typography usage across the glogger frontend, conducted 2026-04-23.

## Current State

### Base Configuration

The global base is set in [`src/assets/css/base.css`](../../src/assets/css/base.css):
- **Font family:** `monospace` on `body` and `button`
- **Font size:** `14px` on `body` (equivalent to Tailwind's `text-sm`)
- **Text color:** `--color-text-primary` (`#d4d4d4`)

No other font families are used anywhere in the codebase. There are zero uses of `font-sans` or `font-serif`. The app is 100% monospace. Explicit `font-mono` classes appear 218 times, reinforcing the base setting redundantly in component classes like `.btn` and `.input` in [`components.css`](../../src/assets/css/components.css).

There is no Tailwind config file. Tailwind v4 is loaded via the Vite plugin, and theme tokens are defined in [`src/assets/css/theme.css`](../../src/assets/css/theme.css). No typography-specific tokens (font sizes, line heights, letter spacing) are defined in the theme -- only colors.

### Font Size Distribution

Across 286 Vue files, font size classes appear roughly 2,450 times:

| Class | Pixel Equivalent | Occurrences | Role |
|-------|-----------------|-------------|------|
| `text-xs` | 12px | 1,204 | Dominant size everywhere |
| `text-sm` | 14px (= body base) | 409 | Secondary body text, inputs |
| `text-[0.65rem]` | 10.4px | 338 | Section labels, NPC panels |
| `text-[0.6rem]` | 9.6px | 172 | Micro labels, stat annotations |
| `text-[10px]` | 10px | 108 | Build planner headings |
| `text-[0.72rem]` | 11.5px | 51 | Various detail views |
| `text-lg` | 18px | 34 | Screen titles, section headers |
| `text-xl` | 20px | 27 | Chat screen titles |
| `text-[0.7rem]` | 11.2px | 22 | Miscellaneous |
| `text-[0.55rem]` | 8.8px | 21 | Tiny annotations, keyboard hints |
| `text-base` | 16px | 23 | Sparse, inconsistent usage |
| `text-[0.82rem]` | 13.1px | 9 | Tooltip body text |
| `text-2xl` | 24px | 8 | App title ("glogger") |
| `text-[11px]` | 11px | 6 | Date picker cells |
| `text-[0.85rem]` | 13.6px | 6 | Misc |
| `text-[9px]` | 9px | 5 | Build planner micro labels |
| `text-[0.5rem]` | 8px | 3 | Inventory grid fallback icons |
| `text-3xl` | 30px | 2 | Splash screen title |

**There are 13 distinct arbitrary (non-standard) text sizes** in addition to the 6 standard Tailwind sizes. Many of these are within 1-2px of each other and serve the same semantic role.

### Font Weight Distribution

| Class | Occurrences |
|-------|-------------|
| `font-semibold` | 236 |
| `font-bold` | 159 |
| `font-medium` | 129 |
| `font-normal` | 42 |

Weight usage is fairly consistent. `font-semibold` dominates for headings and labels, `font-bold` for emphasis and data values, `font-medium` for secondary labels.

### Line Height and Letter Spacing

Line height classes are used sparingly (~100 total): `leading-relaxed` (most common, ~65 uses), `leading-none` (~25), `leading-tight` (~6), `leading-snug` (~5). Most text has no explicit line-height, relying on the browser default.

Letter spacing is used more deliberately (~250 total): `tracking-widest` (129), `tracking-wider` (60), `tracking-wide` (60), `tracking-normal` (4). Wide tracking is paired with uppercase labels, which is a consistent pattern.

### Numeric Data

`tabular-nums` is used in ~50 places for numeric alignment in tables and data displays. This is a good practice but inconsistently applied -- many numeric values in tables do not use it.

## Heading Patterns

Headings are inconsistent across the app. There is no standardized heading system.

### h1 Usage (5 instances)
- Startup screens: `text-2xl font-bold text-accent-gold tracking-wide`
- Splash screen: `text-3xl font-bold text-accent-gold tracking-wide`
- Dev panel: `text-base font-bold text-accent-gold`

### h2 Usage (~25 instances, 8+ distinct patterns)
- Chat screens: `text-xl font-medium text-text-primary m-0` (most common, 8 uses)
- Character views: `text-lg font-semibold text-text-primary` (3 uses)
- Dashboard: `text-accent-gold text-lg m-0`
- Aggregate view: `text-accent-gold text-2xl m-0`
- Various other one-off patterns

### h3 Usage (~60 instances, 15+ distinct patterns)
- Section headers: `text-sm font-semibold text-text-secondary uppercase tracking-wider` (most common cluster, ~20 uses with minor variations in margin)
- Panel titles: `text-sm font-semibold text-text-primary` (7 uses)
- Surveying: `text-xs uppercase tracking-widest text-accent-blue font-semibold` (4 uses)
- NPC sections: `text-[0.65rem] uppercase tracking-widest text-text-secondary font-semibold` (4 uses)
- Many one-off variations

### h4 Usage (~45 instances)
- Build planner: `text-[10px] font-semibold text-text-muted uppercase tracking-wider` (dominant, ~15 uses)
- Other panels: `text-xs font-semibold text-text-muted uppercase tracking-wider` (~10 uses)

## Screen-Type Patterns

### Dashboard / Overview Screens
Heaviest use of `text-xs` (169 occurrences in Dashboard alone). Stat cards use `text-xl font-bold` for hero values and `text-xs uppercase tracking-wide text-text-muted` for labels. Tables use `text-sm` for body rows and `text-xs` for column headers.

### Chat Screens
Most consistent typography. Screen titles use `text-xl font-medium`. Message timestamps use `text-xs font-mono`. Message body uses `text-sm`. Sender names use `font-semibold` with channel-colored text.

### Build Planner
Densest information. Pushes text sizes down to `text-[9px]` and `text-[10px]` for micro-labels. Heavy use of `text-[0.65rem]` and `text-[0.6rem]`. Has the most arbitrary sizes of any feature area.

### Surveying / Analytics
Uses `text-xs` almost exclusively (49 out of 53 text-size uses). Section headers use `text-xs uppercase tracking-widest text-accent-blue`. Annotations use `text-[0.6rem]`.

### NPC Detail Panels
Consistent within themselves: section headers use `text-[0.65rem] uppercase tracking-widest text-text-dim`. Data rows use `text-xs`. Annotations use `text-[0.55rem]` and `text-[0.6rem]`.

### Tooltips (Shared Components)
Entity name: `text-sm font-bold` with entity color. Description: `text-xs leading-relaxed italic text-text-secondary`. Stats/details: `text-xs`. Tags/badges: `text-[0.65rem] uppercase tracking-wide`.

### Data Tables
Tables on Character screens use `text-sm` with `text-xs` column headers. Aggregate view tables use `text-sm`. StallTracker tables use `text-xs` with `tabular-nums`. No consistent standard.

## Style Block Violations

13 Vue files contain `<style>` blocks, which violates the project convention documented in [`styling.md`](styling.md). Most are for overlay/modal positioning or scoped deep selectors (`:deep()`). The only typography-relevant one is [`LoreBrowser.vue`](../../src/components/DataBrowser/LoreBrowser.vue), which uses scoped styles for `font-size: 1.25rem` and `font-size: 1.1rem` on dynamically rendered lore book HTML.

## Accessibility Concerns

### Text Below Minimum Readable Size
WCAG recommends a minimum of 12px for body text. Several sizes used in the app fall well below this:

| Size | Pixel | Occurrences | Concern |
|------|-------|-------------|---------|
| `text-[0.5rem]` | 8px | 3 | Extremely small, even for decorative |
| `text-[0.55rem]` | 8.8px | 21 | Below readable threshold for many users |
| `text-[9px]` | 9px | 5 | Below readable threshold |
| `text-[0.6rem]` | 9.6px | 172 | Borderline; heavy use across the app |

Combined, there are ~200 instances of text below 10px. Many of these are labels or annotations paired with `text-text-dim` (#6a6a6a), creating a double readability problem: tiny text in low-contrast color.

### Contrast Concerns
The `text-dim` token (`#6a6a6a`) on `surface-dark` (`#111111`) has a contrast ratio of approximately 3.6:1, which passes WCAG AA for large text but fails for normal text (requires 4.5:1). When used at sub-10px sizes, this is the worst accessibility combination in the app. `text-muted` (`#808080`) on `#111111` is approximately 4.6:1, which barely passes AA.

### Monospace Readability
The entire app uses the system monospace font. Monospace fonts are inherently less readable for prose than proportional fonts. This is acceptable for a data-heavy tool but worth noting for longer text blocks (lore books, help content, changelogs).

## Proposed Standards

### Typography Scale

Define a fixed scale and stop using arbitrary values. All sizes below map to the body base of 14px:

| Token | Size | Line Height | Use Case |
|-------|------|-------------|----------|
| **Title** | `text-xl` (20px) | default | Screen titles (h2 in chat, dashboard headers) |
| **Heading** | `text-lg` (18px) | default | Section titles within a screen |
| **Subheading** | `text-sm` (14px) | default | Panel/card titles, table section headers |
| **Body** | (inherited, 14px) | default | Default reading text, no class needed |
| **Small** | `text-xs` (12px) | default | Table data, list items, most UI content |
| **Caption** | `text-[10px]` | `leading-tight` | Micro labels, section dividers, badges |
| **Tiny** | `text-[10px]` | `leading-none` | Use sparingly; only for dense data panels like build planner |

Key changes from current state:
- **Eliminate** `text-[0.5rem]`, `text-[0.55rem]`, `text-[9px]` -- nothing below 10px
- **Consolidate** `text-[0.6rem]` (9.6px), `text-[0.65rem]` (10.4px), `text-[0.7rem]` (11.2px), `text-[0.72rem]` (11.5px) into `text-[10px]`
- **Consolidate** `text-[0.82rem]` (13.1px), `text-[0.85rem]` (13.6px), `text-[0.9rem]` (14.4px) into `text-xs` (12px) or `text-sm` (14px)
- **Eliminate** `text-[11px]` -- use `text-xs` (12px) instead
- **Keep** `text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`, `text-2xl` as the standard Tailwind sizes

### Heading Classes

Standardize heading patterns as component classes in `components.css`:

| Pattern | Intended Use |
|---------|-------------|
| Screen title: `text-xl font-medium text-text-primary` | h2 at top of each tab/screen |
| Section header: `text-sm font-semibold text-text-secondary uppercase tracking-wider` | h3 for major sections |
| Panel label: `text-xs font-semibold text-text-muted uppercase tracking-wider` | h4 for sub-sections |
| Micro label: `text-[10px] font-semibold text-text-muted uppercase tracking-wider` | h5 for dense UI panels |

### Font Weight Rules

| Weight | Use Case |
|--------|----------|
| `font-bold` | Hero numbers, emphasized values |
| `font-semibold` | Headings, labels |
| `font-medium` | Interactive elements (buttons, links) |
| (default/normal) | Body text |

### Letter Spacing Rules

| Tracking | Use Case |
|----------|----------|
| `tracking-widest` | Uppercase section dividers |
| `tracking-wider` | Uppercase sub-labels |
| `tracking-wide` | Uppercase inline badges |
| (default) | Everything else |

### Numeric Data

All numeric values in tables and data displays should use `tabular-nums` for proper column alignment.

## Migration Notes

### What Needs to Change

1. **Arbitrary size consolidation (highest impact, ~700 occurrences across ~147 files):** Replace all arbitrary `text-[0.Xrem]`, `text-[Npx]` values with the nearest standard size. This is the single largest change. The build planner subsystem (27 files) is the densest area.

2. **Heading standardization (~170 heading tags across ~60 files):** Align heading classes to the proposed patterns. Most h2 and h3 tags need class adjustments.

3. **Remove redundant `font-mono` (~218 occurrences):** Since `body` already sets `font-family: monospace`, explicit `font-mono` on child elements is redundant. Keep it only in `components.css` class definitions (`.btn`, `.input`) where it documents intent.

4. **Add `tabular-nums` to numeric table columns:** Audit all `<td>` elements displaying numbers and add `tabular-nums` where missing.

5. **Minimum size enforcement:** Eliminate all text below 10px (currently ~30 instances in ~10 files).

6. **Consider component classes for heading patterns:** The most common heading patterns could become `.section-heading`, `.panel-label`, etc. in `components.css`, reducing class repetition.

### Suggested Migration Order

1. Fix accessibility issues first (sub-10px text, low-contrast combinations)
2. Consolidate arbitrary sizes to standard scale
3. Standardize heading patterns
4. Clean up redundant `font-mono`
5. Add missing `tabular-nums`
