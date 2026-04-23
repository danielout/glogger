<template>
  <PaneLayout screen-key="db-lorebooks" :left-pane="{ title: 'Lorebooks', defaultWidth: 340, minWidth: 260, maxWidth: 500 }">
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
            placeholder="Search lorebooks…"
            autofocus />
          <span v-if="loading" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="filteredBooks.length" class="text-text-dim text-xs min-w-6 text-right">{{
            filteredBooks.length
          }}</span>
        </div>

        <!-- Category filter -->
        <select v-model="selectedCategory" class="input text-xs">
          <option value="">All Categories</option>
          <option v-for="cat in categories" :key="cat.key" :value="cat.key">
            {{ cat.title || cat.key }}
          </option>
        </select>

        <div v-if="!allBooks.length && !loading" class="text-text-dim text-xs italic py-1">
          No lorebooks loaded
        </div>

        <div v-else-if="filteredBooks.length === 0 && (query || selectedCategory)" class="text-text-dim text-xs italic py-1">
          No lorebooks found
        </div>

        <ul ref="listRef" v-else class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(book, idx) in filteredBooks"
            :key="book.id"
            class="flex items-baseline gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.id === book.id, 'bg-surface-elevated': selectedIndex === idx && selected?.id !== book.id }"
            @click="selectBook(book)">
            <span class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap text-text-primary/75">
              {{ book.title || 'Untitled' }}
            </span>
            <span class="text-[10px] text-text-dim shrink-0">{{ book.category }}</span>
          </li>
        </ul>
      </div>
      </template>
    </template>

    <!-- Right panel: book detail / reader -->
    <div
      class="h-full overflow-y-auto border-l border-surface-elevated p-4 flex flex-col gap-4"
      :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select a book to read
        </div>

        <template v-else>
          <!-- Header -->
          <div class="flex gap-3 items-start">
            <div class="flex-1 min-w-0">
              <div class="text-base font-bold mb-1 text-accent-gold">
                {{ selected.title || 'Untitled' }}
              </div>
              <div class="flex flex-wrap gap-x-3 gap-y-0.5 text-xs text-text-dim">
                <span v-if="selected.category">
                  {{ categoryTitle(selected.category) }}
                </span>
                <span v-if="selected.location_hint" class="text-text-secondary">
                  {{ selected.location_hint }}
                </span>
              </div>
            </div>

            <button
              class="bg-transparent border-none cursor-pointer px-1 py-0 text-sm shrink-0 transition-colors"
              :class="isFav ? 'text-accent-gold' : 'text-text-dim hover:text-accent-gold'"
              :title="isFav ? 'Remove from favorites' : 'Add to favorites'"
              @click="dataBrowserStore.toggleFavorite({ type: 'lorebook', reference: String(selected.id), label: selected.title || 'Untitled' })"
            >&#x2605;</button>
            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Book text -->
          <div
            v-if="selected.text"
            class="book-text prose prose-invert max-w-none text-sm leading-relaxed"
            v-html="sanitizeBookHtml(selected.text)"
          />
          <div v-else class="text-text-dim italic text-sm">No text content available.</div>

          <!-- Keywords -->
          <div v-if="selected.keywords.length" class="flex flex-col gap-1.5">
            <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="kw in selected.keywords"
                :key="kw"
                class="text-xs px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-entity-item">
                {{ kw }}
              </span>
            </div>
          </div>

          <!-- Raw JSON -->
          <div v-if="settingsStore.settings.showRawJsonInDataBrowser" class="flex flex-col gap-1.5">
            <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-xs text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected, null, 2) }}</pre>
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
import type { LorebookEntry, LorebookCategoryInfo } from "../../types/gameData";

const store = useGameDataStore();
const settingsStore = useSettingsStore();
const dataBrowserStore = useDataBrowserStore();

const isFav = computed(() =>
  selected.value ? dataBrowserStore.isFavorite("lorebook", String(selected.value.id)) : false
);

const query = ref("");
const selectedCategory = ref("");
const allBooks = ref<LorebookEntry[]>([]);
const categories = ref<LorebookCategoryInfo[]>([]);
const filteredBooks = ref<LorebookEntry[]>([]);
const selected = ref<LorebookEntry | null>(null);
const loading = ref(false);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);

onMounted(async () => {
  if (store.status === "ready") {
    await loadAll();
  }
});

watch(() => store.status, async (newStatus) => {
  if (newStatus === "ready") {
    await loadAll();
  }
});

async function loadAll() {
  loading.value = true;
  try {
    const [books, cats] = await Promise.all([
      store.getAllLorebooks(),
      store.getLorebookCategories(),
    ]);
    allBooks.value = books;
    categories.value = cats;
    applyFilters();
  } finally {
    loading.value = false;
  }
}

function applyFilters() {
  let result = allBooks.value;

  if (selectedCategory.value) {
    result = result.filter(b => b.category === selectedCategory.value);
  }

  if (query.value.trim()) {
    const q = query.value.toLowerCase();
    result = result.filter(b =>
      (b.title?.toLowerCase().includes(q)) ||
      (b.location_hint?.toLowerCase().includes(q)) ||
      (b.text?.toLowerCase().includes(q))
    );
  }

  filteredBooks.value = result;
}

watch([query, selectedCategory], () => {
  applyFilters();
});

watch(filteredBooks, () => {
  selectedIndex.value = 0;
});

useKeyboard({
  listNavigation: {
    items: filteredBooks,
    selectedIndex,
    onConfirm: (index: number) => {
      const book = filteredBooks.value[index];
      if (book) selectBook(book);
    },
    scrollContainerRef: listRef,
  },
});

function selectBook(book: LorebookEntry) {
  selected.value = book;
  dataBrowserStore.addToHistory({ type: "lorebook", reference: String(book.id), label: book.title || "Untitled" });
}

function clearSelection() {
  selected.value = null;
}

function categoryTitle(key: string): string {
  const cat = categories.value.find(c => c.key === key);
  return cat?.title || key;
}

/** Sanitize book HTML and convert newlines to line breaks */
function sanitizeBookHtml(html: string): string {
  // The book text uses simple HTML tags: h1, b, i, br, p
  // Strip any tags we don't recognize for safety
  let result = html.replace(/<\/?(?!h1|h2|h3|b|i|em|strong|br|p|\/)[^>]*>/gi, '');
  // Convert literal \n to <br> for line breaks
  result = result.replace(/\n/g, '<br>');
  return result;
}
</script>

<style scoped>
.book-text :deep(h1) {
  font-size: 1.25rem;
  font-weight: bold;
  margin-bottom: 0.75rem;
  color: var(--color-accent-gold, #c8aa6e);
}

.book-text :deep(h2) {
  font-size: 1.1rem;
  font-weight: bold;
  margin-bottom: 0.5rem;
  color: var(--color-accent-gold, #c8aa6e);
}

.book-text :deep(b),
.book-text :deep(strong) {
  color: var(--color-text-secondary, #b0b0b0);
}

.book-text :deep(i),
.book-text :deep(em) {
  color: var(--color-text-dim, #808080);
}
</style>
