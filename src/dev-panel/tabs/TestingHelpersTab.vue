<template>
  <div class="space-y-6">
    <h3 class="m-0 text-text-primary">Testing Helpers</h3>
    <p class="text-text-muted text-xs">Tools for testing app behavior.</p>

    <!-- Toast triggers -->
    <section class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">Toast Notifications</h4>
      <p class="text-text-muted text-xs mb-3">
        Trigger toast notifications in the <strong>main window</strong> via Tauri events.
        These fire in the main app, not this dev panel.
      </p>
      <div class="flex flex-wrap gap-2">
        <button class="btn btn-secondary text-xs" @click="fireToast('success')">
          Success Toast
        </button>
        <button class="btn btn-secondary text-xs" @click="fireToast('info')">
          Info Toast
        </button>
        <button class="btn btn-secondary text-xs" @click="fireToast('warning')">
          Warning Toast
        </button>
        <button class="btn btn-secondary text-xs" @click="fireToast('error')">
          Error Toast
        </button>
      </div>

      <div class="mt-3">
        <label class="text-xs text-text-muted block mb-1">Custom message</label>
        <div class="flex gap-2">
          <input
            v-model="customMessage"
            type="text"
            placeholder="Custom toast message..."
            class="flex-1 bg-surface-elevated border border-border-default rounded px-3 py-1.5 text-sm text-text-primary"
          />
          <StyledSelect
            v-model="customType"
            :options="toastTypeOptions"
            size="sm"
          />
          <button class="btn btn-primary text-xs" @click="fireToast(customType, customMessage)">
            Send
          </button>
        </div>
      </div>
    </section>

    <!-- Local toast preview (in dev panel) -->
    <section class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">Local Toast Preview</h4>
      <p class="text-text-muted text-xs mb-3">
        Preview toasts inside this dev panel window.
      </p>
      <div class="flex flex-wrap gap-2">
        <button class="btn btn-secondary text-xs" @click="toast.success('Success toast preview')">
          Success
        </button>
        <button class="btn btn-secondary text-xs" @click="toast.info('Info toast preview')">
          Info
        </button>
        <button class="btn btn-secondary text-xs" @click="toast.warn('Warning toast preview')">
          Warning
        </button>
        <button class="btn btn-secondary text-xs" @click="toast.error('Error toast preview')">
          Error
        </button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { emit } from "@tauri-apps/api/event";
import { useToast } from "../../composables/useToast";
import StyledSelect from "../../components/Shared/StyledSelect.vue";

const toast = useToast();

const customMessage = ref("");
const customType = ref("info");

const toastTypeOptions = [
  { value: "success", label: "Success" },
  { value: "info", label: "Info" },
  { value: "warning", label: "Warning" },
  { value: "error", label: "Error" },
];

function fireToast(type: string, message?: string) {
  const msg = message || `Test ${type} toast from dev panel`;
  emit("dev-toast", { type, message: msg });
}
</script>
