import type { InjectionKey } from "vue";
import { inject, provide } from "vue";

export type EntityType = "item" | "quest" | "skill" | "npc" | "ability" | "recipe" | "area" | "enemy";

export interface EntityNavigationTarget {
  type: EntityType;
  id: string | number;
}

export type NavigateToEntityFn = (target: EntityNavigationTarget) => void;

export const ENTITY_NAV_KEY: InjectionKey<NavigateToEntityFn> = Symbol("entity-navigation");

export function provideEntityNavigation(fn: NavigateToEntityFn) {
  provide(ENTITY_NAV_KEY, fn);
}

export function useEntityNavigation() {
  const navigateToEntity = inject(ENTITY_NAV_KEY, () => {
    console.warn("Entity navigation not provided");
  });
  return { navigateToEntity };
}
