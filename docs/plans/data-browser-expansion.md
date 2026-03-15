# Data Browser Expansion Plan

**Status:** In Progress - 5 of 7 Phases Complete (71%)
**Last Updated:** 2026-03-13
**Completed:** Phases 1-5 (Skills, Abilities, Recipes, Quests, NPCs)
**Remaining:** Phases 6-7 (Cross-References, Polish)
**Goal:** Implement Skills, Abilities, Recipes, Quests, and NPCs tabs in the data browser to match the functionality and polish of the Items tab.

---

## Current State (Updated)

### ✅ Implemented
- **Items Tab:** Fully functional search, detail view, icon loading, keywords display
- **Skills Tab:** Full browser with search, detail view, related abilities list
- **Abilities Tab:** Skill-filtered browser with search and detail view
- **Recipes Tab:** Complete browser with ingredients, results, XP rewards, prerequisites
- **Quests Tab:** Full browser with filtering, objectives, requirements, rewards, dialog text
- **NPCs Tab:** Full browser with area filtering, favor preferences, skill training, gift items
- **Backend Commands:**
  - Skills: `get_all_skills`, `get_skill_by_name`
  - Abilities: `get_abilities_for_skill`
  - Recipes: `get_recipes_for_item`, `get_recipes_using_item`, `search_recipes`, `get_recipes_for_skill`, `get_items_batch`
  - Quests: `get_all_quests`, `search_quests`, `get_quest_by_key`
- **CDN Management:**
  - All 27 data files added to download list
  - Force refresh CDN button in Settings
  - CDN status display (version, item/skill counts)
- **Fixed Issues:**
  - Skills parser now correctly handles name-keyed JSON format
  - Debug logging for parsing diagnostics

### ❌ Still Missing
- **Cross-References:** Links between related entities (e.g., click a recipe's skill to jump to that skill)
- **Shared Components:** No extracted reusable components yet (EntityIcon, KeywordTag, etc.)
- **Polish:** Keyboard navigation, loading states, empty states improvements

---

## Implementation Pattern (from Items)

The Items tab demonstrates a clean, reusable pattern:

### Component Structure
```
ItemSearch.vue
├── Search Panel (left, 300px fixed width)
│   ├── Search bar with debounced input
│   ├── Results list (scrollable)
│   └── Result count / hint text
└── Detail Panel (right, flexible width)
    ├── Header (icon, name, metadata)
    ├── Sections (keywords, effects, etc.)
    └── Raw JSON dump
```

### Key Features
1. **Debounced search** (250ms delay)
2. **Icon loading** with cache via `getIconPath()`
3. **Keyword highlighting** (lint keywords styled differently)
4. **Metadata display** (ID, icon ID, value, stack size)
5. **Expandable sections** (keywords, effects)
6. **Raw JSON view** for debugging

### Backend Integration
- Query method in `gameDataStore.ts` calls Rust command
- Results returned as typed arrays
- Icon paths resolved lazily and cached

---

## Completed Implementation Summary

### Phase 1: Skills Browser ✅
**Files Created:**
- `src/components/SkillBrowser.vue` - Full skills browser component
- Backend: Skills parser fixed to handle name-keyed JSON format

**Features:**
- Search all skills by name or description
- Alphabetically sorted skill list
- Detail view with icon, description, XP table
- **Related Abilities section** - Lists all abilities for selected skill
- Keyword display
- Raw JSON dump

**Key Fix:** Changed skills parser from `parse_id_map` to `parse_string_map` because skills.json uses skill names as keys (e.g., "Alchemy") instead of "Skill_1" format.

### Phase 2: Abilities Browser ✅
**Files Created:**
- `src/components/AbilityBrowser.vue` - Skill-filtered abilities browser

**Features:**
- **Skill filter dropdown** - Select which skill's abilities to view
- Shows "All Skills" option
- Abilities sorted by level (ascending)
- Detail view with level, skill, description
- Icon support
- Search/filter within selected skill's abilities

### Phase 3: Recipes Browser ✅
**Files Created:**
- `src/components/RecipeBrowser.vue` - Complete recipe browser
- Backend commands: `search_recipes`, `get_recipes_for_skill`, `get_items_batch`

**Features:**
- **Skill filter dropdown** - Browse recipes by crafting skill
- Recipes sorted by skill level requirement
- **Ingredients section** - Shows each ingredient with:
  - Item name (resolved via batch lookup)
  - Stack size
  - Chance to consume percentage
- **Results section** - Shows each result with:
  - Item name
  - Stack size
  - Percent chance (if applicable)
- **XP Rewards** - Skill, XP amount, first-time XP bonus
- **Prerequisites** - Displays prereq recipe name
- Search/filter within selected skill

**Backend Enhancement:** Added `get_items_batch()` command for efficient bulk item lookups to resolve ingredient/result item names.

### Phase 4: Quests Browser ✅
**Files Created:**
- `src/components/QuestBrowser.vue` - Basic quest browser
- `src/types/gameData/quests.ts` - Quest type definitions
- Backend commands: `get_all_quests`, `search_quests`, `get_quest_by_key`

**Features:**
- Search all quests
- Display quest name (from DisplayName field)
- Show quest level if available
- Raw JSON display

**Note:** Full quest parsing (objectives, rewards, requirements) deferred to future enhancement. Currently displays raw JSON for detailed inspection.

### CDN & Infrastructure ✅
**Files Modified:**
- `src-tauri/src/cdn.rs` - Added all 27 data files to download list
- `src/components/Settings.vue` - Added CDN management section

**Features:**
- **Force Refresh CDN** button in Settings
- **CDN Status Display** showing:
  - Cached version vs Remote version
  - Up-to-date status
  - Item and skill counts
- Debug logging for parsing diagnostics
- All 27 game data files now download properly

---

## CDN Data Files & Parsing

### Complete File List (27 files)

The app downloads and parses the following files from the Project: Gorgon CDN:

**Core Game Data:**
- `items.json` - All items (10,704 items) - Uses `Item_X` keys
- `skills.json` - All skills (~182 skills) - **Uses skill names as keys** (e.g., "Alchemy")
- `abilities.json` - All abilities - Uses `Ability_X` keys
- `recipes.json` - All crafting recipes (4,422 recipes) - Uses `Recipe_X` keys
- `npcs.json` - All NPCs (337 NPCs) - Uses NPC internal names as keys
- `quests.json` - All quests - Uses quest internal names as keys

**Supporting Data:**
- `effects.json` - Effect definitions (23,055 effects)
- `areas.json` - Area/zone definitions (36 areas)
- `attributes.json` - Attribute definitions
- `xptables.json` - XP progression tables
- `advancementtables.json` - Skill advancement tables
- `abilitykeywords.json` - Ability keyword definitions
- `abilitydynamicdots.json` - Dynamic DoT data
- `abilitydynamicspecialvalues.json` - Special ability values
- `ai.json` - AI behavior definitions
- `directedgoals.json` - Quest/goal flow data
- `itemuses.json` - Item usage definitions
- `landmarks.json` - Landmark/location data
- `lorebooks.json` - Lorebook content
- `lorebookinfo.json` - Lorebook metadata
- `playertitles.json` - Player title definitions
- `sources_abilities.json` - Ability acquisition sources
- `sources_items.json` - Item drop sources
- `sources_recipes.json` - Recipe acquisition sources
- `storagevaults.json` - Storage vault definitions
- `tsysclientinfo.json` - Client info
- `tsysprofiles.json` - T-Sys profile data

### Parsing Notes

**Key Format Variations:**
1. **ID-based keys** (e.g., `Item_1`, `Ability_42`): Use `parse_id_map()`
   - Items, Abilities, Recipes, XP Tables, Player Titles
2. **String-based keys** (skill names, NPC names): Use `parse_string_map()`
   - Skills, NPCs, Quests, Areas, Attributes, AI, etc.

**Special Cases:**
- **Skills:** JSON uses skill names as keys (e.g., "Alchemy", "Cooking"). Each skill has an `Id` field inside the object.
- **Composite Data:** Some files like `lorebooks`, `sources`, `tsys`, `ability_dynamic` combine multiple JSON files.

### Manual CDN Refresh

Users can force a CDN refresh from **Settings > Game Data (CDN)** which:
1. Re-downloads all 27 data files
2. Re-parses all game data
3. Updates the in-memory cache
4. Shows updated item/skill counts

---

## Phase 1: Skills Browser

**Goal:** Add a Skills tab with search and detail view.

### Tasks

#### 1.1 Create `SkillBrowser.vue`
- **Search panel:** Text input, search by skill name
- **Results list:** All skills sorted alphabetically (no search limit needed — only ~60 skills)
- **Detail view:**
  - Icon (if present)
  - Name, ID
  - Description (if present)
  - XP table name
  - Keywords
  - **Related abilities section:** List all abilities for this skill (call `getAbilitiesForSkill()`)
  - **Trainers section:** List NPCs that train this skill (use `GameData.npcs_by_skill` index once NPCs are loaded)
  - Raw JSON dump

#### 1.2 Update `gameDataStore.ts`
- Already has `getAllSkills()` and `getSkillByName()` ✅
- No changes needed

#### 1.3 Update `DataBrowser.vue`
- Add `'skills'` to `DataView` union type
- Add "Skills" tab button
- Add conditional render for `<SkillBrowser />`

#### 1.4 Styling
- Reuse existing classes from `ItemSearch.vue`
- Consider extracting shared styles to a common stylesheet (`data-browser-common.css`)

---

## Phase 2: Abilities Browser

**Goal:** Add an Abilities tab with filtering by skill.

### Tasks

#### 2.1 Create `AbilityBrowser.vue`
- **Filter panel (left):**
  - Dropdown to filter by skill (populated from `getAllSkills()`)
  - "All Skills" option shows all abilities
  - Search input for ability name (optional enhancement)
- **Results list:**
  - Show abilities for selected skill (or all if "All Skills")
  - Sort by level ascending
  - Display: `[Lv X] Ability Name`
- **Detail view:**
  - Icon (if present)
  - Name, ID
  - Skill, Level
  - Description (if present)
  - Keywords
  - **Recipes using this ability:** Link to recipes that grant this ability (requires new backend index or search — defer to Phase 4)
  - Raw JSON dump

#### 2.2 Backend Enhancement (Optional)
- Current implementation only supports `get_abilities_for_skill(skill: String)`
- Could add `search_abilities(query: String)` for name-based search
- **Decision:** Start without search; add if needed

#### 2.3 Update `DataBrowser.vue`
- Add `'abilities'` to `DataView` type
- Add "Abilities" tab button
- Add conditional render for `<AbilityBrowser />`

---

## Phase 3: Recipes Browser

**Goal:** Add a Recipes tab with skill filtering and detailed ingredient/result views.

### Tasks

#### 3.1 Create `RecipeBrowser.vue`
- **Filter panel (left):**
  - Dropdown to filter by skill (populated from `getAllSkills()`)
  - Search input for recipe name (new backend command needed)
- **Results list:**
  - Show recipes for selected skill (use `GameData.recipes_by_skill` index)
  - Display: `[SkillName Lv X] Recipe Name`
  - Sort by skill level requirement
- **Detail view:**
  - Icon (if present)
  - Name, ID, Internal name
  - Skill, Skill level requirement
  - Description (if present)
  - **Ingredients section:**
    - List each ingredient with:
      - Item icon (clickable → jump to Items tab with that item selected)
      - Item name (resolve via `getItem(item_id)`)
      - Stack size
      - Chance to consume (if < 100%)
  - **Results section:**
    - List each result item with:
      - Item icon (clickable → jump to Items tab)
      - Item name
      - Stack size
      - Percent chance (if < 100%)
  - **XP Rewards:**
    - Reward skill, XP amount, first-time XP (if different)
  - **Prerequisites:**
    - Prereq recipe (if present, link to it)
  - Keywords
  - Raw JSON dump

#### 3.2 Backend Enhancements
**New Rust Commands:**
```rust
// In src-tauri/src/cdn_commands.rs
#[tauri::command]
pub async fn search_recipes(
    query: String,
    limit: Option<usize>,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeInfo>, String>

#[tauri::command]
pub async fn get_recipes_for_skill(
    skill: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<RecipeInfo>, String>
```

**gameDataStore.ts additions:**
```typescript
async function searchRecipes(query: string, limit = 50): Promise<RecipeInfo[]>
async function getRecipesForSkill(skill: string): Promise<RecipeInfo[]>
```

#### 3.3 Cross-Reference Helpers
**New Rust command for batch item lookups:**
```rust
#[tauri::command]
pub async fn get_items_batch(
    ids: Vec<u32>,
    state: State<'_, GameDataState>,
) -> Result<HashMap<u32, ItemInfo>, String>
```
- Efficiently fetch multiple items at once for ingredient/result display
- Returns a map keyed by ID for fast lookup

#### 3.4 Update `DataBrowser.vue`
- Add `'recipes'` to `DataView` type
- Add "Recipes" tab button
- Add conditional render for `<RecipeBrowser />`

---

## Phase 4: Quests Browser

**Goal:** Add a Quests tab with search and detailed quest information.

### Tasks

#### 4.1 Create `QuestBrowser.vue`
- **Search panel (left):**
  - Text input for quest name/description search
  - Filter by quest giver (NPC) once NPCs are integrated
- **Results list:**
  - Show quests matching search
  - Display: `Quest Name (Level)`
  - Sort by level or alphabetically
- **Detail view:**
  - Name, Internal Key
  - Description (if present)
  - **Quest Requirements:**
    - Level requirement
    - Prerequisite quests (clickable → jump to that quest)
    - Required items/skills (if present)
  - **Objectives section:**
    - List all objectives with:
      - Objective description
      - Target count (if applicable)
      - Completion criteria
  - **Rewards section:**
    - XP rewards
    - Item rewards (clickable → jump to Items tab)
    - Currency rewards
    - Favor rewards
  - **Quest Giver section:**
    - NPC name (clickable → jump to NPCs tab once implemented)
    - Location/Area
  - **Follow-up Quests:**
    - List quests that require this quest (if present)
  - Keywords (if present)
  - Raw JSON dump

#### 4.2 Backend Enhancements
**New Rust Commands:**
```rust
// In src-tauri/src/cdn_commands.rs
#[tauri::command]
pub async fn get_all_quests(
    state: State<'_, GameDataState>,
) -> Result<Vec<QuestInfo>, String>

#[tauri::command]
pub async fn search_quests(
    query: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<QuestInfo>, String>

#[tauri::command]
pub async fn get_quest_by_key(
    key: String,
    state: State<'_, GameDataState>,
) -> Result<Option<QuestInfo>, String>
```

**gameDataStore.ts additions:**
```typescript
async function getAllQuests(): Promise<QuestInfo[]>
async function searchQuests(query: string): Promise<QuestInfo[]>
async function getQuestByKey(key: string): Promise<QuestInfo | null>
```

**TypeScript type definitions:**
```typescript
// In src/types/gameData.ts
export interface QuestInfo {
  // Define based on actual quest data structure from CDN
  internal_name?: string
  display_name?: string
  description?: string
  level?: number
  requirements?: QuestRequirement[]
  objectives?: QuestObjective[]
  rewards?: QuestReward[]
  quest_giver?: string
  raw: any
}

export interface QuestRequirement {
  type: string
  value: any
}

export interface QuestObjective {
  description: string
  target_count?: number
}

export interface QuestReward {
  type: string
  item_id?: number
  xp?: number
  currency?: number
  favor?: number
}
```

**Note:** Quest data structure needs to be enhanced in `quests.rs` to properly parse fields. Currently it only stores raw JSON.

#### 4.3 Enhance `quests.rs` Parser
Currently the quests module only stores raw JSON. Need to parse actual quest fields:
- Parse quest name, description, level
- Parse requirements array
- Parse objectives array
- Parse rewards array
- Parse quest giver information
- Build quest prerequisite index

#### 4.4 Update `DataBrowser.vue`
- Add `'quests'` to `DataView` type
- Add "Quests" tab button
- Add conditional render for `<QuestBrowser />`

---

## Phase 5: NPCs Browser

**Goal:** Add an NPCs tab with location filtering and favor preferences.

### Tasks

#### 5.1 Create `NpcBrowser.vue`
- **Filter panel (left):**
  - Dropdown to filter by area (extract unique areas from `GameData.npcs`)
  - Search input for NPC name
- **Results list:**
  - Show NPCs matching filter/search
  - Display: `NPC Name (Area)`
  - Sort alphabetically
- **Detail view:**
  - No icon (NPCs don't have icon_id in current data)
  - Name, Internal Key
  - Description
  - **Location section:**
    - Area name
    - Area friendly name
  - **Training section:**
    - List skills this NPC trains (clickable → jump to Skills tab)
  - **Favor section:**
    - List preferences with:
      - Desire level (Love, Like, etc.)
      - Item/keyword name
      - Preference value
    - Sort by preference value descending
  - **Gift Items:**
    - List specific item gifts (if present)
  - Raw JSON dump

#### 5.2 Backend Enhancements
**New Rust Commands:**
```rust
#[tauri::command]
pub async fn get_all_npcs(
    state: State<'_, GameDataState>,
) -> Result<Vec<NpcInfo>, String>

#[tauri::command]
pub async fn search_npcs(
    query: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<NpcInfo>, String>

#[tauri::command]
pub async fn get_npcs_in_area(
    area: String,
    state: State<'_, GameDataState>,
) -> Result<Vec<NpcInfo>, String>
```

**gameDataStore.ts additions:**
```typescript
async function getAllNpcs(): Promise<NpcInfo[]>
async function searchNpcs(query: string): Promise<NpcInfo[]>
async function getNpcsInArea(area: string): Promise<NpcInfo[]>
```

#### 5.3 Update `DataBrowser.vue`
- Add `'npcs'` to `DataView` type
- Add "NPCs" tab button
- Add conditional render for `<NpcBrowser />`

---

## Phase 6: Cross-References & Navigation

**Goal:** Enable seamless navigation between related entities.

### Tasks

#### 6.1 Implement Tab Switching with Context
**New pattern:**
- DataBrowser emits events or uses a shared state to track:
  - Current tab
  - Selected entity in each tab (preserved when switching away)
- Example: User viewing "Amazing Leather Shoes" recipe → clicks "Amazing Leather" ingredient → switches to Items tab with that item selected

**Implementation:**
```typescript
// In DataBrowser.vue
const selectedItems = ref<{ [key: string]: any }>({
  items: null,
  skills: null,
  abilities: null,
  recipes: null,
  quests: null,
  npcs: null,
})

function navigateToItem(itemId: number) {
  selectedItems.value.items = itemId
  currentDataView.value = 'items'
}

function navigateToSkill(skillName: string) {
  selectedItems.value.skills = skillName
  currentDataView.value = 'skills'
}
```

Pass `selectedItems` and `navigate*` functions as props to each browser component.

#### 6.2 Add Clickable Links
- **RecipeBrowser:** Ingredient/result items link to Items tab
- **RecipeBrowser:** Skill requirement links to Skills tab
- **QuestBrowser:** Item rewards link to Items tab
- **QuestBrowser:** Prerequisite quests link to other quests
- **QuestBrowser:** Quest giver links to NPCs tab
- **SkillBrowser:** Related abilities list items link to Abilities tab
- **SkillBrowser:** Trainers link to NPCs tab
- **AbilityBrowser:** Skill name links to Skills tab
- **NpcBrowser:** Trained skills link to Skills tab
- **NpcBrowser:** Given quests link to Quests tab
- **ItemSearch:** Add "Recipes" section showing recipes that produce/use this item (link to Recipes tab)

#### 6.3 Visual Feedback
- Style clickable entity names with:
  - Underline on hover
  - Color: `#7ec8e3` (blue accent)
  - Cursor: pointer
- Add a small icon (→) after clickable links

---

## Phase 7: Polish & Enhancements

**Goal:** Improve usability, performance, and visual consistency.

### Tasks

#### 7.1 Shared Component Extraction
Create reusable components:
- `EntityIcon.vue` — handles icon loading for any entity type with an icon_id
- `KeywordTag.vue` — displays a keyword with appropriate styling
- `DetailSection.vue` — wraps section label + content
- `RawJsonDump.vue` — collapsible raw JSON viewer

#### 7.2 Loading States
- Add skeleton loaders for:
  - Results list while searching
  - Detail panel while loading related data
- Use same spinner animation as Items tab

#### 7.3 Empty States
- Show helpful messages when:
  - No results found
  - No skill selected (Abilities tab)
  - No area selected (NPCs tab)

#### 7.4 Keyboard Navigation
- Arrow keys to navigate results list
- Enter to select result
- Escape to clear selection

#### 7.5 Performance Optimization
- Virtualize long results lists (e.g., all abilities)
- Debounce all search inputs consistently (250ms)
- Cache resolved item names for recipe ingredients

#### 7.6 Accessibility
- Add ARIA labels to search inputs
- Ensure keyboard focus is visible
- Add alt text to all icons

#### 7.7 Visual Consistency
- Extract common CSS to `src/styles/data-browser.css`
- Ensure all tabs use same:
  - Panel widths
  - Font sizes
  - Color palette
  - Spacing

---

## Implementation Order & Status

| Phase | Status | Effort | Value | Dependencies | Notes |
|-------|--------|--------|-------|--------------|-------|
| Phase 1: Skills | ✅ **DONE** | Medium | High | None | Fixed parser for name-keyed JSON |
| Phase 2: Abilities | ✅ **DONE** | Medium | High | Phase 1 | Skill-filtered browser |
| Phase 3: Recipes | ✅ **DONE** | High | Very High | Phase 1, 2 | Full ingredient/result display |
| Phase 4: Quests | ✅ **DONE** | Medium | High | Phase 1 | Basic browser (full parsing pending) |
| Phase 5: NPCs | ✅ **DONE** | Medium | Medium | Phase 1 | Area filtering, favor prefs, training |
| Phase 6: Cross-References | ⏳ **TODO** | High | Very High | Phases 1-5 | Navigation between entities |
| Phase 7: Polish | ⏳ **TODO** | Medium | Medium | Phases 1-6 | Shared components, keyboard nav |

---

## Testing Checklist (per phase)

### Functionality
- [ ] Search/filter works correctly
- [ ] Results list displays expected data
- [ ] Detail view shows all relevant fields
- [ ] Icons load correctly (where applicable)
- [ ] Raw JSON dump matches backend data
- [ ] Related entities display correctly

### Performance
- [ ] Search debounce prevents excessive queries
- [ ] Large result sets render smoothly
- [ ] Icon loading doesn't block UI

### UX
- [ ] Empty states are informative
- [ ] Loading states are visible
- [ ] Keyboard navigation works
- [ ] Responsive layout (resizes gracefully)

### Integration
- [ ] Tab switching preserves state
- [ ] Cross-references navigate correctly
- [ ] TypeScript types match Rust types
- [ ] No console errors or warnings

---

## Remaining Work (Phases 5-7)

### Phase 5: NPCs Browser (TODO)

**Required Backend Work:**
1. Add NPC query commands to `cdn_commands.rs`:
   - `get_all_npcs()` - Return all NPCs
   - `search_npcs(query)` - Search by name
   - `get_npcs_in_area(area)` - Filter by area/zone
2. Export these commands in `lib.rs`
3. Add TypeScript wrappers in `gameDataStore.ts`

**Frontend Component:**
- Create `src/components/NpcBrowser.vue`
- Area/zone filter dropdown
- NPC search
- Detail view showing:
  - Name, location/area
  - **Training section** - Skills this NPC trains
  - **Favor section** - Favor preferences with desire levels
  - Gift items
  - Raw JSON dump

**Estimated Effort:** 2-3 hours

### Phase 6: Cross-References & Navigation (TODO)

**Goal:** Make related entities clickable to jump between tabs.

**Implementation Plan:**
1. **Add navigation state to DataBrowser.vue:**
   ```typescript
   const selectedEntities = ref({
     items: null,
     skills: null,
     abilities: null,
     recipes: null,
     quests: null,
     npcs: null,
   })

   function navigateTo(tab: DataView, entity: any) {
     selectedEntities.value[tab] = entity
     currentDataView.value = tab
   }
   ```

2. **Pass navigation props to all browser components**

3. **Add clickable links in each browser:**
   - RecipeBrowser: Ingredient/result items → Items tab
   - RecipeBrowser: Skill name → Skills tab
   - SkillBrowser: Abilities → Abilities tab
   - SkillBrowser: Trainers → NPCs tab
   - AbilityBrowser: Skill name → Skills tab
   - NpcBrowser: Trained skills → Skills tab
   - QuestBrowser: Item rewards → Items tab
   - QuestBrowser: Quest giver → NPCs tab

4. **Style clickable links:**
   - Blue accent color (#7ec8e3)
   - Underline on hover
   - Cursor pointer
   - Optional arrow icon (→)

**Estimated Effort:** 4-6 hours

### Phase 7: Polish & Enhancements (TODO)

**Shared Components:**
- Extract `EntityIcon.vue` for icon loading
- Extract `KeywordTag.vue` for keyword display
- Extract `DetailSection.vue` for section wrapping
- Extract `RawJsonDump.vue` for collapsible JSON viewer
- Create `src/styles/data-browser.css` for shared styles

**Loading States:**
- Add skeleton loaders for results lists
- Add spinners for detail panel loading
- Show loading state during batch item lookups

**Empty States:**
- "No results found" messages
- "Select a skill" prompts
- Helpful hints when no filter selected

**Keyboard Navigation:**
- Arrow keys to navigate results
- Enter to select
- Escape to clear selection
- Tab order for accessibility

**Performance:**
- Virtual scrolling for long lists (e.g., all abilities)
- Consistent 250ms debounce on all searches
- Cache resolved item names in recipes

**Accessibility:**
- ARIA labels on inputs
- Visible keyboard focus
- Alt text for all icons
- Screen reader announcements

**Estimated Effort:** 6-8 hours

---

## Future Enhancements (Post-Phase 7)

### Advanced Search
- Multi-field search (e.g., "items with keyword X and value > Y")
- Regex support in search boxes
- Save/load search filters

### Data Visualization
- Skill tree graph showing ability progression
- Recipe dependency graph
- NPC favor preference heatmap

### Export Features
- Export search results to CSV/JSON
- Copy entity data to clipboard
- Generate shareable links (if adding a backend/URL routing)

### Comparison Mode
- Side-by-side comparison of two items/recipes/abilities
- Diff highlighting for similar entities

### Notes & Bookmarks
- User-annotated notes on entities (stored locally)
- Bookmark favorite items/recipes for quick access

---

## Technical Debt & Considerations

### Backend Data Completeness
- Some stub modules (`effects`, `areas`, etc.) are not fully parsed
- NPCs module partially implemented (basic fields only)
- May need to enhance parsers as data browser exposes edge cases

### CDN File Download
- ✅ **FIXED:** All 27 data files now added to `DATA_FILES` list
- Users can force refresh via Settings > Game Data (CDN)
- Automatically downloads on version mismatch

### Index Rebuilding
- `GameData` has several prebuilt indices (`recipes_by_skill`, `npcs_by_skill`)
- Adding new cross-reference features may require new indices
- Consider making index building extensible or lazy

### TypeScript Type Safety
- Recipe ingredient/result arrays need proper typing for display
- Consider creating view model types that extend base types with resolved names

---

## Open Questions

1. **Should we paginate large result sets (e.g., all abilities)?**
   - Recommendation: No pagination; use virtual scrolling instead for simplicity

2. **How should we handle missing/null fields in detail views?**
   - Recommendation: Hide sections entirely if all fields are null; show "—" for individual null fields

3. **Should the Raw JSON section be collapsed by default?**
   - Recommendation: Collapsed by default; expand on click

4. **Do we need a "Recently Viewed" feature?**
   - Recommendation: Defer to future enhancements; not critical for MVP

5. **Should we support multiple tabs open at once (like a browser)?**
   - Recommendation: No; single active tab with preserved state is sufficient

---

## Success Metrics

### Phase Completion Criteria
- All tabs (Items, Skills, Abilities, Recipes, Quests, NPCs) functional
- All major fields displayed in detail views
- Cross-references navigable
- No regressions in existing functionality
- Zero TypeScript errors
- Zero Rust warnings for used code

### User Experience Goals
- Search response time < 100ms for cached data
- Icon loading < 500ms per icon (CDN dependent)
- Zero UI freezes during navigation
- Consistent visual style across all tabs

---

## Notes for Implementer

- Start with Phase 1 (Skills) as it's the simplest and establishes patterns
- Reuse `ItemSearch.vue` structure heavily; copy-paste and adapt
- Test each phase fully before moving to the next
- Keep commits small and scoped to single phases
- Update this document if requirements change during implementation

---

## Issues Fixed During Implementation

### 1. Skills Loading as 0 ✅
**Problem:** Skills were showing as 0 count, with console output:
```
skills.json: Warning: skipped 182 entries with invalid keys
skills.json: Parsed 0 raw skills
```

**Root Cause:** The skills.json file uses **skill names as keys** (e.g., `"Alchemy"`, `"Cooking"`) instead of the expected `"Skill_1"`, `"Skill_2"` pattern used by items and abilities.

**Solution:**
- Changed skills parser from `parse_id_map()` to `parse_string_map()`
- Added `Id` field to `RawSkill` struct to extract the ID from inside each skill object
- Updated parser to use skill names as HashMap keys, then build final map keyed by ID

**Files Changed:**
- `src-tauri/src/game_data/skills.rs` - Parser logic and struct definition
- `src-tauri/src/game_data/mod.rs` - Added debug logging for diagnostics

### 2. Missing CDN Files ✅
**Problem:** Console showing 18+ file-not-found warnings:
```
Warning: Failed to read C:\...\attributes.json: The system cannot find the file specified
Warning: Failed to read C:\...\quests.json: The system cannot find the file specified
... (18 more)
```

**Root Cause:** Only 8 files were in the `DATA_FILES` constant, but the code was trying to load 27 files.

**Solution:** Added all 27 data files to the `DATA_FILES` array in `cdn.rs`:
- attributes, xptables, advancementtables
- abilitykeywords, abilitydynamicdots, abilitydynamicspecialvalues
- ai, directedgoals, itemuses, landmarks
- lorebooks, lorebookinfo, playertitles, quests
- sources_abilities, sources_items, sources_recipes
- storagevaults, tsysprofiles

**Files Changed:**
- `src-tauri/src/cdn.rs` - `DATA_FILES` constant

### 3. No Manual CDN Refresh ✅
**Problem:** Users couldn't force a re-download when new files were added to the list.

**Solution:** Added CDN management UI to Settings:
- Status display showing cached vs remote version
- Item and skill counts
- "Force Refresh CDN Data" button
- Success/error feedback

**Files Changed:**
- `src/components/Settings.vue` - New CDN section with status and refresh button

### 4. Silent Parsing Failures ✅
**Problem:** When parsing failed, errors were silently swallowed with no diagnostics.

**Solution:** Added debug logging throughout parsers:
- Count of entries with invalid keys
- Count of raw entries parsed
- Count of final typed entries created
- Warnings for skipped entries

**Files Changed:**
- `src-tauri/src/game_data/mod.rs` - `parse_id_map()` logging
- `src-tauri/src/game_data/skills.rs` - Skills-specific logging
