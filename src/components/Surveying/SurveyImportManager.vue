<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center">
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/50" @click="emit('close')" />

      <!-- Dialog -->
      <div class="relative bg-surface-base border border-border-default rounded-lg shadow-xl w-[600px] max-w-[90vw] max-h-[80vh] flex flex-col">
        <!-- Header -->
        <div class="px-4 pt-4 pb-2 flex justify-between items-center">
          <h3 class="text-sm font-semibold text-text-primary">Manage Imported Data</h3>
          <button @click="emit('close')"
            class="text-text-muted hover:text-text-primary transition-colors text-lg leading-none px-1">
            &times;
          </button>
        </div>

        <!-- Content -->
        <div class="px-4 pb-4 overflow-y-auto flex-1">
          <div v-if="imports.length === 0" class="text-text-dim text-sm italic py-4 text-center">
            No imported data sets.
          </div>
          <div v-else class="flex flex-col gap-2">
            <div
              v-for="imp in imports"
              :key="imp.id"
              class="bg-surface-elevated border border-border-default rounded p-3 flex items-center justify-between gap-3"
            >
              <div class="flex-1 min-w-0">
                <!-- Editable label -->
                <div v-if="editing === imp.id" class="flex items-center gap-2">
                  <input
                    ref="editInputRef"
                    v-model="editValue"
                    type="text"
                    class="bg-surface-base border border-border-default rounded px-2 py-1 text-sm text-text-primary flex-1 focus:border-accent-gold/50 focus:outline-none"
                    @keydown.enter="saveRename(imp)"
                    @keydown.escape="editing = null"
                  />
                  <button @click="saveRename(imp)"
                    class="px-2 py-1 text-xs rounded bg-accent-gold/20 border border-accent-gold/30 text-accent-gold hover:bg-accent-gold/30 transition-all">
                    Save
                  </button>
                  <button @click="editing = null"
                    class="px-2 py-1 text-xs rounded bg-surface-elevated border border-border-default text-text-muted hover:text-text-secondary transition-all">
                    Cancel
                  </button>
                </div>
                <div v-else class="flex items-center gap-2">
                  <span class="text-sm font-medium text-text-primary truncate">{{ imp.label }}</span>
                  <button @click="startRename(imp)"
                    class="text-text-muted hover:text-text-secondary transition-colors text-xs shrink-0"
                    title="Rename">
                    Rename
                  </button>
                </div>
                <div class="text-[0.65rem] text-text-muted mt-0.5">
                  <span v-if="imp.source_player">From: {{ imp.source_player }} &middot; </span>
                  {{ imp.session_count }} sessions &middot;
                  {{ imp.event_count }} events &middot;
                  Imported {{ imp.imported_at }}
                </div>
              </div>
              <button
                @click="handleDelete(imp)"
                :disabled="deleting === imp.id"
                class="px-2.5 py-1 text-xs rounded bg-red-900/20 border border-red-700/30 text-red-400 hover:bg-red-900/40 transition-all shrink-0"
              >
                {{ deleting === imp.id ? "Removing..." : "Remove" }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../../composables/useToast";

interface SurveyImportInfo {
  id: number;
  label: string;
  source_player: string | null;
  session_count: number;
  event_count: number;
  imported_at: string;
}

const emit = defineEmits<{
  close: [];
  deleted: [];
}>();

const toast = useToast();
const imports = ref<SurveyImportInfo[]>([]);
const deleting = ref<number | null>(null);
const editing = ref<number | null>(null);
const editValue = ref("");
const editInputRef = ref<HTMLInputElement[]>();

onMounted(async () => {
  await loadImports();
});

async function loadImports() {
  try {
    imports.value = await invoke<SurveyImportInfo[]>("get_survey_imports");
  } catch (e) {
    toast.error(`Failed to load imports: ${e}`);
  }
}

async function startRename(imp: SurveyImportInfo) {
  editing.value = imp.id;
  editValue.value = imp.label;
  await nextTick();
  editInputRef.value?.[0]?.focus();
  editInputRef.value?.[0]?.select();
}

async function saveRename(imp: SurveyImportInfo) {
  const newLabel = editValue.value.trim();
  if (!newLabel || newLabel === imp.label) {
    editing.value = null;
    return;
  }
  try {
    await invoke("rename_survey_import", { importId: imp.id, label: newLabel });
    imp.label = newLabel;
    editing.value = null;
  } catch (e) {
    toast.error(`Failed to rename: ${e}`);
  }
}

async function handleDelete(imp: SurveyImportInfo) {
  if (!confirm(`Remove "${imp.label}"? This will delete all ${imp.session_count} imported sessions and their data.`)) {
    return;
  }

  deleting.value = imp.id;
  try {
    await invoke("delete_survey_import", { importId: imp.id });
    toast.success(`Removed "${imp.label}"`);
    await loadImports();
    emit("deleted");
    if (imports.value.length === 0) {
      emit("close");
    }
  } catch (e) {
    toast.error(`Failed to remove import: ${e}`);
  } finally {
    deleting.value = null;
  }
}
</script>
