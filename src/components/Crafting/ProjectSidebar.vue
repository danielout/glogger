<template>
  <!-- Collapsed state -->
  <div v-if="collapsed" class="shrink-0 flex flex-col items-center py-2 gap-2">
    <button
      class="text-text-muted text-xs cursor-pointer bg-transparent border border-surface-elevated rounded px-1.5 py-1 hover:text-text-primary hover:border-border-default"
      title="Show project list"
      @click="collapsed = false">
      ▸
    </button>
    <span class="text-text-muted text-[0.6rem] [writing-mode:vertical-lr] select-none">Projects</span>
  </div>

  <!-- Expanded state -->
  <div v-else class="w-56 shrink-0 flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <button
          class="text-text-muted text-xs cursor-pointer bg-transparent border-none hover:text-text-primary"
          title="Collapse project list"
          @click="collapsed = true">
          ◂
        </button>
        <h3 class="text-text-primary text-sm font-semibold m-0">Projects</h3>
      </div>
      <button
        class="text-accent-gold text-xs cursor-pointer bg-transparent border border-accent-gold/30 rounded px-2 py-0.5 hover:bg-accent-gold/10"
        @click="showNewProject = true">
        + New
      </button>
    </div>

    <!-- New project inline form -->
    <div v-if="showNewProject" class="flex gap-1">
      <input
        ref="newProjectInput"
        v-model="newProjectName"
        class="input flex-1 text-xs"
        placeholder="Project name..."
        @keyup.enter="createProject"
        @keyup.escape="showNewProject = false" />
      <button
        class="text-accent-gold text-xs cursor-pointer bg-transparent border-none hover:text-accent-gold/70"
        :disabled="!newProjectName.trim()"
        @click="createProject">
        Save
      </button>
    </div>

    <!-- Empty state -->
    <div v-if="store.projects.length === 0 && !showNewProject" class="text-text-dim text-xs italic py-4">
      No projects yet. Create one to start planning crafts.
    </div>

    <!-- Project list -->
    <ul class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated rounded">
      <li
        v-for="project in store.projects"
        :key="project.id"
        class="px-3 py-2 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e] group"
        :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': store.activeProject?.id === project.id }"
        @click="store.loadProject(project.id)">
        <div class="flex items-center justify-between">
          <span class="text-text-primary/75 font-medium">{{ project.name }}</span>
          <span class="text-text-muted text-[0.65rem]">{{ project.entry_count }} recipes</span>
        </div>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useCraftingStore } from "../../stores/craftingStore";

const store = useCraftingStore();

const collapsed = ref(false);
const showNewProject = ref(false);
const newProjectName = ref("");
const newProjectInput = ref<HTMLInputElement | null>(null);

onMounted(() => {
  store.loadProjects();
});

async function createProject() {
  if (!newProjectName.value.trim()) return;
  const id = await store.createProject(newProjectName.value.trim());
  newProjectName.value = "";
  showNewProject.value = false;
  await store.loadProject(id);
}
</script>
