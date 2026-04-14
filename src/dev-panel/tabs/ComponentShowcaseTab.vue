<template>
  <div class="space-y-6">
    <h3 class="m-0 text-text-primary">Component Showcase</h3>
    <p class="text-text-muted text-xs">Interactive preview of shared UI components.</p>

    <!-- AccordionSection -->
    <section class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">AccordionSection</h4>
      <AccordionSection :default-open="true">
        <template #title>Example Accordion</template>
        <template #badge>
          <span class="text-text-muted text-xs font-mono">3 items</span>
        </template>
        <div class="text-xs text-text-secondary">
          This is the accordion content. It supports a title slot, badge slot, and default content slot.
        </div>
      </AccordionSection>
      <div class="mt-2">
        <AccordionSection :default-open="false">
          <template #title>Collapsed by Default</template>
          <div class="text-xs text-text-secondary">
            This one starts closed via <code class="text-accent-gold">:default-open="false"</code>.
          </div>
        </AccordionSection>
      </div>
    </section>

    <!-- StyledSelect -->
    <section class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">StyledSelect</h4>
      <div class="flex flex-wrap gap-4 items-end">
        <div>
          <label class="text-xs text-text-muted block mb-1">Default</label>
          <StyledSelect
            v-model="selectValue"
            :options="selectOptions"
          />
        </div>
        <div>
          <label class="text-xs text-text-muted block mb-1">Small</label>
          <StyledSelect
            v-model="selectValue"
            :options="selectOptions"
            size="sm"
          />
        </div>
        <div>
          <label class="text-xs text-text-muted block mb-1">Full width</label>
          <StyledSelect
            v-model="selectValue"
            :options="selectOptions"
            :full-width="true"
            class="w-48"
          />
        </div>
      </div>
      <p class="text-xs text-text-muted mt-2">Selected: <code class="text-accent-gold">{{ selectValue }}</code></p>
    </section>

    <!-- TabBar -->
    <section class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">TabBar</h4>
      <TabBar
        v-model="tabValue"
        :tabs="demotabs"
      />
      <div class="mt-2 p-2 bg-surface-elevated rounded text-xs text-text-secondary">
        Active tab: <code class="text-accent-gold">{{ tabValue }}</code>
      </div>
    </section>

    <!-- EmptyState -->
    <section class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">EmptyState</h4>
      <div class="flex gap-4">
        <div class="flex-1 border border-border-default rounded h-32">
          <EmptyState primary="No data found" secondary="Try adjusting your filters" variant="panel" />
        </div>
        <div class="flex-1 border border-border-default rounded p-2">
          <EmptyState primary="Nothing here" variant="compact" />
        </div>
      </div>
    </section>

    <!-- NumberInput -->
    <section class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">NumberInput</h4>
      <div class="flex flex-wrap gap-6 items-end">
        <div>
          <label class="text-xs text-text-muted block mb-1">Small</label>
          <NumberInput v-model="numberSm" :min="0" :max="99" size="sm" />
        </div>
        <div>
          <label class="text-xs text-text-muted block mb-1">Medium (default)</label>
          <NumberInput v-model="numberMd" :min="0" :max="999" />
        </div>
        <div>
          <label class="text-xs text-text-muted block mb-1">Large</label>
          <NumberInput v-model="numberLg" :min="0" :max="9999" size="lg" />
        </div>
        <div>
          <label class="text-xs text-text-muted block mb-1">Step = 5</label>
          <NumberInput v-model="numberStep" :min="0" :max="100" :step="5" />
        </div>
        <div>
          <label class="text-xs text-text-muted block mb-1">Disabled</label>
          <NumberInput v-model="numberDisabled" disabled />
        </div>
      </div>
      <p class="text-xs text-text-muted mt-2">
        Values: sm=<code class="text-accent-gold">{{ numberSm }}</code>
        md=<code class="text-accent-gold">{{ numberMd }}</code>
        lg=<code class="text-accent-gold">{{ numberLg }}</code>
        step=<code class="text-accent-gold">{{ numberStep }}</code>
      </p>
    </section>

    <!-- ModalDialog -->
    <section class="border border-border-default rounded p-4">
      <h4 class="text-text-secondary text-sm mb-3 mt-0">ModalDialog</h4>
      <div class="flex gap-2">
        <button class="btn btn-secondary text-xs" @click="showPromptModal = true">
          Prompt Modal
        </button>
        <button class="btn btn-secondary text-xs" @click="showConfirmModal = true">
          Confirm Modal
        </button>
        <button class="btn btn-secondary text-xs" @click="showDangerModal = true">
          Danger Modal
        </button>
      </div>
      <p v-if="lastModalResult" class="text-xs text-text-muted mt-2">
        Last result: <code class="text-accent-gold">{{ lastModalResult }}</code>
      </p>

      <ModalDialog
        v-model:show="showPromptModal"
        title="Enter a Value"
        type="prompt"
        placeholder="Type something..."
        @confirm="lastModalResult = `prompt: ${$event}`"
        @cancel="lastModalResult = 'cancelled'"
      />
      <ModalDialog
        v-model:show="showConfirmModal"
        title="Are you sure?"
        type="confirm"
        message="This is a confirmation dialog. Do you want to proceed?"
        confirm-label="Proceed"
        @confirm="lastModalResult = 'confirmed'"
        @cancel="lastModalResult = 'cancelled'"
      />
      <ModalDialog
        v-model:show="showDangerModal"
        title="Delete Everything?"
        type="confirm"
        message="This action cannot be undone."
        confirm-label="Delete"
        :danger="true"
        @confirm="lastModalResult = 'danger confirmed'"
        @cancel="lastModalResult = 'cancelled'"
      />
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import AccordionSection from "../../components/Shared/AccordionSection.vue";
import StyledSelect from "../../components/Shared/StyledSelect.vue";
import TabBar from "../../components/Shared/TabBar.vue";
import EmptyState from "../../components/Shared/EmptyState.vue";
import ModalDialog from "../../components/Shared/ModalDialog.vue";
import NumberInput from "../../components/Shared/NumberInput.vue";

const selectValue = ref("option1");
const selectOptions = [
  { value: "option1", label: "First Option" },
  { value: "option2", label: "Second Option" },
  { value: "option3", label: "Third Option" },
  { value: "option4", label: "Fourth Option" },
];

const tabValue = ref("tab1");
const demotabs = [
  { id: "tab1", label: "First" },
  { id: "tab2", label: "Second" },
  { id: "tab3", label: "Third" },
];

const numberSm = ref(5);
const numberMd = ref(42);
const numberLg = ref(100);
const numberStep = ref(25);
const numberDisabled = ref(10);

const showPromptModal = ref(false);
const showConfirmModal = ref(false);
const showDangerModal = ref(false);
const lastModalResult = ref("");
</script>
