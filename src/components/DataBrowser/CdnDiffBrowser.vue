<template>
  <PaneLayout
    screen-key="db-cdn-diff"
    :left-pane="{ title: 'Files', defaultWidth: 240, minWidth: 180, maxWidth: 360 }"
    :right-pane="{ title: 'Entries', defaultWidth: 280, minWidth: 200, maxWidth: 420 }"
  >
    <!-- Left pane: file list -->
    <template #left>
      <div class="flex flex-col h-full">
        <!-- Load button / status -->
        <div class="p-3 border-b border-border-default">
          <button
            v-if="!summaryLoaded && !summaryLoading"
            class="btn btn-primary w-full text-xs"
            @click="loadSummary"
          >
            Compare with Previous Version
          </button>
          <div v-else-if="summaryLoading" class="text-xs text-accent-gold flex items-center gap-2">
            <span class="animate-spin">&#x27F3;</span>
            Downloading &amp; comparing...
          </div>
          <div v-else-if="summaryError" class="text-xs text-accent-red">
            {{ summaryError }}
            <button class="btn btn-secondary text-xs mt-2 w-full" @click="loadSummary">Retry</button>
          </div>
          <div v-else class="text-xs text-text-muted">
            v{{ currentVersion - 1 }} &rarr; v{{ currentVersion }}
          </div>
        </div>

        <!-- File list -->
        <div v-if="summaryLoaded" class="flex-1 overflow-y-auto">
          <template v-for="(file, idx) in sortedSummary" :key="file.file_name">
            <!-- Section divider between data files and translation files -->
            <div
              v-if="isTranslationFile(file.file_name) && idx > 0 && !isTranslationFile(sortedSummary[idx - 1].file_name)"
              class="px-3 py-1.5 text-[0.6rem] uppercase tracking-wider text-text-dim bg-surface-dark/40 border-b border-border-default"
            >
              Translation Strings
            </div>
            <button
              class="w-full text-left bg-transparent border-none px-3 py-2 cursor-pointer text-xs font-mono transition-colors hover:bg-surface-elevated border-b border-border-default"
              :class="{
                'bg-surface-elevated! text-text-primary': selectedFile === file.file_name,
                'text-text-secondary': selectedFile !== file.file_name,
                'opacity-40': !fileHasChanges(file),
              }"
              @click="selectFile(file.file_name)"
            >
              <div class="flex items-center justify-between">
                <span>{{ file.file_name }}</span>
                <div v-if="fileHasChanges(file)" class="flex gap-1.5">
                  <span v-if="file.added_count" class="text-accent-green">+{{ file.added_count }}</span>
                  <span v-if="file.removed_count" class="text-accent-red">-{{ file.removed_count }}</span>
                  <span v-if="file.changed_count" class="text-accent-gold">~{{ file.changed_count }}</span>
                </div>
                <span v-else class="text-text-dim">--</span>
              </div>
            </button>
          </template>
        </div>
      </div>
    </template>

    <!-- Center: diff detail for selected entry -->
    <div class="h-full overflow-y-auto">
      <template v-if="!selectedFile">
        <EmptyState primary="Select a file" secondary="Choose a data file from the left to see its changes." variant="panel" />
      </template>

      <template v-else-if="!selectedEntry">
        <EmptyState primary="Select an entry" secondary="Pick an entry from the right panel to see its diff." variant="panel" />
      </template>

      <template v-else>
        <div class="p-4">
          <h3 class="text-sm font-mono text-text-primary mb-1">{{ selectedEntry.key }}</h3>
          <div v-if="selectedEntry.label" class="text-xs text-text-muted mb-4">{{ selectedEntry.label }}</div>

          <!-- Added entry: show all fields -->
          <template v-if="activeSection === 'added' && selectedEntry.data">
            <div class="text-xs text-accent-green font-semibold mb-3">Added Entry</div>
            <JsonBlock :value="selectedEntry.data" />
          </template>

          <!-- Removed entry: show all fields -->
          <template v-else-if="activeSection === 'removed' && selectedEntry.old_data">
            <div class="text-xs text-accent-red font-semibold mb-3">Removed Entry</div>
            <JsonBlock :value="selectedEntry.old_data" />
          </template>

          <!-- Changed entry: show field diffs -->
          <template v-else-if="activeSection === 'changed'">
            <div class="text-xs text-accent-gold font-semibold mb-3">
              {{ selectedEntry.field_changes.length }} changed field{{ selectedEntry.field_changes.length === 1 ? '' : 's' }}
            </div>

            <div
              v-for="change in selectedEntry.field_changes"
              :key="change.field"
              class="mb-4 border border-border-default rounded overflow-hidden"
            >
              <div class="px-3 py-1.5 bg-surface-elevated text-xs font-mono text-text-primary border-b border-border-default">
                {{ change.field }}
              </div>
              <div class="grid grid-cols-2 divide-x divide-border-default">
                <div class="p-3">
                  <div class="text-[0.6rem] text-text-dim mb-1.5 uppercase tracking-wider">Old</div>
                  <JsonBlock :value="change.old_value" compact />
                </div>
                <div class="p-3">
                  <div class="text-[0.6rem] text-text-dim mb-1.5 uppercase tracking-wider">New</div>
                  <JsonBlock :value="change.new_value" compact />
                </div>
              </div>
            </div>
          </template>
        </div>
      </template>
    </div>

    <!-- Right pane: entry list for selected file -->
    <template #right>
      <div class="h-full flex flex-col">
        <template v-if="!selectedFile">
          <EmptyState primary="No file selected" secondary="Pick a file from the left." variant="compact" />
        </template>

        <template v-else-if="fileLoading">
          <div class="p-3">
            <SkeletonLoader variant="text" :lines="4" />
          </div>
        </template>

        <template v-else-if="fileError">
          <div class="p-3 text-xs text-accent-red">{{ fileError }}</div>
        </template>

        <template v-else-if="fileDiff">
          <!-- Section tabs -->
          <div class="shrink-0 flex gap-1 px-3 py-2 border-b border-border-default bg-surface-dark/30">
            <button
              v-for="section in diffSections"
              :key="section.id"
              class="px-2 py-1 bg-transparent border-none text-xs font-mono rounded cursor-pointer transition-colors hover:bg-surface-elevated"
              :class="{
                'text-accent-gold! bg-surface-elevated!': activeSection === section.id,
                'text-text-secondary': activeSection !== section.id,
                'opacity-40 cursor-default!': section.count === 0,
              }"
              :disabled="section.count === 0"
              @click="activeSection = section.id"
            >
              {{ section.label }} ({{ section.count }})
            </button>
          </div>

          <!-- Entry list -->
          <div class="flex-1 overflow-y-auto">
            <div v-if="activeEntries.length === 0" class="p-3 text-xs text-text-dim">
              No {{ activeSection }} entries.
            </div>
            <button
              v-for="entry in activeEntries"
              :key="entry.key"
              class="w-full text-left bg-transparent border-none px-3 py-2 cursor-pointer text-xs transition-colors hover:bg-surface-elevated border-b border-border-default"
              :class="{
                'bg-surface-elevated! text-text-primary': selectedEntry?.key === entry.key,
                'text-text-secondary': selectedEntry?.key !== entry.key,
              }"
              @click="selectedEntry = entry"
            >
              <div class="font-mono text-text-primary">{{ entry.label || entry.key }}</div>
              <div v-if="entry.label" class="text-text-dim mt-0.5">{{ entry.key }}</div>
              <div v-if="activeSection === 'changed' && entry.field_changes.length" class="text-text-dim mt-0.5">
                {{ entry.field_changes.length }} field{{ entry.field_changes.length === 1 ? '' : 's' }}
              </div>
            </button>
          </div>
        </template>
      </div>
    </template>
  </PaneLayout>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import PaneLayout from "../Shared/PaneLayout.vue";
import EmptyState from "../Shared/EmptyState.vue";
import JsonBlock from "./JsonBlock.vue";
import SkeletonLoader from "../Shared/SkeletonLoader.vue";

// ── Types ────────────────────────────────────────────────────────────────────

interface FileDiffSummary {
  file_name: string;
  added_count: number;
  removed_count: number;
  changed_count: number;
}

interface FieldChange {
  field: string;
  old_value: unknown;
  new_value: unknown;
}

interface EntryDiff {
  key: string;
  label: string | null;
  field_changes: FieldChange[];
  data: unknown | null;
  old_data: unknown | null;
}

interface FileDiff {
  file_name: string;
  added: EntryDiff[];
  removed: EntryDiff[];
  changed: EntryDiff[];
}

// ── Summary state ────────────────────────────────────────────────────────────

const summaryLoading = ref(false);
const summaryLoaded = ref(false);
const summaryError = ref<string | null>(null);
const summary = ref<FileDiffSummary[]>([]);
const currentVersion = ref(0);

const TRANSLATION_FILE_NAMES = new Set([
  "strings_abilities",
  "strings_ai",
  "strings_areas",
  "strings_attributes",
  "strings_directedgoals",
  "strings_effects",
  "strings_items",
  "strings_lorebookinfo",
  "strings_lorebooks",
  "strings_npcs",
  "strings_playertitles",
  "strings_quests",
  "strings_recipes",
  "strings_requested",
  "strings_skills",
  "strings_storagevaults",
  "strings_tsysclientinfo",
  "strings_ui",
]);

function isTranslationFile(name: string): boolean {
  return TRANSLATION_FILE_NAMES.has(name);
}

/** Data files first (sorted by change count), then translation files (same sort). */
const sortedSummary = computed(() => {
  const byChanges = (a: FileDiffSummary, b: FileDiffSummary) => {
    const aChanges = a.added_count + a.removed_count + a.changed_count;
    const bChanges = b.added_count + b.removed_count + b.changed_count;
    if (aChanges !== bChanges) return bChanges - aChanges;
    return a.file_name.localeCompare(b.file_name);
  };
  const data = summary.value.filter((f) => !isTranslationFile(f.file_name));
  const translation = summary.value.filter((f) => isTranslationFile(f.file_name));
  return [...data.sort(byChanges), ...translation.sort(byChanges)];
});

function fileHasChanges(file: FileDiffSummary): boolean {
  return file.added_count + file.removed_count + file.changed_count > 0;
}

async function loadSummary() {
  summaryLoading.value = true;
  summaryError.value = null;
  try {
    const status = await invoke<{ cached_version: number | null }>("get_cache_status");
    currentVersion.value = status.cached_version ?? 0;

    summary.value = await invoke<FileDiffSummary[]>("cdn_diff_summary");
    summaryLoaded.value = true;
  } catch (e: any) {
    summaryError.value = e.toString();
  } finally {
    summaryLoading.value = false;
  }
}

// ── File diff state ──────────────────────────────────────────────────────────

const selectedFile = ref<string | null>(null);
const fileLoading = ref(false);
const fileError = ref<string | null>(null);
const fileDiff = ref<FileDiff | null>(null);
const activeSection = ref<"added" | "removed" | "changed">("changed");
const selectedEntry = ref<EntryDiff | null>(null);

const diffSections = computed(() => {
  if (!fileDiff.value) return [];
  return [
    { id: "added" as const, label: "Added", count: fileDiff.value.added.length },
    { id: "removed" as const, label: "Removed", count: fileDiff.value.removed.length },
    { id: "changed" as const, label: "Changed", count: fileDiff.value.changed.length },
  ];
});

const activeEntries = computed(() => {
  if (!fileDiff.value) return [];
  return fileDiff.value[activeSection.value] ?? [];
});

async function selectFile(fileName: string) {
  if (selectedFile.value === fileName) return;
  selectedFile.value = fileName;
  selectedEntry.value = null;
  fileDiff.value = null;
  fileError.value = null;
  fileLoading.value = true;

  try {
    fileDiff.value = await invoke<FileDiff>("cdn_diff_file", { fileName });
    // Auto-select the first non-empty section
    if (fileDiff.value.changed.length) activeSection.value = "changed";
    else if (fileDiff.value.added.length) activeSection.value = "added";
    else if (fileDiff.value.removed.length) activeSection.value = "removed";
  } catch (e: any) {
    fileError.value = e.toString();
  } finally {
    fileLoading.value = false;
  }
}
</script>
