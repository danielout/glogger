# Crafting > Price Helper (Integrated into Projects)

## Purpose

A crafting price calculator integrated directly into the Projects tab. When another player asks you to craft something, toggle "Pricing Mode" on a project to figure out what to charge — accounting for materials they provide, materials you source, and your crafting fee.

## How It Works

The Price Helper is not a separate screen. It's a mode toggle on any crafting project:

1. Open the **Projects** tab and select (or create) a project with the recipes you'll be crafting
2. Click **"Enable Pricing"** in the right pane — this activates pricing mode
3. The materials panel gains a **"They Give"** column — enter what the customer is supplying
4. The right pane gains a **Crafting Fee** section — configure your fee structure
5. A **Pricing Summary** appears at the bottom of the materials panel with the "Charge Customer" total

## Data Storage

Pricing data lives directly on the `crafting_projects` table (added in migration v20):
- `fee_config` — JSON column storing the fee structure
- `customer_provides` — JSON column mapping material keys to customer-supplied quantities

This means pricing data persists with the project and is included when duplicating projects.

## Fee Configuration

Three fee components, all combinable:
- **Per-craft fee** — flat gold amount multiplied by total craft count across all entries
- **Material %** — percentage of material cost, with configurable basis:
  - `total` — percentage of all materials (yours + theirs)
  - `yours` — percentage of only the materials you source
  - `theirs` — percentage of only the materials the customer supplies
- **Flat fee** — fixed gold amount for the entire project

The **Charge Customer** total = your material cost + all fee components.

## Default Fee Config

Global default fee configuration is stored via `useViewPrefs('price-helper-defaults')`. Users can "Save as Default" from any project's fee config, or "Reset" a project's fees back to the default.

## Files

The pricing feature is integrated into existing Projects files:
- `src/components/Crafting/ProjectsTab.vue` — orchestrator (pricing state, fee save/load, price resolution)
- `src/components/Crafting/ProjectRecipePanel.vue` — pricing toggle button + fee config section
- `src/components/Crafting/ProjectMaterialsPanel.vue` — customer-provides column + pricing summary
- `src/composables/usePriceCalculator.ts` — reactive fee calculation logic
- `src/composables/useRecipeCost.ts` — material price resolution (market/craft/vendor)
- `src-tauri/src/db/crafting_commands.rs` — project CRUD includes fee_config and customer_provides

## Material Price Resolution

When pricing mode is active, material prices are resolved after the standard material resolution completes. Prices use the same priority as `useRecipeCost`: market price > vendor fallback (value x 1.5). Dynamic/keyword ingredients show as unpriced.

## Design Decision: Integration vs Standalone

The Price Helper was initially built as a standalone tab with its own DB tables (`price_helper_quotes`, `price_helper_entries`). It was integrated into Projects because:
- Nearly all data and resolution logic overlapped with Projects
- Users would need to constantly swap between tabs
- Adding pricing as a project mode eliminates duplication and provides a seamless workflow

The standalone tables (from migration v19) remain in the database but are unused.
