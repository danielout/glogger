// TypeScript types matching Rust PlayerEvent enum (#[serde(tag = "kind")])

// === Item Events ===

export interface ItemAddedEvent {
  kind: 'ItemAdded'
  timestamp: string
  item_name: string
  instance_id: number
  slot_index: number
  is_new: boolean
}

export interface ItemStackChangedEvent {
  kind: 'ItemStackChanged'
  timestamp: string
  instance_id: number
  item_name: string | null
  item_type_id: number
  old_stack_size: number
  new_stack_size: number
  delta: number
  from_server: boolean
}

export type DeleteContext = 'StorageTransfer' | 'VendorSale' | 'Consumed' | 'Unknown'

export interface ItemDeletedEvent {
  kind: 'ItemDeleted'
  timestamp: string
  instance_id: number
  item_name: string | null
  context: DeleteContext
}

// === Skill Events ===

export interface SkillSnapshot {
  skill_type: string
  raw: number
  bonus: number
  xp: number
  tnl: number
  max: number
}

export interface SkillsLoadedEvent {
  kind: 'SkillsLoaded'
  timestamp: string
  skills: SkillSnapshot[]
}

// === NPC Events ===

export interface InteractionStartedEvent {
  kind: 'InteractionStarted'
  timestamp: string
  entity_id: number
  interaction_type: number
  npc_name: string
}

export interface FavorChangedEvent {
  kind: 'FavorChanged'
  timestamp: string
  npc_id: number
  npc_name: string
  delta: number
  is_gift: boolean
}

// === Vendor Events ===

export interface VendorSoldEvent {
  kind: 'VendorSold'
  timestamp: string
  price: number
  item_name: string
  instance_id: number
  is_buyback: boolean
}

export interface VendorStackUpdatedEvent {
  kind: 'VendorStackUpdated'
  timestamp: string
  instance_id: number
  item_type_id: number
  new_stack_size: number
  price: number
}

// === Storage Events ===

export interface StorageDepositEvent {
  kind: 'StorageDeposit'
  timestamp: string
  npc_id: number
  slot: number
  item_name: string
  instance_id: number
}

export interface StorageWithdrawalEvent {
  kind: 'StorageWithdrawal'
  timestamp: string
  npc_id: number
  instance_id: number
  quantity: number
}

// === Action Events ===

export interface DelayLoopStartedEvent {
  kind: 'DelayLoopStarted'
  timestamp: string
  duration: number
  action_type: string
  label: string
  entity_id: number
  abort_condition: string
}

// === Screen/Book Events ===

export interface ScreenTextEvent {
  kind: 'ScreenText'
  timestamp: string
  category: string
  message: string
}

export interface BookOpenedEvent {
  kind: 'BookOpened'
  timestamp: string
  title: string
  content: string
  book_type: string
}

// === Union Type ===

export type PlayerEvent =
  | ItemAddedEvent
  | ItemStackChangedEvent
  | ItemDeletedEvent
  | SkillsLoadedEvent
  | InteractionStartedEvent
  | FavorChangedEvent
  | VendorSoldEvent
  | VendorStackUpdatedEvent
  | StorageDepositEvent
  | StorageWithdrawalEvent
  | DelayLoopStartedEvent
  | ScreenTextEvent
  | BookOpenedEvent
