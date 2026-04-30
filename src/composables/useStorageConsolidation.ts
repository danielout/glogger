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
    // Classify each move:
    //   - localMoves: source and target in same zone (no carrying)
    //   - cross-zone: pickup at source zone, dropoff at target zone
    //
    // Key insight: dropoffs should only appear at a zone AFTER the
    // corresponding pickup zone has been visited. We order zones so
    // pickup-only zones come first, then zones that are both pickup
    // and dropoff, then dropoff-only zones.

    const perZonePickups = new Map<string, PlannedMove[]>();
    const perZoneDropoffs = new Map<string, PlannedMove[]>();
    const perZoneLocal = new Map<string, PlannedMove[]>();

    for (const move of moves) {
      const from = move.fromAreaKey;
      const to = move.toAreaKey;
      const sameZone = from && to && from === to;

      if (sameZone && isRoutableZone(from)) {
        if (!perZoneLocal.has(from)) perZoneLocal.set(from, []);
        perZoneLocal.get(from)!.push(move);
      } else {
        if (isRoutableZone(from)) {
          if (!perZonePickups.has(from)) perZonePickups.set(from, []);
          perZonePickups.get(from)!.push(move);
        }
        if (isRoutableZone(to)) {
          if (!perZoneDropoffs.has(to)) perZoneDropoffs.set(to, []);
          perZoneDropoffs.get(to)!.push(move);
        }
      }
    }

    // Collect all zones and classify them for ordering
    const allZones = new Set([
      ...perZonePickups.keys(),
      ...perZoneDropoffs.keys(),
      ...perZoneLocal.keys(),
    ]);

    // Helper: resolve friendly area name
    function zoneFriendlyName(zone: string): string {
      for (const v of gameState.storageVaults) {
        if (v.area === zone && v.area_name) return v.area_name;
      }
      return zone;
    }

    const zoneStops: ZoneStop[] = [];
    for (const zone of allZones) {
      const pickups = perZonePickups.get(zone) ?? [];
      const dropoffs = perZoneDropoffs.get(zone) ?? [];
      const localMoves = perZoneLocal.get(zone) ?? [];
      const allMovesHere = [...pickups, ...dropoffs, ...localMoves];

      zoneStops.push({
        areaKey: zone,
        areaName: zoneFriendlyName(zone),
        pickups,
        dropoffs,
        localMoves,
        completed: allMovesHere.length > 0 && allMovesHere.every((m) => m.completed),
      });
    }

    // Order zones so the route makes sense:
    // 1. Zones with pickups but no dropoffs (pure sources — visit first)
    // 2. Zones with both pickups and dropoffs (swap stops)
    // 3. Zones with dropoffs but no pickups (pure destinations — visit last)
    // 4. Zones with only local moves (can be done anytime)
    // Within each group, sort by action count descending.
    function zoneOrder(zs: ZoneStop): number {
      const hasPickup = zs.pickups.length > 0;
      const hasDropoff = zs.dropoffs.length > 0;
      const hasLocal = zs.localMoves.length > 0;
      if (hasPickup && !hasDropoff) return 0;  // pure source
      if (hasPickup && hasDropoff) return 1;    // swap
      if (!hasPickup && hasDropoff) return 2;   // pure destination
      if (hasLocal) return 3;                   // local only
      return 4;
    }

    zoneStops.sort((a, b) => {
      const orderDiff = zoneOrder(a) - zoneOrder(b);
      if (orderDiff !== 0) return orderDiff;
      const aCount = a.pickups.length + a.dropoffs.length + a.localMoves.length;
      const bCount = b.pickups.length + b.dropoffs.length + b.localMoves.length;
      return bCount - aCount;
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

  // ── Action completion ───────────────────────────────────────────────
  // Each cross-zone move has two actions: pickup and dropoff.
  // Local moves have one action (the rearrangement).
  // We track completion of each action separately.

  /** Completed actions, keyed as "pickup|item|vault" or "dropoff|item|vault" or "local|item|from|to" */
  const completedActions = ref<Set<string>>(new Set());

  function pickupKey(move: PlannedMove): string {
    return `pickup|${move.itemName}|${move.fromVaultKey}`;
  }

  function dropoffKey(move: PlannedMove): string {
    return `dropoff|${move.itemName}|${move.toVaultKey}`;
  }

  function localKey(move: PlannedMove): string {
    return `local|${move.itemName}|${move.fromVaultKey}|${move.toVaultKey}`;
  }

  function isPickupDone(move: PlannedMove): boolean {
    return completedActions.value.has(pickupKey(move));
  }

  function isDropoffDone(move: PlannedMove): boolean {
    return completedActions.value.has(dropoffKey(move));
  }

  function isLocalDone(move: PlannedMove): boolean {
    return completedActions.value.has(localKey(move));
  }

  function togglePickup(move: PlannedMove) {
    const key = pickupKey(move);
    const next = new Set(completedActions.value);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    completedActions.value = next;
  }

  function toggleDropoff(move: PlannedMove) {
    const key = dropoffKey(move);
    const next = new Set(completedActions.value);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    completedActions.value = next;
  }

  function toggleLocal(move: PlannedMove) {
    const key = localKey(move);
    const next = new Set(completedActions.value);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    completedActions.value = next;
  }

  // Legacy compat for plan.moves[].completed
  function isMoveCompleted(move: PlannedMove): boolean {
    if (move.fromAreaKey === move.toAreaKey) return isLocalDone(move);
    return isPickupDone(move) && isDropoffDone(move);
  }

  function resetCompletion() {
    completedActions.value = new Set();
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

  /**
   * Items currently in the player's carry bag: pickup was checked done
   * but the corresponding dropoff has NOT been checked done yet.
   */
  const carryBag = computed<Set<string>>(() => {
    const bag = new Set<string>();
    for (const move of plan.value.moves) {
      if (move.fromAreaKey === move.toAreaKey) continue; // local moves don't carry
      if (isPickupDone(move) && !isDropoffDone(move)) {
        bag.add(`${move.itemName}|${move.fromVaultKey}`);
      }
    }
    return bag;
  });

  /** Is this dropoff item currently in the player's carry bag? */
  function isInCarryBag(move: PlannedMove): boolean {
    return carryBag.value.has(`${move.itemName}|${move.fromVaultKey}`);
  }

  /** The zone stop for the player's current location, filtered for actionability */
  const currentZoneStop = computed<ZoneStop | null>(() => {
    if (!currentZone.value) return null;
    const raw = plan.value.zoneStops.find((zs) => zs.areaKey === currentZone.value);
    if (!raw) return null;

    // Filter dropoffs to only items actually in the carry bag
    return {
      ...raw,
      dropoffs: raw.dropoffs.filter((d) => isInCarryBag(d)),
    };
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
    carryBag,
    remainingZoneStops,
    completedCount,
    totalCount,
    // Action tracking (separate pickup/dropoff/local checkboxes)
    isPickupDone,
    isDropoffDone,
    isLocalDone,
    togglePickup,
    toggleDropoff,
    toggleLocal,
    isInCarryBag,
    isMoveCompleted,
    resetCompletion,
    routeStops,
  };
}
