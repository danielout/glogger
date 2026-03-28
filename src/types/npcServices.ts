export interface NpcServiceBase {
  type: string
  favor: string
}

export interface StoreService extends NpcServiceBase {
  type: 'Store'
  capIncreases: StoreCapIncrease[]
}

export interface StoreCapIncrease {
  tier: string
  maxGold: number
  itemTypes: string[]
}

export interface TrainingService extends NpcServiceBase {
  type: 'Training'
  skills: string[]
  unlocks: string[]
}

export interface BarterService extends NpcServiceBase {
  type: 'Barter'
  additionalUnlocks: string[]
}

export interface ConsignmentService extends NpcServiceBase {
  type: 'Consignment'
  itemTypes: string[]
  unlocks: string[]
}

export interface StorageService extends NpcServiceBase {
  type: 'Storage'
  spaceIncreases: string[]
}

export interface GenericService extends NpcServiceBase {
  type: string
}

export type NpcService = StoreService | TrainingService | BarterService | ConsignmentService | StorageService | GenericService

function parseCapIncrease(raw: string): StoreCapIncrease {
  const [tier, gold, types] = raw.split(':')
  return {
    tier: tier ?? '',
    maxGold: Number(gold) || 0,
    itemTypes: types ? types.split(',') : [],
  }
}

export function parseServices(raw: unknown[] | null | undefined): NpcService[] {
  if (!raw || !Array.isArray(raw)) return []

  return raw.map((entry): NpcService => {
    const obj = entry as Record<string, unknown>
    const type = (obj.Type as string) ?? 'Unknown'
    const favor = (obj.Favor as string) ?? 'Despised'

    switch (type) {
      case 'Store':
        return {
          type: 'Store',
          favor,
          capIncreases: Array.isArray(obj.CapIncreases)
            ? (obj.CapIncreases as string[]).map(parseCapIncrease)
            : [],
        }
      case 'Training':
        return {
          type: 'Training',
          favor,
          skills: Array.isArray(obj.Skills) ? (obj.Skills as string[]) : [],
          unlocks: Array.isArray(obj.Unlocks) ? (obj.Unlocks as string[]) : [],
        }
      case 'Barter':
        return {
          type: 'Barter',
          favor,
          additionalUnlocks: Array.isArray(obj.AdditionalUnlocks) ? (obj.AdditionalUnlocks as string[]) : [],
        }
      case 'Consignment':
        return {
          type: 'Consignment',
          favor,
          itemTypes: Array.isArray(obj.ItemTypes) ? (obj.ItemTypes as string[]) : [],
          unlocks: Array.isArray(obj.Unlocks) ? (obj.Unlocks as string[]) : [],
        }
      case 'Storage':
        return {
          type: 'Storage',
          favor,
          spaceIncreases: Array.isArray(obj.SpaceIncreases) ? (obj.SpaceIncreases as string[]) : [],
        }
      default:
        return { type, favor }
    }
  })
}
