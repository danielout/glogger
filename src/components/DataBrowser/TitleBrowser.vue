<template>
  <PaneLayout screen-key="db-titles" :left-pane="{ title: 'Titles', defaultWidth: 360, minWidth: 280, maxWidth: 500 }">
    <template #left>
      <!-- Status banner if data not ready -->
      <div v-if="store.status !== 'ready'" class="p-4 text-sm">
        <span v-if="store.status === 'loading'" class="text-accent-gold"
          >⟳ Loading game data…</span
        >
        <span v-else-if="store.status === 'error'" class="text-accent-red"
          >✕ {{ store.errorMessage }}</span
        >
      </div>

      <template v-else>
      <div class="flex flex-col gap-2 h-full overflow-hidden">
        <div class="flex items-center gap-2 relative">
          <input
            v-model="query"
            class="input flex-1"
            placeholder="Search titles…"
            autofocus />
          <span v-if="loading" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="filteredTitles.length" class="text-text-dim text-xs min-w-6 text-right">{{
            filteredTitles.length
          }}</span>
        </div>

        <div v-if="!allTitles.length && !loading" class="text-text-dim text-xs italic py-1">
          No titles loaded
        </div>

        <div v-else-if="filteredTitles.length === 0 && query" class="text-text-dim text-xs italic py-1">
          No titles found for "{{ query }}"
        </div>

        <ul ref="listRef" v-else class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(title, idx) in filteredTitles"
            :key="title.id"
            class="flex items-baseline gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.id === title.id, 'bg-surface-elevated': selectedIndex === idx && selected?.id !== title.id }"
            @click="selectTitle(title)">
            <span class="text-text-dim text-[0.72rem] min-w-12 shrink-0">#{{ title.id }}</span>
            <span
              class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap"
              :style="parseColorTag(title.title).color ? { color: parseColorTag(title.title).color! } : {}"
              :class="{ 'text-text-primary/75': !parseColorTag(title.title).color }">
              {{ parseColorTag(title.title).text || 'Untitled' }}
            </span>
            <span v-if="title.account_wide" class="text-[0.65rem] text-[#c0a0e0] shrink-0">acct</span>
          </li>
        </ul>
      </div>
      </template>
    </template>

    <!-- Right panel: title detail -->
    <div
      class="h-full overflow-y-auto border-l border-surface-elevated p-4 flex flex-col gap-4"
      :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select a title to inspect
        </div>

        <template v-else>
          <div class="flex gap-3 items-start">
            <div class="flex-1 min-w-0">
              <div
                class="text-base font-bold mb-1"
                :style="parseColorTag(selected.title).color ? { color: parseColorTag(selected.title).color! } : {}"
                :class="{ 'text-accent-gold': !parseColorTag(selected.title).color }">
                {{ parseColorTag(selected.title).text || 'Untitled' }}
              </div>
              <div class="text-xs text-text-dim mb-1">
                ID: <span class="text-text-secondary font-mono">{{ selected.id }}</span>
              </div>
              <div v-if="selected.tooltip" class="text-xs text-text-secondary italic">
                {{ selected.tooltip }}
              </div>
            </div>

            <button
              class="bg-transparent border-none cursor-pointer px-1 py-0 text-sm shrink-0 transition-colors"
              :class="isFav ? 'text-accent-gold' : 'text-text-dim hover:text-accent-gold'"
              :title="isFav ? 'Remove from favorites' : 'Add to favorites'"
              @click="dataBrowserStore.toggleFavorite({ type: 'title', reference: String(selected.id), label: parseColorTag(selected.title).text || 'Untitled' })"
            >&#x2605;</button>
            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Scope -->
          <div v-if="selected.account_wide || selected.soul_wide" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Scope</div>
            <div class="flex flex-wrap gap-1">
              <span v-if="selected.account_wide" class="text-[0.72rem] px-1.5 py-0.5 bg-[#2a1a3a] border border-[#4a2a5a] text-[#c0a0e0]">Account-Wide</span>
              <span v-if="selected.soul_wide" class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a2a3a] border border-[#2a4a5a] text-[#a0c0e0]">Soul-Wide</span>
            </div>
          </div>

          <!-- Keywords -->
          <div v-if="selected.keywords.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="kw in selected.keywords"
                :key="kw"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-entity-item">
                {{ kw }}
              </span>
            </div>
          </div>

          <!-- Raw JSON -->
          <div v-if="settingsStore.settings.showRawJsonInDataBrowser" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-[0.72rem] text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected, null, 2) }}</pre>
          </div>
        </template>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from "../Shared/PaneLayout.vue";
import { ref, computed, onMounted, watch } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useDataBrowserStore } from "../../stores/dataBrowserStore";
import { useKeyboard } from "../../composables/useKeyboard";
import type { PlayerTitleInfo } from "../../types/gameData";

/** Strip color tags and return { text, color } */
function parseColorTag(raw: string | null): { text: string; color: string | null } {
  if (!raw) return { text: "", color: null };
  const match = raw.match(/^<color=(#[0-9a-fA-F]{6})>(.*?)<\/color>$/s);
  if (match) return { text: match[2], color: match[1] };
  // Handle partial/malformed tags — strip any remaining tags
  const stripped = raw.replace(/<\/?color[^>]*>/g, "");
  return { text: stripped, color: null };
}

const store = useGameDataStore();
const settingsStore = useSettingsStore();
const dataBrowserStore = useDataBrowserStore();

const isFav = computed(() =>
  selected.value ? dataBrowserStore.isFavorite("title", String(selected.value.id)) : false
);

const query = ref("");
const allTitles = ref<PlayerTitleInfo[]>([]);
const filteredTitles = ref<PlayerTitleInfo[]>([]);
const selected = ref<PlayerTitleInfo | null>(null);
const loading = ref(false);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);

onMounted(async () => {
  if (store.status === "ready") {
    await loadAllTitles();
  }
});

watch(() => store.status, async (newStatus) => {
  if (newStatus === "ready") {
    await loadAllTitles();
  }
});

async function loadAllTitles() {
  loading.value = true;
  try {
    allTitles.value = await store.getAllPlayerTitles();
    filteredTitles.value = allTitles.value;
  } finally {
    loading.value = false;
  }
}

watch(query, (val) => {
  if (!val.trim()) {
    filteredTitles.value = allTitles.value;
    return;
  }
  const q = val.toLowerCase();
  filteredTitles.value = allTitles.value.filter(title =>
    parseColorTag(title.title).text.toLowerCase().includes(q) ||
    title.tooltip?.toLowerCase().includes(q)
  );
});

watch(filteredTitles, () => {
  selectedIndex.value = 0;
});

useKeyboard({
  listNavigation: {
    items: filteredTitles,
    selectedIndex,
    onConfirm: (index: number) => {
      const title = filteredTitles.value[index];
      if (title) selectTitle(title);
    },
    scrollContainerRef: listRef,
  },
});

function selectTitle(title: PlayerTitleInfo) {
  selected.value = title;
  dataBrowserStore.addToHistory({ type: "title", reference: String(title.id), label: parseColorTag(title.title).text || "Untitled" });
}

function clearSelection() {
  selected.value = null;
}
</script>
