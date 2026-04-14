# Data Browser — Lorebooks

## Overview

Browse and read in-game lorebooks — stories, histories, god lore, plot books, notes, and more. Provides a readable book viewer that's easier to use than the in-game UI.

## Data Sources

- `lorebooks.json` — book entries with title, category, text (HTML-formatted), location hint, keywords, visibility
- `lorebookinfo.json` — category metadata (title, subtitle, sort order)

## Search & Filtering

- Text search across title, body text, and location hint (computed, no debounce)
- Category dropdown filter (Gods, History, Stories, Misc, Notes and Signs, Plot, Volunteer Guides)
- All books loaded on mount; filtering is client-side

## List View

- Book title on the left, category label on the right
- Sorted alphabetically by title

## Detail View

- **Title** — gold header text
- **Category** — resolved to display name from lorebookinfo categories
- **Location hint** — where the book can be found in-game
- **Book text** — HTML-rendered content with formatting (headings, bold, italic) and newline-to-`<br>` conversion
- **Keywords** — blue badges (typically area keywords)
- **Raw JSON** — toggled via settings

## Backend Commands

- `get_all_lorebooks` — all books sorted by title
- `get_lorebook_categories` — category metadata sorted by sort_title
- `search_lorebooks(query, category?)` — filtered search with optional category constraint
