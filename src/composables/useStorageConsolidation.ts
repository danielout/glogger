import { ref, computed, type ComputedRef } from "vue";
import { useGameStateStore } from "../stores/gameStateStore";

// ── Types ───────────────────────────────────────────────────────────────────

/** A single item that needs to move from one vault to another */
export interface PlannedMove {
  itemName: string;
  quantity: number;
  fromVaultKey: string;
  fromVaultName: string;
  fromAreaKey: string | null;
  toVaultKey: string;
  toVaultName: string;
  toAreaKey: string | null;
  /** Why this move was suggested */
  reason: "duplicate" | "type_specific";
  /** Has the player completed this move (auto-detected or manual check) */
  completed: boolean;
}

/** All moves grouped by zone, with pickups and dropoffs separated */
export interface ZoneStop {
  areaKey: string;
  areaName: string;
  /** Items to pick up from vaults here and carry to another zone */
  pickups: PlannedMove[];
  /** Items arriving from another zone to deposit in vaults here */
  dropoffs: PlannedMove[];
  /** Items moving between two vaults in this same zone (no carrying needed) */
  localMoves: PlannedMove[];
  completed: boolean;
}

export interface ConsolidationPlan {
  moves: PlannedMove[];
  zoneStops: ZoneStop[];
  slotsSaved: number;
  itemsToMove: number;
  zonesInvolved: number;
  typeSpecificSuggestions: number;
}

// ── Helpers ─────────────────────────────────────────────────────────────────

function isRoutableZone(zone: string | null): zone is string {
  if (!zone) return false;
  if (zone === "*") return false;
  if (!zone.startsWith("Area")) return false;
  return true;
}

// ── Composable ──────────────────────────────────────────────────────────────

export function useStorageConsolidation() {
  const gameState = useGameStateStore();

  const wizardActive = ref(false);
  const completedMoves = ref<Set<string>>(new Set());

  // ── Vault helpers ───────────────────────────────────────────────────────

  function vaultName(key: string): string {
    const detail = gameState.storageVaultsByKey[key];
    if (detail?.npc_friendly_name) return detail.npc_friendly_name;
    return key;
  }

  function vaultArea(key: string): string | null {
    return gameState.storageVaultsByKey[key]?.area ?? null;
  }

  // ── Plan generation ───────────────────────────────────────────────────

  const plan: ComputedRef<ConsolidationPlan> = computed(() => {
    const moves: PlannedMove[] = [];

    // ── Step 1: Find duplicates ──────────────────────────────────────

    // Group: item_name → vault_key → total_quantity
    const itemVaults = new Map<string, Map<string, number>>();
    for (const item of gameState.storage) {
      if (!itemVaults.has(item.item_name)) {
        itemVaults.set(item.item_name, new Map());
      }
      const vm = itemVaults.get(item.item_name)!;
      vm.set(item.vault_key, (vm.get(item.vault_key) ?? 0) + item.stack_size);
    }

    for (const [itemName, vaultMap] of itemVaults) {
      if (vaultMap.size < 2) continue;

      // Target: vault with the most of this item
      let bestVault = "";
      let bestQty = 0;
      for (const [vk, qty] of vaultMap) {
        if (qty > bestQty) {
          bestVault = vk;
          bestQty = qty;
        }
      }

      // Move from all other vaults to the best
      for (const [vk, qty] of vaultMap) {
        if (vk === bestVault) continue;
        const moveKey = `${itemName}|${vk}|${bestVault}`;
        moves.push({
          itemName,
          quantity: qty,
          fromVaultKey: vk,
          fromVaultName: vaultName(vk),
          fromAreaKey: vaultArea(vk),
          toVaultKey: bestVault,
          toVaultName: vaultName(bestVault),
          toAreaKey: vaultArea(bestVault),
          reason: "duplicate",
          completed: completedMoves.value.has(moveKey),
        });
      }
    }

    // ── Step 2: Type-specific vault opportunities ────────────────────
    // (Future: check items in generic vaults that could go to type-specific ones)

    // ── Build zone stops ─────────────────────────────────────────────
    // Separate moves into:
    //   - localMoves: source and target are in the same zone (rearrange locally)
    //   - pickups: source is here, target is in a different zone (carry out)
    //   - dropoffs: target is here, source is in a different zone (deposit from carry)

    const zonePickups = new Map<string, PlannedMove[]>();
    const zoneDropoffs = new Map<string, PlannedMove[]>();
    const zoneLocalMoves = new Map<string, PlannedMove[]>();

    for (const move of moves) {
      const from = move.fromAreaKey;
      const to = move.toAreaKey;
      const sameZone = from && to && from === to;

      if (sameZone && isRoutableZone(from)) {
        // Local move — both vaults in the same zone
        if (!zoneLocalMoves.has(from)) zoneLocalMoves.set(from, []);
        zoneLocalMoves.get(from)!.push(move);
      } else {
        // Cross-zone move
        if (isRoutableZone(from)) {
          if (!zonePickups.has(from)) zonePickups.set(from, []);
          zonePickups.get(from)!.push(move);
        }
        if (isRoutableZone(to)) {
          if (!zoneDropoffs.has(to)) zoneDropoffs.set(to, []);
          zoneDropoffs.get(to)!.push(move);
        }
      }
    }

    // Merge into zone stops
    const allZones = new Set([
      ...zonePickups.keys(),
      ...zoneDropoffs.keys(),
      ...zoneLocalMoves.keys(),
    ]);
    const zoneStops: ZoneStop[] = [];
    for (const zone of allZones) {
      const pickups = zonePickups.get(zone) ?? [];
      const dropoffs = zoneDropoffs.get(zone) ?? [];
      const localMoves = zoneLocalMoves.get(zone) ?? [];

      // Resolve friendly area name from any vault in this zone
      let friendlyName = zone;
      for (const v of gameState.storageVaults) {
        if (v.area === zone && v.area_name) {
          friendlyName = v.area_name;
          break;
        }
      }

      const allMovesHere = [...pickups, ...dropoffs, ...localMoves];
      zoneStops.push({
        areaKey: zone,
        areaName: friendlyName,
        pickups,
        dropoffs,
        localMoves,
        completed: allMovesHere.length > 0 && allMovesHere.every((m) => m.completed),
      });
    }

    // Sort: zones with only local moves last, then by total action count
    zoneStops.sort((a, b) => {
      const aCross = a.pickups.length + a.dropoffs.length;
      const bCross = b.pickups.length + b.dropoffs.length;
      if (aCross === 0 && bCross > 0) return 1;
      if (bCross === 0 && aCross > 0) return -1;
      return (bCross + b.localMoves.length) - (aCross + a.localMoves.length);
    });

    // ── Stats ────────────────────────────────────────────────────────

    // Slots saved = number of source stacks being consolidated (each move frees 1 slot)
    const slotsSaved = moves.filter((m) => !m.completed).length;

    return {
      moves,
      zoneStops,
      slotsSaved,
      itemsToMove: moves.filter((m) => !m.completed).length,
      zonesInvolved: zoneStops.length,
      typeSpecificSuggestions: moves.filter((m) => m.reason === "type_specific").length,
    };
  });

  // ── Move completion ─────────────────────────────────────────────────

  function moveKey(move: PlannedMove): string {
    return `${move.itemName}|${move.fromVaultKey}|${move.toVaultKey}`;
  }

  function toggleMoveCompleted(move: PlannedMove) {
    const key = moveKey(move);
    const next = new Set(completedMoves.value);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    completedMoves.value = next;
  }

  function isMoveCompleted(move: PlannedMove): boolean {
    return completedMoves.value.has(moveKey(move));
  }

  function resetCompletion() {
    completedMoves.value = new Set();
  }

  // ── Wizard mode ─────────────────────────────────────────────────────

  function startWizard() {
    wizardActive.value = true;
    resetCompletion();
  }

  function stopWizard() {
    wizardActive.value = false;
  }

  /** Current zone from game state */
  const currentZone = computed(() => {
    const area = gameState.world?.area as { area_name?: string } | null;
    return area?.area_name ?? null;
  });

  /** The zone stop for the player's current location, if any */
  const currentZoneStop = computed<ZoneStop | null>(() => {
    if (!currentZone.value) return null;
    return plan.value.zoneStops.find((zs) => zs.areaKey === currentZone.value) ?? null;
  });

  /** Zone stops not yet completed, excluding current zone */
  const remainingZoneStops = computed(() =>
    plan.value.zoneStops.filter((zs) => !zs.completed && zs.areaKey !== currentZone.value)
  );

  const completedCount = computed(() => plan.value.moves.filter((m) => m.completed).length);
  const totalCount = computed(() => plan.value.moves.length);

  // ── Route stops for trip planner ────────────────────────────────────

  const routeStops = computed(() => {
    const stops: { zone: string; purpose: string; details: string }[] = [];
    for (const zs of plan.value.zoneStops) {
      if (zs.completed) continue;
      for (const p of zs.pickups) {
        if (p.completed) continue;
        stops.push({
          zone: zs.areaKey,
          purpose: "pickup",
          details: `Pick up ${p.itemName} x${p.quantity} from ${p.fromVaultName}`,
        });
      }
      for (const d of zs.dropoffs) {
        if (d.completed) continue;
        stops.push({
          zone: zs.areaKey,
          purpose: "deposit",
          details: `Deposit ${d.itemName} at ${d.toVaultName}`,
        });
      }
    }
    return stops;
  });

  return {
    plan,
    wizardActive,
    startWizard,
    stopWizard,
    currentZone,
    currentZoneStop,
    remainingZoneStops,
    completedCount,
    totalCount,
    toggleMoveCompleted,
    isMoveCompleted,
    resetCompletion,
    routeStops,
  };
}
