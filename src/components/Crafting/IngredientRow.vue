<template>
  <div class="flex items-center gap-2 py-1.5 px-2 text-xs border-b border-surface-dark/50">
    <!-- Item display -->
    <template v-if="ingredient.item_id !== null">
      <ItemInline :reference="ingredient.item_name" />
    </template>
    <template v-else-if="ingredient.is_dynamic">
      <span
        class="text-text-secondary cursor-pointer hover:text-accent-gold"
        @click="showKeywordPicker = !showKeywordPicker">
        <span class="text-accent-gold/60 text-[0.65rem] mr-1">&#9670;</span>
        {{ ingredient.item_name }}
      </span>
    </template>
    <template v-else>
      <span class="text-text-muted italic">{{ ingredient.item_name }}</span>
    </template>

    <div class="ml-auto flex items-center gap-2 shrink-0">
      <!-- Quantity display: per-craft amount -->
      <span class="text-text-primary font-mono">
        ×{{ ingredient.per_craft }}
      </span>

      <!-- Chance to consume indicator -->
      <span
        v-if="ingredient.chance_to_consume < 1"
        class="text-accent-gold cursor-help"
        :title="`${Math.round(ingredient.chance_to_consume * 100)}% chance to consume per craft. ~${ingredient.expected_quantity} needed across all crafts.`">
        *
      </span>

      <!-- Craftable toggle -->
      <button
        v-if="ingredient.is_craftable && ingredient.children.length === 0"
        class="text-[0.65rem] cursor-pointer bg-transparent border rounded px-1.5 py-0.5 transition-colors"
        :class="isMarkedForCrafting
          ? 'text-accent-gold border-accent-gold/40 bg-accent-gold/10'
          : 'text-text-muted border-border-light hover:text-accent-gold hover:border-accent-gold/30'"
        :title="isMarkedForCrafting ? 'Remove from intermediate crafts' : 'Also craft this ingredient'"
        @click="$emit('toggle-intermediate', ingredient.item_id)">
        {{ isMarkedForCrafting ? '✓ craft' : '+ craft' }}
      </button>
    </div>

    <!-- Dynamic ingredient picker popup -->
    <DynamicIngredientPicker
      v-if="showKeywordPicker && ingredient.is_dynamic"
      :item-keys="ingredient.item_keys"
      class="absolute z-50 mt-1"
      @close="showKeywordPicker = false" />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import type { ResolvedIngredient } from "../../types/crafting";
import ItemInline from "../Shared/Item/ItemInline.vue";
import DynamicIngredientPicker from "./DynamicIngredientPicker.vue";

defineProps<{
  ingredient: ResolvedIngredient
  isMarkedForCrafting: boolean
}>();

defineEmits<{
  'toggle-intermediate': [itemId: number | null]
}>();

const showKeywordPicker = ref(false);
</script>
