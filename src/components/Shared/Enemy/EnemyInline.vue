<template>
  <span
    class="inline-flex items-center text-entity-enemy font-medium border-b border-dotted border-entity-enemy/30 cursor-pointer hover:underline"
    @click="handleClick"
  >
    {{ reference }}
  </span>
</template>

<script setup lang="ts">
import { useEntityNavigation } from "../../../composables/useEntityNavigation";

const props = defineProps<{
  reference: string;
}>();

const { navigateToEntity } = useEntityNavigation();

function handleClick() {
  navigateToEntity({ type: "enemy", id: props.reference });
}
</script>
