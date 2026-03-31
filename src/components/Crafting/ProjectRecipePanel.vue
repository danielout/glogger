<template>
  <div class="shrink-0 overflow-y-auto border border-surface-elevated rounded p-4">
    <EmptyState v-if="!activeProject" variant="panel" primary="No project selected" secondary="Select or create a project to see recipes." />

    <div v-else class="flex flex-col gap-4">
      <!-- Project header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <h3 class="text-accent-gold text-base font-semibold m-0">{{ activeProject.name }}</h3>
          <button
            class="text-text-muted text-xs cursor-pointer bg-transparent border-none hover:text-text-primary"
            @click="editProjectName">
            edit
          </button>
        </div>
        <div class="flex gap-2">
          <button
            class="text-text-muted text-xs cursor-pointer bg-transparent border border-border-light rounded px-2 py-0.5 hover:text-text-primary hover:border-border-default"
            @click="$emit('duplicate')">
            Duplicate
          </button>
          <button
            class="text-accent-red/70 text-xs cursor-pointer bg-transparent border border-accent-red/20 rounded px-2 py-0.5 hover:text-accent-red hover:border-accent-red/40"
            @click="$emit('delete')">
            Delete
          </button>
        </div>
      </div>

      <!-- Group assignment -->
      <div class="flex items-center gap-2 text-xs">
        <span class="text-text-dim shrink-0">Group:</span>
        <div class="relative flex-1">
          <input
            v-model="groupInput"
            class="input w-full text-xs"
            placeholder="None"
            @input="onGroupInput"
            @blur="commitGroup"
            @keyup.enter="commitGroup"
            list="group-suggestions" />
          <datalist id="group-suggestions">
            <option v-for="name in existingGroupNames" :key="name" :value="name" />
          </datalist>
        </div>
        <button
          v-if="groupInput"
          class="text-text-muted text-[0.65rem] cursor-pointer bg-transparent border-none hover:text-accent-red"
          title="Remove from group"
          @click="clearGroup">
          &#10005;
        </button>
      </div>

      <!-- Add recipe search -->
      <div class="flex gap-2">
        <input
          v-model="addRecipeQuery"
          class="input flex-1 text-xs"
          placeholder="Search recipes to add..."
          @input="debouncedRecipeSearch" />
        <input
          v-model.number="addRecipeQty"
          type="number"
          min="1"
          class="input w-16 text-xs text-center"
          placeholder="Qty" />
      </div>

      <!-- Recipe search results -->
      <ul
        v-if="addRecipeResults.length > 0"
        class="list-none m-0 p-0 border border-surface-elevated rounded max-h-40 overflow-y-auto -mt-2">
        <li
          v-for="recipe in addRecipeResults"
          :key="recipe.id"
          class="flex items-baseline gap-2 px-3 py-1.5 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
          @click="addRecipeToProject(recipe)">
          <span class="text-text-muted text-[0.72rem] min-w-12 shrink-0">
            [{{ recipe.skill ?? '?' }} {{ recipe.skill_level_req ?? 0 }}]
          </span>
          <span class="text-text-primary/75">{{ recipe.name }}</span>
        </li>
      </ul>

      <!-- Empty state -->
      <div v-if="activeProject.entries.length === 0" class="text-text-dim text-xs italic py-4">
        No recipes added yet. Search above to add recipes to this project.
      </div>

      <!-- Recipe entries -->
      <div v-else class="flex flex-col gap-1">
        <div class="flex items-center justify-between">
          <h4 class="text-text-secondary text-xs font-semibold uppercase tracking-wide m-0">Recipe List</h4>
          <div class="flex gap-1">
            <button
              class="text-text-muted text-[0.65rem] cursor-pointer bg-transparent border-none hover:text-text-primary"
              @click="expandAll">
              Expand All
            </button>
            <span class="text-text-muted/40 text-[0.65rem]">|</span>
            <button
              class="text-text-muted text-[0.65rem] cursor-pointer bg-transparent border-none hover:text-text-primary"
              @click="collapseAll">
              Collapse All
            </button>
          </div>
        </div>
        <ProjectEntryCard
          v-for="entry in activeProject.entries"
          :key="entry.id"
          ref="entryCards"
          :entry="entry"
          :intermediate-expansions="intermediateExpansions"
          :stock-target="stockTargets.get(entry.id)"
          @update-qty="(entryId, qty) => $emit('update-qty', entryId, qty)"
          @remove="(entryId) => $emit('remove', entryId)"
          @toggle-intermediate="(entryId, itemId) => $emit('toggle-intermediate', entryId, itemId)"
          @update-target-stock="(entryId, ts) => $emit('update-target-stock', entryId, ts)" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useCraftingStore } from "../../stores/craftingStore";
import type { CraftingProject } from "../../types/crafting";
import type { RecipeInfo } from "../../types/gameData/recipes";
import EmptyState from "../Shared/EmptyState.vue";
import ProjectEntryCard from "./ProjectEntryCard.vue";

const props = defineProps<{
  activeProject: CraftingProject | null
  intermediateExpansions: Map<string, boolean>
  stockTargets: Map<number, { effectiveQty: number; currentStock: number }>
}>();

defineEmits<{
  'duplicate': []
  'delete': []
  'update-qty': [entryId: number, qty: number]
  'remove': [entryId: number]
  'toggle-intermediate': [entryId: number, itemId: number | null]
  'update-target-stock': [entryId: number, targetStock: number | null]
}>();

const gameData = useGameDataStore();
const store = useCraftingStore();

const entryCards = ref<InstanceType<typeof ProjectEntryCard>[]>([]);

const addRecipeQuery = ref("");
const addRecipeQty = ref(1);
const addRecipeResults = ref<RecipeInfo[]>([]);
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

const groupInput = ref("");

// Sync groupInput when active project changes
watch(() => props.activeProject, (p) => {
  groupInput.value = p?.group_name ?? "";
}, { immediate: true });

// Collect existing group names for autocomplete
const existingGroupNames = computed(() => {
  const names = new Set<string>();
  for (const p of store.projects) {
    if (p.group_name) names.add(p.group_name);
  }
  return Array.from(names).sort();
});

function onGroupInput() {
  // Just let the input update reactively; commit on blur/enter
}

function commitGroup() {
  if (!props.activeProject) return;
  const newGroup = groupInput.value.trim() || null;
  if (newGroup !== (props.activeProject.group_name ?? null)) {
    store.updateProject(props.activeProject.id, props.activeProject.name, props.activeProject.notes, newGroup);
  }
}

function clearGroup() {
  groupInput.value = "";
  commitGroup();
}

function editProjectName() {
  if (!props.activeProject) return;
  const name = prompt("Project name:", props.activeProject.name);
  if (name && name.trim()) {
    store.updateProject(props.activeProject.id, name.trim(), props.activeProject.notes, props.activeProject.group_name);
  }
}

function debouncedRecipeSearch() {
  if (searchTimeout) clearTimeout(searchTimeout);
  searchTimeout = setTimeout(async () => {
    if (!addRecipeQuery.value.trim()) {
      addRecipeResults.value = [];
      return;
    }
    addRecipeResults.value = await gameData.searchRecipes(addRecipeQuery.value, 20);
  }, 250);
}

async function addRecipeToProject(recipe: RecipeInfo) {
  if (!props.activeProject) return;
  await store.addEntry(
    props.activeProject.id,
    recipe.id,
    recipe.name,
    Math.max(1, addRecipeQty.value),
  );
  addRecipeQuery.value = "";
  addRecipeResults.value = [];
  addRecipeQty.value = 1;
}

function expandAll() {
  for (const card of entryCards.value) {
    card.expanded = true;
  }
}

function collapseAll() {
  for (const card of entryCards.value) {
    card.expanded = false;
  }
}
</script>
