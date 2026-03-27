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
        :class="[borderClass, interactive ? '' : 'pointer-events-none']"
        :style="tooltipStyle"
        @mouseenter="onTooltipMouseEnter"
        @mouseleave="onTooltipMouseLeave"
      >
        <slot name="tooltip" />
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, type CSSProperties } from "vue";
import { useTooltip } from "../../composables/useTooltip";

const props = defineProps<{
  delay?: number;
  disabled?: boolean;
  borderClass?: string;
  interactive?: boolean;
}>();

const emit = defineEmits<{
  hover: [];
}>();

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
  interactive: props.interactive,
  onHover: () => emit("hover"),
});

function onMouseEnter() {
  if (anchorEl.value) {
    anchorRect.value = anchorEl.value.getBoundingClientRect();
  }
  baseMouseEnter();
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
