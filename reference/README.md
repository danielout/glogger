# Reference — not live code

This directory holds **restored copies** of the legacy survey tracker files that were deleted during the Phase 5 nuke-and-pave rewrite. They are kept here as references for the rebuild of Session / Session History / Analytics screens so we match the original layout, information density, and UX that users valued.

**Nothing in here is wired into the running app.** The paths under `reference/` mirror the original paths (e.g. `reference/src/components/Surveying/…` corresponds to what used to live at `src/components/Surveying/…`) purely so cross-referencing is easy when rebuilding.

## What's in here

- **`src/components/Surveying/`** — the legacy Vue tab components: `SessionTab`, `SessionSidebar`, `SessionCard`, `SurveyView`, `SurveyLog`, `SurveyLootGrid`, `SurveyTypeAccordion`, `SurveyImportManager`.
- **`src/components/Surveying/Analytics/`** — the full 11-component Analytics suite: `OverviewView`, `CrossZoneComparison`, `ItemCostCalculator`, `ZoneDetailView`, `ZoneRewardsCard`, `ZoneSpeedBonusCard`, `ProfitRateCard`, `SurveyTypeDetailView`, `SurveyTypeDistributionChart`, `AnalyticsHeader`, `AnalyticsViewNav`.
- **`src/stores/surveyStore.ts`** — the legacy Pinia store the components consumed.
- **`src-tauri/src/survey_parser.rs`, `survey_persistence.rs`** — the legacy backend parser + persistence.
- **`src-tauri/src/db/player_commands_survey_events.rs`, `survey_sharing_commands.rs`** — the legacy Tauri commands that fed the UI.
- **`docs/features/screens/economics/`** — the legacy per-sub-tab docs (session / historical / analytics).

## Use

When rebuilding a screen:
- Open the matching reference file to see information density, layout decisions, and component composition.
- Rebuild on the new backend (the `src-tauri/src/survey/` module), using current app conventions (PaneLayout, ItemInline, real `.vue` components, StatCard, SearchableSelect, current color tokens).
- Don't copy the legacy code verbatim — the data shapes, store, and backend are all different now. Use it as a reference for **what** to build, not **how**.

## Removal

When all three screens (Session, History, Analytics) are rebuilt and shipped, this directory can be deleted. Until then, leave it alone.
