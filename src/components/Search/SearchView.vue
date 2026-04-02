<template>
  <PaneLayout screen-key="search">
    <div class="flex flex-col gap-4 max-w-4xl mx-auto w-full">
      <!-- Search bar -->
      <div class="flex items-center gap-2 bg-surface-elevated border border-border-default rounded-lg px-4 py-2.5">
        <span class="text-text-muted">&#x1F50D;</span>
        <input
          ref="inputRef"
          v-model="query"
          type="text"
          placeholder="Search everything..."
          class="flex-1 bg-transparent border-none text-sm text-text-primary placeholder-text-muted focus:outline-none"
        />
        <span v-if="totalResults > 0" class="text-xs text-text-muted">{{ totalResults }} results</span>
      </div>

      <!-- Category filter toggles -->
      <div class="flex flex-wrap gap-2">
        <button
          v-for="cat in allCategories"
          :key="cat.key"
          class="px-2.5 py-1 text-xs rounded-full border cursor-pointer transition-colors"
          :class="enabledCategories.has(cat.key)
            ? 'bg-accent-gold/15 border-accent-gold/40 text-accent-gold'
            : 'bg-surface-elevated border-border-default text-text-muted hover:text-text-secondary'"
          @click="toggleCategory(cat.key)"
        >
          {{ cat.label }}
          <span v-if="categoryCounts[cat.key]" class="ml-1 opacity-70">({{ categoryCounts[cat.key] }})</span>
        </button>
      </div>

      <!-- Results -->
      <div v-if="query.length >= 2" class="flex flex-col gap-3">
        <template v-for="category in filteredCategories" :key="category.name">
          <div class="bg-surface-base border border-border-default rounded-lg overflow-hidden">
            <!-- Category header (collapsible) -->
            <button
              class="w-full flex items-center gap-2 px-4 py-2 text-left cursor-pointer border-none bg-transparent hover:bg-surface-elevated transition-colors"
              @click="toggleCollapse(category.name)"
            >
              <span class="text-xs transition-transform" :class="collapsed.has(category.name) ? '' : 'rotate-90'">&#x25B6;</span>
              <span class="text-xs font-semibold text-text-secondary uppercase tracking-wider flex-1">{{ category.name }}</span>
              <span class="text-xs text-text-muted">{{ category.results.length }}</span>
            </button>

            <!-- Category results -->
            <div v-if="!collapsed.has(category.name)" class="border-t border-border-default">
              <div
                v-for="(result, idx) in category.results"
                :key="idx"
                class="flex items-center gap-3 px-4 py-2 hover:bg-surface-elevated cursor-pointer transition-colors border-b border-border-default last:border-b-0"
                @click="navigateToResult(result)"
              >
                <div class="flex-1 min-w-0">
                  <div class="text-sm" @click.stop="navigateToResult(result)">
                    <!-- Use inline components for entity types that have tooltips -->
                    <ItemInline v-if="category.name === 'Game Items' || category.name === 'Your Items'" :reference="result.label" :show-icon="true" />
                    <RecipeInline v-else-if="category.name === 'Game Recipes'" :reference="result.label" :show-icon="true" />
                    <NpcInline v-else-if="category.name === 'NPCs'" :reference="String(result.navigation.entityId ?? result.label)" />
                    <QuestInline v-else-if="category.name === 'Quests'" :reference="String(result.navigation.entityId ?? result.label)" />
                    <SkillInline v-else-if="category.name === 'Your Skills'" :reference="result.label" :show-icon="true" />
                    <span v-else class="text-text-primary">{{ result.label }}</span>
                  </div>
                  <div v-if="result.detail" class="text-xs text-text-muted truncate">{{ result.detail }}</div>
                </div>
                <span class="text-[0.6rem] text-text-muted shrink-0">{{ categoryNavHint(result) }}</span>
              </div>
            </div>
          </div>
        </template>

        <!-- No results -->
        <div v-if="filteredCategories.length === 0 && !loading" class="text-center py-12 text-sm text-text-muted">
          No results for "{{ query }}"
        </div>
      </div>

      <!-- Initial state -->
      <div v-else class="text-center py-16 text-text-muted">
        <div class="text-sm">Type at least 2 characters to search</div>
        <div class="text-xs mt-2">Searches your items, skills, game data, NPCs, recipes, quests, and market values</div>
      </div>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, nextTick, onMounted } from "vue"
import PaneLayout from "../Shared/PaneLayout.vue"
import ItemInline from "../Shared/Item/ItemInline.vue"
import RecipeInline from "../Shared/Recipe/RecipeInline.vue"
import NpcInline from "../Shared/NPC/NpcInline.vue"
import QuestInline from "../Shared/Quest/QuestInline.vue"
import SkillInline from "../Shared/Skill/SkillInline.vue"
import { useQuickSearch, type SearchResult } from "../../composables/useQuickSearch"

const emit = defineEmits<{
  navigate: [result: SearchResult]
}>()

const inputRef = ref<HTMLInputElement>()
const query = ref("")
const collapsed = reactive(new Set<string>())

// No cap for the dedicated page
const { categories, loading } = useQuickSearch(query, { cap: 50 })

const allCategories = [
  { key: "Your Items", label: "Your Items" },
  { key: "Your Skills", label: "Your Skills" },
  { key: "Game Items", label: "Game Items" },
  { key: "Game Recipes", label: "Game Recipes" },
  { key: "NPCs", label: "NPCs" },
  { key: "Quests", label: "Quests" },
  { key: "Market Values", label: "Market Values" },
]

const enabledCategories = reactive(new Set(allCategories.map(c => c.key)))

const filteredCategories = computed(() =>
  categories.value.filter(c => enabledCategories.has(c.name))
)

const totalResults = computed(() =>
  filteredCategories.value.reduce((sum, c) => sum + c.results.length, 0)
)

const categoryCounts = computed(() => {
  const counts: Record<string, number> = {}
  for (const c of categories.value) {
    counts[c.name] = c.results.length
  }
  return counts
})

function toggleCategory(key: string) {
  if (enabledCategories.has(key)) {
    enabledCategories.delete(key)
  } else {
    enabledCategories.add(key)
  }
}

function toggleCollapse(name: string) {
  if (collapsed.has(name)) {
    collapsed.delete(name)
  } else {
    collapsed.add(name)
  }
}

function categoryNavHint(result: SearchResult): string {
  if (result.navigation.entityType) {
    return `Data Browser › ${result.navigation.subTab}`
  }
  const hints: Record<string, string> = {
    inventory: "Inventory",
    character: "Character",
    crafting: "Crafting",
    economics: "Economics",
  }
  return hints[result.navigation.view] ?? result.navigation.view
}

function navigateToResult(result: SearchResult) {
  emit("navigate", result)
}

onMounted(async () => {
  await nextTick()
  inputRef.value?.focus()
})

// Re-focus input when view becomes visible
watch(query, () => {
  // keep focus in the input
})
</script>
