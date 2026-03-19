import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useGameDataStore } from "./gameDataStore";
import { useSettingsStore } from "./settingsStore";
import { useInventoryStore } from "./inventoryStore";
import type {
  CraftDetectionEvent,
  CraftingHistoryRecipe,
  CraftingProject,
  CraftingProjectSummary,
  CraftingTracker,
  FlattenedMaterial,
  IntermediateCraft,
  LevelingPlan,
  LevelingPlanStep,
  LevelingStrategy,
  MaterialAvailability,
  MaterialNeed,
  RecipeCompletionEntry,
  ResolvedIngredient,
  ResolvedRecipe,
  SkillCraftingStats,
  TrackedRecipeEntry,
  WorkOrderSnapshotData,
  EnrichedWorkOrder,
} from "../types/crafting";
import type { RecipeInfo } from "../types/gameData/recipes";
import type { PlayerEvent } from "../types/playerEvents";

export const useCraftingStore = defineStore("crafting", () => {
  const gameData = useGameDataStore();

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

  async function loadProjects() {
    projects.value = await invoke<CraftingProjectSummary[]>("get_crafting_projects");
  }

  async function loadProject(id: number) {
    activeProject.value = await invoke<CraftingProject>("get_crafting_project", { projectId: id });
  }

  async function createProject(name: string, notes?: string) {
    const id = await invoke<number>("create_crafting_project", {
      input: { name, notes },
    });
    await loadProjects();
    return id;
  }

  async function updateProject(id: number, name: string, notes: string) {
    await invoke("update_crafting_project", { input: { id, name, notes } });
    await loadProjects();
    if (activeProject.value?.id === id) {
      activeProject.value.name = name;
      activeProject.value.notes = notes;
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

  async function addEntry(projectId: number, recipeId: number, recipeName: string, quantity: number) {
    await invoke("add_project_entry", {
      input: { project_id: projectId, recipe_id: recipeId, recipe_name: recipeName, quantity },
    });
    if (activeProject.value?.id === projectId) {
      await loadProject(projectId);
    }
    await loadProjects();
  }

  async function updateEntry(entryId: number, quantity: number, expandedIngredientIds: number[] = []) {
    await invoke("update_project_entry", {
      input: { id: entryId, quantity, expanded_ingredient_ids: expandedIngredientIds },
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
   */
  async function resolveRecipeIngredients(
    recipe: RecipeInfo,
    desiredQuantity: number,
    expandIntermediates: boolean = false,
    visited: Set<number> = new Set(),
    expandItemIds?: Set<number>,
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
        const producingRecipes = filterRecipes(await gameData.getRecipesForItem(ing.item_id));
        if (producingRecipes.length > 0) {
          isCraftable = true;
          const sourceRecipe = producingRecipes[0];
          sourceRecipeId = sourceRecipe.id;
          sourceRecipeName = sourceRecipe.name;

          // Recursively resolve (with cycle detection)
          visited.add(ing.item_id);
          const subResolved = await resolveRecipeIngredients(
            sourceRecipe,
            expectedQty,
            true,
            visited,
            expandItemIds,
          );
          children = subResolved.ingredients;
          childCraftsNeeded = subResolved.craft_count;
          visited.delete(ing.item_id);
        }
      } else if (ing.item_id) {
        // Just check if craftable (without expanding)
        const producingRecipes = filterRecipes(await gameData.getRecipesForItem(ing.item_id));
        isCraftable = producingRecipes.length > 0;
      }

      // Get item name
      let itemName = ing.description ?? "Unknown item";
      if (ing.item_id) {
        const item = await gameData.getItem(ing.item_id);
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

    // Estimate cost: sum of ingredient vendor values
    const itemIds = ingredients
      .filter((i) => i.item_id !== null && i.children.length === 0)
      .map((i) => i.item_id!);

    let estimatedCost = 0;
    if (itemIds.length > 0) {
      const items = await gameData.getItemsBatch(itemIds);
      for (const ing of ingredients) {
        if (ing.item_id && ing.children.length === 0) {
          const item = items[ing.item_id];
          if (item?.value) {
            estimatedCost += item.value * 1.5 * ing.expected_quantity;
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
        if (ing.children.length > 0) {
          // This is an expanded intermediate — recurse into children
          walk(ing.children);
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
   * These are ingredients that have been expanded (children.length > 0).
   */
  function collectIntermediates(ingredients: ResolvedIngredient[]): IntermediateCraft[] {
    const intermediates: IntermediateCraft[] = [];

    function walk(list: ResolvedIngredient[]) {
      for (const ing of list) {
        if (ing.children.length > 0 && ing.source_recipe_id !== null && ing.item_id !== null) {
          intermediates.push({
            recipe_name: ing.source_recipe_name ?? ing.item_name,
            recipe_id: ing.source_recipe_id,
            item_name: ing.item_name,
            item_id: ing.item_id,
            crafts_needed: ing.crafts_needed,
            quantity_produced: ing.expected_quantity,
          });
          // Also collect nested intermediates
          walk(ing.children);
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
   * Dynamic (keyword-based) materials are skipped since they can't be inventory-checked.
   */
  async function checkMaterialAvailability(
    materials: Map<string, FlattenedMaterial>,
  ): Promise<MaterialNeed[]> {
    const settingsStore = useSettingsStore();
    const inventoryStore = useInventoryStore();

    // Filter to only concrete items (skip dynamic/keyword entries)
    const concreteMaterials = Array.from(materials.values()).filter(
      (m) => m.item_id !== null && !m.is_dynamic,
    );
    const itemIds = concreteMaterials.map((m) => m.item_id!);
    if (itemIds.length === 0) return [];

    // 1. Query storage vaults from latest snapshot
    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;

    let storageData: MaterialAvailability[] = [];
    if (characterName && serverName) {
      try {
        storageData = await invoke<MaterialAvailability[]>("check_material_availability", {
          characterName,
          serverName,
          itemTypeIds: itemIds,
        });
      } catch (e) {
        console.warn("[crafting] Failed to check storage availability:", e);
      }
    }

    // Build lookup from storage results
    const storageMap = new Map<number, MaterialAvailability>();
    for (const item of storageData) {
      storageMap.set(item.item_type_id, item);
    }

    // 2. Check live inventory
    const liveInventory = new Map<number, number>();
    for (const item of inventoryStore.items) {
      if (item.item_type_id && itemIds.includes(item.item_type_id)) {
        liveInventory.set(
          item.item_type_id,
          (liveInventory.get(item.item_type_id) ?? 0) + item.stack_size,
        );
      }
    }

    // 3. Get vendor prices for cost estimation
    const itemData = await gameData.getItemsBatch(itemIds);

    // 4. Build MaterialNeed list
    const needs: MaterialNeed[] = [];
    for (const mat of concreteMaterials) {
      const itemId = mat.item_id!;
      const storage = storageMap.get(itemId);
      const invQty = liveInventory.get(itemId) ?? 0;
      const storageQty = storage?.storage_quantity ?? 0;
      const totalHave = invQty + storageQty;
      const shortfall = Math.max(0, mat.expected_quantity - totalHave);

      const item = itemData[itemId];
      const vendorPrice = item?.value ? item.value * 1.5 : null;

      needs.push({
        item_id: itemId,
        item_name: mat.item_name,
        quantity_needed: mat.expected_quantity,
        inventory_have: invQty,
        storage_have: storageQty,
        vault_breakdown: storage?.vault_breakdown ?? [],
        shortfall,
        vendor_price: vendorPrice,
      });
    }

    return needs.sort((a, b) => a.item_name.localeCompare(b.item_name));
  }

  // ── Leveling Optimizer ───────────────────────────────────────────────────

  /**
   * Build a cumulative XP array from the per-level XP amounts.
   * Input: xpAmounts[i] = XP needed to go from level i to level i+1
   * Output: cumXp[i] = total XP needed to reach level i+1 from level 0
   */
  function buildCumulativeXp(xpAmounts: number[]): number[] {
    const cumXp: number[] = [];
    let total = 0;
    for (const amount of xpAmounts) {
      total += amount;
      cumXp.push(total);
    }
    return cumXp;
  }

  /**
   * Calculate XP needed to go from currentLevel to targetLevel using per-level XP table.
   * xpAmounts[i] = XP needed to go from level i to level i+1
   */
  function calculateXpNeeded(xpAmounts: number[], currentLevel: number, targetLevel: number): number {
    if (targetLevel <= currentLevel || targetLevel > xpAmounts.length) return 0;
    let total = 0;
    for (let i = currentLevel; i < targetLevel && i < xpAmounts.length; i++) {
      total += xpAmounts[i];
    }
    return total;
  }

  /**
   * Estimate the ingredient cost for one craft of a recipe.
   */
  async function estimateRecipeCost(recipe: RecipeInfo): Promise<number> {
    const itemIds = recipe.ingredients
      .filter((i) => i.item_id !== null)
      .map((i) => i.item_id!);
    if (itemIds.length === 0) return 0;

    const items = await gameData.getItemsBatch(itemIds);
    let cost = 0;
    for (const ing of recipe.ingredients) {
      if (ing.item_id) {
        const item = items[ing.item_id];
        if (item?.value) {
          const chanceToConsume = ing.chance_to_consume ?? 1;
          cost += item.value * 1.5 * ing.stack_size * chanceToConsume;
        }
      }
    }
    return Math.round(cost);
  }

  /**
   * Generate a leveling plan for a skill from currentLevel to targetLevel.
   */
  async function generateLevelingPlan(
    skillName: string,
    currentLevel: number,
    targetLevel: number,
    strategy: LevelingStrategy,
    includeUnlearnedRecipes: boolean = true,
    excludedRecipeIds: Set<number> = new Set(),
  ): Promise<LevelingPlan> {
    const settingsStore = useSettingsStore();
    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;

    // 1. Get XP table for this skill (per-level amounts → cumulative)
    const xpAmounts = await invoke<number[]>("get_xp_table_for_skill", { skillName });
    const xpTable = buildCumulativeXp(xpAmounts);
    const xpNeeded = calculateXpNeeded(xpAmounts, currentLevel, targetLevel);

    if (xpNeeded <= 0) {
      return {
        skill_name: skillName,
        current_level: currentLevel,
        target_level: targetLevel,
        strategy,
        xp_needed: 0,
        xp_from_first_time: 0,
        xp_from_grinding: 0,
        levels: [],
        steps: [],
        total_cost: 0,
        total_crafts: 0,
      };
    }

    // 2. Get all recipes that reward XP in this skill
    // reward_skill on recipes uses internal names (e.g., "JewelryCrafting"),
    // but skillName is the display name (e.g., "Jewelry Crafting").
    // Look up the internal name for proper matching.
    const skillInfo = await gameData.getSkillByName(skillName);
    const skillInternalName = skillInfo?.internal_name ?? skillName;

    const allRecipes = filterRecipes(await gameData.getRecipesForSkill(skillName));
    const relevantRecipes = allRecipes.filter(
      (r) => r.reward_skill === skillInternalName && ((r.reward_skill_xp ?? 0) > 0 || (r.reward_skill_xp_first_time ?? 0) > 0),
    );

    // 3. Get recipe completions to identify first-time bonus opportunities
    let completions: RecipeCompletionEntry[] = [];
    if (characterName && serverName) {
      try {
        completions = await invoke<RecipeCompletionEntry[]>("get_latest_recipe_completions", {
          characterName,
          serverName,
        });
      } catch {
        // No snapshot available — treat all as unknown
      }
    }
    const completionMap = new Map(completions.map((c) => [c.recipe_key, c.completions]));
    const hasCompletionData = completions.length > 0;

    // 4. Build candidate list with costs
    interface RecipeCandidate {
      recipe: RecipeInfo
      xpPerCraft: number
      firstTimeXp: number
      alreadyCrafted: boolean
      isKnown: boolean
      costPerCraft: number
      xpPerGold: number
    }

    const candidates: RecipeCandidate[] = [];
    for (const recipe of relevantRecipes) {
      // Skip manually excluded recipes
      if (excludedRecipeIds.has(recipe.id)) continue;
      // Skip recipes above target level (never reachable in this plan)
      if ((recipe.skill_level_req ?? 0) > targetLevel) continue;

      const internalName = recipe.internal_name ?? "";
      const isKnown = !hasCompletionData || completionMap.has(internalName);
      const alreadyCrafted = completionMap.has(internalName) && (completionMap.get(internalName)! > 0);

      // Skip unlearned recipes if toggle is off
      if (!includeUnlearnedRecipes && hasCompletionData && !isKnown) {
        continue;
      }

      const xpPerCraft = recipe.reward_skill_xp ?? 0;
      const firstTimeXp = alreadyCrafted ? 0 : (recipe.reward_skill_xp_first_time ?? 0);
      const costPerCraft = await estimateRecipeCost(recipe);
      const xpPerGold = costPerCraft > 0 ? xpPerCraft / costPerCraft : Infinity;

      candidates.push({
        recipe,
        xpPerCraft,
        firstTimeXp,
        alreadyCrafted,
        isKnown,
        costPerCraft,
        xpPerGold,
      });
    }

    // 5. Level-by-level planning
    // At each level, pick the best option:
    //   - First-time bonus available? Craft the highest-XP one once.
    //   - No bonuses (or cost-efficient only)? Grind the best XP/gold recipe until we level.
    // Re-evaluate at each level-up since new recipes unlock and others drop off.

    const levels: import("../types/crafting").LevelingPlanLevel[] = [];
    const usedFirstTime = new Set<number>();
    let xpFromFirstTime = 0;
    let xpFromGrinding = 0;
    let simLevel = currentLevel;
    let cumulativeXp = currentLevel > 0 ? (xpTable[currentLevel - 1] ?? 0) : 0;
    const targetCumulativeXp = xpTable[targetLevel - 1] ?? 0;

    /** Helper: get candidates available at a given level */
    function candidatesAtLevel(level: number) {
      return candidates.filter((c) => {
        if ((c.recipe.skill_level_req ?? 0) > level) return false;
        const dropOff = c.recipe.reward_skill_xp_drop_off_level;
        if (dropOff && level >= dropOff) return false;
        // Must give XP from crafting or from first-time bonus
        return c.xpPerCraft > 0 || c.firstTimeXp > 0;
      });
    }

    /** Helper: pick best grind recipe at a level by strategy (must have repeatable XP) */
    function pickGrindRecipe(level: number) {
      const available = candidatesAtLevel(level).filter((c) => c.xpPerCraft > 0);
      if (strategy === "first-time-rush") {
        return available.sort((a, b) => b.xpPerCraft - a.xpPerCraft)[0] ?? null;
      }
      return available.sort((a, b) => {
        if (a.costPerCraft === 0 && b.costPerCraft === 0) return b.xpPerCraft - a.xpPerCraft;
        if (a.costPerCraft === 0) return -1;
        if (b.costPerCraft === 0) return 1;
        return b.xpPerGold - a.xpPerGold;
      })[0] ?? null;
    }

    // Current level segment being built
    let currentLevelSteps: LevelingPlanStep[] = [];
    let levelSegmentStart = simLevel;

    /** Finalize the current level segment and start a new one */
    function finalizeLevelSegment(newLevel: number) {
      if (currentLevelSteps.length > 0) {
        const xpForLevel = xpAmounts[levelSegmentStart] ?? 0;
        levels.push({
          from_level: levelSegmentStart,
          to_level: levelSegmentStart + 1,
          xp_needed: xpForLevel,
          steps: currentLevelSteps,
          total_xp: currentLevelSteps.reduce((s, st) => s + st.total_xp, 0),
          total_crafts: currentLevelSteps.reduce((s, st) => s + st.craft_count, 0),
          total_cost: currentLevelSteps.reduce((s, st) => s + st.estimated_cost, 0),
        });
        currentLevelSteps = [];
      }
      levelSegmentStart = newLevel;
    }

    const MAX_ITERATIONS = 500;
    let iterations = 0;

    while (cumulativeXp < targetCumulativeXp && iterations++ < MAX_ITERATIONS) {
      // Try first-time bonuses at this level (unless cost-efficient only)
      if (strategy !== "cost-efficient") {
        const bonusCandidates = candidatesAtLevel(simLevel)
          .filter((c) => !c.alreadyCrafted && c.firstTimeXp > 0 && !usedFirstTime.has(c.recipe.id))
          .sort((a, b) => (b.xpPerCraft + b.firstTimeXp) - (a.xpPerCraft + a.firstTimeXp));

        for (const c of bonusCandidates) {
          if (cumulativeXp >= targetCumulativeXp) break;
          const stepXp = c.xpPerCraft + c.firstTimeXp;
          usedFirstTime.add(c.recipe.id);

          currentLevelSteps.push({
            recipe_id: c.recipe.id,
            recipe_name: c.recipe.name,
            craft_count: 1,
            xp_per_craft: c.xpPerCraft,
            xp_first_time: c.firstTimeXp,
            total_xp: stepXp,
            estimated_cost: c.costPerCraft,
            skill_level_req: c.recipe.skill_level_req,
            already_crafted: false,
            is_known: c.isKnown,
            xp_drop_off_level: c.recipe.reward_skill_xp_drop_off_level ?? null,
          });

          cumulativeXp += stepXp;
          xpFromFirstTime += stepXp;

          // Check if we leveled up
          if (simLevel < xpTable.length && cumulativeXp >= (xpTable[simLevel] ?? Infinity)) {
            simLevel++;
            finalizeLevelSegment(simLevel);
            break; // restart the while loop at new level
          }
        }
      }

      if (cumulativeXp >= targetCumulativeXp) break;

      // Re-check level after bonuses
      while (simLevel < xpTable.length && cumulativeXp >= (xpTable[simLevel] ?? Infinity)) {
        finalizeLevelSegment(simLevel + 1);
        simLevel++;
      }

      // Grind: pick best recipe, craft until we level up or reach target
      const grindRecipe = pickGrindRecipe(simLevel);
      if (!grindRecipe) break;

      const xpRemaining = Math.min(
        targetCumulativeXp - cumulativeXp,
        simLevel < xpTable.length ? (xpTable[simLevel] ?? targetCumulativeXp) - cumulativeXp : targetCumulativeXp - cumulativeXp,
      );

      if (xpRemaining <= 0) break;

      const craftsNeeded = Math.ceil(xpRemaining / grindRecipe.xpPerCraft);
      const stepXp = craftsNeeded * grindRecipe.xpPerCraft;

      currentLevelSteps.push({
        recipe_id: grindRecipe.recipe.id,
        recipe_name: grindRecipe.recipe.name,
        craft_count: craftsNeeded,
        xp_per_craft: grindRecipe.xpPerCraft,
        xp_first_time: 0,
        total_xp: stepXp,
        estimated_cost: craftsNeeded * grindRecipe.costPerCraft,
        skill_level_req: grindRecipe.recipe.skill_level_req,
        already_crafted: grindRecipe.alreadyCrafted,
        is_known: grindRecipe.isKnown,
        xp_drop_off_level: grindRecipe.recipe.reward_skill_xp_drop_off_level ?? null,
      });

      cumulativeXp += stepXp;
      xpFromGrinding += stepXp;

      // Update simulated level
      while (simLevel < xpTable.length && cumulativeXp >= (xpTable[simLevel] ?? Infinity)) {
        simLevel++;
        finalizeLevelSegment(simLevel);
      }
    }

    // Finalize any remaining steps
    if (currentLevelSteps.length > 0) {
      const xpForLevel = xpAmounts[levelSegmentStart] ?? 0;
      levels.push({
        from_level: levelSegmentStart,
        to_level: Math.min(levelSegmentStart + 1, targetLevel),
        xp_needed: xpForLevel,
        steps: currentLevelSteps,
        total_xp: currentLevelSteps.reduce((s, st) => s + st.total_xp, 0),
        total_crafts: currentLevelSteps.reduce((s, st) => s + st.craft_count, 0),
        total_cost: currentLevelSteps.reduce((s, st) => s + st.estimated_cost, 0),
      });
    }

    // Merge adjacent levels that use the exact same recipe set (for cleaner display)
    const mergedLevels: typeof levels = [];
    for (const lvl of levels) {
      const prev = mergedLevels[mergedLevels.length - 1];
      if (
        prev
        && prev.steps.length === 1 && lvl.steps.length === 1
        && prev.steps[0].recipe_id === lvl.steps[0].recipe_id
        && prev.steps[0].xp_first_time === 0 && lvl.steps[0].xp_first_time === 0
      ) {
        // Merge: same grind recipe across consecutive levels
        prev.to_level = lvl.to_level;
        prev.xp_needed += lvl.xp_needed;
        prev.steps[0].craft_count += lvl.steps[0].craft_count;
        prev.steps[0].total_xp += lvl.steps[0].total_xp;
        prev.steps[0].estimated_cost += lvl.steps[0].estimated_cost;
        prev.total_xp += lvl.total_xp;
        prev.total_crafts += lvl.total_crafts;
        prev.total_cost += lvl.total_cost;
      } else {
        mergedLevels.push(lvl);
      }
    }

    const allSteps = mergedLevels.flatMap((l) => l.steps);
    const totalCost = allSteps.reduce((sum, s) => sum + s.estimated_cost, 0);
    const totalCrafts = allSteps.reduce((sum, s) => sum + s.craft_count, 0);

    return {
      skill_name: skillName,
      current_level: currentLevel,
      target_level: targetLevel,
      strategy,
      xp_needed: xpNeeded,
      xp_from_first_time: xpFromFirstTime,
      xp_from_grinding: xpFromGrinding,
      levels: mergedLevels,
      steps: allSteps,
      total_cost: totalCost,
      total_crafts: totalCrafts,
    };
  }

  /**
   * Get the player's current skill level from the latest character snapshot.
   */
  async function getSkillLevel(skillName: string): Promise<{ level: number; xpTowardNext: number; xpNeededForNext: number } | null> {
    const settingsStore = useSettingsStore();
    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;
    if (!characterName || !serverName) return null;

    try {
      const result = await invoke<[number, number, number] | null>("get_latest_skill_level", {
        characterName,
        serverName,
        skillName,
      });
      if (!result) return null;
      return { level: result[0], xpTowardNext: result[1], xpNeededForNext: result[2] };
    } catch {
      return null;
    }
  }

  // ── Crafting History ─────────────────────────────────────────────────────

  /**
   * Get all recipe completions merged with CDN recipe data.
   * Returns enriched list sorted by completions (descending).
   */
  async function getCraftingHistory(): Promise<CraftingHistoryRecipe[]> {
    const settingsStore = useSettingsStore();
    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;
    if (!characterName || !serverName) return [];

    let completions: RecipeCompletionEntry[] = [];
    try {
      completions = await invoke<RecipeCompletionEntry[]>("get_latest_recipe_completions", {
        characterName,
        serverName,
      });
    } catch {
      return [];
    }

    // Enrich with CDN recipe data
    const results: CraftingHistoryRecipe[] = [];
    for (const entry of completions) {
      const recipe = await gameData.getRecipeByName(entry.recipe_key);
      results.push({
        recipe_key: entry.recipe_key,
        recipe_name: recipe?.name ?? entry.recipe_key,
        completions: entry.completions,
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
    const settingsStore = useSettingsStore();
    const characterName = settingsStore.settings.activeCharacterName;
    const serverName = settingsStore.settings.activeServerName;
    if (!characterName || !serverName) return [];

    // Get completions
    let completions: RecipeCompletionEntry[] = [];
    try {
      completions = await invoke<RecipeCompletionEntry[]>("get_latest_recipe_completions", {
        characterName,
        serverName,
      });
    } catch {
      return [];
    }

    const completionSet = new Set(
      completions.filter((c) => c.completions > 0).map((c) => c.recipe_key),
    );
    const completionMap = new Map(completions.map((c) => [c.recipe_key, c.completions]));

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
        const key = r.internal_name ?? r.name;
        if (completionSet.has(key)) {
          craftedCount++;
          totalCompletions += completionMap.get(key) ?? 0;
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
      const item = await gameData.getItem(primaryOutput.item_id);
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
      const recipe = await gameData.getRecipeByName(entry.recipe_name);
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
   * Handle a player event to detect crafted items.
   */
  function handleCraftDetection(event: PlayerEvent) {
    if (!tracker.value?.active) return;

    // We detect crafts via ItemAdded — when a new item appears that matches a recipe output.
    // ItemStackChanged with positive delta on an existing stack also counts (e.g., stackable outputs).
    if (event.kind === "ItemAdded" && event.is_new) {
      const lookup = buildOutputLookup();
      const idx = lookup.get(event.item_name);
      if (idx !== undefined) {
        // Will be updated when ItemStackChanged arrives with the actual quantity
        // For now, mark that we saw this item appear
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
        entry.detected_output += event.delta;
        entry.crafts_completed = Math.floor(entry.detected_output / entry.output_per_craft);

        // Update the most recent log entry for this item if it has quantity 0
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

        // Trigger reactivity
        tracker.value = { ...tracker.value };
      }
    }
  }

  // Listen for player events
  listen<PlayerEvent>("player-event", (event) => {
    handleCraftDetection(event.payload);
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
        const item = await gameData.getItem(typeId);
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
      const quest = await gameData.getQuestByInternalName(questKey);
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
        const item = await gameData.getItemByInternalName(itemInternalName);
        if (item) {
          itemName = item.name;
          // Find a recipe that produces this item (filtering Max-Enchanted)
          const recipes = filterRecipes(await gameData.getRecipesForItem(item.id));
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

  return {
    // Project state
    projects,
    activeProject,
    loadProjects,
    loadProject,
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
    // Leveling optimizer
    generateLevelingPlan,
    getSkillLevel,
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
    // Work orders
    getWorkOrders,
  };
});
