import { ref, type Ref } from 'vue'
import { useGameDataStore } from '../stores/gameDataStore'
import { useMarketStore } from '../stores/marketStore'
import type { RecipeInfo } from '../types/gameData/recipes'

export interface IngredientCost {
  item_id: number | null
  item_name: string
  stack_size: number
  chance_to_consume: number
  expected_quantity: number
  /** Best unit price (market → craft cost → vendor fallback) */
  unit_price: number | null
  /** Total cost for this ingredient line */
  total_price: number | null
  /** How the price was sourced */
  price_source: 'market' | 'craft' | 'vendor' | null
  /** If craftable, the recursive cost to craft one unit */
  craft_unit_cost: number | null
  /** If craftable, the recipe name */
  craft_recipe_name: string | null
  /** Whether item_keys / dynamic ingredient */
  is_dynamic: boolean
}

export interface RecipeCostBreakdown {
  /** Total cost to craft 1x of this recipe */
  total_cost: number | null
  /** Per-ingredient breakdown */
  ingredients: IngredientCost[]
  /** True if any ingredient had no price at all */
  has_unknown_prices: boolean
  /** Cost per output unit (total_cost / output stack size) */
  cost_per_unit: number | null
}

/**
 * Recursively compute the material cost for a recipe.
 *
 * For each ingredient:
 * - Use market price if available
 * - If the ingredient is craftable, recursively compute the craft cost
 *   and use min(market, craft cost) as the effective price
 * - Fall back to estimated price (value × 2) if no market or craft price
 */
export function useRecipeCost() {
  const gameData = useGameDataStore()
  const marketStore = useMarketStore()

  const breakdown = ref<RecipeCostBreakdown | null>(null) as Ref<RecipeCostBreakdown | null>
  const loading = ref(false)

  function getMarketPrice(itemId: number): number | null {
    const market = marketStore.valuesByItemId[itemId]
    return market ? market.market_value : null
  }

  function getVendorPrice(vendorValue: number | null | undefined): number | null {
    if (vendorValue) return vendorValue * 2
    return null
  }

  /**
   * Compute the cost to craft one unit of a recipe's primary output.
   * Returns null if cost cannot be determined.
   * visited prevents infinite recursion on circular recipes.
   */
  async function computeCraftCostPerUnit(
    recipe: RecipeInfo,
    visited: Set<number>,
  ): Promise<number | null> {
    const outputPerCraft = recipe.result_items[0]?.stack_size ?? 1
    const primaryChance = (recipe.result_items[0]?.percent_chance ?? 100) / 100
    const effectiveOutput = outputPerCraft * primaryChance

    let totalCraftCost = 0
    let hasUnknown = false

    // Resolve all ingredient items in one batch
    const itemIds = recipe.ingredients
      .filter(i => i.item_id !== null)
      .map(i => i.item_id!)
    const items = itemIds.length > 0
      ? await gameData.resolveItemsBatch(itemIds.map(String))
      : {}

    for (const ing of recipe.ingredients) {
      const chanceToConsume = ing.chance_to_consume ?? 1
      const expectedQty = ing.stack_size * chanceToConsume

      if (ing.item_id === null) {
        // Dynamic/keyword ingredient — can't price
        hasUnknown = true
        continue
      }

      const unitPrice = await resolveIngredientUnitPrice(ing.item_id, items[String(ing.item_id)]?.value, visited)
      if (unitPrice === null) {
        hasUnknown = true
        continue
      }
      totalCraftCost += unitPrice * expectedQty
    }

    if (hasUnknown && totalCraftCost === 0) return null
    return totalCraftCost / effectiveOutput
  }

  /**
   * Find the best unit price for an ingredient:
   * market price vs recursive craft cost vs vendor fallback.
   * Returns [price, source].
   */
  async function resolveIngredientUnitPrice(
    itemId: number,
    vendorValue: number | null | undefined,
    visited: Set<number>,
  ): Promise<number | null> {
    const marketPrice = getMarketPrice(itemId)
    const vendorPrice = getVendorPrice(vendorValue)

    // Try to compute craft cost if not in a cycle
    let craftCost: number | null = null
    if (!visited.has(itemId)) {
      const producingRecipes = await gameData.getRecipesForItem(itemId)
      if (producingRecipes.length > 0) {
        visited.add(itemId)
        craftCost = await computeCraftCostPerUnit(producingRecipes[0], visited)
        visited.delete(itemId)
      }
    }

    // Pick the cheapest known price
    const candidates = [marketPrice, craftCost, vendorPrice].filter(
      (p): p is number => p !== null,
    )
    return candidates.length > 0 ? Math.min(...candidates) : null
  }

  function classifyPriceSource(
    itemId: number,
    craftCost: number | null,
    chosenPrice: number | null,
  ): 'market' | 'craft' | 'vendor' | null {
    if (chosenPrice === null) return null
    const marketPrice = getMarketPrice(itemId)
    if (marketPrice !== null && chosenPrice === marketPrice) return 'market'
    if (craftCost !== null && chosenPrice === craftCost) return 'craft'
    return 'vendor'
  }

  async function computeBreakdown(recipe: RecipeInfo): Promise<RecipeCostBreakdown> {
    const outputPerCraft = recipe.result_items[0]?.stack_size ?? 1
    const primaryChance = (recipe.result_items[0]?.percent_chance ?? 100) / 100
    const effectiveOutput = outputPerCraft * primaryChance

    // Resolve all ingredient items in one batch
    const itemIds = recipe.ingredients
      .filter(i => i.item_id !== null)
      .map(i => i.item_id!)
    const items = itemIds.length > 0
      ? await gameData.resolveItemsBatch(itemIds.map(String))
      : {}

    const ingredientCosts: IngredientCost[] = []
    let totalCost = 0
    let hasUnknown = false

    for (const ing of recipe.ingredients) {
      const chanceToConsume = ing.chance_to_consume ?? 1
      const expectedQty = ing.stack_size * chanceToConsume
      const isDynamic = ing.item_id === null && (ing.item_keys?.length ?? 0) > 0

      if (ing.item_id === null) {
        ingredientCosts.push({
          item_id: null,
          item_name: ing.description || ing.item_keys?.join(' / ') || 'Unknown',
          stack_size: ing.stack_size,
          chance_to_consume: chanceToConsume,
          expected_quantity: expectedQty,
          unit_price: null,
          total_price: null,
          price_source: null,
          craft_unit_cost: null,
          craft_recipe_name: null,
          is_dynamic: isDynamic,
        })
        hasUnknown = true
        continue
      }

      const item = items[String(ing.item_id)]
      const itemName = item?.name || `Item #${ing.item_id}`
      const vendorValue = item?.value

      // Get market price
      const marketPrice = getMarketPrice(ing.item_id)
      const vendorPrice = getVendorPrice(vendorValue)

      // Try craft cost
      let craftCost: number | null = null
      let craftRecipeName: string | null = null
      const visited = new Set<number>()
      const producingRecipes = await gameData.getRecipesForItem(ing.item_id)
      if (producingRecipes.length > 0) {
        visited.add(ing.item_id)
        craftCost = await computeCraftCostPerUnit(producingRecipes[0], visited)
        craftRecipeName = producingRecipes[0].name
      }

      // Pick cheapest
      const candidates = [marketPrice, craftCost, vendorPrice].filter(
        (p): p is number => p !== null,
      )
      const unitPrice = candidates.length > 0 ? Math.min(...candidates) : null
      const priceSource = classifyPriceSource(ing.item_id, craftCost, unitPrice)
      const totalPrice = unitPrice !== null ? Math.round(unitPrice * expectedQty) : null

      if (unitPrice === null) hasUnknown = true
      if (totalPrice !== null) totalCost += totalPrice

      ingredientCosts.push({
        item_id: ing.item_id,
        item_name: itemName,
        stack_size: ing.stack_size,
        chance_to_consume: chanceToConsume,
        expected_quantity: expectedQty,
        unit_price: unitPrice,
        total_price: totalPrice,
        price_source: priceSource,
        craft_unit_cost: craftCost,
        craft_recipe_name: craftRecipeName,
        is_dynamic: isDynamic,
      })
    }

    return {
      total_cost: totalCost > 0 || !hasUnknown ? Math.round(totalCost) : null,
      ingredients: ingredientCosts,
      has_unknown_prices: hasUnknown,
      cost_per_unit: totalCost > 0 ? Math.round(totalCost / effectiveOutput) : null,
    }
  }

  async function calculate(recipe: RecipeInfo) {
    loading.value = true
    try {
      breakdown.value = await computeBreakdown(recipe)
    } catch (e) {
      console.error('[useRecipeCost] Failed to compute cost:', e)
      breakdown.value = null
    } finally {
      loading.value = false
    }
  }

  return {
    breakdown,
    loading,
    calculate,
  }
}

export function formatGold(amount: number): string {
  const rounded = Math.round(amount)
  if (rounded >= 0) return rounded.toLocaleString() + 'g'
  return '-' + Math.abs(rounded).toLocaleString() + 'g'
}
