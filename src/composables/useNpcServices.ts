import {
  parseServices,
  type NpcService,
  type StoreService,
  type StorageService,
  type TrainingService,
  type BarterService,
  type StoreCapIncrease,
} from '../types/npcServices'
import type { NpcInfo } from '../types/gameData/npcs'
import { tierIndex } from './useFavorTiers'

export function getServices(npc: NpcInfo): NpcService[] {
  return parseServices(npc.services)
}

export function getStoreService(npc: NpcInfo): StoreService | null {
  return getServices(npc).find((s): s is StoreService => s.type === 'Store') ?? null
}

export function getStorageService(npc: NpcInfo): StorageService | null {
  return getServices(npc).find((s): s is StorageService => s.type === 'Storage') ?? null
}

export function getTrainingService(npc: NpcInfo): TrainingService | null {
  return getServices(npc).find((s): s is TrainingService => s.type === 'Training') ?? null
}

export function getBarterService(npc: NpcInfo): BarterService | null {
  return getServices(npc).find((s): s is BarterService => s.type === 'Barter') ?? null
}

export function goldCapAtTier(npc: NpcInfo, tier: string): StoreCapIncrease | null {
  const store = getStoreService(npc)
  if (!store || !store.capIncreases.length) return null

  const targetIdx = tierIndex(tier)
  let best: StoreCapIncrease | null = null
  let bestIdx = Infinity

  for (const cap of store.capIncreases) {
    const capIdx = tierIndex(cap.tier)
    // Lower index = higher tier. We want the entry whose tier matches,
    // or the highest tier that is still at or below the given tier (capIdx >= targetIdx).
    if (capIdx === targetIdx) return cap
    if (capIdx >= targetIdx && capIdx < bestIdx) {
      best = cap
      bestIdx = capIdx
    }
  }
  return best
}

export function maxGoldCap(npc: NpcInfo): StoreCapIncrease | null {
  const store = getStoreService(npc)
  if (!store || !store.capIncreases.length) return null

  let best: StoreCapIncrease | null = null
  let bestIdx = Infinity

  for (const cap of store.capIncreases) {
    const idx = tierIndex(cap.tier)
    if (idx < bestIdx) {
      best = cap
      bestIdx = idx
    }
  }
  return best
}

export function hasVendor(npc: NpcInfo): boolean {
  return getServices(npc).some(s => s.type === 'Store')
}

/** True if NPC has a Store service with CapIncreases — i.e. they actually buy items from players. */
export function hasBuyCapacity(npc: NpcInfo): boolean {
  const store = getStoreService(npc)
  return store != null && store.capIncreases.length > 0
}

export function hasStorage(npc: NpcInfo): boolean {
  return getServices(npc).some(s => s.type === 'Storage')
}

export function hasTraining(npc: NpcInfo): boolean {
  return getServices(npc).some(s => s.type === 'Training')
}
