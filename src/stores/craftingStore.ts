import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { useGameDataStore } from "./gameDataStore";
import { useSettingsStore } from "./settingsStore";
import { useGameStateStore } from "./gameStateStore";
import { useMarketStore } from "./marketStore";
import type {
  CraftDetectionEvent,
  CraftingHistoryRecipe,
  CraftingProject,
  CraftingProjectEntry,
  CraftingProjectSummary,
  FeeConfig,
  CraftingTracker,
  DynamicItemBreakdown,
  FlattenedMaterial,
  IntermediateCraft,
  MaterialAvailability,
  MaterialNeed,
  ResolvedIngredient,
  ResolvedRecipe,
  SkillCraftingStats,
  TrackedRecipeEntry,
  WorkOrderSnapshotData,
  EnrichedWorkOrder,
  LevelingPlanLevel,
} from "../types/crafting";
import type { RecipeInfo } from "../types/gameData/recipes";
import type { PlayerEvent } from "../types/playerEvents";
import { type GameStateSkill } from "../types/gameState";

export const useCraftingStore = defineStore("crafting", () => {
  const gameData = useGameDataStore();
  const marketStore = useMarketStore();

  // ── Price helpers ──────────────────────────────────────────────────────────

  /**
   * Get the best known price for an item: market price if set, otherwise
   * a rough estimate of 2× vendor buy value (what you'd get selling to an NPC).
   */
  function getItemPrice(itemId: number, vendorValue: number | null | undefined): number | null {
    const market = marketStore.valuesByItemId[itemId];
    if (market) return market.market_value;
    if (vendorValue) return vendorValue * 2;
    return null;
  }

  /** Rough acquisition cost estimate when no market price is available */
  function getEstimatedPrice(vendorValue: number | null | undefined): number | null {
    if (vendorValue) return vendorValue * 2;
    return null;
  }

  // ── Recipe lookup cache ─────────────────────────────────────────────────────
  // Recipe data is static within a CDN version, so we cache lookups to avoid
  // repeated IPC round-trips during ingredient resolution.

  const recipesForItemCache = new Map<number, RecipeInfo[]>();

  async function getCachedRecipesForItem(itemId: number): Promise<RecipeInfo[]> {
    const cached = recipesForItemCache.get(itemId);
    if (cached) return cached;
    const recipes = await gameData.getRecipesForItem(itemId);
    recipesForItemCache.set(itemId, recipes);
    return recipes;
  }

  // ── Recipe filtering helpers ────────────────────────────────────────────────

  const MAX_ENCHANTED_RE = /\(Max-Enchanted\)/i;

  /** Filter out Max-Enchanted recipes if the setting is enabled. */
  function filterRecipes(recipes: RecipeInfo[]): RecipeInfo[] {
    const settingsStore = useSettingsStore();
    if (!settingsStore.settings.excludeMaxEnchantedRecipes) return recipes;
    return recipes.filter((r) => !MAX_ENCHANTED_RE.test(r.name));
  }

  // ── Project state ─────────────────────────────────────────────────────────

  const projects = ref<CraftingProjectSummary[]>([]);
  const activeProject = ref<CraftingProject | null>(null);
  /** When set, the sidebar is showing a group summary instead of a single project */
  const activeGroupName = ref<string | null>(null);

  async function loadProjects() {
    projects.value = await invoke<CraftingProjectSummary[]>("get_crafting_projects");
  }

  async function loadProject(id: number) {
    activeGroupName.value = null;
    const raw = await invoke<any>("get_crafting_project", { projectId: id });
    activeProject.value = {
      ...raw,
      fee_config: typeof raw.fee_config === 'string' ? JSON.parse(raw.fee_config) : raw.fee_config,
      customer_provides: typeof raw.customer_provides === 'string' ? JSON.parse(raw.customer_provides) : raw.customer_provides,
    } as CraftingProject;
  }

  function selectGroup(groupName: string) {
    activeProject.value = null;
    activeGroupName.value = groupName;
  }

  function clearGroupSelection() {
    activeGroupName.value = null;
  }

  /** Get all projects belonging to a group */
  function getProjectsInGroup(groupName: string): CraftingProjectSummary[] {
    return projects.value.filter((p) => p.group_name === groupName);
  }

  async function createProject(name: string, notes?: string, groupName?: string, feeConfig?: FeeConfig) {
    const id = await invoke<number>("create_crafting_project", {
      input: {
        name,
        notes,
        group_name: groupName ?? null,
        fee_config: feeConfig ? JSON.stringify(feeConfig) : null,
      },
    });
    await loadProjects();
    return id;
  }

  async function updateProject(
    id: number,
    name: string,
    notes: string,
    groupName?: string | null,
    feeConfig?: FeeConfig | null,
    customerProvides?: Record<string, number> | null,
  ) {
    await invoke("update_crafting_project", {
      input: {
        id,
        name,
        notes,
        group_name: groupName ?? null,
        fee_config: feeConfig ? JSON.stringify(feeConfig) : null,
        customer_provides: customerProvides ? JSON.stringify(customerProvides) : null,
      },
    });
    await loadProjects();
    if (activeProject.value?.id === id) {
      activeProject.value.name = name;
      activeProject.value.notes = notes;
      activeProject.value.group_name = groupName ?? null;
      if (feeConfig) activeProject.value.fee_config = feeConfig;
      if (customerProvides) activeProject.value.customer_provides = customerProvides;
    }
  }

  async function deleteProject(id: number) {
    await invoke("delete_crafting_project", { projectId: id });
    if (activeProject.value?.id === id) activeProject.value = null;
    await loadProjects();
  }

  async function duplicateProject(id: number) {
    const newId = await invoke<number>("duplicate_crafting_project", { projectId: id });
    await loadProjects();
    return newId;
  }

  async function addEntry(projectId: number, recipeId: number, recipeName: string, quantity: number, targetStock?: number | null) {
    await invoke("add_project_entry", {
      input: { project_id: projectId, recipe_id: recipeId, recipe_name: recipeName, quantity, target_stock: targetStock ?? null },
    });
    if (activeProject.value?.id === projectId) {
      await loadProject(projectId);
    }
    await loadProjects();
  }

  async function updateEntry(entryId: number, quantity: number, expandedIngredientIds: number[] = [], targetStock?: number | null) {
    await invoke("update_project_entry", {
      input: { id: entryId, quantity, expanded_ingredient_ids: expandedIngredientIds, target_stock: targetStock ?? null },
    });
    if (activeProject.value) {
      await loadProject(activeProject.value.id);
    }
  }

  async function removeEntry(entryId: number) {
    await invoke("remove_project_entry", { entryId });
    if (activeProject.value) {
      await loadProject(activeProject.value.id);
    }
    await loadProjects();
  }

  // ── Dependency Resolver ───────────────────────────────────────────────────

  /**
   * Core resolver: given a recipe and craft count, compute ingredient tree.
   * expandItemIds: if provided, only expand intermediates whose item_id is in this set.
   * If not provided, falls back to the boolean expandIntermediates flag.
   * intermediateStock: if provided, maps item_id → quantity on hand. When expanding
   * an intermediate, only the shortfall (needed - stock) is resolved into sub-ingredients.
   */
  async function resolveRecipeIngredients(
    recipe: RecipeInfo,
    desiredQuantity: number,
    expandIntermediates: boolean = false,
    visited: Set<number> = new Set(),
    expandItemIds?: Set<number>,
    intermediateStock?: Map<number, number>,
  ): Promise<ResolvedRecipe> {
    // Calculate how many crafts are needed
    const outputPerCraft = recipe.result_items[0]?.stack_size ?? 1;
    const primaryChance = (recipe.result_items[0]?.percent_chance ?? 100) / 100;
    const effectiveOutput = outputPerCraft * primaryChance;
    const craftCount = Math.ceil(desiredQuantity / effectiveOutput);

    // Resolve each ingredient
    const ingredients: ResolvedIngredient[] = [];
    for (const ing of recipe.ingredients) {
      const chanceToConsume = ing.chance_to_consume ?? 1;
      const totalNeeded = ing.stack_size * craftCount;
      const expectedQty = Math.ceil(totalNeeded * chanceToConsume);
      const isDynamic = ing.item_id === null && ing.item_keys.length > 0;

      // Check if this ingredient is itself craftable
      let isCraftable = false;
      let children: ResolvedIngredient[] = [];
      let sourceRecipeId: number | null = null;
      let sourceRecipeName: string | null = null;
      let childCraftsNeeded = 0;

      // Determine whether to expand this ingredient
      const shouldExpand = ing.item_id
        ? expandItemIds
          ? expandItemIds.has(ing.item_id)
          : expandIntermediates
        : false;

      if (ing.item_id && shouldExpand && !visited.has(ing.item_id)) {
        const producingRecipes = filterRecipes(await getCachedRecipesForItem(ing.item_id));
        if (producingRecipes.length > 0) {
          isCraftable = true;
          const sourceRecipe = producingRecipes[0];
          sourceRecipeId = sourceRecipe.id;
          sourceRecipeName = sourceRecipe.name;

          // Subtract stock on hand — only craft the shortfall.
          // Consume allocated stock so later entries don't double-count it.
          const onHand = intermediateStock?.get(ing.item_id) ?? 0;
          const toCraft = Math.max(0, expectedQty - onHand);
          if (intermediateStock && onHand > 0) {
            const consumed = Math.min(onHand, expectedQty);
            intermediateStock.set(ing.item_id, onHand - consumed);
          }

          // Recursively resolve (with cycle detection)
          visited.add(ing.item_id);
          if (toCraft > 0) {
            const subResolved = await resolveRecipeIngredients(
              sourceRecipe,
              toCraft,
              true,
              visited,
              expandItemIds,
              intermediateStock,
            );
            children = subResolved.ingredients;
            childCraftsNeeded = subResolved.craft_count;
          }
          visited.delete(ing.item_id);
        }
      } else if (ing.item_id) {
        // Just check if craftable (without expanding)
        const producingRecipes = filterRecipes(await getCachedRecipesForItem(ing.item_id));
        isCraftable = producingRecipes.length > 0;
      }

      // Get item name
      let itemName = ing.description ?? "Unknown item";
      if (ing.item_id) {
        const item = await gameData.resolveItem(ing.item_id);
        if (item) itemName = item.name;
      }

      ingredients.push({
        item_id: ing.item_id,
        item_name: itemName,
        per_craft: ing.stack_size,
        quantity_needed: totalNeeded,
        chance_to_consume: chanceToConsume,
        expected_quantity: expectedQty,
        is_craftable: isCraftable,
        source_recipe_id: sourceRecipeId,
        source_recipe_name: sourceRecipeName,
        children,
        crafts_needed: childCraftsNeeded,
        item_keys: ing.item_keys ?? [],
        is_dynamic: isDynamic,
      });
    }

    // Estimate cost: sum of market prices (falling back to vendor price * 1.5)
    const itemIds = ingredients
      .filter((i) => i.item_id !== null && i.children.length === 0)
      .map((i) => i.item_id!);

    let estimatedCost = 0;
    if (itemIds.length > 0) {
      const items = await gameData.resolveItemsBatch(itemIds.map(String));
      for (const ing of ingredients) {
        if (ing.item_id && ing.children.length === 0) {
          const item = items[String(ing.item_id)];
          const price = getItemPrice(ing.item_id, item?.value);
          if (price) {
            estimatedCost += price * ing.expected_quantity;
          }
        }
      }
    }

    return {
      recipe_id: recipe.id,
      recipe_name: recipe.name,
      craft_count: craftCount,
      desired_quantity: desiredQuantity,
      output_per_craft: outputPerCraft,
      xp_per_craft: recipe.reward_skill_xp ?? 0,
      xp_first_time: recipe.reward_skill_xp_first_time ?? 0,
      total_xp: (recipe.reward_skill_xp ?? 0) * craftCount,
      reward_skill: recipe.reward_skill,
      ingredients,
      estimated_cost: Math.round(estimatedCost),
    };
  }

  /**
   * Flatten a resolved recipe's ingredient tree into a deduplicated list
   * of leaf-node materials (raw materials the player actually needs).
   * Now includes dynamic (keyword-based) ingredients and carries chance_to_consume.
   */
  function flattenIngredients(ingredients: ResolvedIngredient[]): Map<string, FlattenedMaterial> {
    const flat = new Map<string, FlattenedMaterial>();

    function walk(list: ResolvedIngredient[]) {
      for (const ing of list) {
        if (ing.source_recipe_id !== null) {
          // Expanded intermediate — recurse into children (if any).
          // Stock-satisfied intermediates may have no children but are
          // still handled by collectIntermediates, not the flat list.
          if (ing.children.length > 0) {
            walk(ing.children);
          }
        } else {
          // Leaf node — determine key
          let key: string;
          if (ing.item_id !== null) {
            key = String(ing.item_id);
          } else if (ing.is_dynamic && ing.item_keys.length > 0) {
            key = `kw:${ing.item_keys[0]}`;
          } else {
            continue; // Skip ingredients with no item and no keywords
          }
          const existing = flat.get(key);
          if (existing) {
            existing.quantity += ing.quantity_needed;
            existing.expected_quantity += ing.expected_quantity;
          } else {
            flat.set(key, {
              key,
              item_id: ing.item_id,
              item_name: ing.item_name,
              quantity: ing.quantity_needed,
              chance_to_consume: ing.chance_to_consume,
              expected_quantity: ing.expected_quantity,
              is_dynamic: ing.is_dynamic,
              item_keys: ing.item_keys,
              is_craftable: ing.is_craftable,
            });
          }
        }
      }
    }

    walk(ingredients);
    return flat;
  }

  /**
   * Collect intermediate crafts from the resolved ingredient tree.
   * An intermediate is any ingredient that has a source recipe (was marked
   * for expansion), regardless of whether it actually has children — stock
   * may fully cover the need, leaving children empty.
   */
  function collectIntermediates(ingredients: ResolvedIngredient[]): IntermediateCraft[] {
    const intermediates: IntermediateCraft[] = [];

    function walk(list: ResolvedIngredient[]) {
      for (const ing of list) {
        if (ing.source_recipe_id !== null && ing.item_id !== null) {
          intermediates.push({
            recipe_name: ing.source_recipe_name ?? ing.item_name,
            recipe_id: ing.source_recipe_id,
            item_name: ing.item_name,
            item_id: ing.item_id,
            crafts_needed: ing.crafts_needed,
            quantity_produced: ing.expected_quantity,
          });
          // Also collect nested intermediates
          if (ing.children.length > 0) {
            walk(ing.children);
          }
        }
      }
    }

    walk(ingredients);
    return intermediates;
  }

  // ── Material Availability ─────────────────────────────────────────────────

  /**
   * Given a flat material map (from flattenIngredients), check what the player
   * has across live inventory and the latest storage snapshot.
   * Dynamic (keyword-based) materials are resolved via user preferences to
   * determine which concrete items to check.
   */
  async function checkMaterialAvailability(
    materials: Map<string, FlattenedMaterial>,
  ): Promise<MaterialNeed[]> {
    const settingsStore = useSettingsStore();

    // Separate concrete and dynamic materials
    const concreteMaterials = Array.from(materials.values()).filter(
      (m) => m.item_id !== null && !m.is_dynamic,
    );
    const dynamicMaterials = Array.from(materials.values()).filter(
      (m) => m.is_dynamic && m.item_keys.length > 0,
    );

    // Resolve dynamic keywords → enabled concrete item IDs
    const dynamicItemIds: number[] = [];
    const dynamicResolvedMap = new Map<string, number[]>(); // key → enabled item IDs
    for (const mat of dynamicMaterials) {
      const keyword = mat.item_keys[0];
      const disabledSet = getDynamicItemDisabledSet(keyword);
      const allItems = await gameData.getItemsByKeyword(keyword);
      const enabledIds = allItems
        .filter((item) => !disabledSet.has(item.id))
        .map((item) => item.id);
      dynamicResolvedMap.set(mat.key, enabledIds);
      dynamicItemIds.push(...enabledIds);
    }

    // Combine all item IDs for a single batch inventory query
    const concreteIds = concreteMaterials.map((m) => m.item_id!);
    const allItemIds = [...concreteIds, ...dynamicItemIds];
    if (allItemIds.length === 0 && dynamicMaterials.length === 0) return [];

    // 1. Query storage vaults from latest snapshot
    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;

    let storageData: MaterialAvailability[] = [];
    if (characterName && serverName && allItemIds.length > 0) {
      try {
        // Deduplicate IDs for the batch query
        const uniqueIds = [...new Set(allItemIds)];
        storageData = await invoke<MaterialAvailability[]>("check_material_availability", {
          characterName,
          serverName,
          itemTypeIds: uniqueIds,
        });
      } catch (e) {
        console.warn("[crafting] Failed to check storage availability:", e);
      }
    }

    // Build lookup from backend results
    const availMap = new Map<number, MaterialAvailability>();
    for (const item of storageData) {
      availMap.set(item.item_type_id, item);
    }

    // 2. Get vendor prices and vendor-purchasable item IDs for cost estimation
    const [itemData, vendorItemIds] = await Promise.all([
      gameData.resolveItemsBatch(allItemIds.length > 0 ? [...new Set(allItemIds)].map(String) : []),
      invoke<number[]>('get_vendor_purchasable_item_ids'),
    ]);
    const vendorSet = new Set(vendorItemIds);

    // 3. Build MaterialNeed list — concrete items
    const needs: MaterialNeed[] = [];
    for (const mat of concreteMaterials) {
      const itemId = mat.item_id!;
      const avail = availMap.get(itemId);
      const invQty = avail?.inventory_quantity ?? 0;
      const storageQty = avail?.storage_quantity ?? 0;
      const totalHave = invQty + storageQty;
      const shortfall = Math.max(0, mat.expected_quantity - totalHave);

      const item = itemData[String(itemId)];
      const vendorValue = vendorSet.has(itemId) ? (item?.value ?? null) : null;
      const price = getItemPrice(itemId, item?.value);

      needs.push({
        item_id: itemId,
        item_name: mat.item_name,
        quantity_needed: mat.expected_quantity,
        inventory_have: invQty,
        storage_have: storageQty,
        vault_breakdown: avail?.vault_breakdown ?? [],
        shortfall,
        vendor_price: vendorValue ? getEstimatedPrice(vendorValue) : null,
        unit_price: price,
        is_craftable: mat.is_craftable,
      });
    }

    // 4. Build MaterialNeed list — dynamic items (aggregate across enabled concrete items)
    for (const mat of dynamicMaterials) {
      const enabledIds = dynamicResolvedMap.get(mat.key) ?? [];

      // Sum inventory and storage across all enabled items for this keyword
      let totalInv = 0;
      let totalStorage = 0;
      const combinedVaultBreakdown: MaterialNeed["vault_breakdown"] = [];
      const breakdown: DynamicItemBreakdown[] = [];
      for (const id of enabledIds) {
        const avail = availMap.get(id);
        if (avail) {
          totalInv += avail.inventory_quantity;
          totalStorage += avail.storage_quantity;
          for (const vb of avail.vault_breakdown) {
            // Tag each vault entry with the concrete item info for pickup list resolution
            combinedVaultBreakdown.push({
              ...vb,
              item_id: avail.item_type_id,
              item_name: avail.item_name,
            });
          }
          // Track per-item breakdown for materials display
          if (avail.inventory_quantity > 0 || avail.storage_quantity > 0) {
            breakdown.push({
              item_id: avail.item_type_id,
              item_name: avail.item_name,
              inventory_qty: avail.inventory_quantity,
              storage_qty: avail.storage_quantity,
            });
          }
        }
      }
      const totalHave = totalInv + totalStorage;
      const shortfall = Math.max(0, mat.expected_quantity - totalHave);

      needs.push({
        item_id: 0, // sentinel for dynamic
        item_name: mat.item_name,
        quantity_needed: mat.expected_quantity,
        inventory_have: totalInv,
        storage_have: totalStorage,
        vault_breakdown: combinedVaultBreakdown,
        shortfall,
        vendor_price: null,
        unit_price: null,
        is_craftable: false,
        is_dynamic: true,
        item_keys: mat.item_keys,
        dynamic_breakdown: breakdown.length > 0 ? breakdown.sort((a, b) => a.item_name.localeCompare(b.item_name)) : undefined,
      });
    }

    return needs.sort((a, b) => a.item_name.localeCompare(b.item_name));
  }

  // ── Dynamic item preferences ──────────────────────────────────────────────

  /**
   * Get the set of disabled item IDs for a keyword.
   * Default = all items enabled (empty disabled set).
   */
  function getDynamicItemDisabledSet(keyword: string): Set<number> {
    const settingsStore = useSettingsStore();
    const dynamicItems = settingsStore.settings.viewPreferences?.dynamicItems as
      | Record<string, number[]>
      | undefined;
    const disabled = dynamicItems?.[keyword];
    return new Set(disabled ?? []);
  }

  /**
   * Toggle an item enabled/disabled for a keyword and persist.
   */
  function setDynamicItemDisabled(keyword: string, itemId: number, disabled: boolean) {
    const settingsStore = useSettingsStore();
    const allPrefs = { ...(settingsStore.settings.viewPreferences ?? {}) };
    const dynamicItems = { ...((allPrefs.dynamicItems as Record<string, number[]>) ?? {}) };
    const current = new Set(dynamicItems[keyword] ?? []);

    if (disabled) {
      current.add(itemId);
    } else {
      current.delete(itemId);
    }

    dynamicItems[keyword] = Array.from(current);
    allPrefs.dynamicItems = dynamicItems;
    settingsStore.updateSettings({ viewPreferences: allPrefs });
  }

  /**
   * Bulk-set all items for a keyword as enabled or disabled.
   */
  function setAllDynamicItems(keyword: string, itemIds: number[], disabled: boolean) {
    const settingsStore = useSettingsStore();
    const allPrefs = { ...(settingsStore.settings.viewPreferences ?? {}) };
    const dynamicItems = { ...((allPrefs.dynamicItems as Record<string, number[]>) ?? {}) };

    if (disabled) {
      dynamicItems[keyword] = [...itemIds];
    } else {
      dynamicItems[keyword] = [];
    }

    allPrefs.dynamicItems = dynamicItems;
    settingsStore.updateSettings({ viewPreferences: allPrefs });
  }

  /**
   * Query total available stock (inventory + storage) for a set of item IDs.
   * Returns a map of itemId → total quantity on hand.
   */
  async function queryItemStock(itemIds: number[]): Promise<Map<number, number>> {
    const settingsStore = useSettingsStore();
    const result = new Map<number, number>();
    if (itemIds.length === 0) return result;

    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;
    if (!characterName || !serverName) return result;

    try {
      const data = await invoke<MaterialAvailability[]>("check_material_availability", {
        characterName,
        serverName,
        itemTypeIds: itemIds,
      });
      for (const item of data) {
        result.set(item.item_type_id, item.total_available);
      }
    } catch (e) {
      console.warn("[crafting] Failed to query item stock:", e);
    }
    return result;
  }

  // ── Stock Targets ────────────────────────────────────────────────────────

  interface StockTargetResult {
    effectiveQty: number
    currentStock: number
  }

  /**
   * For entries with target_stock set, resolve the output item's current
   * inventory+storage count and compute how many to craft.
   * Returns a map of entry.id → { effectiveQty, currentStock }.
   */
  async function resolveStockTargets(
    entries: CraftingProjectEntry[],
  ): Promise<Map<number, StockTargetResult>> {
    const settingsStore = useSettingsStore();
    const result = new Map<number, StockTargetResult>();

    const targetEntries = entries.filter((e) => e.target_stock !== null);
    if (targetEntries.length === 0) return result;

    // Resolve recipes to get output item IDs
    const recipeOutputs = new Map<number, { itemId: number; outputPerCraft: number; primaryChance: number }>();
    for (const entry of targetEntries) {
      const recipe = await gameData.resolveRecipe(entry.recipe_name);
      if (!recipe || recipe.result_items.length === 0) continue;
      const primary = recipe.result_items[0];
      recipeOutputs.set(entry.id, {
        itemId: primary.item_id,
        outputPerCraft: primary.stack_size,
        primaryChance: (primary.percent_chance ?? 100) / 100,
      });
    }

    // Batch query availability for all output items
    const outputItemIds = Array.from(new Set(
      Array.from(recipeOutputs.values()).map((o) => o.itemId),
    ));

    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;

    let availData: MaterialAvailability[] = [];
    if (characterName && serverName && outputItemIds.length > 0) {
      try {
        availData = await invoke<MaterialAvailability[]>("check_material_availability", {
          characterName,
          serverName,
          itemTypeIds: outputItemIds,
        });
      } catch (e) {
        console.warn("[crafting] Failed to check stock targets:", e);
      }
    }

    const availMap = new Map<number, number>();
    for (const item of availData) {
      availMap.set(item.item_type_id, item.total_available);
    }

    // Compute effective quantities
    for (const entry of targetEntries) {
      const output = recipeOutputs.get(entry.id);
      if (!output) continue;

      const currentStock = availMap.get(output.itemId) ?? 0;
      const needed = Math.max(0, entry.target_stock! - currentStock);

      result.set(entry.id, { effectiveQty: needed, currentStock });
    }

    return result;
  }

  // ── Leveling Helper ──────────────────────────────────────────────────────

  /**
   * Estimate the total ingredient cost for one craft of a recipe,
   * using app-wide pricing (market → vendor × 1.5 fallback).
   */
  async function estimateRecipeCost(recipe: RecipeInfo): Promise<number> {
    const itemIds = recipe.ingredients
      .filter((i) => i.item_id !== null)
      .map((i) => i.item_id!);
    if (itemIds.length === 0) return 0;

    const items = await gameData.resolveItemsBatch(itemIds.map(String));
    let cost = 0;
    for (const ing of recipe.ingredients) {
      if (ing.item_id) {
        const item = items[String(ing.item_id)];
        const price = getItemPrice(ing.item_id, item?.value);
        if (price) {
          const chanceToConsume = ing.chance_to_consume ?? 1;
          cost += price * ing.stack_size * chanceToConsume;
        }
      }
    }
    return Math.round(cost);
  }

  /**
   * Get the player's current skill level from game state.
   * Resolves display name → CDN internal name → game state lookup.
   */
  async function getSkillLevel(skillName: string): Promise<{ baseLevel: number; bonusLevels: number; totalLevel: number; xpTowardNext: number; xpNeededForNext: number } | null> {
    const gameStateStore = useGameStateStore()

    function toResult(skill: GameStateSkill) {
      return { baseLevel: skill.base_level, bonusLevels: skill.bonus_levels, totalLevel: skill.level, xpTowardNext: skill.xp, xpNeededForNext: skill.tnl }
    }

    // Try direct lookup first (works if skillName is already an internal name)
    let skill = gameStateStore.skillsByName[skillName]
    if (skill) return toResult(skill)

    // Resolve display name → internal name via CDN
    const skillInfo = await gameData.resolveSkill(skillName)
    if (skillInfo?.internal_name) {
      skill = gameStateStore.skillsByName[skillInfo.internal_name]
      if (skill) return toResult(skill)
    }

    return null
  }

  // ── Leveling Tab State (persists across tab switches) ───────────────────

  const levelingState = ref<{
    selectedSkill: string
    currentLevel: number
    bonusLevels: number
    snapshotLevel: number | null
    xpBuffPercent: number
    xpTable: number[]
    planLevels: LevelingPlanLevel[]
    /** XP already earned toward the current level (from game state or manual entry) */
    startingXp: number
  }>({
    selectedSkill: "",
    currentLevel: 0,
    bonusLevels: 0,
    snapshotLevel: null,
    xpBuffPercent: 0,
    xpTable: [],
    planLevels: [],
    startingXp: 0,
  })

  /**
   * Create a crafting project from the current leveling plan.
   * Aggregates entries across all levels by recipe_id, summing craft counts.
   * Converts craft counts to desired output quantities so the project's
   * material resolver (which divides by output-per-craft) arrives at the
   * correct number of crafts.
   */
  async function createProjectFromLevelingPlan(planName: string): Promise<number> {
    const plan = levelingState.value.planLevels
    // Aggregate entries across all levels by recipe_id (craft counts)
    const recipeMap = new Map<number, { recipe_name: string; total_crafts: number }>()
    for (const lvl of plan) {
      for (const entry of lvl.entries) {
        const existing = recipeMap.get(entry.recipe_id)
        if (existing) {
          existing.total_crafts += entry.craft_count
        } else {
          recipeMap.set(entry.recipe_id, {
            recipe_name: entry.recipe_name,
            total_crafts: entry.craft_count,
          })
        }
      }
    }

    const projectId = await createProject(planName)
    for (const [recipeId, data] of recipeMap) {
      // Project entries use "desired output quantity" — the material resolver
      // divides by output-per-craft to derive craft count. Multiply craft
      // count by the recipe's output per craft so the math round-trips.
      const recipe = await gameData.resolveRecipe(recipeId)
      const outputPerCraft = recipe?.result_items[0]?.stack_size ?? 1
      const desiredQuantity = data.total_crafts * outputPerCraft
      await addEntry(projectId, recipeId, data.recipe_name, desiredQuantity)
    }
    return projectId
  }

  // ── Crafting History ─────────────────────────────────────────────────────

  /**
   * Get all recipe completions merged with CDN recipe data.
   * Returns enriched list sorted by completions (descending).
   */
  async function getCraftingHistory(): Promise<CraftingHistoryRecipe[]> {
    const gameStateStore = useGameStateStore();
    const completionMap = gameStateStore.recipeCompletions;
    if (Object.keys(completionMap).length === 0) return [];

    // Enrich with CDN recipe data
    const results: CraftingHistoryRecipe[] = [];
    for (const [recipeKey, completions] of Object.entries(completionMap)) {
      const recipe = await gameData.resolveRecipe(recipeKey);
      results.push({
        recipe_key: recipeKey,
        recipe_name: recipe?.name ?? recipeKey,
        completions,
        skill: recipe?.skill ?? null,
        reward_skill: recipe?.reward_skill ?? null,
        skill_level_req: recipe?.skill_level_req ?? null,
      });
    }

    return results.sort((a, b) => b.completions - a.completions);
  }

  /**
   * Compute per-skill crafting stats: total recipes available (from CDN),
   * how many the player has crafted, completion percentages.
   */
  async function getSkillCraftingStats(): Promise<SkillCraftingStats[]> {
    const gameStateStore = useGameStateStore();
    const completionMap = gameStateStore.recipeCompletions;
    const completionSet = gameStateStore.knownRecipeKeys;

    // Get all skills with XP tables (crafting-capable skills)
    const allSkills = await gameData.getAllSkills();
    const craftSkills = allSkills.filter((s) => s.xp_table !== null);

    const stats: SkillCraftingStats[] = [];
    for (const skill of craftSkills) {
      const recipes = await gameData.getRecipesForSkill(skill.name);
      // Count recipes that reward XP in this skill
      const relevantRecipes = recipes.filter((r) => r.reward_skill === skill.name);
      if (relevantRecipes.length === 0) continue;

      let craftedCount = 0;
      let totalCompletions = 0;
      for (const r of relevantRecipes) {
        const key = `Recipe_${r.id}`;
        if (completionSet.has(key)) {
          craftedCount++;
          totalCompletions += completionMap[key] ?? 0;
        }
      }

      stats.push({
        skill_name: skill.name,
        total_recipes: relevantRecipes.length,
        crafted_recipes: craftedCount,
        total_completions: totalCompletions,
        uncrafted_count: relevantRecipes.length - craftedCount,
        completion_percent: relevantRecipes.length > 0
          ? Math.round((craftedCount / relevantRecipes.length) * 100)
          : 0,
      });
    }

    return stats.sort((a, b) => a.skill_name.localeCompare(b.skill_name));
  }

  // ── Live Crafting Detection ────────────────────────────────────────────

  const tracker = ref<CraftingTracker | null>(null);
  const craftLog = ref<CraftDetectionEvent[]>([]);
  const CRAFT_LOG_MAX = 100;

  /**
   * Build an output-name → entry index lookup from the tracker.
   * Multiple recipes can produce the same item, so we map to the first match.
   */
  function buildOutputLookup(): Map<string, number> {
    const lookup = new Map<string, number>();
    if (!tracker.value) return lookup;
    for (let i = 0; i < tracker.value.entries.length; i++) {
      const entry = tracker.value.entries[i];
      if (!lookup.has(entry.output_item_name)) {
        lookup.set(entry.output_item_name, i);
      }
    }
    return lookup;
  }

  /**
   * Start tracking crafting progress for a project.
   * Resolves recipe outputs so we know what items to watch for.
   */
  async function startTracking(projectId: number | null, recipes: { recipe: RecipeInfo; targetQuantity: number }[]) {
    const entries: TrackedRecipeEntry[] = [];
    for (const { recipe, targetQuantity } of recipes) {
      const primaryOutput = recipe.result_items[0];
      if (!primaryOutput) continue;

      // Get the item name for matching
      const item = await gameData.resolveItem(primaryOutput.item_id);
      if (!item) continue;

      entries.push({
        recipe_id: recipe.id,
        recipe_name: recipe.name,
        output_item_name: item.name,
        output_item_type_id: primaryOutput.item_id,
        output_per_craft: primaryOutput.stack_size,
        target_quantity: targetQuantity,
        detected_output: 0,
        crafts_completed: 0,
        baseline_completion_count: null,
        manual_adjustment: 0,
      });
    }

    tracker.value = {
      project_id: projectId,
      entries,
      started_at: new Date().toISOString(),
      active: true,
    };
    craftLog.value = [];
  }

  /**
   * Start tracking for the active project's entries.
   */
  async function startProjectTracking() {
    if (!activeProject.value) return;

    const recipes: { recipe: RecipeInfo; targetQuantity: number }[] = [];
    for (const entry of activeProject.value.entries) {
      const recipe = await gameData.resolveRecipe(entry.recipe_name);
      if (recipe) {
        recipes.push({ recipe, targetQuantity: entry.quantity });
      }
    }
    await startTracking(activeProject.value.id, recipes);
  }

  /**
   * Start tracking for a single recipe (quick calc mode).
   */
  async function startQuickCalcTracking(recipe: RecipeInfo, targetQuantity: number) {
    await startTracking(null, [{ recipe, targetQuantity }]);
  }

  function stopTracking() {
    if (tracker.value) {
      tracker.value.active = false;
    }
  }

  function clearTracking() {
    tracker.value = null;
    craftLog.value = [];
  }

  /**
   * Build recipe_id → entry index lookup for RecipeUpdated matching.
   */
  function buildRecipeIdLookup(): Map<number, number> {
    const lookup = new Map<number, number>();
    if (!tracker.value) return lookup;
    for (let i = 0; i < tracker.value.entries.length; i++) {
      lookup.set(tracker.value.entries[i].recipe_id, i);
    }
    return lookup;
  }

  /**
   * Manually adjust the detected output for a tracked recipe.
   * Positive values add crafts, negative values subtract.
   */
  function adjustTrackedOutput(recipeId: number, delta: number) {
    if (!tracker.value) return;
    const entry = tracker.value.entries.find(e => e.recipe_id === recipeId);
    if (!entry) return;
    entry.manual_adjustment += delta;
    const adjustedOutput = entry.detected_output + entry.manual_adjustment;
    entry.crafts_completed = Math.floor(Math.max(0, adjustedOutput) / entry.output_per_craft);
    tracker.value = { ...tracker.value };
  }

  /**
   * Handle a player event to detect crafted items.
   * Uses RecipeUpdated (authoritative) as the primary signal, with
   * ItemAdded/ItemStackChanged as a fallback for non-tracked recipes.
   */
  function handleCraftDetection(event: PlayerEvent) {
    if (!tracker.value?.active) return;

    // Primary detection: RecipeUpdated provides authoritative completion count
    if (event.kind === "RecipeUpdated") {
      const recipeLookup = buildRecipeIdLookup();
      const idx = recipeLookup.get(event.recipe_id);
      if (idx !== undefined) {
        const entry = tracker.value.entries[idx];

        if (entry.baseline_completion_count === null) {
          // First RecipeUpdated for this entry — set baseline
          entry.baseline_completion_count = event.completion_count;
        }

        // Calculate crafts since tracking started
        const craftsSinceStart = event.completion_count - entry.baseline_completion_count;
        const newOutput = craftsSinceStart * entry.output_per_craft;

        if (newOutput > entry.detected_output) {
          const delta = newOutput - entry.detected_output;
          entry.detected_output = newOutput;
          entry.crafts_completed = Math.floor(
            Math.max(0, entry.detected_output + entry.manual_adjustment) / entry.output_per_craft,
          );

          craftLog.value.unshift({
            timestamp: event.timestamp,
            recipe_name: entry.recipe_name,
            item_name: entry.output_item_name,
            quantity: delta,
          });
          if (craftLog.value.length > CRAFT_LOG_MAX) {
            craftLog.value.length = CRAFT_LOG_MAX;
          }

          tracker.value = { ...tracker.value };
        }
      }
      return;
    }

    // Fallback detection: ItemAdded/ItemStackChanged for items matching tracked outputs
    if (event.kind === "ItemAdded" && event.is_new) {
      const lookup = buildOutputLookup();
      const idx = lookup.get(event.item_name);
      if (idx !== undefined) {
        const entry = tracker.value.entries[idx];
        // Defer quantity counting to ItemStackChanged
        craftLog.value.unshift({
          timestamp: event.timestamp,
          recipe_name: entry.recipe_name,
          item_name: event.item_name,
          quantity: 0, // Updated by stack change
        });
        if (craftLog.value.length > CRAFT_LOG_MAX) {
          craftLog.value.length = CRAFT_LOG_MAX;
        }
      }
    } else if (event.kind === "ItemStackChanged" && event.delta > 0) {
      const itemName = event.item_name ?? "";
      const lookup = buildOutputLookup();
      const idx = lookup.get(itemName);
      if (idx !== undefined) {
        const entry = tracker.value.entries[idx];
        // Only use item-based detection if we haven't received RecipeUpdated for this entry
        // (RecipeUpdated is authoritative and already counts output)
        if (entry.baseline_completion_count !== null) return;

        entry.detected_output += event.delta;
        entry.crafts_completed = Math.floor(
          Math.max(0, entry.detected_output + entry.manual_adjustment) / entry.output_per_craft,
        );

        const recentLog = craftLog.value.find(
          (l) => l.item_name === itemName && l.quantity === 0,
        );
        if (recentLog) {
          recentLog.quantity = event.delta;
        } else {
          craftLog.value.unshift({
            timestamp: event.timestamp,
            recipe_name: entry.recipe_name,
            item_name: itemName,
            quantity: event.delta,
          });
          if (craftLog.value.length > CRAFT_LOG_MAX) {
            craftLog.value.length = CRAFT_LOG_MAX;
          }
        }

        tracker.value = { ...tracker.value };
      }
    }
  }

  // Listen for player events (batched from backend)
  listen<PlayerEvent[]>("player-events-batch", (event) => {
    for (const pe of event.payload) {
      handleCraftDetection(pe);
    }
  });

  // ── Work Orders ──────────────────────────────────────────────────────────

  /**
   * Load and enrich work orders from the character snapshot.
   * Enriches each quest key with CDN quest data, item info, and matching recipes.
   */
  async function getWorkOrders(includeInventoryScrolls: boolean = false): Promise<EnrichedWorkOrder[]> {
    const settingsStore = useSettingsStore();
    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;
    if (!characterName || !serverName) return [];

    // Get work order data from snapshot
    const snapshot = await invoke<WorkOrderSnapshotData>("get_work_orders_from_snapshot", {
      characterName,
      serverName,
    });

    const activeSet = new Set(snapshot.active);
    const completedSet = new Set(snapshot.completed);

    // Collect all quest keys to enrich
    const questKeys = [...snapshot.active];

    // If including inventory scrolls, look up work order items from inventory/storage
    const inventoryQuestKeys = new Set<string>();
    if (includeInventoryScrolls && snapshot.inventory_item_ids.length > 0) {
      for (const typeId of snapshot.inventory_item_ids) {
        const item = await gameData.resolveItem(typeId);
        if (item?.bestow_quest) {
          // bestow_quest is the quest InternalName (e.g. "Toolcrafting_GrandShapingHammers")
          const questInternalName = item.bestow_quest;
          if (!activeSet.has(questInternalName)) {
            questKeys.push(questInternalName);
            inventoryQuestKeys.add(questInternalName);
          }
        }
      }
    }

    // Enrich each work order with quest + item + recipe data
    const results: EnrichedWorkOrder[] = [];
    for (const questKey of questKeys) {
      const quest = await gameData.resolveQuest(questKey);
      if (!quest) continue;
      const raw = quest.raw;

      // Only process actual work orders
      if (!raw.Keywords?.includes("WorkOrder")) continue;

      // Extract objective info
      const collectObjective = raw.Objectives?.find((o) => o.Type === "Collect");
      const itemInternalName = collectObjective?.ItemName ?? null;
      const quantity = collectObjective?.Number ?? 0;

      // Find the item
      let itemName: string | null = null;
      let recipeId: number | null = null;
      let recipeName: string | null = null;

      if (itemInternalName) {
        const item = await gameData.resolveItem(itemInternalName);
        if (item) {
          itemName = item.name;
          // Find a recipe that produces this item (filtering Max-Enchanted)
          const recipes = filterRecipes(await getCachedRecipesForItem(item.id));
          if (recipes.length > 0) {
            recipeId = recipes[0].id;
            recipeName = recipes[0].name;
          }
        }
      }

      // Extract rewards
      let industryXp = 0;
      let goldReward = 0;
      for (const reward of raw.Rewards ?? []) {
        if (reward.T === "SkillXp" && reward.Skill === "Industry") {
          industryXp = reward.Xp ?? 0;
        }
        if (reward.T === "WorkOrderCurrency" && reward.Currency === "Gold") {
          goldReward = reward.Amount ?? 0;
        }
      }

      // Extract Industry level requirement
      let industryLevelReq: number | null = null;
      for (const req of raw.Requirements ?? []) {
        if (req.T === "MinSkillLevel" && req.Skill === "Industry") {
          industryLevelReq = req.Level as number ?? null;
        }
      }

      results.push({
        quest_key: questKey,
        name: raw.Name ?? questKey,
        craft_skill: raw.WorkOrderSkill ?? null,
        item_internal_name: itemInternalName,
        item_name: itemName,
        quantity,
        industry_xp: industryXp,
        gold_reward: goldReward,
        industry_level_req: industryLevelReq,
        is_active: activeSet.has(questKey),
        is_completed: completedSet.has(questKey),
        is_in_inventory: inventoryQuestKeys.has(questKey),
        recipe_id: recipeId,
        recipe_name: recipeName,
      });
    }

    // Sort: active first, then by craft skill, then by name
    results.sort((a, b) => {
      if (a.is_active !== b.is_active) return a.is_active ? -1 : 1;
      const skillCmp = (a.craft_skill ?? "").localeCompare(b.craft_skill ?? "");
      if (skillCmp !== 0) return skillCmp;
      return a.name.localeCompare(b.name);
    });

    return results;
  }

  /**
   * Export material needs as a shareable text file via save dialog.
   */
  async function exportMaterialList(
    projectName: string,
    needs: MaterialNeed[],
  ): Promise<void> {
    const filePath = await save({
      filters: [{ name: "Text File", extensions: ["txt"] }],
      defaultPath: `${projectName.replace(/[^a-zA-Z0-9_-]/g, '_')}-materials.txt`,
    })

    if (!filePath) return

    const lines: string[] = [`Materials for: ${projectName}`, ""]

    // Items you need to acquire
    const shortfalls = needs.filter(n => n.shortfall > 0)
    if (shortfalls.length > 0) {
      lines.push("=== Still Needed ===")
      for (const n of shortfalls) {
        const cost = n.unit_price ? ` (~${Math.round(n.unit_price * n.shortfall).toLocaleString()}g est.)` : ""
        lines.push(`  ${n.item_name} x${n.shortfall}${cost}`)
      }
      lines.push("")
    }

    // Full material summary
    lines.push("=== Full Material List ===")
    for (const n of needs) {
      const have = n.inventory_have + n.storage_have
      const status = have >= n.quantity_needed ? "[OK]" : `[Need ${n.shortfall} more]`
      lines.push(`  ${n.item_name}: need ${n.quantity_needed}, have ${have} ${status}`)
    }

    const content = lines.join("\n") + "\n"
    await invoke("export_text_file", { filePath, content })
  }

  return {
    // Project state
    projects,
    activeProject,
    activeGroupName,
    loadProjects,
    loadProject,
    selectGroup,
    clearGroupSelection,
    getProjectsInGroup,
    createProject,
    updateProject,
    deleteProject,
    duplicateProject,
    addEntry,
    updateEntry,
    removeEntry,
    // Dependency resolver
    resolveRecipeIngredients,
    flattenIngredients,
    collectIntermediates,
    // Material availability
    checkMaterialAvailability,
    exportMaterialList,
    // Dynamic item preferences
    getDynamicItemDisabledSet,
    setDynamicItemDisabled,
    setAllDynamicItems,
    // Stock targets
    resolveStockTargets,
    queryItemStock,
    // Leveling helper
    estimateRecipeCost,
    getSkillLevel,
    levelingState,
    createProjectFromLevelingPlan,
    // Crafting history
    getCraftingHistory,
    getSkillCraftingStats,
    // Live crafting detection
    tracker,
    craftLog,
    startProjectTracking,
    startQuickCalcTracking,
    stopTracking,
    clearTracking,
    adjustTrackedOutput,
    // Work orders
    getWorkOrders,
  };
});
