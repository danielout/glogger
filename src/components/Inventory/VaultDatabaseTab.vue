<template>
  <div class="flex flex-col gap-4 h-full">
    <!-- Header with search and summary -->
    <div class="flex items-center gap-4 shrink-0">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search vaults by name, area, or stored items..."
        class="flex-1 px-3 py-1.5 bg-bg-tertiary border border-border-primary rounded text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-primary"
      />
      <div class="text-xs text-text-secondary whitespace-nowrap">
        {{ totalStoredItems }} items across {{ vaultsWithItems }} vaults
      </div>
    </div>

    <!-- Area cards grid -->
    <div class="flex-1 min-h-0 overflow-y-auto pr-1">
      <EmptyState
        v-if="gameState.storageVaults.length === 0"
        primary="No vault data loaded"
        secondary="Vault metadata is loaded from CDN game data"
      />
      <EmptyState
        v-else-if="filteredAreaGroups.length === 0 && searchQuery"
        primary="No matching vaults"
        :secondary="`No vaults matching '${searchQuery}'`"
      />
      <div v-else class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2">
        <VaultAreaCard
          v-for="group in filteredAreaGroups"
          :key="group.areaKey"
          :area-name="group.areaName"
          :area-key="group.areaKey"
          :vaults="group.vaults"
          :total-used="group.totalUsed"
          :total-unlocked="group.totalUnlocked"
          :total-max-possible="group.totalMaxPossible"
          :search-query="searchQuery"
          @select="selectedArea = selectedArea === group.areaKey ? null : group.areaKey"
          :selected="selectedArea === group.areaKey"
        />
      </div>

      <!-- Selected area detail panel -->
      <div v-if="selectedAreaGroup" class="mt-3 bg-bg-secondary border border-border-primary rounded p-3">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-text-primary">{{ selectedAreaGroup.areaName }}</span>
            <span class="text-xs text-text-muted">{{ selectedAreaGroup.vaults.length }} vaults</span>
          </div>
          <button
            class="text-xs text-text-muted hover:text-text-primary cursor-pointer bg-transparent border-none"
            @click="selectedArea = null"
          >&times; close</button>
        </div>
        <div class="flex flex-col gap-2">
          <VaultRow
            v-for="vault in selectedAreaGroup.vaults"
            :key="vault.detail.key"
            :detail="vault.detail"
            :items="vault.items"
            :max-possible-slots="vault.maxPossibleSlots"
            :unlocked-slots="vault.unlockedSlots"
            :favor-tier="vault.favorTier"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useGameStateStore } from "../../stores/gameStateStore";
import type { StorageVaultDetail, GameStateStorageItem } from "../../types/gameState";
import EmptyState from "../Shared/EmptyState.vue";
import VaultAreaCard from "./VaultAreaCard.vue";
import VaultRow from "./VaultRow.vue";

export interface VaultEntry {
  detail: StorageVaultDetail;
  items: GameStateStorageItem[];
  maxPossibleSlots: number | null;
  unlockedSlots: number | null;
  favorTier: string | null;
}

interface AreaGroup {
  areaKey: string;
  areaName: string;
  vaults: VaultEntry[];
  totalUsed: number;
  totalUnlocked: number | null;
  totalMaxPossible: number | null;
}

const gameState = useGameStateStore();
const searchQuery = ref("");
const selectedArea = ref<string | null>(null);

onMounted(() => {
  gameState.loadStorageVaults();
});

/** Build vault entries with items and capacity for every CDN vault */
const allVaultEntries = computed<VaultEntry[]>(() =>
  gameState.storageVaults.map(detail => ({
    detail,
    items: gameState.storageByVault[detail.key] ?? [],
    maxPossibleSlots: gameState.getVaultMaxPossibleSlots(detail),
    unlockedSlots: gameState.getVaultUnlockedSlots(detail),
    favorTier: gameState.getVaultFavorTier(detail),
  }))
);

/** Group all vaults by area */
const allAreaGroups = computed<AreaGroup[]>(() => {
  const areaMap = new Map<string, VaultEntry[]>();

  for (const entry of allVaultEntries.value) {
    const areaKey = entry.detail.area ?? "*";
    if (!areaMap.has(areaKey)) areaMap.set(areaKey, []);
    areaMap.get(areaKey)!.push(entry);
  }

  const groups: AreaGroup[] = [];
  for (const [areaKey, vaults] of areaMap) {
    const areaName = areaKey === "*"
      ? "Location-Independent"
      : vaults[0].detail.area_name ?? areaKey;

    // Sort vaults: those with items first, then alphabetical by display name
    vaults.sort((a, b) => {
      if (a.items.length > 0 && b.items.length === 0) return -1;
      if (a.items.length === 0 && b.items.length > 0) return 1;
      const nameA = a.detail.npc_friendly_name ?? a.detail.key;
      const nameB = b.detail.npc_friendly_name ?? b.detail.key;
      return nameA.localeCompare(nameB);
    });

    const totalUsed = vaults.reduce((sum, v) => sum + v.items.length, 0);
    let totalUnlocked: number | null = null;
    let totalMaxPossible: number | null = null;
    for (const v of vaults) {
      if (v.unlockedSlots != null && v.unlockedSlots > 0) {
        totalUnlocked = (totalUnlocked ?? 0) + v.unlockedSlots;
      }
      if (v.maxPossibleSlots != null && v.maxPossibleSlots > 0) {
        totalMaxPossible = (totalMaxPossible ?? 0) + v.maxPossibleSlots;
      }
    }

    groups.push({ areaKey, areaName, vaults, totalUsed, totalUnlocked, totalMaxPossible });
  }

  // Sort: areas with items first, then alphabetical. "*" sorts last.
  groups.sort((a, b) => {
    if (a.areaKey === "*") return 1;
    if (b.areaKey === "*") return -1;
    const aHasItems = a.totalUsed > 0;
    const bHasItems = b.totalUsed > 0;
    if (aHasItems && !bHasItems) return -1;
    if (!aHasItems && bHasItems) return 1;
    return a.areaName.localeCompare(b.areaName);
  });

  return groups;
});

/** Filter area groups by search query */
const filteredAreaGroups = computed<AreaGroup[]>(() => {
  if (!searchQuery.value.trim()) return allAreaGroups.value;
  const q = searchQuery.value.toLowerCase();

  return allAreaGroups.value
    .map(group => {
      const matchingVaults = group.vaults.filter(vault => {
        const name = vault.detail.npc_friendly_name ?? vault.detail.key;
        if (name.toLowerCase().includes(q)) return true;
        if (vault.detail.key.toLowerCase().includes(q)) return true;
        if (group.areaName.toLowerCase().includes(q)) return true;
        if (vault.items.some(item => item.item_name.toLowerCase().includes(q))) return true;
        return false;
      });

      if (matchingVaults.length === 0) return null;

      const totalUsed = matchingVaults.reduce((sum, v) => sum + v.items.length, 0);
      let totalUnlocked: number | null = null;
      let totalMaxPossible: number | null = null;
      for (const v of matchingVaults) {
        if (v.unlockedSlots != null && v.unlockedSlots > 0) {
          totalUnlocked = (totalUnlocked ?? 0) + v.unlockedSlots;
        }
        if (v.maxPossibleSlots != null && v.maxPossibleSlots > 0) {
          totalMaxPossible = (totalMaxPossible ?? 0) + v.maxPossibleSlots;
        }
      }

      return { ...group, vaults: matchingVaults, totalUsed, totalUnlocked, totalMaxPossible };
    })
    .filter((g): g is AreaGroup => g !== null);
});

const selectedAreaGroup = computed(() =>
  selectedArea.value
    ? filteredAreaGroups.value.find(g => g.areaKey === selectedArea.value) ?? null
    : null
);

const totalStoredItems = computed(() =>
  gameState.storage.reduce((sum, item) => sum + item.stack_size, 0)
);

const vaultsWithItems = computed(() =>
  Object.keys(gameState.storageByVault).length
);
</script>
