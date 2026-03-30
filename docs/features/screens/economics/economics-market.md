# Economics — Market Prices

## Overview

A player-maintained price database for tracking market values of items. Used throughout the app for valuation (inventory worth, farming profit, survey economics).

## How It Works

- **Search and browse** existing market values by item name or ID
- **Add prices** via autocomplete item picker — search for an item, enter its market value
- **Edit inline** — click any price to modify it with real-time save
- **Delete** prices no longer needed
- **Notes** field per item for context (e.g., "checked 2024-03-15")

## Price Modes

Toggle between two modes (via Settings):
- **Universal** — prices apply across all servers (`server="*"`)
- **Per-Server** — prices scoped to the active server

## Valuation Modes

Six options controlling how item values are calculated app-wide:
1. Highest of market or vendor
2. Market price only
3. Vendor price only
4. Market if available, else vendor
5. Vendor if available, else market
6. Always zero

## Import / Export

- **Export** — copies all market values as JSON to clipboard
- **Import** — paste JSON with conflict resolution strategies:
  - **Newest wins** — keep whichever entry has a more recent timestamp
  - **Overwrite** — imported values always win
  - **Keep existing** — existing values always win

## Tauri Commands

- `get_market_values() → Vec<MarketValue>`
- `get_market_value(item_type_id) → Option<MarketValue>`
- `set_market_value(item_type_id, item_name, market_value, notes)`
- `delete_market_value(item_type_id)`
- `export_market_values() → String` (JSON)
- `import_market_values(json_data, strategy) → ImportResult`
