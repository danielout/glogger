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

// ── Discovery types ─────────────────────────────────────────────────────────

export interface BrewingDiscovery {
  id: number;
  character: string;
  recipe_id: number;
  ingredient_ids: number[];
  power: string;
  power_tier: number;
  effect_label: string | null;
  race_restriction: string | null;
  item_name: string | null;
  first_seen_at: string;
  last_seen_at: string;
}

export interface BrewingScanResult {
  new_discoveries: number;
  updated_discoveries: number;
  total_brewing_items: number;
}

// ── TSys power info (from batch resolve) ────────────────────────────────────

export interface ResolvedEffect {
  label: string;
  value: string;
  display_type: string;
  formatted: string;
  icon_id: number | null;
}

export interface TsysPowerInfo {
  internal_name: string;
  skill: string | null;
  prefix: string | null;
  suffix: string | null;
  slots: string[];
  tier_effects: string[];
  tier_effects_structured: ResolvedEffect[];
  icon_id: number | null;
}

// ── Effect pool labels ──────────────────────────────────────────────────────

/** Strip the trailing number from a pool name to get the base category */
export function parsePoolName(pool: string): { base: string; weight: number } {
  const match = pool.match(/^(.+?)(\d+)$/);
  if (match) {
    return { base: match[1], weight: parseInt(match[2]) };
  }
  return { base: pool, weight: 0 };
}

/** Human-friendly labels for effect pool base categories */
const POOL_LABELS: Record<string, { label: string; description: string }> = {
  Partying: {
    label: "Party & Dance",
    description: "Dance appreciation, alcohol tolerance, and social effects",
  },
  Gathering: {
    label: "Gathering",
    description: "Bonuses to foraging, lumberjack, mining, and other gathering skills",
  },
  SkillSpecificPowerCosts: {
    label: "Ability Costs",
    description: "Reduced power costs for specific combat skill abilities",
  },
  Endurance: {
    label: "Endurance",
    description: "Max health, max power, sprint speed, and survival effects",
  },
  RacialBonuses: {
    label: "Racial Bonuses",
    description: "Race-specific effects — may be restricted to certain races!",
  },
  BasicMitigation: {
    label: "Mitigation",
    description: "Damage mitigation against various damage types",
  },
  DamageVsAnatomy: {
    label: "Damage vs Creatures",
    description: "Bonus damage against specific creature types (orcs, undead, etc.)",
  },
  DirectDamageBoosts: {
    label: "Direct Damage",
    description: "Flat increases to damage dealt by specific skills",
  },
  EliteFighting: {
    label: "Elite Combat",
    description: "Bonuses when fighting elite and boss enemies",
  },
  Angling: {
    label: "Fishing",
    description: "Fishing-related bonuses",
  },
  CorpseActions: {
    label: "Corpse Actions",
    description: "Bonuses to burial, autopsy, and other corpse interactions",
  },
  FishAndGame: {
    label: "Fish & Game",
    description: "Hunting, fishing, and animal-related bonuses",
  },
  SkillBaseDamage: {
    label: "Skill Damage",
    description: "Base damage boosts for specific combat skills",
  },
  TBD: {
    label: "Placeholder",
    description: "Effects not yet implemented by the developers",
  },
};

/** Get a friendly label for an effect pool name like "Partying4" */
export function getPoolLabel(pool: string): string {
  const { base } = parsePoolName(pool);
  return POOL_LABELS[base]?.label ?? base;
}

/** Get a description for an effect pool name */
export function getPoolDescription(pool: string): string {
  const { base } = parsePoolName(pool);
  return POOL_LABELS[base]?.description ?? "";
}
