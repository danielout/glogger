import type { QuestInfo, QuestRequirement } from '../types/gameData'
import { skillTotalLevel, type GameStateSkill, type GameStateFavor } from '../types/gameState'
import { isTierAtOrAbove, tierDisplayName } from './useFavorTiers'
import { extractNpcKeyFromFavorPath, extractNpcDisplayFromFavorPath } from '../utils/questDisplay'

export type RequirementStatus = 'met' | 'unmet' | 'unknown'

export interface EvaluatedRequirement {
  requirement: QuestRequirement
  status: RequirementStatus
  detail: string
}

export type QuestEligibility = 'eligible' | 'partial' | 'unknown' | 'ineligible'

const ELIGIBILITY_ORDER: Record<QuestEligibility, number> = {
  eligible: 0,
  partial: 1,
  unknown: 2,
  ineligible: 3,
}

export function eligibilitySort(a: QuestEligibility, b: QuestEligibility): number {
  return ELIGIBILITY_ORDER[a] - ELIGIBILITY_ORDER[b]
}

export function evaluateRequirement(
  req: QuestRequirement,
  skillsByName: Record<string, GameStateSkill>,
  favorByNpc: Record<string, GameStateFavor>,
): EvaluatedRequirement {
  if (req.T === 'MinSkillLevel' && req.Skill && req.MinSkillLevel != null) {
    const skill = skillsByName[req.Skill]
    const playerLevel = skill ? skillTotalLevel(skill) : 0
    const needed = req.MinSkillLevel
    if (playerLevel >= needed) {
      return { requirement: req, status: 'met', detail: `${req.Skill} ${playerLevel}/${needed}` }
    }
    return { requirement: req, status: 'unmet', detail: `${req.Skill} ${playerLevel}/${needed}` }
  }

  if (req.T === 'MinFavorLevel' && req.Npc && req.Level) {
    const npcKey = extractNpcKeyFromFavorPath(req.Npc)
    const npcDisplay = extractNpcDisplayFromFavorPath(req.Npc)
    const requiredTier = String(req.Level)
    const favor = favorByNpc[npcKey]
    const playerTier = favor?.favor_tier ?? null

    if (isTierAtOrAbove(playerTier, requiredTier)) {
      return {
        requirement: req,
        status: 'met',
        detail: `${npcDisplay}: ${tierDisplayName(playerTier!)} (need ${tierDisplayName(requiredTier)})`,
      }
    }
    const playerLabel = playerTier ? tierDisplayName(playerTier) : 'Unknown'
    return {
      requirement: req,
      status: favor ? 'unmet' : 'unknown',
      detail: `${npcDisplay}: ${playerLabel} (need ${tierDisplayName(requiredTier)})`,
    }
  }

  if (req.T === 'ActiveCombatSkill' && req.Skill) {
    const skill = skillsByName[req.Skill]
    if (skill) {
      return { requirement: req, status: 'met', detail: `Active skill: ${req.Skill}` }
    }
    return { requirement: req, status: 'unknown', detail: `Active skill: ${req.Skill}` }
  }

  if (req.T === 'QuestCompleted' && req.Quest) {
    return { requirement: req, status: 'unknown', detail: `Complete quest: ${req.Quest}` }
  }

  if (req.T === 'MinLevel' && req.Level) {
    return { requirement: req, status: 'unknown', detail: `Character level ${req.Level}` }
  }

  if (req.T === 'Race') {
    return { requirement: req, status: 'unknown', detail: `Race requirement` }
  }

  if (req.T === 'Or') {
    return { requirement: req, status: 'unknown', detail: 'One of several requirements' }
  }

  const label = req.T ? req.T.replace(/([A-Z])/g, ' $1').trim() : 'Unknown requirement'
  return { requirement: req, status: 'unknown', detail: label }
}

export function evaluateQuestEligibility(
  quest: QuestInfo,
  skillsByName: Record<string, GameStateSkill>,
  favorByNpc: Record<string, GameStateFavor>,
): { eligibility: QuestEligibility; requirements: EvaluatedRequirement[] } {
  const rawReqs = quest.raw?.Requirements
  if (!rawReqs || !Array.isArray(rawReqs) || rawReqs.length === 0) {
    return { eligibility: 'eligible', requirements: [] }
  }

  const evaluated = rawReqs.map(r => evaluateRequirement(r, skillsByName, favorByNpc))
  let met = 0
  let unmet = 0
  let unknown = 0

  for (const e of evaluated) {
    if (e.status === 'met') met++
    else if (e.status === 'unmet') unmet++
    else unknown++
  }

  let eligibility: QuestEligibility
  if (unmet === 0 && unknown === 0) {
    eligibility = 'eligible'
  } else if (unmet > 0) {
    eligibility = met > 0 ? 'partial' : 'ineligible'
  } else {
    // unmet === 0, unknown > 0
    eligibility = met > 0 ? 'partial' : 'unknown'
  }

  return { eligibility, requirements: evaluated }
}

export function eligibilityLabel(e: QuestEligibility): string {
  switch (e) {
    case 'eligible': return 'Eligible'
    case 'partial': return 'Partial'
    case 'unknown': return 'Unknown'
    case 'ineligible': return 'Not Met'
  }
}

export function eligibilityClasses(e: QuestEligibility): string {
  switch (e) {
    case 'eligible': return 'bg-green-400/20 border-green-400/40 text-green-300'
    case 'partial': return 'bg-yellow-400/20 border-yellow-400/40 text-yellow-300'
    case 'unknown': return 'bg-surface-elevated border-border-default text-text-muted'
    case 'ineligible': return 'bg-red-400/20 border-red-400/40 text-red-300'
  }
}

export function requirementStatusIcon(status: RequirementStatus): string {
  switch (status) {
    case 'met': return '\u2714'     // checkmark
    case 'unmet': return '\u2716'   // cross
    case 'unknown': return '?'
  }
}

export function requirementStatusColor(status: RequirementStatus): string {
  switch (status) {
    case 'met': return 'text-green-400'
    case 'unmet': return 'text-red-400'
    case 'unknown': return 'text-text-dim'
  }
}
