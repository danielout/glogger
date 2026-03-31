<template>
  <div class="flex h-[calc(100vh-200px)]">
    <!-- Left: project sidebar (collapsible) -->
    <ProjectSidebar />

    <!-- Middle: materials panel (flex, gets most space) -->
    <ProjectMaterialsPanel
      class="mx-3"
      :active-project="store.activeProject"
      :active-group-name="store.activeGroupName"
      :group-project-names="groupProjectNames"
      :group-entries="groupEntries"
      :stock-targets="stockTargets"
      :materials="projectMaterials"
      :intermediates="projectIntermediates"
      :expanded-item-ids="expandedItemIds"
      :intermediate-stock="intermediateStockMap"
      :material-needs="materialNeeds"
      :resolving="resolvingAll"
      :checking-availability="checkingAvailability"
      @resolve="onResolve"
      @toggle-intermediate="toggleIntermediateGlobal" />

    <!-- Resize handle (hidden in group view) -->
    <template v-if="!store.activeGroupName">
      <div
        class="w-1.5 shrink-0 cursor-col-resize flex items-center justify-center hover:bg-accent-gold/20 rounded transition-colors"
        :class="{ 'bg-accent-gold/30': isResizing }"
        @mousedown="startResize">
        <div class="w-px h-8 bg-border-default rounded-full" />
      </div>

      <!-- Right: recipe list (resizable width) -->
      <ProjectRecipePanel
        :style="{ width: `${recipePanelWidth}px` }"
        :active-project="store.activeProject"
        :intermediate-expansions="intermediateExpansions"
        :stock-targets="stockTargets"
        @duplicate="duplicateProject"
        @delete="deleteProject"
        @update-qty="updateEntryQty"
        @remove="(entryId) => store.removeEntry(entryId)"
        @toggle-intermediate="toggleIntermediate"
        @update-target-stock="updateEntryTargetStock" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useCraftingStore } from "../../stores/craftingStore";
import type { FlattenedMaterial, IntermediateCraft, MaterialNeed } from "../../types/crafting";
import ProjectSidebar from "./ProjectSidebar.vue";
import ProjectRecipePanel from "./ProjectRecipePanel.vue";
import ProjectMaterialsPanel from "./ProjectMaterialsPanel.vue";

const gameData = useGameDataStore();
const store = useCraftingStore();

// ── State ─────────────────────────────────────────────────────────────────────

const resolvingAll = ref(false);
const projectMaterials = ref(new Map<string, FlattenedMaterial>());
const projectIntermediates = ref<IntermediateCraft[]>([]);
const checkingAvailability = ref(false);
const materialNeeds = ref<MaterialNeed[]>([]);

/** Stock on hand for expanded intermediate items. Maps item_id → quantity */
const intermediateStockMap = ref(new Map<number, number>());

/** Stock target resolution results. Maps entry.id → { effectiveQty, currentStock } */
const stockTargets = ref(new Map<number, { effectiveQty: number; currentStock: number }>());

/** Combined entries when viewing a group summary */
const groupEntries = ref<import("../../types/crafting").CraftingProjectEntry[]>([]);

/** Tracks which ingredients are marked "also craft this". Key: "{entryId}:{itemId}" */
const intermediateExpansions = ref(new Map<string, boolean>());

/** Project names in the active group (for display in materials panel header) */
const groupProjectNames = computed(() => {
  if (!store.activeGroupName) return [];
  return store.getProjectsInGroup(store.activeGroupName).map((p) => p.name);
});

/** Set of item IDs currently marked for intermediate crafting (project-wide) */
const expandedItemIds = computed(() => {
  const ids = new Set<number>();
  for (const [key, value] of intermediateExpansions.value) {
    if (value) {
      const itemId = parseInt(key.split(":")[1], 10);
      if (!isNaN(itemId)) ids.add(itemId);
    }
  }
  return ids;
});

// Rebuild intermediateExpansions from persisted entry data when project loads, then auto-resolve
watch(() => store.activeProject, (project) => {
  const map = new Map<string, boolean>();
  if (project) {
    for (const entry of project.entries) {
      for (const itemId of entry.expanded_ingredient_ids) {
        map.set(`${entry.id}:${itemId}`, true);
      }
    }
  }
  intermediateExpansions.value = map;

  // Auto-resolve materials when a project with entries is loaded
  if (project && project.entries.length > 0) {
    resolveProject();
  } else {
    // Clear stale data when switching to empty/null project
    projectMaterials.value = new Map();
    projectIntermediates.value = [];
    materialNeeds.value = [];
  }
}, { immediate: true });

// ── Group summary watcher ─────────────────────────────────────────────────────

watch(() => store.activeGroupName, async (groupName) => {
  if (!groupName) return;

  // Clear single-project state
  intermediateExpansions.value = new Map();
  stockTargets.value = new Map();

  await resolveGroup(groupName);
}, { immediate: false });

// ── Resize logic ──────────────────────────────────────────────────────────────

const MIN_PANEL_WIDTH = 320;
const MAX_PANEL_WIDTH = 700;
const DEFAULT_PANEL_WIDTH = 420;

const recipePanelWidth = ref(DEFAULT_PANEL_WIDTH);
const isResizing = ref(false);
let startX = 0;
let startWidth = 0;

function startResize(e: MouseEvent) {
  isResizing.value = true;
  startX = e.clientX;
  startWidth = recipePanelWidth.value;
  document.addEventListener("mousemove", onResize);
  document.addEventListener("mouseup", stopResize);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
}

function onResize(e: MouseEvent) {
  // Dragging left increases width (panel is on the right)
  const delta = startX - e.clientX;
  const newWidth = Math.min(MAX_PANEL_WIDTH, Math.max(MIN_PANEL_WIDTH, startWidth + delta));
  recipePanelWidth.value = newWidth;
}

function stopResize() {
  isResizing.value = false;
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", stopResize);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
}

onBeforeUnmount(() => {
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", stopResize);
});

function onResolve() {
  if (store.activeGroupName) {
    resolveGroup(store.activeGroupName);
  } else {
    resolveProject();
  }
}

// ── Actions ───────────────────────────────────────────────────────────────────

async function duplicateProject() {
  if (!store.activeProject) return;
  const newId = await store.duplicateProject(store.activeProject.id);
  await store.loadProject(newId);
}

async function deleteProject() {
  if (!store.activeProject) return;
  if (!confirm(`Delete project "${store.activeProject.name}"?`)) return;
  await store.deleteProject(store.activeProject.id);
  projectMaterials.value = new Map();
  projectIntermediates.value = [];
  materialNeeds.value = [];
}

async function updateEntryQty(entryId: number, qty: number) {
  if (!qty || qty < 1) return;
  const entry = store.activeProject?.entries.find((e) => e.id === entryId);
  await store.updateEntry(entryId, qty, entry?.expanded_ingredient_ids ?? [], entry?.target_stock);
}

async function updateEntryTargetStock(entryId: number, targetStock: number | null) {
  const entry = store.activeProject?.entries.find((e) => e.id === entryId);
  if (!entry) return;
  await store.updateEntry(entryId, entry.quantity, entry.expanded_ingredient_ids, targetStock);
  // Re-resolve to recalculate materials with new target
  if (projectMaterials.value.size > 0 || targetStock !== null) {
    resolveProject();
  }
}

/**
 * Toggle an intermediate craft globally across all entries in the active project.
 * Called from the materials panel's centralized intermediate management.
 */
async function toggleIntermediateGlobal(itemId: number) {
  const entries = store.activeProject?.entries;
  if (!entries) return;

  const isCurrentlyExpanded = expandedItemIds.value.has(itemId);

  // Apply to ALL entries — toggle the item ID in every entry's expansion map
  for (const entry of entries) {
    const key = `${entry.id}:${itemId}`;
    if (isCurrentlyExpanded) {
      intermediateExpansions.value.delete(key);
    } else {
      intermediateExpansions.value.set(key, true);
    }

    // Persist for this entry
    const ids: number[] = [];
    for (const [k, v] of intermediateExpansions.value) {
      if (v && k.startsWith(`${entry.id}:`)) {
        const id = parseInt(k.split(":")[1], 10);
        if (!isNaN(id)) ids.push(id);
      }
    }
    await store.updateEntry(entry.id, entry.quantity, ids, entry.target_stock);
  }

  if (projectMaterials.value.size > 0) {
    resolveProject();
  }
}

async function toggleIntermediate(_entryId: number, itemId: number | null) {
  // Delegate to the global toggle — intermediates apply project-wide
  if (itemId === null) return;
  await toggleIntermediateGlobal(itemId);
}

let resolveGeneration = 0;

async function resolveGroup(groupName: string) {
  const gen = ++resolveGeneration;

  resolvingAll.value = true;
  projectMaterials.value = new Map();
  projectIntermediates.value = [];
  materialNeeds.value = [];

  try {
    const groupProjects = store.getProjectsInGroup(groupName);
    const projectIds = groupProjects.map((p) => p.id);

    // Load full project data for each project in the group
    const fullProjects: import("../../types/crafting").CraftingProject[] = [];
    for (const pid of projectIds) {
      const project = await invoke<import("../../types/crafting").CraftingProject>("get_crafting_project", { projectId: pid });
      if (gen !== resolveGeneration) return;
      fullProjects.push(project);
    }

    const allEntries = fullProjects.flatMap((p) => p.entries);
    groupEntries.value = allEntries;

    // Rebuild intermediateExpansions from all group entries
    const map = new Map<string, boolean>();
    for (const entry of allEntries) {
      for (const itemId of entry.expanded_ingredient_ids) {
        map.set(`${entry.id}:${itemId}`, true);
      }
    }
    intermediateExpansions.value = map;

    // Resolve stock targets across all entries
    const targets = await store.resolveStockTargets(allEntries);
    if (gen !== resolveGeneration) return;
    stockTargets.value = targets;

    const combinedMaterials = new Map<string, FlattenedMaterial>();
    const allIntermediates: IntermediateCraft[] = [];

    // Collect intermediate expansions from all entries
    const expandItemIds = new Set<number>();
    for (const project of fullProjects) {
      for (const entry of project.entries) {
        for (const itemId of entry.expanded_ingredient_ids) {
          expandItemIds.add(itemId);
        }
      }
    }

    // Pre-fetch stock for expanded intermediates
    let intermediateStock: Map<number, number> | undefined;
    if (expandItemIds.size > 0) {
      intermediateStock = await store.queryItemStock(Array.from(expandItemIds));
      if (gen !== resolveGeneration) return;
      intermediateStockMap.value = intermediateStock;
    } else {
      intermediateStockMap.value = new Map();
    }

    for (const entry of allEntries) {
      if (gen !== resolveGeneration) return;

      const targetInfo = targets.get(entry.id);
      const quantity = targetInfo ? targetInfo.effectiveQty : entry.quantity;
      if (quantity <= 0) continue;

      const recipe = await gameData.resolveRecipe(entry.recipe_name);
      if (!recipe) continue;

      const resolved = await store.resolveRecipeIngredients(
        recipe,
        quantity,
        false,
        new Set(),
        expandItemIds.size > 0 ? expandItemIds : undefined,
        intermediateStock,
      );

      const entryIntermediates = store.collectIntermediates(resolved.ingredients);
      allIntermediates.push(...entryIntermediates);

      const flat = store.flattenIngredients(resolved.ingredients);
      for (const [key, mat] of flat) {
        const existing = combinedMaterials.get(key);
        if (existing) {
          existing.quantity += mat.quantity;
          existing.expected_quantity += mat.expected_quantity;
        } else {
          combinedMaterials.set(key, { ...mat });
        }
      }
    }

    if (gen !== resolveGeneration || store.activeGroupName !== groupName) return;

    projectMaterials.value = combinedMaterials;
    projectIntermediates.value = allIntermediates;
    materialNeeds.value = [];

    if (combinedMaterials.size > 0) {
      await checkProjectAvailability();
    }
  } catch (e) {
    console.error("[crafting] Group resolve failed:", e);
  } finally {
    if (gen === resolveGeneration) {
      resolvingAll.value = false;
    }
  }
}

async function resolveProject() {
  if (!store.activeProject) return;
  const projectId = store.activeProject.id;
  const gen = ++resolveGeneration;

  resolvingAll.value = true;
  projectMaterials.value = new Map();
  projectIntermediates.value = [];

  try {
    // Resolve stock targets first (for entries in target mode)
    const targets = await store.resolveStockTargets(store.activeProject.entries);
    if (gen !== resolveGeneration) return;
    stockTargets.value = targets;

    const combinedMaterials = new Map<string, FlattenedMaterial>();
    const allIntermediates: IntermediateCraft[] = [];

    const expandItemIds = new Set<number>();
    for (const [key, value] of intermediateExpansions.value) {
      if (value) {
        const itemId = parseInt(key.split(":")[1], 10);
        if (!isNaN(itemId)) expandItemIds.add(itemId);
      }
    }

    // Pre-fetch stock for expanded intermediates so resolver can subtract on-hand
    let intermediateStock: Map<number, number> | undefined;
    if (expandItemIds.size > 0) {
      intermediateStock = await store.queryItemStock(Array.from(expandItemIds));
      if (gen !== resolveGeneration) return;
      intermediateStockMap.value = intermediateStock;
    } else {
      intermediateStockMap.value = new Map();
    }

    for (const entry of store.activeProject.entries) {
      if (gen !== resolveGeneration) return; // project changed, abort

      // Use effective quantity from stock target if available, else entry.quantity
      const targetInfo = targets.get(entry.id);
      const quantity = targetInfo ? targetInfo.effectiveQty : entry.quantity;
      if (quantity <= 0) continue; // target met, no materials needed

      const recipe = await gameData.resolveRecipe(entry.recipe_name);
      if (!recipe) continue;

      const resolved = await store.resolveRecipeIngredients(
        recipe,
        quantity,
        false,
        new Set(),
        expandItemIds.size > 0 ? expandItemIds : undefined,
        intermediateStock,
      );

      const entryIntermediates = store.collectIntermediates(resolved.ingredients);
      allIntermediates.push(...entryIntermediates);

      const flat = store.flattenIngredients(resolved.ingredients);
      for (const [key, mat] of flat) {
        const existing = combinedMaterials.get(key);
        if (existing) {
          existing.quantity += mat.quantity;
          existing.expected_quantity += mat.expected_quantity;
        } else {
          combinedMaterials.set(key, { ...mat });
        }
      }
    }

    // Guard: don't apply stale results if project changed during resolve
    if (gen !== resolveGeneration || store.activeProject?.id !== projectId) return;

    projectMaterials.value = combinedMaterials;
    projectIntermediates.value = allIntermediates;
    materialNeeds.value = [];

    // Auto-check availability after resolve
    if (combinedMaterials.size > 0) {
      await checkProjectAvailability();
    }
  } catch (e) {
    console.error("[crafting] Project resolve failed:", e);
  } finally {
    if (gen === resolveGeneration) {
      resolvingAll.value = false;
    }
  }
}

async function checkProjectAvailability() {
  if (projectMaterials.value.size === 0) return;
  checkingAvailability.value = true;
  try {
    materialNeeds.value = await store.checkMaterialAvailability(projectMaterials.value);
  } catch (e) {
    console.error("[crafting] Availability check failed:", e);
  } finally {
    checkingAvailability.value = false;
  }
}
</script>
