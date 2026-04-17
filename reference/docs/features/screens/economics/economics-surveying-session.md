# Economics — Surveying: Session Tab

Parent: [economics-surveying.md](economics-surveying.md)

## Overview

The Session tab is the active surveying view. It provides real-time tracking of an in-progress survey session with a three-column layout: stats sidebar (left), survey type breakdown with loot (center), and a collapsible activity log (right).

## Files

- `src/components/Surveying/SessionTab.vue` — tab root, layout orchestration
- `src/components/Surveying/SessionSidebar.vue` — left sidebar with stats, XP, economics
- `src/components/Surveying/SurveyTypeAccordion.vue` — center panel per-type breakdown table
- `src/components/Surveying/SurveyLootGrid.vue` — reusable loot grid (used in accordion and speed bonus)
- `src/components/Surveying/SurveyLog.vue` — activity log entries
- `src/stores/surveyStore.ts` — all session state and computed values

## Session Lifecycle

### Starting a Session
- **Manual start** — click "Start Manual Session" when no session is active
- **Auto-start** — triggered by the backend when a survey event is detected and no session exists

### During a Session
- **Pause/Resume** — pauses elapsed time tracking; visual indicator shows paused state with pulsing amber dot
- **End** — manually ends the session; backend can also auto-end when all completable (non-motherlode) maps are used
- **Name/Notes** — editable inline; name field in the sidebar header, notes in a textarea below controls

### After a Session Ends
- A "Session ended" banner appears with a "New Session" button
- The session data remains visible for review until a new session is started or reset
- **Reset** — clears all session state from the frontend

## Layout

### Left Sidebar (`SessionSidebar`)

A fixed-width (w-56) sidebar with four card sections:

**Status & Controls**
- Status dot: green (active), amber pulsing (paused), gray (ended)
- Editable session name
- Start/end timestamps and elapsed time
- Pause/Resume, End, and Reset buttons
- Manual mode indicator
- Session notes textarea

**Stats**
- Maps Crafted — number of survey maps crafted during the session
- Completed — number of surveys completed (loot collected)
- Avg Time — average time per survey completion

**XP Gained**
- Surveying XP (green) — gained from crafting maps
- Mining XP (red) — gained from completing surveys
- Geology XP (amber) — gained from completing surveys
- Each skill shows estimated surveys/crafts to next level (when data available)

**Economics** (shown when revenue or cost > 0)
- Revenue — total value of all loot (from market/vendor prices)
- Cost — total crafting material cost
- Profit — revenue minus cost
- Per Survey — profit divided by completions
- Per Hour — profit rate based on elapsed time

### Center Panel

**Survey Type Accordion (`SurveyTypeAccordion`)**

A table with one row per survey type used in the session. Columns:
- Type — survey internal name
- Done — number completed
- Revenue — total loot value for this type
- Cost — crafting material cost for this type
- Profit — net profit
- Profit/ea — per-survey profit

Each row expands to show a `SurveyLootGrid` with primary loot drops for that type (item name, count, drop percentage, per-hour rate).

**Speed Bonus Loot**

A separate `SurveyLootGrid` below the accordion showing all speed bonus drops across all survey types. Only visible when speed bonus loot exists.

**Crafting Materials Consumed**

Shows all materials consumed during map crafting as `ItemInline` chips with quantities. Only visible when materials have been tracked.

### Right Sidebar (Activity Log)

A collapsible panel (w-72 expanded, w-8 collapsed) containing `SurveyLog` — a chronological feed of survey events (map crafted, survey used, completed, loot corrections, etc.).

## Loot Tracking

### Primary Loot
Standard drops from survey completions. Tracked per survey type and displayed in the type accordion.

### Speed Bonus Loot
Extra drops awarded for fast survey completions. Tracked globally (not per type) since speed bonus is area-wide. Displayed in a separate section below the accordion.

### Crafting Materials
Materials consumed during map crafting (tracked via the crafting window in the survey parser). Displayed as item chips at the bottom of the center panel.

### Reactive Pricing
All item values update reactively when market prices change in the economics store, so revenue/profit figures stay current.
