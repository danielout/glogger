<template>
  <div class="flex h-[calc(100vh-200px)]">
    <!-- Left: project sidebar (collapsible) -->
    <ProjectSidebar />

    <!-- Middle: materials panel (flex, gets most space) -->
    <ProjectMaterialsPanel
      class="mx-3"
      :active-project="store.activeProject"
      :materials="projectMaterials"
      :intermediates="projectIntermediates"
      :material-needs="materialNeeds"
      :resolving="resolvingAll"
      :checking-availability="checkingAvailability"
      @resolve="resolveProject"
      @check-availability="checkProjectAvailability" />

    <!-- Resize handle -->
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
      @duplicate="duplicateProject"
      @delete="deleteProject"
      @update-qty="updateEntryQty"
      @remove="(entryId) => store.removeEntry(entryId)"
      @toggle-intermediate="toggleIntermediate" />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from "vue";
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

/** Tracks which ingredients are marked "also craft this". Key: "{entryId}:{itemId}" */
const intermediateExpansions = ref(new Map<string, boolean>());

// Rebuild intermediateExpansions from persisted entry data when project loads
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
}, { immediate: true });

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
  await store.updateEntry(entryId, qty, entry?.expanded_ingredient_ids ?? []);
}

async function toggleIntermediate(entryId: number, itemId: number | null) {
  if (itemId === null) return;
  const key = `${entryId}:${itemId}`;
  const current = intermediateExpansions.value.get(key) ?? false;
  const newValue = !current;

  if (newValue) {
    intermediateExpansions.value.set(key, true);
  } else {
    intermediateExpansions.value.delete(key);
  }

  // Persist: collect all expanded item IDs for this entry
  const entry = store.activeProject?.entries.find((e) => e.id === entryId);
  if (entry) {
    const ids: number[] = [];
    for (const [k, v] of intermediateExpansions.value) {
      if (v && k.startsWith(`${entryId}:`)) {
        const id = parseInt(k.split(":")[1], 10);
        if (!isNaN(id)) ids.push(id);
      }
    }
    await store.updateEntry(entryId, entry.quantity, ids);
  }

  if (projectMaterials.value.size > 0) {
    resolveProject();
  }
}

async function resolveProject() {
  if (!store.activeProject) return;
  resolvingAll.value = true;
  projectMaterials.value = new Map();
  projectIntermediates.value = [];

  try {
    const combinedMaterials = new Map<string, FlattenedMaterial>();
    const allIntermediates: IntermediateCraft[] = [];

    const expandItemIds = new Set<number>();
    for (const [key, value] of intermediateExpansions.value) {
      if (value) {
        const itemId = parseInt(key.split(":")[1], 10);
        if (!isNaN(itemId)) expandItemIds.add(itemId);
      }
    }

    for (const entry of store.activeProject.entries) {
      const recipe = await gameData.resolveRecipe(entry.recipe_name);
      if (!recipe) continue;

      const resolved = await store.resolveRecipeIngredients(
        recipe,
        entry.quantity,
        false,
        new Set(),
        expandItemIds.size > 0 ? expandItemIds : undefined,
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

    projectMaterials.value = combinedMaterials;
    projectIntermediates.value = allIntermediates;
    materialNeeds.value = [];
  } catch (e) {
    console.error("[crafting] Project resolve failed:", e);
  } finally {
    resolvingAll.value = false;
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
