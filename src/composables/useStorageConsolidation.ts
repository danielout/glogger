import { ref, computed, type ComputedRef } from "vue";
import { useGameStateStore } from "../stores/gameStateStore";

// ── Types ───────────────────────────────────────────────────────────────────

export interface ConsolidationLocation {
  vaultKey: string;
  displayName: string;
  areaKey: string | null;
  areaName: string | null;
  quantity: number;
}

export interface ConsolidationCandidate {
  itemName: string;
  totalQuantity: number;
  locations: ConsolidationLocation[];
  /** The vault key chosen as consolidation target */
  targetVaultKey: string;
  targetDisplayName: string;
  targetAreaKey: string | null;
}

export type TargetStrategy = "most_items" | "specific_vault";

export interface RouteStop {
  zone: string;
  purpose: "pickup" | "deposit";
  details: string;
}

// ── Composable ──────────────────────────────────────────────────────────────

export function useStorageConsolidation() {
  const gameState = useGameStateStore();

  const targetStrategy = ref<TargetStrategy>("most_items");
  const specificVaultKey = ref<string | null>(null);
  const selectedItems = ref<Set<string>>(new Set());

  // ── Vault display name helper ─────────────────────────────────────────

  function vaultDisplayName(vaultKey: string): string {
    const detail = gameState.storageVaultsByKey[vaultKey];
    if (detail?.npc_friendly_name) return detail.npc_friendly_name;
    if (detail?.area_name) return `${detail.area_name} - ${vaultKey}`;
    return vaultKey;
  }

  function vaultAreaKey(vaultKey: string): string | null {
    return gameState.storageVaultsByKey[vaultKey]?.area ?? null;
  }

  function vaultAreaName(vaultKey: string): string | null {
    return gameState.storageVaultsByKey[vaultKey]?.area_name ?? null;
  }

  // ── All items in 2+ locations ─────────────────────────────────────────

  const allCandidates: ComputedRef<ConsolidationCandidate[]> = computed(() => {
    // Group storage items by item name -> vault key -> total quantity
    const itemMap = new Map<string, Map<string, number>>();

    for (const item of gameState.storage) {
      if (!itemMap.has(item.item_name)) {
        itemMap.set(item.item_name, new Map());
      }
      const vaultMap = itemMap.get(item.item_name)!;
      vaultMap.set(
        item.vault_key,
        (vaultMap.get(item.vault_key) ?? 0) + item.stack_size
      );
    }

    const candidates: ConsolidationCandidate[] = [];

    for (const [itemName, vaultMap] of itemMap) {
      if (vaultMap.size < 2) continue;

      const locations: ConsolidationLocation[] = [];
      let totalQuantity = 0;

      for (const [vaultKey, quantity] of vaultMap) {
        totalQuantity += quantity;
        locations.push({
          vaultKey,
          displayName: vaultDisplayName(vaultKey),
          areaKey: vaultAreaKey(vaultKey),
          areaName: vaultAreaName(vaultKey),
          quantity,
        });
      }

      // Sort locations by quantity descending
      locations.sort((a, b) => b.quantity - a.quantity);

      // Determine target based on strategy
      const target = resolveTarget(itemName, locations);

      candidates.push({
        itemName,
        totalQuantity,
        locations,
        targetVaultKey: target.vaultKey,
        targetDisplayName: target.displayName,
        targetAreaKey: target.areaKey,
      });
    }

    // Sort by location count descending, then by name
    candidates.sort((a, b) => {
      const locDiff = b.locations.length - a.locations.length;
      if (locDiff !== 0) return locDiff;
      return a.itemName.localeCompare(b.itemName);
    });

    return candidates;
  });

  function resolveTarget(
    _itemName: string,
    locations: ConsolidationLocation[]
  ): { vaultKey: string; displayName: string; areaKey: string | null } {
    if (
      targetStrategy.value === "specific_vault" &&
      specificVaultKey.value
    ) {
      return {
        vaultKey: specificVaultKey.value,
        displayName: vaultDisplayName(specificVaultKey.value),
        areaKey: vaultAreaKey(specificVaultKey.value),
      };
    }
    // "most_items" -- pick the location with the most of this item
    const best = locations[0]; // already sorted descending by quantity
    return {
      vaultKey: best.vaultKey,
      displayName: best.displayName,
      areaKey: best.areaKey,
    };
  }

  // ── Selection management ──────────────────────────────────────────────

  /** Initialize selection with all candidates checked */
  function selectAll() {
    selectedItems.value = new Set(
      allCandidates.value.map((c) => c.itemName)
    );
  }

  function deselectAll() {
    selectedItems.value = new Set();
  }

  function toggleItem(itemName: string) {
    const next = new Set(selectedItems.value);
    if (next.has(itemName)) {
      next.delete(itemName);
    } else {
      next.add(itemName);
    }
    selectedItems.value = next;
  }

  function isSelected(itemName: string): boolean {
    return selectedItems.value.has(itemName);
  }

  // ── Selected candidates (filtered by checkbox selection) ──────────────

  const selectedCandidates = computed(() =>
    allCandidates.value.filter((c) => selectedItems.value.has(c.itemName))
  );

  // ── Route stop generation ─────────────────────────────────────────────

  /**
   * Check if a zone key is a valid routable area.
   * "*" means portable/global storage (Saddlebag, Council Storage, etc.) — player always has access.
   */
  function isRoutableZone(zone: string | null): zone is string {
    if (!zone) return false;
    if (zone === "*") return false;
    if (!zone.startsWith("Area")) return false;
    return true;
  }

  /** True if this vault is portable (player carries it everywhere) */
  function isPortableVault(areaKey: string | null): boolean {
    return areaKey === "*";
  }

  const routeStops = computed<RouteStop[]>(() => {
    const stops: RouteStop[] = [];

    for (const candidate of selectedCandidates.value) {
      // Source locations: all locations except the target
      for (const loc of candidate.locations) {
        if (loc.vaultKey === candidate.targetVaultKey) continue;
        // Portable vaults (Saddlebag, Council Storage, etc.) need no pickup stop —
        // the player always has access to them
        if (isPortableVault(loc.areaKey)) continue;
        if (!isRoutableZone(loc.areaKey)) continue;

        stops.push({
          zone: loc.areaKey,
          purpose: "pickup",
          details: `Pick up ${candidate.itemName} x${loc.quantity} from ${loc.displayName}`,
        });
      }

      // Deposit location — skip if target is portable storage (no travel needed)
      if (isPortableVault(candidate.targetAreaKey)) continue;
      if (!isRoutableZone(candidate.targetAreaKey)) continue;
      stops.push({
        zone: candidate.targetAreaKey,
        purpose: "deposit",
        details: `Deposit ${candidate.itemName} at ${candidate.targetDisplayName}`,
      });
    }

    return stops;
  });

  // ── Available vaults (for specific vault picker) ──────────────────────

  const availableVaults = computed(() => {
    const vaults: { key: string; displayName: string; areaName: string | null }[] = [];
    for (const detail of gameState.storageVaults) {
      if (detail.area) {
        vaults.push({
          key: detail.key,
          displayName: detail.npc_friendly_name ?? detail.key,
          areaName: detail.area_name ?? null,
        });
      }
    }
    // Sort by area name then vault name
    vaults.sort((a, b) => {
      const areaCmp = (a.areaName ?? "").localeCompare(b.areaName ?? "");
      if (areaCmp !== 0) return areaCmp;
      return a.displayName.localeCompare(b.displayName);
    });
    return vaults;
  });

  // ── Summary stats ─────────────────────────────────────────────────────

  const candidateCount = computed(() => allCandidates.value.length);

  const totalLocationCount = computed(() =>
    allCandidates.value.reduce((sum, c) => sum + c.locations.length, 0)
  );

  return {
    // Strategy
    targetStrategy,
    specificVaultKey,

    // Data
    allCandidates,
    selectedCandidates,
    candidateCount,
    totalLocationCount,
    availableVaults,

    // Selection
    selectedItems,
    selectAll,
    deselectAll,
    toggleItem,
    isSelected,

    // Route
    routeStops,
  };
}
