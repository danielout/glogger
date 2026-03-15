# Shared Components

Reusable components for displaying game entity references throughout the app. These provide a consistent look and behavior — color-coded names, hover tooltips, and click-to-navigate — so every part of the UI renders entities the same way.

## Entity Colors

Each entity type has a dedicated color token defined in [`theme.css`](../../src/assets/css/theme.css). Use these via Tailwind utilities (`text-entity-item`, `bg-entity-item/10`, `border-entity-item/50`, etc.).

| Entity   | Token            | Color     | Usage                  |
|----------|------------------|-----------|------------------------|
| Item     | `entity-item`    | `#7ec8e3` | Blue — items, loot     |
| Quest    | `entity-quest`   | `#e0c060` | Gold — quests          |
| Skill    | `entity-skill`   | `#5cb85c` | Green — skills         |
| NPC      | `entity-npc`     | `#e0965c` | Orange — NPCs          |
| Ability  | `entity-ability` | `#b07ce0` | Purple — abilities     |
| Recipe   | `entity-recipe`  | `#c8a05c` | Warm gold — recipes    |
| Area     | `entity-area`    | `#6a9fb5` | Teal — zones/areas     |
| Enemy    | `entity-enemy`   | `#d9534f` | Red — enemies/mobs     |

## Component Overview

```
src/components/Shared/
├── GameIcon.vue                 # Reusable icon with loading/fallback
├── EntityTooltipWrapper.vue     # Slot-based tooltip positioning
├── ItemCard.vue                 # Survey loot card (count + percentage)
├── Item/
│   ├── ItemTooltip.vue          # Tooltip content (presentational)
│   ├── ItemInline.vue           # Inline text reference
│   ├── ItemIconOnly.vue         # Icon-only reference
│   └── ItemIconPlus.vue         # Card with icon + name + value + type
├── Quest/
│   ├── QuestTooltip.vue         # Tooltip content
│   └── QuestInline.vue          # Inline text reference
├── Skill/
│   ├── SkillTooltip.vue         # Tooltip content
│   └── SkillInline.vue          # Inline text reference
├── NPC/
│   ├── NpcTooltip.vue           # Tooltip content
│   └── NpcInline.vue            # Inline text reference
├── Recipe/
│   ├── RecipeTooltip.vue        # Tooltip content
│   └── RecipeInline.vue         # Inline text reference
├── Ability/
│   ├── AbilityTooltip.vue       # Tooltip content
│   └── AbilityInline.vue        # Inline text reference
├── Area/
│   └── AreaInline.vue           # Placeholder (no backend data yet)
└── Enemy/
    └── EnemyInline.vue          # Placeholder (no backend data yet)
```

## Composables

Shared logic lives in [`src/composables/`](../../src/composables/):

- **`useTooltip(options?)`** — Tooltip show/hide with configurable delay. Returns `showTooltip`, `onMouseEnter`, `onMouseLeave`, `cleanup`. Calls optional `onHover` callback on first mouseenter for lazy data loading.
- **`useGameIcon()`** — Loads icon via `gameDataStore.getIconPath()` + `convertFileSrc()`. Returns `iconSrc`, `iconLoading`, `loadIcon(iconId)`. Memoizes per icon ID.
- **`useEntityNavigation()`** — Provides/injects a `navigateToEntity(target)` function for click-to-browse. The provider in [`App.vue`](../../src/App.vue) switches to the Data Browser view and passes a nav target to [`DataBrowser.vue`](../../src/components/DataBrowser/DataBrowser.vue).

## Inline Components

Inline components are designed to sit naturally within a block of text. They render as `inline-flex` elements with entity-colored text, sized to match surrounding content.

**Behavior:** Hover shows a rich tooltip after a 500ms delay. Click navigates to the entity's Data Browser tab.

### Item

```vue
<!-- Basic inline item reference -->
<ItemInline name="Amazing Longsword" />

<!-- Without icon -->
<ItemInline name="Amazing Longsword" :show-icon="false" />
```

**Props:** `name: string`, `showIcon?: boolean` (default `true`)

Data is loaded lazily from `gameDataStore.getItemByName()` on first hover.

### Quest

```vue
<QuestInline quest-key="Quest_SomeName" />
```

**Props:** `questKey: string`

Displays the quest's friendly name once loaded (falls back to the key). Data loaded from `gameDataStore.getQuestByKey()`.

### Skill

```vue
<SkillInline name="Sword" />
<SkillInline name="Sword" :show-icon="false" />
```

**Props:** `name: string`, `showIcon?: boolean` (default `true`)

Data loaded from `gameDataStore.getSkillByName()`.

### NPC

```vue
<!-- Name only (no tooltip) -->
<NpcInline name="Joeh" />

<!-- With pre-loaded data (shows tooltip on hover) -->
<NpcInline name="Joeh" :npc="npcInfoObject" />
```

**Props:** `name: string`, `npc?: NpcInfo`

The `npc` prop is optional. If provided, the tooltip shows full NPC details (area, trained skills, preferences). If omitted, the component renders as a styled name with no tooltip — this avoids expensive lookups when you just need the visual callout.

### Ability

```vue
<AbilityInline :ability="abilityInfoObject" />
```

**Props:** `ability: AbilityInfo`

Requires the full `AbilityInfo` object because the store only supports `getAbilitiesForSkill()`, not individual lookups. The calling component (which already fetched the list) passes the data directly.

### Recipe

```vue
<RecipeInline name="Brewed Mudbeer" />
<RecipeInline name="Brewed Mudbeer" :show-icon="false" />
```

**Props:** `name: string`, `showIcon?: boolean` (default `true`)

Data loaded from `gameDataStore.getRecipeByName()`.

### Area (placeholder)

```vue
<AreaInline name="Serbule" />
```

**Props:** `name: string`

Renders as styled text with a dotted underline. No tooltip, no click navigation. Will be upgraded when backend area data is available.

### Enemy (placeholder)

```vue
<EnemyInline name="Feral Cow" />
```

**Props:** `name: string`

Same as AreaInline — styled placeholder only.

## Item-Specific Components

### ItemIconOnly

Just an icon. Hover shows the full item tooltip. Click navigates to the item in the Data Browser.

```vue
<ItemIconOnly name="Amazing Longsword" />
<ItemIconOnly name="Amazing Longsword" size="lg" />
```

**Props:** `name: string`, `size?: "xs" | "sm" | "md" | "lg"` (default `"sm"`)

### ItemIconPlus

A rectangular card showing a large icon alongside the item's name, vendor value, and type keyword.

```vue
<ItemIconPlus name="Amazing Longsword" />
```

**Props:** `name: string`

### ItemCard

A survey loot card showing item icon, count, and drop percentage. Used by the Surveying session view. Hover shows item tooltip.

```vue
<ItemCard item-name="Iron Ore" :count="15" :percentage="42.3" />
```

**Props:** `itemName: string`, `count: number`, `percentage: number`

## Tooltip Components

Tooltip components are **presentational** — they receive already-loaded data as props and render the tooltip body. They are not used standalone; they are placed inside the `#tooltip` slot of `EntityTooltipWrapper`.

| Component          | Props                                |
|--------------------|--------------------------------------|
| `ItemTooltip`      | `item: ItemInfo`, `iconSrc: string \| null` |
| `QuestTooltip`     | `quest: QuestInfo`                   |
| `SkillTooltip`     | `skill: SkillInfo`, `iconSrc: string \| null` |
| `NpcTooltip`       | `npc: NpcInfo`                       |
| `AbilityTooltip`   | `ability: AbilityInfo`, `iconSrc: string \| null` |
| `RecipeTooltip`    | `recipe: RecipeInfo`, `iconSrc: string \| null` |

If you need a custom card or display that still wants the standard tooltip on hover, use `EntityTooltipWrapper` directly:

```vue
<EntityTooltipWrapper border-class="border-entity-item/50" @hover="loadMyData">
  <!-- Your custom trigger content -->
  <div>My custom item display</div>

  <template #tooltip>
    <ItemTooltip v-if="myItem" :item="myItem" :icon-src="myIconSrc" />
  </template>
</EntityTooltipWrapper>
```

## Base Components

### GameIcon

Renders an icon by `icon_id` with loading spinner and `?` fallback.

```vue
<GameIcon :icon-id="item.icon_id" alt="Item name" size="md" />
```

**Props:** `iconId: number | null | undefined`, `alt?: string`, `size?: "xs" | "sm" | "md" | "lg"`

Sizes: `xs` = 16px, `sm` = 20px (default), `md` = 32px, `lg` = 48px.

### EntityTooltipWrapper

Wraps any content with tooltip-on-hover behavior. Handles delay, positioning, and show/hide state.

```vue
<EntityTooltipWrapper
  :delay="500"
  border-class="border-entity-skill/50"
  @hover="onFirstHover"
>
  <slot />              <!-- trigger content -->
  <template #tooltip>
    <slot name="tooltip" />  <!-- tooltip body -->
  </template>
</EntityTooltipWrapper>
```

**Props:** `delay?: number` (default 500), `disabled?: boolean`, `borderClass?: string`

**Events:** `hover` — emitted once on first mouseenter, use for lazy data loading.

## Navigation

Inline components call `navigateToEntity({ type, id })` on click, which is provided via Vue's provide/inject from [`App.vue`](../../src/App.vue). This switches the app to the Data Browser view and activates the correct tab.

The mapping from entity type to Data Browser tab:

| Entity Type | Browser Tab |
|-------------|-------------|
| `item`      | Items       |
| `skill`     | Skills      |
| `ability`   | Abilities   |
| `recipe`    | Recipes     |
| `quest`     | Quests      |
| `npc`       | NPCs        |
| `area`      | *(no-op)*   |
| `enemy`     | *(no-op)*   |

To use navigation in a custom component:

```vue
<script setup lang="ts">
import { useEntityNavigation } from "../../composables/useEntityNavigation";
const { navigateToEntity } = useEntityNavigation();

function handleClick() {
  navigateToEntity({ type: "item", id: "Amazing Longsword" });
}
</script>
```
