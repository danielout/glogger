import type { NpcInfo, NpcPreference } from '../types/gameData/npcs'

const GIFT_DESIRES = new Set(['Love', 'Like'])

/**
 * Strip the `=value` suffix from CDN item keywords.
 * e.g. "SnailShell=50" → "SnailShell"
 */
export function stripKeywordValue(keyword: string): string {
  const eqIdx = keyword.indexOf('=')
  return eqIdx === -1 ? keyword : keyword.substring(0, eqIdx)
}

export function matchesPreference(itemKeywords: string[], pref: NpcPreference): boolean {
  const prefLower = pref.keywords.map(k => k.toLowerCase())
  return itemKeywords.some(ik => prefLower.includes(stripKeywordValue(ik).toLowerCase()))
}

export function findMatchingPreferences(
  itemKeywords: string[],
  npc: NpcInfo,
): { pref: NpcPreference; desire: string; value: number }[] {
  return npc.preferences
    .filter(p => GIFT_DESIRES.has(p.desire) && matchesPreference(itemKeywords, p))
    .map(p => ({ pref: p, desire: p.desire, value: p.pref }))
    .sort((a, b) => b.value - a.value)
}

export function findGiftableItems(
  npc: NpcInfo,
  items: { name: string; keywords: string[] }[],
): { item: { name: string; keywords: string[] }; bestPref: NpcPreference }[] {
  const seen = new Set<string>()
  const results: { item: { name: string; keywords: string[] }; bestPref: NpcPreference }[] = []

  for (const item of items) {
    if (seen.has(item.name)) continue

    const matches = findMatchingPreferences(item.keywords, npc)
    if (matches.length > 0) {
      seen.add(item.name)
      results.push({ item, bestPref: matches[0].pref })
    }
  }

  return results.sort((a, b) => b.bestPref.pref - a.bestPref.pref)
}

export function findInterestedNpcs(
  itemKeywords: string[],
  allNpcs: NpcInfo[],
): { npc: NpcInfo; bestPref: NpcPreference }[] {
  const results: { npc: NpcInfo; bestPref: NpcPreference }[] = []

  for (const npc of allNpcs) {
    const matches = findMatchingPreferences(itemKeywords, npc)
    if (matches.length > 0) {
      results.push({ npc, bestPref: matches[0].pref })
    }
  }

  return results.sort((a, b) => b.bestPref.pref - a.bestPref.pref)
}
