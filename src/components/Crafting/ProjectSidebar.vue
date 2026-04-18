<template>
  <div class="flex flex-col gap-2 px-2">
    <div class="flex items-center justify-between">
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

    <!-- Sort control -->
    <select
      v-model="sortMode"
      class="px-2 py-0.5 bg-surface-base border border-border-default rounded text-[0.65rem] text-text-muted cursor-pointer">
      <option value="recent">Recent</option>
      <option value="az">A-Z</option>
      <option value="za">Z-A</option>
    </select>

    <!-- Empty state -->
    <EmptyState v-if="store.projects.length === 0 && !showNewProject" variant="compact" primary="No projects yet" secondary="Create one to start planning crafts." />

    <!-- Project list (grouped) -->
    <ul class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated rounded">
      <!-- Grouped projects -->
      <template v-for="group in groupedProjects.groups" :key="group.name">
        <li
          class="px-3 py-1.5 cursor-pointer border-b border-surface-dark text-xs bg-surface-dark/40 sticky top-0 z-10 select-none hover:bg-surface-dark/60"
          :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': store.activeGroupName === group.name }"
          @click="store.selectGroup(group.name)">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-1.5">
              <span
                class="text-text-secondary text-xs w-3 hover:text-text-primary"
                @click.stop="toggleGroup(group.name)">
                {{ collapsedGroups.has(group.name) ? '&#9656;' : '&#9662;' }}
              </span>
              <span class="text-text-secondary font-semibold">{{ group.name }}</span>
            </div>
            <span class="text-text-muted text-[0.6rem]">{{ group.projects.length }}</span>
          </div>
        </li>
        <template v-if="!collapsedGroups.has(group.name)">
          <li
            v-for="project in group.projects"
            :key="project.id"
            class="px-3 pl-6 py-2 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': store.activeProject?.id === project.id }"
            @click="store.loadProject(project.id)">
            <div class="flex items-center justify-between">
              <span class="text-text-primary/75 font-medium">{{ project.name }}</span>
              <span class="text-text-muted text-[0.65rem]">{{ project.entry_count }} recipes</span>
            </div>
          </li>
        </template>
      </template>

      <!-- Ungrouped projects -->
      <li
        v-for="project in groupedProjects.ungrouped"
        :key="project.id"
        class="px-3 py-2 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
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
import { ref, reactive, computed, onMounted, nextTick, watch } from "vue";
import { useCraftingStore } from "../../stores/craftingStore";
import type { CraftingProjectSummary } from "../../types/crafting";
import EmptyState from "../Shared/EmptyState.vue";

const store = useCraftingStore();

const showNewProject = ref(false);
const newProjectName = ref("");
const newProjectInput = ref<HTMLInputElement | null>(null);
const sortMode = ref<'recent' | 'az' | 'za'>('recent');
const collapsedGroups = reactive(new Set<string>());

watch(showNewProject, (v) => {
  if (v) nextTick(() => newProjectInput.value?.focus());
});

const sortedProjects = computed(() => {
  const list = [...store.projects];
  switch (sortMode.value) {
    case 'az': return list.sort((a, b) => a.name.localeCompare(b.name));
    case 'za': return list.sort((a, b) => b.name.localeCompare(a.name));
    default: return list;
  }
});

const groupedProjects = computed(() => {
  const groupMap = new Map<string, CraftingProjectSummary[]>();
  const ungrouped: CraftingProjectSummary[] = [];

  for (const p of sortedProjects.value) {
    if (p.group_name) {
      const key = p.group_name;
      if (!groupMap.has(key)) groupMap.set(key, []);
      groupMap.get(key)!.push(p);
    } else {
      ungrouped.push(p);
    }
  }

  const groups = Array.from(groupMap.entries())
    .map(([name, projects]) => ({ name, projects }));

  switch (sortMode.value) {
    case 'az':
      groups.sort((a, b) => a.name.localeCompare(b.name));
      break;
    case 'za':
      groups.sort((a, b) => b.name.localeCompare(a.name));
      break;
    default:
      // "recent": sort groups by most recently updated project within each group
      groups.sort((a, b) => {
        const aMax = Math.max(...a.projects.map(p => new Date(p.updated_at).getTime()));
        const bMax = Math.max(...b.projects.map(p => new Date(p.updated_at).getTime()));
        return bMax - aMax;
      });
  }

  return { groups, ungrouped };
});

function toggleGroup(name: string) {
  if (collapsedGroups.has(name)) {
    collapsedGroups.delete(name);
  } else {
    collapsedGroups.add(name);
  }
}

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
