# Using the Data Browser

The Data Browser lets you search and explore all of Project: Gorgon's game data — items, skills, abilities, recipes, quests, and NPCs. Data is downloaded from the CDN and cached locally, so browsing is fast and works offline after the first load.

## Getting Started

1. Open the Data Browser from the navigation bar
2. Select a tab (Items, Skills, Abilities, Recipes, Quests, or NPCs)
3. Use the search bar or filters on the left panel to find entries
4. Click any result to view its details in the right panel

## Tabs

### Items

Search items by name. Results show the item ID, name, and an "unobtainable" badge where applicable. The detail panel shows:

- Icon, ID, icon ID, value, stack size
- Description
- Keywords
- Effect descriptions

### Skills

Browse all skills alphabetically or search by name/description. Selecting a skill shows:

- Icon and description
- XP table reference
- Keywords
- **Related Abilities** — all abilities for this skill, sorted by level

### Abilities

Filter abilities by skill using the dropdown, or browse all skills at once. Search within the selected skill's abilities. The detail view shows:

- Icon, ID, skill, level
- Description and keywords

### Recipes

Filter recipes by crafting skill or search by name. The detail view includes:

- **Ingredients** — each ingredient with item name, stack size, and chance to consume
- **Results** — each result item with stack size and success percentage
- **XP Rewards** — skill, XP amount, and first-time bonus (if different)
- **Prerequisites** — required prerequisite recipe, if any
- Keywords

### Quests

Search quests by name or description. Use the filter panel to narrow by:

- **Area** — quest location
- **Sort** — by Name, Level, or Area
- **Cancellable** — filter by whether the quest can be cancelled

The detail view shows quest description, dialog text, requirements, objectives, rewards (favor, XP, items), and quest metadata.

### NPCs

Filter NPCs by area or search by name/description. The detail view shows:

- Name, area, and description
- **Trains Skills** — skills this NPC can teach
- **Favor Preferences** — what items/keywords the NPC loves, likes, dislikes, or hates, sorted by preference strength and color-coded
- **Favorite Gift Items** — specific items the NPC loves

## CDN Data Management

Game data is downloaded from the Project: Gorgon CDN on first launch and cached locally. The app checks for updates on startup and downloads new data when the CDN version changes.

To force a re-download (e.g., after a game update):

1. Go to **Settings > Game Data (CDN)**
2. View the current cache status (version, item/skill counts)
3. Click **Force Refresh CDN Data**

The app works offline using cached data if the CDN is unreachable.

## Icons

Entity icons are fetched lazily from the CDN when you select an entity and cached locally for the session. If an icon hasn't loaded yet, you'll see a placeholder until it arrives.
