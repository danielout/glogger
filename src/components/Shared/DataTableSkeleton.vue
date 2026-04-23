<template>
  <div class="w-full animate-pulse">
    <!-- Header row -->
    <div v-if="showHeader" class="flex gap-4 border-b border-border-default pb-2 mb-2">
      <div
        v-for="col in columns"
        :key="`header-${col}`"
        class="h-4 rounded bg-surface-elevated flex-1"
        :class="headerWidthClass(col)" />
    </div>

    <!-- Data rows -->
    <div
      v-for="row in rows"
      :key="`row-${row}`"
      class="flex gap-4 py-2 border-b border-border-default/40">
      <div
        v-for="col in columns"
        :key="`cell-${row}-${col}`"
        class="h-3 rounded bg-surface-elevated flex-1"
        :class="cellWidthClass(row, col)" />
    </div>
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  columns?: number;
  rows?: number;
  showHeader?: boolean;
}>(), {
  columns: 4,
  rows: 5,
  showHeader: true,
});

/** Header cells use wider bars (70-90% of their flex space). */
function headerWidthClass(col: number): string {
  const widths = ["max-w-[90%]", "max-w-[80%]", "max-w-[70%]", "max-w-[85%]"];
  return widths[(col - 1) % widths.length];
}

/** Body cells vary width by a simple hash of row+col for visual variety. */
function cellWidthClass(row: number, col: number): string {
  const widths = [
    "max-w-[60%]",
    "max-w-[80%]",
    "max-w-[45%]",
    "max-w-[70%]",
    "max-w-[55%]",
    "max-w-[75%]",
    "max-w-[50%]",
  ];
  return widths[((row * 3 + col * 7) % widths.length)];
}
</script>
