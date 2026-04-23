<template>
  <div :style="{ paddingLeft: `${depth * 16}px` }">
    <div class="flex items-center gap-2 py-1 border-b border-surface-dark text-xs group">
      <!-- Expand indicator for craftable items -->
      <span
        v-if="ingredient.children.length > 0"
        class="text-text-secondary cursor-pointer select-none w-3 text-center text-xs"
        @click="expanded = !expanded">
        {{ expanded ? '▾' : '▸' }}
      </span>
      <span v-else class="w-3 text-center text-surface-elevated">·</span>

      <!-- Item name -->
      <ItemInline :reference="ingredient.item_name" />

      <!-- Quantity -->
      <span class="text-text-primary ml-auto">
        ×{{ ingredient.expected_quantity }}
      </span>

      <!-- Chance to consume indicator -->
      <span
        v-if="ingredient.chance_to_consume < 1"
        class="text-text-muted text-[10px]"
        :title="`${Math.round(ingredient.chance_to_consume * 100)}% chance to consume per craft`">
        ({{ Math.round(ingredient.chance_to_consume * 100) }}%)
      </span>

      <!-- Craftable badge -->
      <span
        v-if="ingredient.is_craftable && ingredient.children.length === 0"
        class="text-accent-gold text-[10px] opacity-70"
        title="This item can be crafted from other ingredients">
        craftable
      </span>

      <!-- Source recipe if expanded -->
      <span
        v-if="ingredient.source_recipe_name && ingredient.children.length > 0"
        class="text-text-muted text-[10px]">
        via {{ ingredient.source_recipe_name }}
        <span v-if="ingredient.crafts_needed > 1">({{ ingredient.crafts_needed }}x)</span>
      </span>
    </div>

    <!-- Children (expanded intermediates) -->
    <template v-if="expanded && ingredient.children.length > 0">
      <IngredientTreeNode
        v-for="child in ingredient.children"
        :key="child.item_id ?? child.item_name"
        :ingredient="child"
        :depth="depth + 1" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { ResolvedIngredient } from "../../types/crafting";
import ItemInline from "../Shared/Item/ItemInline.vue";

const props = defineProps<{
  ingredient: ResolvedIngredient
  depth: number
}>();

const expanded = ref(props.ingredient.children.length > 0);
</script>
