export interface QuestObjective {
  Description: string
  Type: string
  Number?: number
  Target?: string
  ItemName?: string
}

export interface QuestReward {
  T: string
  Skill?: string
  Xp?: number
  Amount?: number
  Currency?: string
}

export interface QuestRewardItem {
  Item: string
  StackSize: number
}

export interface QuestRequirement {
  T: string
  Quest?: string
  Level?: number | string
  Skill?: string
  MinSkillLevel?: number
  Npc?: string
}

export interface QuestData {
  Name?: string
  InternalName?: string
  Description?: string
  DisplayedLocation?: string
  PrefaceText?: string
  SuccessText?: string
  MidwayText?: string
  Version?: number
  IsCancellable?: boolean
  ReuseTime_Minutes?: number
  ReuseTime_Days?: number
  FavorNpc?: string
  Reward_Favor?: number
  Objectives?: QuestObjective[]
  Rewards?: QuestReward[]
  Rewards_Items?: QuestRewardItem[]
  Rewards_NamedLootProfile?: string
  Requirements?: QuestRequirement[]
  Keywords?: string[]
  WorkOrderSkill?: string
  Level?: number
}

export interface QuestInfo {
  internal_name: string
  raw: QuestData
}
