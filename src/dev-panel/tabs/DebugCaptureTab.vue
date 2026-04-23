<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h3 class="m-0 text-text-primary">Debug Capture</h3>
      <span
        v-if="status?.active"
        class="text-xs px-2 py-0.5 rounded bg-red-900/40 text-red-400 animate-pulse">
        RECORDING
      </span>
      <span
        v-else-if="status?.pending_save"
        class="text-xs px-2 py-0.5 rounded bg-yellow-900/40 text-yellow-400">
        READY TO SAVE
      </span>
    </div>
    <p class="text-text-muted text-xs">
      Capture raw Player.log and Chat.log lines along with game state snapshots
      for debugging. Lines are streamed to a temp file so long captures are safe.
    </p>

    <!-- Controls -->
    <section class="border border-border-default rounded p-4 space-y-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">Capture Controls</h4>

      <!-- Idle state: start button -->
      <div v-if="!status?.active && !status?.pending_save" class="space-y-3">
        <button
          class="btn btn-primary text-xs"
          :disabled="starting"
          @click="startCapture">
          {{ starting ? 'Starting...' : 'Start Capture' }}
        </button>
        <p class="text-text-muted text-xs">
          Starts buffering all raw log lines to disk and takes a game state snapshot.
        </p>
      </div>

      <!-- Recording state: live stats + stop button -->
      <div v-else-if="status?.active" class="space-y-3">
        <div class="grid grid-cols-3 gap-3 text-xs">
          <div class="bg-surface-elevated rounded p-2">
            <div class="text-text-muted mb-1">Total Lines</div>
            <div class="text-text-primary text-sm">{{ (status.player_line_count + status.chat_line_count).toLocaleString() }}</div>
          </div>
          <div class="bg-surface-elevated rounded p-2">
            <div class="text-text-muted mb-1">Player.log</div>
            <div class="text-accent-gold text-sm">{{ status.player_line_count.toLocaleString() }}</div>
          </div>
          <div class="bg-surface-elevated rounded p-2">
            <div class="text-text-muted mb-1">Chat.log</div>
            <div class="text-accent-gold text-sm">{{ status.chat_line_count.toLocaleString() }}</div>
          </div>
        </div>

        <div class="text-text-muted text-xs">
          Started: <span class="text-text-secondary">{{ formatStartTime }}</span>
        </div>

        <div class="flex gap-2">
          <button
            class="btn btn-primary text-xs"
            :disabled="stopping"
            @click="stopCapture">
            {{ stopping ? 'Stopping...' : 'Stop Recording' }}
          </button>
          <button
            class="btn btn-secondary text-xs"
            :disabled="stopping"
            @click="discardCapture">
            Discard
          </button>
        </div>
      </div>

      <!-- Pending save state: review, edit notes, choose save mode -->
      <div v-else-if="status?.pending_save" class="space-y-3">
        <div class="grid grid-cols-3 gap-3 text-xs">
          <div class="bg-surface-elevated rounded p-2">
            <div class="text-text-muted mb-1">Total Lines</div>
            <div class="text-text-primary text-sm">{{ status.line_count.toLocaleString() }}</div>
          </div>
          <div class="bg-surface-elevated rounded p-2">
            <div class="text-text-muted mb-1">Player.log</div>
            <div class="text-accent-gold text-sm">{{ status.player_line_count.toLocaleString() }}</div>
          </div>
          <div class="bg-surface-elevated rounded p-2">
            <div class="text-text-muted mb-1">Chat.log</div>
            <div class="text-accent-gold text-sm">{{ status.chat_line_count.toLocaleString() }}</div>
          </div>
        </div>

        <div>
          <label class="text-xs text-text-muted block mb-1">Notes (what were you doing?)</label>
          <textarea
            v-model="notes"
            rows="3"
            placeholder="Describe what you were doing, what bug you're trying to capture, etc."
            class="w-full bg-surface-elevated border border-border-default rounded px-3 py-2 text-sm text-text-primary resize-y"
          />
        </div>

        <div class="flex gap-2 flex-wrap">
          <button
            class="btn btn-primary text-xs"
            :disabled="saving"
            @click="saveCapture('normal')">
            {{ saving ? 'Saving...' : 'Save (Normal)' }}
          </button>
          <button
            class="btn btn-secondary text-xs"
            :disabled="saving"
            @click="saveCapture('full')">
            Save (Full)
          </button>
          <button
            class="btn btn-secondary text-xs"
            :disabled="saving"
            @click="discardCapture">
            Discard
          </button>
        </div>
        <p class="text-text-muted text-xs mt-1">
          <strong>Normal</strong> filters engine noise (asset loading, animations, sounds) for easier analysis.
          <strong>Full</strong> keeps every raw line.
        </p>
      </div>
    </section>

    <!-- Last capture result -->
    <section v-if="lastResult" class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">Last Capture</h4>
      <div class="text-xs space-y-1">
        <div v-if="lastResult.success" class="text-green-400">
          Saved {{ lastResult.lineCount?.toLocaleString() }} lines to: {{ lastResult.path }}
        </div>
        <div v-else class="text-red-400">
          {{ lastResult.error }}
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

interface CaptureStatus {
  active: boolean;
  pending_save: boolean;
  started_at: string | null;
  line_count: number;
  player_line_count: number;
  chat_line_count: number;
}

interface CaptureResult {
  line_count: number;
  player_line_count: number;
  chat_line_count: number;
}

const status = ref<CaptureStatus | null>(null);
const notes = ref("");
const starting = ref(false);
const stopping = ref(false);
const saving = ref(false);
const lastResult = ref<{
  success: boolean;
  path?: string;
  lineCount?: number;
  error?: string;
} | null>(null);

let pollInterval: ReturnType<typeof setInterval> | null = null;

const formatStartTime = computed(() => {
  if (!status.value?.started_at) return "\u2014";
  try {
    return new Date(status.value.started_at).toLocaleTimeString();
  } catch {
    return status.value.started_at;
  }
});

async function refreshStatus() {
  try {
    status.value = await invoke<CaptureStatus>("debug_capture_status");
  } catch (e) {
    console.error("Failed to get capture status:", e);
  }
}

async function startCapture() {
  starting.value = true;
  try {
    await invoke("debug_capture_start");
    notes.value = "";
    lastResult.value = null;
    await refreshStatus();
  } catch (e) {
    console.error("Failed to start capture:", e);
    lastResult.value = { success: false, error: String(e) };
  } finally {
    starting.value = false;
  }
}

async function stopCapture() {
  stopping.value = true;
  try {
    await invoke<CaptureResult>("debug_capture_stop");
    await refreshStatus();
  } catch (e) {
    console.error("Failed to stop capture:", e);
    lastResult.value = { success: false, error: String(e) };
  } finally {
    stopping.value = false;
  }
}

async function saveCapture(filterMode: "normal" | "full") {
  saving.value = true;
  try {
    const filePath = await save({
      filters: [{ name: "JSON", extensions: ["json"] }],
      defaultPath: `glogger-capture-${new Date().toISOString().slice(0, 19).replace(/:/g, "-")}.json`,
    });

    if (!filePath) {
      saving.value = false;
      return;
    }

    const result = await invoke<CaptureResult>("debug_capture_save", {
      notes: notes.value,
      filterMode,
      outputPath: filePath,
    });

    lastResult.value = {
      success: true,
      path: filePath,
      lineCount: result.line_count,
    };
    await refreshStatus();
  } catch (e) {
    console.error("Failed to save capture:", e);
    lastResult.value = { success: false, error: String(e) };
    await refreshStatus();
  } finally {
    saving.value = false;
  }
}

async function discardCapture() {
  try {
    await invoke("debug_capture_discard");
    lastResult.value = null;
    await refreshStatus();
  } catch (e) {
    console.error("Failed to discard capture:", e);
  }
}

onMounted(() => {
  refreshStatus();
  pollInterval = setInterval(refreshStatus, 2000);
});

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
});
</script>
