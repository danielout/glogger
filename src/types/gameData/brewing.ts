export type BrewingCategory =
  | "Beer"
  | "BeerKeg"
  | "Wine"
  | "LiquorUnaged"
  | "LiquorFinished"
  | "Utility";

export interface BrewItemEffect {
  tier: number;
  skill_level: number;
  ingredient_slots: string[];
  effect_pools: string[];
}

export interface BrewingVariableSlot {
  keyword: string;
  description: string | null;
  valid_item_ids: number[];
  stack_size: number;
}

export interface BrewingFixedIngredient {
  item_id: number;
  stack_size: number;
  chance_to_consume: number | null;
}

export interface BrewingRecipe {
  recipe_id: number;
  name: string;
  internal_name: string | null;
  description: string | null;
  icon_id: number | null;
  category: BrewingCategory;
  skill_level_req: number;
  xp: number;
  xp_first_time: number | null;
  xp_drop_off_level: number | null;
  usage_delay_message: string | null;
  fixed_ingredients: BrewingFixedIngredient[];
  variable_slots: BrewingVariableSlot[];
  brew_item_effect: BrewItemEffect | null;
  result_item_id: number | null;
}

export interface BrewingIngredient {
  item_id: number;
  name: string;
  internal_name: string | null;
  icon_id: number | null;
  brewing_keywords: string[];
}

/** Display-friendly category labels */
export const CATEGORY_LABELS: Record<BrewingCategory, string> = {
  Beer: "Beers (Glass)",
  BeerKeg: "Beers (Keg)",
  Wine: "Wines",
  LiquorUnaged: "Liquors (Un-Aged)",
  LiquorFinished: "Liquors (Finished)",
  Utility: "Utility",
};

/** Category display order */
export const CATEGORY_ORDER: BrewingCategory[] = [
  "Beer",
  "BeerKeg",
  "Wine",
  "LiquorUnaged",
  "LiquorFinished",
  "Utility",
];
