<template>
  <div
    ref="anchorEl"
    class="relative inline-flex items-center"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <slot />
    <Teleport to="body">
      <div
        v-if="showTooltip && !disabled"
        class="fixed z-[9999] min-w-62 max-w-87 bg-[#1a1a2e] border rounded-md p-3 shadow-lg"
        :class="[borderClass, isInteractive ? '' : 'pointer-events-none']"
        :style="tooltipStyle"
        @mouseenter="onTooltipMouseEnter"
        @mouseleave="onTooltipMouseLeave"
      >
        <!-- Pin button -->
        <button
          v-if="entityType && entityReference"
          class="absolute top-1.5 right-1.5 p-0.5 rounded transition-colors z-10"
          :class="pinned
            ? 'text-accent-blue hover:text-accent-blue-bright'
            : 'text-text-muted hover:text-text-default'"
          :title="pinned ? 'Unpin from shelf' : 'Pin to shelf'"
          @click.stop="togglePin"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" :fill="pinned ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
            <path d="M12 2L12 12M9 4L12 2L15 4" />
            <path d="M5 12H19" />
            <path d="M12 12V22" />
          </svg>
        </button>
        <slot name="tooltip" />
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, type CSSProperties } from "vue";
import { useTooltip } from "../../composables/useTooltip";
import { useReferenceShelfStore } from "../../stores/referenceShelfStore";
import type { EntityType } from "../../composables/useEntityNavigation";

const props = defineProps<{
  delay?: number;
  disabled?: boolean;
  borderClass?: string;
  interactive?: boolean;
  /** Entity type for pin support — omit to hide pin button */
  entityType?: EntityType;
  /** Entity reference string for pin support */
  entityReference?: string;
  /** Display label for pin (resolved name) — falls back to entityReference */
  entityLabel?: string;
}>();

const emit = defineEmits<{
  hover: [];
}>();

const shelf = useReferenceShelfStore();

const pinned = computed(() => {
  if (!props.entityType || !props.entityReference) return false;
  return shelf.isPinned(props.entityType, props.entityReference);
});

// Make tooltip interactive when pin button is present, or when explicitly requested
const isInteractive = computed(() => props.interactive || !!(props.entityType && props.entityReference));

const anchorEl = ref<HTMLElement | null>(null);
const anchorRect = ref<DOMRect | null>(null);

const {
  showTooltip,
  onMouseEnter: baseMouseEnter,
  onMouseLeave,
  onTooltipMouseEnter,
  onTooltipMouseLeave,
} = useTooltip({
  delay: props.delay,
  interactive: isInteractive.value,
  onHover: () => emit("hover"),
});

function onMouseEnter() {
  if (anchorEl.value) {
    anchorRect.value = anchorEl.value.getBoundingClientRect();
  }
  baseMouseEnter();
}

function togglePin() {
  if (!props.entityType || !props.entityReference) return;
  shelf.togglePin({
    type: props.entityType,
    reference: props.entityReference,
    label: props.entityLabel ?? props.entityReference,
  });
}

const tooltipStyle = computed<CSSProperties>(() => {
  if (!anchorRect.value) return {};
  const rect = anchorRect.value;
  return {
    top: `${rect.bottom + 8}px`,
    left: `${rect.left}px`,
  };
});
</script>
