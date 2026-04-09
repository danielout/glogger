# Data Browser — Recipes

## Overview

Browse crafting recipes organized by skill, with full ingredient and result breakdowns.

## Search & Filters

- **Skill filter dropdown** — select a crafting skill to browse its recipes
- **"All Skills" mode** — global recipe search across all skills
- **Text search** — debounced 250ms

## Detail View

- Icon
- ID, skill, skill level requirement, internal name
- **Ingredients** — stack size, specific item (via `ItemInline`) or wildcard/keyword ingredient, chance to consume if < 100%
- **Results** — stack size, output item (via `ItemInline`), percent chance if < 100%
- **Estimated Cost** — per-ingredient and total material cost breakdown. Price sources are prioritized: market price → recursive craft cost (for intermediate products) → vendor price (value × 1.5). Intermediate ingredients that are themselves craftable use the cheapest option between their market price and their own recursive craft cost. Shows price source tags (market/craft/vendor) and per-unit cost for multi-output recipes.
- **XP Rewards** — skill, base XP, first-time bonus, dropoff level
- **Result effects** — green text list
- **Usage info** — action label, delay, sort skill
- **Prerequisites** — required recipe
- **Sources**
- **Keywords**
- **Raw JSON**
