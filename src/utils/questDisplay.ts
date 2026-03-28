import type { QuestInfo, QuestReward, QuestRequirement } from '../types/gameData'

export function getQuestDisplayName(quest: QuestInfo): string {
  return quest.raw?.Name || quest.internal_name || 'Unknown Quest'
}

export function getQuestLevel(quest: QuestInfo): number | null {
  return quest.raw?.Level ?? null
}

export function getQuestArea(quest: QuestInfo): string | null {
  return quest.raw?.DisplayedLocation ?? null
}

export function getObjectiveTypeDisplay(type: string): string {
  const typeMap: Record<string, string> = {
    Kill: 'Kill',
    KillElite: 'Kill Elite',
    Collect: 'Collect',
    Scripted: 'Scripted Event',
    Deliver: 'Deliver',
    Harvest: 'Harvest',
    Loot: 'Loot',
    UseItem: 'Use Item',
    UseRecipe: 'Use Recipe',
    InteractionFlag: 'Interaction',
    BeAttacked: 'Be Attacked',
    GuildEventComplete: 'Guild Event',
    Have: 'Have',
    UseAbility: 'Use Ability',
    GiveGift: 'Give Gift',
    SayInChat: 'Say in Chat',
    Special: 'Special',
    UniqueSpecial: 'Special',
    Angling: 'Angling',
    Bury: 'Bury',
    TipPlayer: 'Tip Player',
    DruidKill: 'Druid Kill',
    DruidScripted: 'Druid Scripted',
    CompleteQuest: 'Complete Quest',
  }
  return typeMap[type] || type
}

export function getRewardTypeDisplay(reward: QuestReward): string {
  if (reward.T === 'SkillXp' && reward.Skill) {
    return `${reward.Skill}: ${reward.Xp} XP`
  }
  if (reward.T === 'CombatXp') {
    return `Combat XP: ${reward.Xp}`
  }
  if (reward.T === 'Currency' && reward.Currency) {
    return `${reward.Amount} ${reward.Currency}`
  }
  if (reward.T === 'GuildXp') {
    return `Guild XP: ${reward.Xp}`
  }
  if (reward.T === 'GuildCredits') {
    return `${reward.Amount} Guild Credits`
  }
  return reward.T
}

export function getRequirementDisplay(req: QuestRequirement): string {
  if (req.T === 'QuestCompleted' && req.Quest) {
    return `Quest: ${req.Quest}`
  }
  if (req.T === 'MinFavorLevel' && req.Npc) {
    const npcName = extractNpcDisplayFromFavorPath(req.Npc)
    return `${npcName}: ${req.Level} favor`
  }
  if (req.T === 'MinSkillLevel' && req.Skill) {
    return `${req.Skill} level ${req.MinSkillLevel}`
  }
  if (req.T === 'ActiveCombatSkill' && req.Skill) {
    return `Active skill: ${req.Skill}`
  }
  return req.T
}

export function formatReuseTime(quest: QuestInfo): string | null {
  if (quest.raw?.ReuseTime_Days) {
    return `${quest.raw.ReuseTime_Days} days`
  }
  if (quest.raw?.ReuseTime_Minutes) {
    const hours = Math.floor(quest.raw.ReuseTime_Minutes / 60)
    const mins = quest.raw.ReuseTime_Minutes % 60
    if (hours > 0 && mins > 0) return `${hours}h ${mins}m`
    if (hours > 0) return `${hours}h`
    return `${mins}m`
  }
  return null
}

/** Extract the CDN key (e.g. "NPC_Foo") from a FavorNpc path like "AreaName/NPC_Foo" */
export function extractNpcKeyFromFavorPath(favorNpc: string): string {
  const parts = favorNpc.split('/')
  return parts[parts.length - 1]
}

/** Extract a display name (e.g. "Foo") from a FavorNpc path like "AreaName/NPC_Foo" */
export function extractNpcDisplayFromFavorPath(favorNpc: string): string {
  const key = extractNpcKeyFromFavorPath(favorNpc)
  return key.replace(/^NPC_/, '').replace(/_/g, ' ')
}
