<template>
  <div
    class="flex items-center gap-3 px-3 py-2 bg-surface-base border border-surface-elevated rounded text-xs group"
    :class="{
      'border-l-2 border-l-accent-gold/50': isSelected,
    }">
    <!-- Checkbox -->
    <input
      type="checkbox"
      :checked="isSelected"
      class="accent-accent-gold shrink-0"
      @change="$emit('toggle', entry.recipe.id)" />

    <!-- Food name -->
    <div class="flex items-center gap-1.5 min-w-0 flex-1">
      <ItemInline :reference="entry.food.name" />
    </div>

    <!-- Food category badge -->
    <span
      :class="[
        'text-[10px] font-semibold shrink-0 px-1.5 py-0.5 rounded',
        entry.food.food_category === 'Meal'
          ? 'bg-orange-500/15 text-orange-400'
          : entry.food.food_category === 'Snack'
            ? 'bg-blue-500/15 text-blue-400'
            : 'bg-green-500/15 text-green-400',
      ]">
      {{ entry.food.food_category }}
    </span>

    <!-- Food level -->
    <span class="text-text-dim shrink-0 w-10 text-right">
      Lv{{ entry.food.food_level }}
    </span>

    <!-- Gourmand req -->
    <span
      v-if="entry.food.gourmand_req"
      class="text-accent-gold/70 shrink-0 w-12 text-right text-[10px]">
      G{{ entry.food.gourmand_req }}
    </span>
    <span v-else class="shrink-0 w-12" />

    <!-- Recipe skill + level -->
    <div class="flex items-center gap-1 shrink-0">
      <SkillInline
        v-if="entry.recipe.skill"
        :reference="entry.recipe.skill"
        :show-icon="true"
        class="text-[10px]" />
      <span class="text-text-muted text-[10px]">
        {{ entry.recipe.skill_level_req ?? '?' }}
      </span>
    </div>

    <!-- Owned count -->
    <span
      v-if="ownedCount > 0"
      class="text-green-400/70 shrink-0 text-[10px] w-14 text-right"
      title="Already in inventory/storage">
      have {{ ownedCount }}
    </span>
    <span v-else class="shrink-0 w-14" />

    <!-- Material status -->
    <div class="shrink-0 w-5 text-center">
      <span
        v-if="materialStatus === 'ready'"
        class="text-green-400"
        title="All materials available">
        &#x2714;
      </span>
      <span
        v-else-if="materialStatus === 'vendor'"
        class="text-accent-gold"
        title="Missing materials are vendor-purchasable">
        &#x25CF;
      </span>
      <span
        v-else-if="materialStatus === 'partial'"
        class="text-yellow-400"
        title="Some materials missing">
        &#x25CF;
      </span>
      <span
        v-else
        class="text-text-muted/30"
        title="Materials not checked">
        &#x2015;
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import SkillInline from '../Shared/Skill/SkillInline.vue'
import { useCooksHelperStore, type HelpfulRecipe } from '../../stores/cooksHelperStore'
import type { MaterialNeed } from '../../types/crafting'

const props = defineProps<{
  entry: HelpfulRecipe
  isSelected: boolean
  materialNeeds: MaterialNeed[] | undefined
}>()

defineEmits<{
  toggle: [recipeId: number]
}>()

const cooksHelper = useCooksHelperStore()
const ownedCount = computed(() => cooksHelper.ownedCount(props.entry.food.name))

const materialStatus = computed<'ready' | 'partial' | 'vendor' | 'unknown'>(() => {
  if (!props.materialNeeds) return 'unknown'
  if (props.materialNeeds.length === 0) return 'ready'
  const allMet = props.materialNeeds.every(n => n.shortfall === 0)
  if (allMet) return 'ready'
  const onlyVendorShort = props.materialNeeds.every(n => n.shortfall === 0 || n.vendor_price !== null)
  if (onlyVendorShort) return 'vendor'
  return 'partial'
})
</script>
